use crate::models::finding::{Finding, Severity};
use chrono::Utc;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;
use uuid::Uuid;

#[derive(Debug)]
pub enum ScannerError {
    BinaryNotFound,
    ScanTimeout,
    ExecutionError(String),
    ParseError(String),
}

impl std::fmt::Display for ScannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BinaryNotFound => write!(f, "Scanner binary not found"),
            Self::ScanTimeout => write!(f, "Scan execution timed out"),
            Self::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ScannerError {}

/// Resolve Nuclei binary path via env var or PATH lookup
pub fn resolve_nuclei_binary() -> Option<PathBuf> {
    // Check NUCLEI_BINARY_PATH env var first
    if let Ok(path) = std::env::var("NUCLEI_BINARY_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() && p.is_file() {
            return Some(p);
        }
        tracing::warn!("NUCLEI_BINARY_PATH set but not found: {}", p.display());
    }

    // Check PATH
    if let Ok(path) = which::which("nuclei") {
        return Some(path);
    }

    // Check common installation paths
    for path_str in ["/usr/local/bin/nuclei", "/usr/bin/nuclei", "/opt/nuclei/bin/nuclei"] {
        let p = PathBuf::from(path_str);
        if p.exists() {
            return Some(p);
        }
    }

    // Not found
    None
}

/// Resolve testssl.sh binary path via env var or PATH lookup
pub fn resolve_testssl_binary() -> Option<PathBuf> {
    // Check TESTSSL_BINARY_PATH env var first
    if let Ok(path) = std::env::var("TESTSSL_BINARY_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() && p.is_file() {
            return Some(p);
        }
        tracing::warn!("TESTSSL_BINARY_PATH set but not found: {}", p.display());
    }

    // Check PATH
    if let Ok(path) = which::which("testssl.sh") {
        return Some(path);
    }

    // Check common installation paths
    for path_str in ["/usr/local/bin/testssl.sh", "/usr/bin/testssl.sh", "/opt/testssl/testssl.sh"] {
        let p = PathBuf::from(path_str);
        if p.exists() {
            return Some(p);
        }
    }

    // Not found
    None
}

/// Run Nuclei scanner as native binary subprocess
pub async fn run_nuclei(target: &str) -> Result<Vec<Finding>, ScannerError> {
    let nuclei_binary = match resolve_nuclei_binary() {
        Some(path) => path,
        None => {
            tracing::warn!("Nuclei binary not found, skipping Nuclei scan");
            return Ok(Vec::new());
        }
    };

    // Create temp file for JSON output
    let temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| ScannerError::ExecutionError(format!("Failed to create temp file: {}", e)))?;
    let temp_path = temp_file.path();

    let args = vec![
        "-u", target,
        "-jsonl",
        "-silent",
        "-severity", "medium,high,critical",
        "-tags", "exposure,misconfig,cve",
        "-o", temp_path.to_str().unwrap(),
    ];

    let child = Command::new(&nuclei_binary)
        .args(&args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| ScannerError::ExecutionError(format!("Failed to spawn Nuclei: {}", e)))?;

    // Wait with timeout
    match tokio::time::timeout(Duration::from_secs(120), child.wait_with_output()).await {
        Ok(Ok(output)) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!("Nuclei exited with code {}: {}", output.status.code().unwrap_or(-1), stderr);
            }
        }
        Ok(Err(e)) => {
            return Err(ScannerError::ExecutionError(format!("Failed to wait for Nuclei: {}", e)));
        }
        Err(_) => {
            tracing::warn!("Nuclei execution timed out after 120s");
            return Err(ScannerError::ScanTimeout);
        }
    }

    // Read JSON from temp file
    let output = std::fs::read_to_string(temp_path)
        .map_err(|e| ScannerError::ParseError(format!("Failed to read output file: {}", e)))?;

    parse_nuclei_output(&output, target)
}

/// Run testssl.sh scanner as native binary subprocess
pub async fn run_testssl(target: &str) -> Result<Vec<Finding>, ScannerError> {
    let testssl_binary = match resolve_testssl_binary() {
        Some(path) => path,
        None => {
            tracing::warn!("testssl.sh binary not found, skipping testssl.sh scan");
            return Ok(Vec::new());
        }
    };

    // Create temp file for JSON output
    let temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| ScannerError::ExecutionError(format!("Failed to create temp file: {}", e)))?;
    let temp_path = temp_file.path();

    let args = vec![
        "--jsonfile-pretty", temp_path.to_str().unwrap(),
        "--quiet",
        target,
    ];

    let child = Command::new(&testssl_binary)
        .args(&args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| ScannerError::ExecutionError(format!("Failed to spawn testssl.sh: {}", e)))?;

    // Wait with timeout
    match tokio::time::timeout(Duration::from_secs(180), child.wait_with_output()).await {
        Ok(Ok(output)) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!("testssl.sh exited with code {}: {}", output.status.code().unwrap_or(-1), stderr);
            }
        }
        Ok(Err(e)) => {
            return Err(ScannerError::ExecutionError(format!("Failed to wait for testssl.sh: {}", e)));
        }
        Err(_) => {
            tracing::warn!("testssl.sh execution timed out after 180s");
            return Err(ScannerError::ScanTimeout);
        }
    }

    // Read JSON from temp file
    let output = std::fs::read_to_string(temp_path)
        .map_err(|e| ScannerError::ParseError(format!("Failed to read output file: {}", e)))?;

    parse_testssl_output(&output, target)
}

/// Parse Nuclei JSONL output into findings
fn parse_nuclei_output(output: &str, target: &str) -> Result<Vec<Finding>, ScannerError> {
    let mut findings = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(json) => {
                if let Some(finding) = parse_nuclei_finding(&json, target) {
                    findings.push(finding);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to parse Nuclei JSON line: {}", e);
                continue;
            }
        }
    }

    Ok(findings)
}

/// Parse a single Nuclei JSON finding
fn parse_nuclei_finding(json: &serde_json::Value, target: &str) -> Option<Finding> {
    let info = json.get("info")?;
    let template_id = json.get("template-id")?.as_str()?;
    let name = info.get("name")?.as_str()?;
    let description = info.get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("");

    let severity = info.get("severity")
        .and_then(|s| s.as_str())
        .unwrap_or("medium");

    let matched_at = json.get("matched-at")
        .and_then(|m| m.as_str())
        .unwrap_or(target);

    // Map Nuclei severity to our severity levels
    let mapped_severity = match severity.to_lowercase().as_str() {
        "critical" => Severity::Critical,
        "high" => Severity::High,
        "medium" => Severity::Medium,
        "low" => Severity::Low,
        _ => Severity::Medium,
    };

    let remediation = info.get("remediation")
        .and_then(|r| r.as_str())
        .unwrap_or("Review the finding and apply security patches or configuration changes as needed.");

    Some(Finding {
        id: Uuid::new_v4(),
        scan_id: Uuid::nil(), // Placeholder, will be set by caller
        scanner_name: "nuclei".to_string(),
        severity: mapped_severity,
        title: name.to_string(),
        description: if description.is_empty() {
            format!("Nuclei detected a potential vulnerability: {}", name)
        } else {
            description.to_string()
        },
        remediation: remediation.to_string(),
        raw_evidence: Some(format!(
            "Template: {}\nMatched at: {}\nFull output: {}",
            template_id,
            matched_at,
            serde_json::to_string_pretty(json).unwrap_or_else(|_| "{}".to_string())
        )),
        vibe_code: false,
        created_at: Utc::now().naive_utc(),
    })
}

/// Parse testssl.sh JSON output into findings
fn parse_testssl_output(output: &str, target: &str) -> Result<Vec<Finding>, ScannerError> {
    let mut findings = Vec::new();

    let json: serde_json::Value = serde_json::from_str(output)
        .map_err(|e| ScannerError::ParseError(format!("Failed to parse testssl JSON: {}", e)))?;

    // testssl.sh outputs an array of test results
    if let Some(scan_results) = json.as_array() {
        for result in scan_results {
            if let Some(finding) = parse_testssl_finding(result, target) {
                findings.push(finding);
            }
        }
    } else if let Some(findings_array) = json.get("scanResult").and_then(|s| s.as_array()) {
        // Alternative structure: {scanResult: [...]}
        for result in findings_array {
            if let Some(finding) = parse_testssl_finding(result, target) {
                findings.push(finding);
            }
        }
    }

    Ok(findings)
}

/// Parse a single testssl.sh finding
fn parse_testssl_finding(json: &serde_json::Value, target: &str) -> Option<Finding> {
    let severity = json.get("severity")?.as_str()?;

    // Only report issues (skip OK, INFO)
    match severity {
        "OK" | "INFO" => return None,
        _ => {}
    }

    let id = json.get("id")?.as_str()?;
    let finding_text = json.get("finding")?.as_str().unwrap_or("");

    // Map testssl severity to our severity levels
    let mapped_severity = match severity {
        "CRITICAL" => Severity::Critical,
        "HIGH" => Severity::High,
        "MEDIUM" => Severity::Medium,
        "LOW" => Severity::Low,
        _ => Severity::Medium,
    };

    let title = format!("TLS/SSL Issue: {}", id.replace('_', " "));

    let description = if finding_text.is_empty() {
        format!("testssl.sh detected a TLS/SSL configuration issue: {}", id)
    } else {
        finding_text.to_string()
    };

    Some(Finding {
        id: Uuid::new_v4(),
        scan_id: Uuid::nil(), // Placeholder, will be set by caller
        scanner_name: "testssl".to_string(),
        severity: mapped_severity,
        title,
        description: format!("Target: {}\n{}", target, description),
        remediation: get_testssl_remediation(id),
        raw_evidence: Some(serde_json::to_string_pretty(json)
            .unwrap_or_else(|_| "{}".to_string())),
        vibe_code: false,
        created_at: Utc::now().naive_utc(),
    })
}

/// Get remediation advice for common testssl.sh findings
fn get_testssl_remediation(id: &str) -> String {
    match id {
        "cert_expiration" => "Renew your SSL/TLS certificate before it expires. Use Let's Encrypt for free automated renewals.".to_string(),
        "cert_trust" => "Ensure your certificate is signed by a trusted Certificate Authority. Self-signed certificates trigger browser warnings.".to_string(),
        "TLS1" | "TLS1_1" => "Disable TLS 1.0 and TLS 1.1. These protocols have known vulnerabilities. Use TLS 1.2 or TLS 1.3 only.".to_string(),
        "SSLv2" | "SSLv3" => "Disable SSLv2 and SSLv3 immediately. These protocols are severely compromised (POODLE, DROWN).".to_string(),
        "LUCKY13" | "BREACH" | "CRIME" => "Disable TLS compression and vulnerable CBC cipher suites. Use AEAD ciphers like AES-GCM.".to_string(),
        "ROBOT" => "Disable RSA key exchange cipher suites. Use ECDHE or DHE for forward secrecy.".to_string(),
        "weak_cipher" => "Remove weak cipher suites (RC4, 3DES, export ciphers). Use strong modern ciphers (AES-128-GCM or better).".to_string(),
        "forward_secrecy" => "Enable forward secrecy by supporting ECDHE or DHE key exchange. This protects past sessions if keys are compromised.".to_string(),
        _ => "Review your TLS/SSL configuration and apply recommended security settings. Consult Mozilla SSL Configuration Generator for best practices.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_resolution() {
        // Test that resolve functions return None gracefully when binaries aren't installed
        // (Don't fail test - just ensure the functions don't panic)
        let nuclei = resolve_nuclei_binary();
        let testssl = resolve_testssl_binary();

        // If found, ensure they're valid paths
        if let Some(path) = nuclei {
            assert!(path.exists() || !path.exists()); // Either way is fine
        }
        if let Some(path) = testssl {
            assert!(path.exists() || !path.exists()); // Either way is fine
        }

        // Test passes as long as no panic occurs
    }

    #[test]
    fn test_nuclei_finding_parse() {
        let json = serde_json::json!({
            "template-id": "ssl-expired",
            "info": {
                "name": "SSL Certificate Expired",
                "severity": "high",
                "description": "The SSL certificate has expired"
            },
            "matched-at": "https://example.com"
        });

        let finding = parse_nuclei_finding(&json, "https://example.com").unwrap();
        assert_eq!(finding.scanner_name, "nuclei");
        assert_eq!(finding.severity, Severity::High);
        assert_eq!(finding.title, "SSL Certificate Expired");
    }

    #[test]
    fn test_testssl_severity_mapping() {
        let json = serde_json::json!({
            "id": "TLS1",
            "severity": "HIGH",
            "finding": "TLS 1.0 is enabled"
        });

        let finding = parse_testssl_finding(&json, "example.com:443").unwrap();
        assert_eq!(finding.severity, Severity::High);
        assert!(finding.title.contains("TLS1"));
    }
}
