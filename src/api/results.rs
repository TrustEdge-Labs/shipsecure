use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use serde_json::json;

use crate::api::errors::ApiError;
use crate::api::scans::AppState;
use crate::models::Severity;
use crate::{db, models};

/// GET /api/v1/results/:token - Get scan results by token
pub async fn get_results_by_token(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // 1. Query scan by token (checks expiry automatically)
    let scan = db::scans::get_scan_by_token(&state.pool, &token)
        .await?
        .ok_or_else(|| {
            ApiError::Custom {
                status: StatusCode::NOT_FOUND,
                error_type: "https://shipsecure.ai/errors/results-not-found".to_string(),
                title: "Results Not Found".to_string(),
                detail: "Results not found or link has expired. Free scan results are available for 3 days.".to_string(),
            }
        })?;

    // 2. Query findings
    let findings = db::findings::get_findings_by_scan(&state.pool, scan.id).await?;

    // 3. Calculate summary
    let mut summary = json!({
        "total": findings.len(),
        "critical": 0,
        "high": 0,
        "medium": 0,
        "low": 0
    });

    for finding in &findings {
        match finding.severity {
            Severity::Critical => {
                summary["critical"] = json!(summary["critical"].as_i64().unwrap_or(0) + 1);
            }
            Severity::High => {
                summary["high"] = json!(summary["high"].as_i64().unwrap_or(0) + 1);
            }
            Severity::Medium => {
                summary["medium"] = json!(summary["medium"].as_i64().unwrap_or(0) + 1);
            }
            Severity::Low => {
                summary["low"] = json!(summary["low"].as_i64().unwrap_or(0) + 1);
            }
        }
    }

    // 4. Build findings array (same format as get_scan)
    let findings_json: Vec<serde_json::Value> = findings
        .iter()
        .map(|f| {
            json!({
                "id": f.id,
                "title": f.title,
                "description": f.description,
                "severity": format!("{:?}", f.severity).to_lowercase(),
                "remediation": f.remediation,
                "scanner_name": f.scanner_name,
                "vibe_code": f.vibe_code,
            })
        })
        .collect();

    // 5. Return JSON response (NO email, NO submitter_ip, NO scan_id for privacy)
    let response = json!({
        "id": scan.results_token,  // Return token instead of internal ID
        "target_url": scan.target_url,
        "status": format!("{:?}", scan.status).to_lowercase(),
        "score": scan.score,
        "tier": scan.tier,
        "expires_at": scan.expires_at,
        "created_at": scan.created_at,
        "completed_at": scan.completed_at,
        "detected_framework": scan.detected_framework,
        "detected_platform": scan.detected_platform,
        "stage_detection": scan.stage_detection,
        "stage_headers": scan.stage_headers,
        "stage_tls": scan.stage_tls,
        "stage_files": scan.stage_files,
        "stage_secrets": scan.stage_secrets,
        "stage_vibecode": scan.stage_vibecode,
        "findings": findings_json,
        "summary": summary,
    });

    Ok(Json(response))
}

/// GET /api/v1/results/:token/download - Download scan results as markdown
pub async fn download_results_markdown(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Response, ApiError> {
    // 1. Query scan by token (checks expiry automatically)
    let scan = db::scans::get_scan_by_token(&state.pool, &token)
        .await?
        .ok_or_else(|| {
            ApiError::Custom {
                status: StatusCode::NOT_FOUND,
                error_type: "https://shipsecure.ai/errors/results-not-found".to_string(),
                title: "Results Not Found".to_string(),
                detail: "Results not found or link has expired. Free scan results are available for 3 days.".to_string(),
            }
        })?;

    // 2. Query findings
    let findings = db::findings::get_findings_by_scan(&state.pool, scan.id).await?;

    // 3. Group findings by severity
    let mut critical_findings = Vec::new();
    let mut high_findings = Vec::new();
    let mut medium_findings = Vec::new();
    let mut low_findings = Vec::new();

    for finding in &findings {
        match finding.severity {
            Severity::Critical => critical_findings.push(finding),
            Severity::High => high_findings.push(finding),
            Severity::Medium => medium_findings.push(finding),
            Severity::Low => low_findings.push(finding),
        }
    }

    // 4. Format timestamps
    let scanned_time = scan
        .completed_at
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "In Progress".to_string());
    let generated_time = Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();

    // 5. Build markdown report
    let detected_framework = scan.detected_framework.as_deref().unwrap_or("Not detected");
    let detected_platform = scan.detected_platform.as_deref().unwrap_or("Not detected");

    let mut markdown = format!(
        "# Security Scan Report\n\n\
         **Target:** {}\n\
         **Grade:** {}\n\
         **Framework:** {}\n\
         **Platform:** {}\n\
         **Scanned:** {}\n\
         **Report generated:** {}\n\n\
         ---\n\n\
         ## Summary\n\n\
         | Severity | Count |\n\
         |----------|-------|\n\
         | Critical | {} |\n\
         | High | {} |\n\
         | Medium | {} |\n\
         | Low | {} |\n\
         | **Total** | **{}** |\n\n\
         ---\n\n\
         ## Findings\n\n",
        scan.target_url,
        scan.score.unwrap_or_else(|| "N/A".to_string()),
        detected_framework,
        detected_platform,
        scanned_time,
        generated_time,
        critical_findings.len(),
        high_findings.len(),
        medium_findings.len(),
        low_findings.len(),
        findings.len()
    );

    // Helper function to add findings section
    let add_findings_section = |md: &mut String, severity: &str, findings_list: Vec<&models::Finding>| {
        if !findings_list.is_empty() {
            md.push_str(&format!("### {}\n\n", severity));
            for (i, finding) in findings_list.iter().enumerate() {
                let severity_label = if finding.vibe_code {
                    format!("[Vibe-Code] {}", severity)
                } else {
                    severity.to_string()
                };

                md.push_str(&format!(
                    "#### {}. {}\n\n\
                     **Severity:** {}\n\
                     **Scanner:** {}\n\n\
                     {}\n\n\
                     **How to fix:**\n\n\
                     {}\n\n\
                     ---\n\n",
                    i + 1,
                    finding.title,
                    severity_label,
                    finding.scanner_name,
                    finding.description,
                    finding.remediation
                ));
            }
        }
    };

    // Add findings by severity
    add_findings_section(&mut markdown, "Critical", critical_findings);
    add_findings_section(&mut markdown, "High", high_findings);
    add_findings_section(&mut markdown, "Medium", medium_findings);
    add_findings_section(&mut markdown, "Low", low_findings);

    // Add footer
    markdown.push_str("---\n\n*Generated by ShipSecure - https://shipsecure.ai*\n");

    // 6. Return with proper headers
    let token_prefix = if token.len() >= 8 {
        &token[..8]
    } else {
        &token
    };
    let filename = format!("trustedge-scan-{}.md", token_prefix);

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/markdown; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        markdown,
    )
        .into_response())
}
