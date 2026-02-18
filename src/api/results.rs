use axum::extract::{Path, State};
use axum::http::header::{self, AUTHORIZATION};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use serde_json::json;

use crate::api::errors::ApiError;
use crate::api::scans::AppState;
use crate::models::Severity;
use crate::{db, models};

/// Extract the bare domain from a scan target URL using the same normalization
/// as `domains::normalize_domain` — lowercase, www-stripped — to prevent
/// normalization mismatch (Phase 32 Pitfall 6).
fn extract_domain_from_url(target_url: &str) -> Option<String> {
    let parsed = url::Url::parse(target_url).ok()?;
    let host = parsed.host_str()?;
    let domain = host.strip_prefix("www.").unwrap_or(host);
    Some(domain.to_lowercase())
}

/// Optionally extract the Clerk user ID from the Authorization header.
///
/// Returns `None` if no Authorization header is present, the token is malformed,
/// or the JWT fails verification. Never fails the request — anonymous callers
/// simply get `None`.
async fn extract_optional_clerk_user(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Option<String> {
    let auth_value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let token = auth_value.strip_prefix("Bearer ")?.trim();
    if token.is_empty() {
        return None;
    }
    let token_data = state.jwt_decoder.decode(token).await.ok()?;
    Some(token_data.claims.sub)
}

/// GET /api/v1/results/:token - Get scan results by token
pub async fn get_results_by_token(
    State(state): State<AppState>,
    Path(token): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    // 1. Optionally extract Clerk user ID from Authorization header
    let authenticated_user_id = extract_optional_clerk_user(&state, &headers).await;

    // 2. Query scan by token (checks expiry automatically)
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

    // 3. Compute owner_verified — identity match AND active domain verification required.
    //    None == None returns false (anonymous scans always gated for anonymous callers).
    //    If domain verification has expired, owner_verified becomes false and results are re-gated.
    let owner_verified = match (&authenticated_user_id, &scan.clerk_user_id) {
        (Some(caller), Some(owner)) if caller == owner => {
            // Identity matches — also check active domain verification
            let domain = extract_domain_from_url(&scan.target_url);
            match domain {
                Some(ref d) => {
                    db::domains::is_domain_verified(&state.pool, caller, d)
                        .await
                        .unwrap_or(false)
                }
                None => false,
            }
        }
        _ => false,
    };

    // 4. Query findings
    let findings = db::findings::get_findings_by_scan(&state.pool, scan.id).await?;

    // 5. Calculate summary
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

    // 6. Build findings array with gating logic
    //    Non-owners: high/critical findings have description/remediation stripped and gated: true
    //    Owners: all findings have full details and gated: false
    let findings_json: Vec<serde_json::Value> = findings
        .iter()
        .map(|f| {
            let is_gated = !owner_verified
                && matches!(f.severity, Severity::High | Severity::Critical);
            if is_gated {
                json!({
                    "id": f.id,
                    "title": f.title,
                    "description": null,
                    "severity": format!("{:?}", f.severity).to_lowercase(),
                    "remediation": null,
                    "scanner_name": f.scanner_name,
                    "vibe_code": f.vibe_code,
                    "gated": true,
                })
            } else {
                json!({
                    "id": f.id,
                    "title": f.title,
                    "description": f.description,
                    "severity": format!("{:?}", f.severity).to_lowercase(),
                    "remediation": f.remediation,
                    "scanner_name": f.scanner_name,
                    "vibe_code": f.vibe_code,
                    "gated": false,
                })
            }
        })
        .collect();

    // 7. Return JSON response (NO email, NO submitter_ip, NO scan_id for privacy)
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
        "owner_verified": owner_verified,
        "findings": findings_json,
        "summary": summary,
    });

    Ok(Json(response))
}

/// GET /api/v1/results/:token/download - Download scan results as markdown
pub async fn download_results_markdown(
    State(state): State<AppState>,
    Path(token): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Response, ApiError> {
    // 1. Optionally extract Clerk user ID from Authorization header
    let authenticated_user_id = extract_optional_clerk_user(&state, &headers).await;

    // 2. Query scan by token (checks expiry automatically)
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

    // 3. Compute owner_verified — identity match AND active domain verification required.
    //    None == None returns false (anonymous scans always gated for anonymous callers).
    //    If domain verification has expired, owner_verified becomes false and results are re-gated.
    let owner_verified = match (&authenticated_user_id, &scan.clerk_user_id) {
        (Some(caller), Some(owner)) if caller == owner => {
            // Identity matches — also check active domain verification
            let domain = extract_domain_from_url(&scan.target_url);
            match domain {
                Some(ref d) => {
                    db::domains::is_domain_verified(&state.pool, caller, d)
                        .await
                        .unwrap_or(false)
                }
                None => false,
            }
        }
        _ => false,
    };

    // 4. Query findings
    let findings = db::findings::get_findings_by_scan(&state.pool, scan.id).await?;

    // 5. Group findings by severity
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

    // 6. Format timestamps
    let scanned_time = scan
        .completed_at
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "In Progress".to_string());
    let generated_time = Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();

    // 7. Build markdown report
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

    // Helper closure to add a section of findings, applying gating for non-owners
    let add_findings_section = |md: &mut String, severity: &str, findings_list: Vec<&models::Finding>| {
        if !findings_list.is_empty() {
            md.push_str(&format!("### {}\n\n", severity));
            for (i, finding) in findings_list.iter().enumerate() {
                let severity_label = if finding.vibe_code {
                    format!("[Vibe-Code] {}", severity)
                } else {
                    severity.to_string()
                };

                // Gate high and critical findings for non-owners in the markdown download
                let is_gated = !owner_verified
                    && matches!(severity, "Critical" | "High");

                let (description, remediation) = if is_gated {
                    (
                        "[Sign up free to view full details — shipsecure.ai]".to_string(),
                        "[Sign up free to view remediation — shipsecure.ai]".to_string(),
                    )
                } else {
                    (finding.description.clone(), finding.remediation.clone())
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
                    description,
                    remediation
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

    // 8. Return with proper headers
    let token_prefix = if token.len() >= 8 {
        &token[..8]
    } else {
        &token
    };
    let filename = format!("shipsecure-scan-{}.md", token_prefix);

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
