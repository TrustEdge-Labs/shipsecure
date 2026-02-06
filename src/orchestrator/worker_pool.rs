use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use uuid::Uuid;
use base64::Engine;

use crate::db::{findings as findings_db, scans as scans_db};
use crate::models::finding::Finding;
use crate::models::scan::ScanStatus;
use crate::scanners::{security_headers, tls, exposed_files, js_secrets, detector, vibecode, remediation};

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

        // Stage 1: Framework/Platform Detection (runs first, feeds downstream)
        let detection = match detector::detect_stack(&target_url).await {
            Ok(result) => {
                // Store detection results in database
                if let Some(ref fw) = result.framework {
                    scans_db::update_detected_framework(&pool, scan_id, fw.to_db()).await.ok();
                }
                if let Some(ref pl) = result.platform {
                    scans_db::update_detected_platform(&pool, scan_id, pl.to_db()).await.ok();
                }
                scans_db::update_scan_stage(&pool, scan_id, "detection", true).await.ok();
                Some(result)
            }
            Err(e) => {
                tracing::warn!("Framework detection failed for scan {}: {}", scan_id, e);
                scans_db::update_scan_stage(&pool, scan_id, "detection", true).await.ok();
                None // Detection failure does NOT fail the scan
            }
        };

        // Extract framework/platform strings for downstream use
        let framework_str = detection.as_ref()
            .and_then(|d| d.framework.as_ref())
            .map(|f| f.to_db().to_string());
        let platform_str = detection.as_ref()
            .and_then(|d| d.platform.as_ref())
            .map(|p| p.to_db().to_string());

        // Run scanners with timeout and retry
        let scanner_results = Self::run_scanners(&pool, scan_id, &target_url, timeout, framework_str, platform_str).await;

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
            Some(score.clone()),
            None,
        ).await?;

        // Generate results token
        let token = Self::generate_results_token();
        let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::days(3);
        scans_db::set_results_token(&pool, scan_id, &token, expires_at).await?;

        // Send email notification (don't fail scan if email fails)
        if let Err(e) = Self::send_completion_email(&pool, scan_id, &target_url, &score, &all_findings, &token).await {
            tracing::warn!("Failed to send completion email for scan {}: {}", scan_id, e);
        }

        Ok(())
    }

    fn generate_results_token() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.r#gen();
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
    }

    async fn send_completion_email(
        pool: &PgPool,
        scan_id: Uuid,
        target_url: &str,
        grade: &str,
        findings: &[Finding],
        results_token: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get scan record to retrieve email
        let scan = scans_db::get_scan(pool, scan_id).await?
            .ok_or("Scan not found")?;

        // Compute findings summary
        let summary = crate::email::FindingsSummary {
            critical: findings.iter().filter(|f| f.severity == crate::models::Severity::Critical).count() as i64,
            high: findings.iter().filter(|f| f.severity == crate::models::Severity::High).count() as i64,
            medium: findings.iter().filter(|f| f.severity == crate::models::Severity::Medium).count() as i64,
            low: findings.iter().filter(|f| f.severity == crate::models::Severity::Low).count() as i64,
            total: findings.len() as i64,
        };

        // Get base URL from environment
        let base_url = std::env::var("TRUSTEDGE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:3001".to_string());

        // Send email
        crate::email::send_scan_complete_email(
            &scan.email,
            target_url,
            grade,
            &summary,
            results_token,
            &base_url,
        ).await?;

        Ok(())
    }

    async fn run_scanners(
        pool: &PgPool,
        scan_id: Uuid,
        target_url: &str,
        _timeout: Duration,
        framework: Option<String>,
        platform: Option<String>,
    ) -> Vec<ScannerResult> {
        // Spawn each scanner independently so stage updates happen as each completes
        let pool_clone1 = pool.clone();
        let pool_clone2 = pool.clone();
        let pool_clone3 = pool.clone();
        let pool_clone4 = pool.clone();
        let pool_clone5 = pool.clone();
        let url1 = target_url.to_string();
        let url2 = target_url.to_string();
        let url3 = target_url.to_string();
        let url4 = target_url.to_string();
        let url5 = target_url.to_string();
        let framework_str_clone = framework.clone();
        let platform_str_clone = platform.clone();

        let headers_handle = tokio::spawn(async move {
            let result = tokio::time::timeout(
                Duration::from_secs(60),
                security_headers::scan_security_headers(&url1)
            ).await;

            // Update stage on completion (success or failure)
            let _ = scans_db::update_scan_stage(&pool_clone1, scan_id, "headers", true).await;

            match result {
                Ok(Ok(findings)) => ScannerResult {
                    scanner_name: "security_headers".to_string(),
                    findings: Some(findings),
                    error: None,
                },
                Ok(Err(e)) => ScannerResult {
                    scanner_name: "security_headers".to_string(),
                    findings: None,
                    error: Some(e.to_string()),
                },
                Err(_) => ScannerResult {
                    scanner_name: "security_headers".to_string(),
                    findings: None,
                    error: Some("Timeout".to_string()),
                },
            }
        });

        let tls_handle = tokio::spawn(async move {
            let result = tokio::time::timeout(
                Duration::from_secs(300), // SSL Labs can be slow
                tls::scan_tls(&url2)
            ).await;

            // Update stage on completion
            let _ = scans_db::update_scan_stage(&pool_clone2, scan_id, "tls", true).await;

            match result {
                Ok(Ok(findings)) => ScannerResult {
                    scanner_name: "tls".to_string(),
                    findings: Some(findings),
                    error: None,
                },
                Ok(Err(e)) => ScannerResult {
                    scanner_name: "tls".to_string(),
                    findings: None,
                    error: Some(e.to_string()),
                },
                Err(_) => ScannerResult {
                    scanner_name: "tls".to_string(),
                    findings: None,
                    error: Some("Timeout".to_string()),
                },
            }
        });

        let files_handle = tokio::spawn(async move {
            let result = tokio::time::timeout(
                Duration::from_secs(60),
                exposed_files::scan_exposed_files(&url3)
            ).await;

            // Update stage on completion
            let _ = scans_db::update_scan_stage(&pool_clone3, scan_id, "files", true).await;

            match result {
                Ok(Ok(findings)) => ScannerResult {
                    scanner_name: "exposed_files".to_string(),
                    findings: Some(findings),
                    error: None,
                },
                Ok(Err(e)) => ScannerResult {
                    scanner_name: "exposed_files".to_string(),
                    findings: None,
                    error: Some(e.to_string()),
                },
                Err(_) => ScannerResult {
                    scanner_name: "exposed_files".to_string(),
                    findings: None,
                    error: Some("Timeout".to_string()),
                },
            }
        });

        let secrets_handle = tokio::spawn(async move {
            let result = tokio::time::timeout(
                Duration::from_secs(60),
                js_secrets::scan_js_secrets(&url4)
            ).await;

            // Update stage on completion
            let _ = scans_db::update_scan_stage(&pool_clone4, scan_id, "secrets", true).await;

            match result {
                Ok(Ok(findings)) => ScannerResult {
                    scanner_name: "js_secrets".to_string(),
                    findings: Some(findings),
                    error: None,
                },
                Ok(Err(e)) => ScannerResult {
                    scanner_name: "js_secrets".to_string(),
                    findings: None,
                    error: Some(e.to_string()),
                },
                Err(_) => ScannerResult {
                    scanner_name: "js_secrets".to_string(),
                    findings: None,
                    error: Some("Timeout".to_string()),
                },
            }
        });

        let vibecode_handle = tokio::spawn(async move {
            let fw_ref = framework_str_clone.as_deref();
            let pl_ref = platform_str_clone.as_deref();

            let result = tokio::time::timeout(
                Duration::from_secs(180), // Nuclei can be slow
                vibecode::scan_vibecode(&url5, fw_ref, pl_ref)
            ).await;

            let _ = scans_db::update_scan_stage(&pool_clone5, scan_id, "vibecode", true).await;

            match result {
                Ok(Ok(mut findings)) => {
                    // Apply framework-specific remediation to vibe-code findings
                    for finding in &mut findings {
                        finding.remediation = remediation::generate_remediation(
                            finding.raw_evidence.as_deref().unwrap_or(""),
                            &finding.title,
                            fw_ref,
                            finding.raw_evidence.as_deref(),
                        );
                    }
                    ScannerResult {
                        scanner_name: "vibecode".to_string(),
                        findings: Some(findings),
                        error: None,
                    }
                }
                Ok(Err(e)) => ScannerResult {
                    scanner_name: "vibecode".to_string(),
                    findings: None,
                    error: Some(e.to_string()),
                },
                Err(_) => ScannerResult {
                    scanner_name: "vibecode".to_string(),
                    findings: None,
                    error: Some("Timeout".to_string()),
                },
            }
        });

        // Await all scanner tasks
        let results = tokio::join!(headers_handle, tls_handle, files_handle, secrets_handle, vibecode_handle);

        vec![
            results.0.unwrap_or_else(|_| ScannerResult {
                scanner_name: "security_headers".to_string(),
                findings: None,
                error: Some("Task panicked".to_string()),
            }),
            results.1.unwrap_or_else(|_| ScannerResult {
                scanner_name: "tls".to_string(),
                findings: None,
                error: Some("Task panicked".to_string()),
            }),
            results.2.unwrap_or_else(|_| ScannerResult {
                scanner_name: "exposed_files".to_string(),
                findings: None,
                error: Some("Task panicked".to_string()),
            }),
            results.3.unwrap_or_else(|_| ScannerResult {
                scanner_name: "js_secrets".to_string(),
                findings: None,
                error: Some("Task panicked".to_string()),
            }),
            results.4.unwrap_or_else(|_| ScannerResult {
                scanner_name: "vibecode".to_string(),
                findings: None,
                error: Some("Task panicked".to_string()),
            }),
        ]
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
    use crate::models::Severity;
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
            vibe_code: false,
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
            vibe_code: false,
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
                vibe_code: false,
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
                vibe_code: false,
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
                vibe_code: false,
                created_at: Utc::now().naive_utc(),
            },
        ];

        let deduplicated = ScanOrchestrator::deduplicate_findings(findings);
        assert_eq!(deduplicated.len(), 2); // Should remove duplicate (scanner1, Finding A)
    }
}
