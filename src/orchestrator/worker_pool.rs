use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::db::{findings as findings_db, scans as scans_db};
use crate::models::finding::Finding;
use crate::models::scan::ScanStatus;
use crate::scanners::security_headers;

#[derive(Debug)]
pub enum OrchestratorError {
    DatabaseError(sqlx::Error),
    AllScannersFailed(String),
}

impl std::fmt::Display for OrchestratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrchestratorError::DatabaseError(e) => write!(f, "Database error: {}", e),
            OrchestratorError::AllScannersFailed(msg) => write!(f, "All scanners failed: {}", msg),
        }
    }
}

impl std::error::Error for OrchestratorError {}

impl From<sqlx::Error> for OrchestratorError {
    fn from(e: sqlx::Error) -> Self {
        OrchestratorError::DatabaseError(e)
    }
}

#[derive(Debug)]
struct ScannerResult {
    scanner_name: String,
    findings: Option<Vec<Finding>>,
    error: Option<String>,
}

pub struct ScanOrchestrator {
    pool: PgPool,
    semaphore: Arc<Semaphore>,
    max_scanner_timeout: Duration,
}

impl ScanOrchestrator {
    /// Create a new scan orchestrator
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `max_concurrent` - Maximum number of concurrent scans (default: 5)
    pub fn new(pool: PgPool, max_concurrent: usize) -> Self {
        Self {
            pool,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_scanner_timeout: Duration::from_secs(60),
        }
    }

    /// Spawn a scan task in the background (fire-and-forget)
    ///
    /// The scan will acquire a semaphore permit, run scanners, and update the database.
    /// Errors are logged but not propagated.
    pub fn spawn_scan(&self, scan_id: Uuid, target_url: String) {
        let pool = self.pool.clone();
        let semaphore = self.semaphore.clone();
        let timeout = self.max_scanner_timeout;

        tokio::spawn(async move {
            // Acquire permit inside the task to avoid blocking the API
            let _permit = semaphore.acquire().await.expect("Semaphore closed");

            if let Err(e) = Self::execute_scan_internal(pool, scan_id, target_url, timeout).await {
                tracing::error!("Scan {} failed: {}", scan_id, e);
            }
        });
    }

    /// Execute a scan synchronously (for testing or controlled execution)
    ///
    /// This acquires a semaphore permit and runs the scan to completion.
    #[allow(dead_code)]
    pub async fn execute_scan(&self, scan_id: Uuid, target_url: String) -> Result<(), OrchestratorError> {
        let _permit = self.semaphore.acquire().await.expect("Semaphore closed");
        Self::execute_scan_internal(self.pool.clone(), scan_id, target_url, self.max_scanner_timeout).await
    }

    async fn execute_scan_internal(
        pool: PgPool,
        scan_id: Uuid,
        target_url: String,
        timeout: Duration,
    ) -> Result<(), OrchestratorError> {
        // Update scan to InProgress
        scans_db::update_scan_status(&pool, scan_id, ScanStatus::InProgress, None, None).await?;

        // Run scanners with timeout and retry
        let scanner_results = Self::run_scanners(&target_url, timeout).await;

        // Check if all scanners failed
        let all_failed = scanner_results.iter().all(|r| r.findings.is_none());

        if all_failed {
            let error_messages: Vec<String> = scanner_results
                .iter()
                .filter_map(|r| r.error.clone())
                .collect();
            let combined_error = error_messages.join("; ");

            scans_db::update_scan_status(
                &pool,
                scan_id,
                ScanStatus::Failed,
                None,
                Some(format!("All scanners failed: {}", combined_error)),
            ).await?;

            return Err(OrchestratorError::AllScannersFailed(combined_error));
        }

        // Collect successful findings
        let mut all_findings: Vec<Finding> = scanner_results
            .into_iter()
            .filter_map(|r| r.findings)
            .flatten()
            .collect();

        // Deduplicate findings by (scanner_name, title)
        all_findings = Self::deduplicate_findings(all_findings);

        // Compute score from findings
        let score = Self::compute_score(&all_findings);

        // Persist findings to database
        findings_db::insert_findings(&pool, scan_id, &all_findings).await?;

        // Update scan to Completed with score
        scans_db::update_scan_status(
            &pool,
            scan_id,
            ScanStatus::Completed,
            Some(score),
            None,
        ).await?;

        Ok(())
    }

    async fn run_scanners(target_url: &str, timeout: Duration) -> Vec<ScannerResult> {
        let mut results = Vec::new();

        // Run security_headers scanner with retry
        let headers_result = Self::run_scanner_with_retry(
            "security_headers",
            || security_headers::scan_security_headers(target_url),
            timeout,
        ).await;
        results.push(headers_result);

        // Future scanners (testssl, nuclei) will be added here in later plans

        results
    }

    async fn run_scanner_with_retry<F, Fut>(
        scanner_name: &str,
        scanner_fn: F,
        timeout: Duration,
    ) -> ScannerResult
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<Vec<Finding>, security_headers::ScannerError>>,
    {
        // First attempt
        match tokio::time::timeout(timeout, scanner_fn()).await {
            Ok(Ok(findings)) => {
                return ScannerResult {
                    scanner_name: scanner_name.to_string(),
                    findings: Some(findings),
                    error: None,
                };
            }
            Ok(Err(e)) => {
                tracing::warn!("Scanner {} failed on first attempt: {}", scanner_name, e);
                // Retry once
                match tokio::time::timeout(timeout, scanner_fn()).await {
                    Ok(Ok(findings)) => {
                        return ScannerResult {
                            scanner_name: scanner_name.to_string(),
                            findings: Some(findings),
                            error: None,
                        };
                    }
                    Ok(Err(e)) => {
                        let error_msg = format!("Failed after retry: {}", e);
                        tracing::error!("Scanner {} {}", scanner_name, error_msg);
                        return ScannerResult {
                            scanner_name: scanner_name.to_string(),
                            findings: None,
                            error: Some(error_msg),
                        };
                    }
                    Err(_) => {
                        let error_msg = "Timeout on retry";
                        tracing::error!("Scanner {} {}", scanner_name, error_msg);
                        return ScannerResult {
                            scanner_name: scanner_name.to_string(),
                            findings: None,
                            error: Some(error_msg.to_string()),
                        };
                    }
                }
            }
            Err(_) => {
                tracing::warn!("Scanner {} timed out on first attempt", scanner_name);
                // Retry once
                match tokio::time::timeout(timeout, scanner_fn()).await {
                    Ok(Ok(findings)) => {
                        return ScannerResult {
                            scanner_name: scanner_name.to_string(),
                            findings: Some(findings),
                            error: None,
                        };
                    }
                    Ok(Err(e)) => {
                        let error_msg = format!("Failed after timeout retry: {}", e);
                        tracing::error!("Scanner {} {}", scanner_name, error_msg);
                        return ScannerResult {
                            scanner_name: scanner_name.to_string(),
                            findings: None,
                            error: Some(error_msg),
                        };
                    }
                    Err(_) => {
                        let error_msg = "Timeout on both attempts";
                        tracing::error!("Scanner {} {}", scanner_name, error_msg);
                        return ScannerResult {
                            scanner_name: scanner_name.to_string(),
                            findings: None,
                            error: Some(error_msg.to_string()),
                        };
                    }
                }
            }
        }
    }

    fn deduplicate_findings(findings: Vec<Finding>) -> Vec<Finding> {
        let mut seen = std::collections::HashSet::new();
        let mut deduplicated = Vec::new();

        for finding in findings {
            let key = (finding.scanner_name.clone(), finding.title.clone());
            if seen.insert(key) {
                deduplicated.push(finding);
            }
        }

        deduplicated
    }

    fn compute_score(findings: &[Finding]) -> String {
        if findings.is_empty() {
            return "A+".to_string();
        }

        // Calculate weighted score based on severity
        let total_weight: i32 = findings.iter().map(|f| f.severity.score_weight()).sum();

        // Convert to letter grade (simple heuristic)
        // 0 = A+, 1-3 = A, 4-7 = B, 8-12 = C, 13-20 = D, 20+ = F
        match total_weight {
            0 => "A+".to_string(),
            1..=3 => "A".to_string(),
            4..=7 => "B".to_string(),
            8..=12 => "C".to_string(),
            13..=20 => "D".to_string(),
            _ => "F".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_compute_score() {
        // No findings = A+
        assert_eq!(ScanOrchestrator::compute_score(&[]), "A+");

        // 1 Low = A (weight 1)
        let findings = vec![Finding {
            id: Uuid::new_v4(),
            scan_id: Uuid::new_v4(),
            scanner_name: "test".to_string(),
            title: "Test".to_string(),
            description: "Test".to_string(),
            severity: Severity::Low,
            remediation: "Test".to_string(),
            raw_evidence: None,
            created_at: Utc::now().naive_utc(),
        }];
        assert_eq!(ScanOrchestrator::compute_score(&findings), "A");

        // 1 Critical = B (weight 10)
        let findings = vec![Finding {
            id: Uuid::new_v4(),
            scan_id: Uuid::new_v4(),
            scanner_name: "test".to_string(),
            title: "Test".to_string(),
            description: "Test".to_string(),
            severity: Severity::Critical,
            remediation: "Test".to_string(),
            raw_evidence: None,
            created_at: Utc::now().naive_utc(),
        }];
        assert_eq!(ScanOrchestrator::compute_score(&findings), "C");
    }

    #[test]
    fn test_deduplicate_findings() {
        let findings = vec![
            Finding {
                id: Uuid::new_v4(),
                scan_id: Uuid::new_v4(),
                scanner_name: "scanner1".to_string(),
                title: "Finding A".to_string(),
                description: "Desc".to_string(),
                severity: Severity::High,
                remediation: "Fix".to_string(),
                raw_evidence: None,
                created_at: Utc::now().naive_utc(),
            },
            Finding {
                id: Uuid::new_v4(),
                scan_id: Uuid::new_v4(),
                scanner_name: "scanner1".to_string(),
                title: "Finding A".to_string(),
                description: "Different desc".to_string(),
                severity: Severity::High,
                remediation: "Fix".to_string(),
                raw_evidence: None,
                created_at: Utc::now().naive_utc(),
            },
            Finding {
                id: Uuid::new_v4(),
                scan_id: Uuid::new_v4(),
                scanner_name: "scanner2".to_string(),
                title: "Finding B".to_string(),
                description: "Desc".to_string(),
                severity: Severity::Medium,
                remediation: "Fix".to_string(),
                raw_evidence: None,
                created_at: Utc::now().naive_utc(),
            },
        ];

        let deduplicated = ScanOrchestrator::deduplicate_findings(findings);
        assert_eq!(deduplicated.len(), 2); // Should remove duplicate (scanner1, Finding A)
    }
}
