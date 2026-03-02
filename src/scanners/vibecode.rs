use crate::models::finding::{Finding, Severity};
use crate::scanners::container::ScannerError;
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;
use uuid::Uuid;

/// Scan for vibe-code specific vulnerabilities using custom Nuclei templates
///
/// This scanner is ShipSecure's differentiator - it runs framework-aware templates
/// that catch vulnerabilities specific to AI-generated applications (vibe-coded apps).
///
/// # Arguments
/// * `target_url` - The URL to scan
/// * `framework` - Optional framework detected (e.g., "nextjs", "react")
/// * `platform` - Optional deployment platform detected (e.g., "vercel", "netlify")
/// * `tier` - Scan tier ("free" or "paid") - paid tier includes additional templates
///
/// # Returns
/// Vec of findings with `vibe_code: true` set on all results
pub async fn scan_vibecode(
    target_url: &str,
    framework: Option<&str>,
    platform: Option<&str>,
    tier: &str,
) -> Result<Vec<Finding>, ScannerError> {
    // Resolve Nuclei binary
    let nuclei_binary = match crate::scanners::container::resolve_nuclei_binary() {
        Some(path) => path,
        None => {
            tracing::warn!("Nuclei binary not found, skipping vibe-code scan");
            return Ok(Vec::new());
        }
    };

    // Get templates directory
    let templates_dir = get_templates_dir();
    if !templates_dir.exists() {
        tracing::error!("Templates directory not found: {}", templates_dir.display());
        return Err(ScannerError::ExecutionError(
            "Templates directory not found".to_string(),
        ));
    }

    // Select which templates to run based on framework/platform
    let template_paths = select_templates(&templates_dir, framework, platform, tier);

    tracing::info!(
        "Running vibe-code scan with {} templates (framework={:?}, platform={:?}, tier={})",
        template_paths.len(),
        framework,
        platform,
        tier
    );

    // Create temp file for JSON output
    let temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| ScannerError::ExecutionError(format!("Failed to create temp file: {}", e)))?;
    let temp_path = temp_file.path();

    // Build args for native Nuclei binary
    let mut args = vec!["-u".to_string(), target_url.to_string()];

    // Add template paths
    if !template_paths.is_empty() {
        args.push("-t".to_string());
        args.push(template_paths.join(","));
    }

    args.extend_from_slice(&[
        "-jsonl".to_string(),
        "-silent".to_string(),
        "-severity".to_string(),
        "critical,high,medium,low".to_string(),
        "-timeout".to_string(),
        "30".to_string(),
        "-o".to_string(),
        temp_path.to_str().unwrap().to_string(),
    ]);

    // Execute binary
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
                tracing::warn!(
                    "Nuclei exited with code {}: {}",
                    output.status.code().unwrap_or(-1),
                    stderr
                );
            }
        }
        Ok(Err(e)) => {
            return Err(ScannerError::ExecutionError(format!(
                "Failed to wait for Nuclei: {}",
                e
            )));
        }
        Err(_) => {
            tracing::warn!("Vibe-code scan timed out after 120s");
            return Err(ScannerError::ScanTimeout);
        }
    }

    // Read JSON from temp file
    let output = std::fs::read_to_string(temp_path)
        .map_err(|e| ScannerError::ParseError(format!("Failed to read output file: {}", e)))?;

    // Parse output and tag all findings as vibe_code
    parse_vibecode_output(&output, target_url)
}

/// Get the templates directory path
fn get_templates_dir() -> PathBuf {
    // Try environment variable first
    if let Ok(dir) = std::env::var("SHIPSECURE_TEMPLATES_DIR") {
        return PathBuf::from(dir);
    }

    // Default to templates/ in current working directory
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("templates/nuclei")
}

/// Select which templates to run based on detected framework and platform
fn select_templates(
    templates_dir: &Path,
    framework: Option<&str>,
    platform: Option<&str>,
    tier: &str,
) -> Vec<String> {
    let mut templates = Vec::new();

    // Universal checks (always run)
    templates.push(
        templates_dir
            .join("supabase-rls.yaml")
            .to_string_lossy()
            .to_string(),
    );
    templates.push(
        templates_dir
            .join("firebase-rules.yaml")
            .to_string_lossy()
            .to_string(),
    );
    templates.push(
        templates_dir
            .join("env-in-build-output.yaml")
            .to_string_lossy()
            .to_string(),
    );

    // Framework-specific checks
    match framework {
        Some("nextjs") | Some("next") => {
            templates.push(
                templates_dir
                    .join("nextjs-env-leak.yaml")
                    .to_string_lossy()
                    .to_string(),
            );
            templates.push(
                templates_dir
                    .join("unprotected-api-routes.yaml")
                    .to_string_lossy()
                    .to_string(),
            );
        }
        None => {
            // Unknown framework - run everything to be safe
            templates.push(
                templates_dir
                    .join("nextjs-env-leak.yaml")
                    .to_string_lossy()
                    .to_string(),
            );
            templates.push(
                templates_dir
                    .join("unprotected-api-routes.yaml")
                    .to_string_lossy()
                    .to_string(),
            );
            templates.push(
                templates_dir
                    .join("netlify-function-exposure.yaml")
                    .to_string_lossy()
                    .to_string(),
            );
            templates.push(
                templates_dir
                    .join("vercel-env-leak.yaml")
                    .to_string_lossy()
                    .to_string(),
            );
        }
        _ => {
            // Known framework but not Next.js - still run API route checks
            templates.push(
                templates_dir
                    .join("unprotected-api-routes.yaml")
                    .to_string_lossy()
                    .to_string(),
            );
        }
    }

    // Platform-specific checks
    match platform {
        Some("vercel") => {
            templates.push(
                templates_dir
                    .join("vercel-env-leak.yaml")
                    .to_string_lossy()
                    .to_string(),
            );
        }
        Some("netlify") => {
            templates.push(
                templates_dir
                    .join("netlify-function-exposure.yaml")
                    .to_string_lossy()
                    .to_string(),
            );
        }
        None => {
            // Unknown platform already handled above in framework=None case
        }
        _ => {}
    }

    // Add paid-tier templates for deeper scanning
    if tier == "paid" {
        templates.push(
            templates_dir
                .join("paid/advanced-env-leak.yaml")
                .to_string_lossy()
                .to_string(),
        );
        templates.push(
            templates_dir
                .join("paid/api-auth-bypass.yaml")
                .to_string_lossy()
                .to_string(),
        );
        templates.push(
            templates_dir
                .join("paid/debug-endpoints.yaml")
                .to_string_lossy()
                .to_string(),
        );
        templates.push(
            templates_dir
                .join("paid/database-exposure.yaml")
                .to_string_lossy()
                .to_string(),
        );
        templates.push(
            templates_dir
                .join("paid/admin-panel-detection.yaml")
                .to_string_lossy()
                .to_string(),
        );
    }

    // Remove duplicates
    templates.sort();
    templates.dedup();
    templates
}

/// Parse Nuclei JSONL output and tag all findings as vibe_code
fn parse_vibecode_output(output: &str, target: &str) -> Result<Vec<Finding>, ScannerError> {
    let mut findings = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(json) => {
                if let Some(mut finding) = parse_nuclei_finding(&json, target) {
                    // Tag as vibe_code - this is the key differentiator
                    finding.vibe_code = true;
                    finding.scanner_name = "vibecode".to_string();

                    // Apply whitelist filtering for false positives
                    if should_filter_finding(&finding) {
                        tracing::debug!(
                            "Filtered false positive: {} (safe publishable key)",
                            finding.title
                        );
                        continue;
                    }

                    findings.push(finding);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to parse Nuclei JSON line: {}", e);
                continue;
            }
        }
    }

    tracing::info!("Vibe-code scan found {} findings", findings.len());
    Ok(findings)
}

/// Parse a single Nuclei finding from JSON
fn parse_nuclei_finding(json: &serde_json::Value, target: &str) -> Option<Finding> {
    let info = json.get("info")?;
    let template_id = json.get("template-id")?.as_str()?;
    let name = info.get("name")?.as_str()?;
    let description = info
        .get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("");

    let severity = info
        .get("severity")
        .and_then(|s| s.as_str())
        .unwrap_or("medium");

    let matched_at = json
        .get("matched-at")
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

    let remediation = info.get("remediation").and_then(|r| r.as_str()).unwrap_or(
        "Review the finding and apply security patches or configuration changes as needed.",
    );

    Some(Finding {
        id: Uuid::new_v4(),
        scan_id: Uuid::nil(), // Placeholder, will be set by caller
        scanner_name: "vibecode".to_string(), // Will be set correctly by caller
        severity: mapped_severity,
        title: name.to_string(),
        description: if description.is_empty() {
            format!("Vibe-code vulnerability detected: {}", name)
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
        vibe_code: false, // Will be set to true by caller
        created_at: Utc::now().naive_utc(),
    })
}

/// Filter out false positives - safe publishable environment variables
fn should_filter_finding(finding: &Finding) -> bool {
    // Whitelist safe NEXT_PUBLIC_ variables that are meant to be public
    let safe_patterns = ["NEXT_PUBLIC_SUPABASE_URL", "NEXT_PUBLIC_SUPABASE_ANON_KEY"];

    let content = format!(
        "{} {} {}",
        finding.title,
        finding.description,
        finding.raw_evidence.as_deref().unwrap_or("")
    );

    for pattern in &safe_patterns {
        if content.contains(pattern) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_selection_nextjs() {
        let templates_dir = PathBuf::from("/templates");
        let templates = select_templates(&templates_dir, Some("nextjs"), None, "free");

        // Should include universal + nextjs-specific
        assert!(templates.contains(&"/templates/supabase-rls.yaml".to_string()));
        assert!(templates.contains(&"/templates/nextjs-env-leak.yaml".to_string()));
        assert!(templates.contains(&"/templates/unprotected-api-routes.yaml".to_string()));
    }

    #[test]
    fn test_template_selection_none() {
        let templates_dir = PathBuf::from("/templates");
        let templates = select_templates(&templates_dir, None, None, "free");

        // Should include everything when unknown
        assert!(templates.len() >= 7);
        assert!(templates.contains(&"/templates/supabase-rls.yaml".to_string()));
        assert!(templates.contains(&"/templates/nextjs-env-leak.yaml".to_string()));
    }

    #[test]
    fn test_template_selection_vercel() {
        let templates_dir = PathBuf::from("/templates");
        let templates = select_templates(&templates_dir, Some("nextjs"), Some("vercel"), "free");

        // Should include vercel-specific
        assert!(templates.contains(&"/templates/vercel-env-leak.yaml".to_string()));
    }

    #[test]
    fn test_template_selection_netlify() {
        let templates_dir = PathBuf::from("/templates");
        let templates = select_templates(&templates_dir, None, Some("netlify"), "free");

        // Should include netlify-specific
        assert!(templates.contains(&"/templates/netlify-function-exposure.yaml".to_string()));
    }

    #[test]
    fn test_template_selection_paid_tier() {
        let templates_dir = PathBuf::from("/templates");
        let templates = select_templates(&templates_dir, Some("nextjs"), None, "paid");

        // Should include base templates + paid templates
        assert!(templates.contains(&"/templates/supabase-rls.yaml".to_string()));
        assert!(templates.contains(&"/templates/paid/advanced-env-leak.yaml".to_string()));
        assert!(templates.contains(&"/templates/paid/api-auth-bypass.yaml".to_string()));
        assert!(templates.contains(&"/templates/paid/debug-endpoints.yaml".to_string()));
        assert!(templates.contains(&"/templates/paid/database-exposure.yaml".to_string()));
        assert!(templates.contains(&"/templates/paid/admin-panel-detection.yaml".to_string()));
    }

    #[test]
    fn test_finding_parsing() {
        let json = serde_json::json!({
            "template-id": "supabase-rls",
            "info": {
                "name": "Supabase RLS Misconfiguration",
                "severity": "critical",
                "description": "RLS is not enabled"
            },
            "matched-at": "https://example.com"
        });

        let finding = parse_nuclei_finding(&json, "https://example.com").unwrap();
        assert_eq!(finding.scanner_name, "vibecode");
        assert_eq!(finding.severity, Severity::Critical);
        assert_eq!(finding.title, "Supabase RLS Misconfiguration");
        assert!(!finding.vibe_code); // Will be set to true by caller
    }

    #[test]
    fn test_whitelist_filtering_safe_keys() {
        let finding = Finding {
            id: Uuid::new_v4(),
            scan_id: Uuid::nil(),
            scanner_name: "vibecode".to_string(),
            title: "Environment leak".to_string(),
            description: "Found NEXT_PUBLIC_SUPABASE_URL in bundle".to_string(),
            severity: Severity::High,
            remediation: "".to_string(),
            raw_evidence: None,
            vibe_code: true,
            created_at: Utc::now().naive_utc(),
        };

        assert!(should_filter_finding(&finding));
    }

    #[test]
    fn test_whitelist_filtering_actual_secret() {
        let finding = Finding {
            id: Uuid::new_v4(),
            scan_id: Uuid::nil(),
            scanner_name: "vibecode".to_string(),
            title: "Secret leak".to_string(),
            description: "Found NEXT_PUBLIC_SECRET_KEY in bundle".to_string(),
            severity: Severity::High,
            remediation: "".to_string(),
            raw_evidence: None,
            vibe_code: true,
            created_at: Utc::now().naive_utc(),
        };

        assert!(!should_filter_finding(&finding));
    }

    #[test]
    fn test_templates_dir_from_env() {
        unsafe {
            std::env::set_var("SHIPSECURE_TEMPLATES_DIR", "/custom/path");
        }
        let dir = get_templates_dir();
        assert_eq!(dir, PathBuf::from("/custom/path"));
        unsafe {
            std::env::remove_var("SHIPSECURE_TEMPLATES_DIR");
        }
    }
}
