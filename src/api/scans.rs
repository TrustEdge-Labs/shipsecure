use axum::extract::{ConnectInfo, Path, State, Extension};
use axum::http::StatusCode;
use axum::Json;
use axum_jwt_auth::{Claims, Decoder};
use metrics_exporter_prometheus::PrometheusHandle;
use serde_json::json;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::api::auth::ClerkClaims;
use crate::api::errors::ApiError;
use crate::api::health::HealthCache;
use crate::models::{CreateScanRequest, Severity};
use crate::orchestrator::ScanOrchestrator;
use crate::{db, rate_limit, ssrf, RequestId};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub orchestrator: Arc<ScanOrchestrator>,
    pub health_cache: HealthCache,
    pub metrics_handle: PrometheusHandle,
    pub shutdown_token: CancellationToken,
    /// JWKS decoder for verifying Clerk JWTs. Implements `JwtDecoder<ClerkClaims>`.
    pub jwt_decoder: Decoder<ClerkClaims>,
}

/// Compute the first instant of the next UTC calendar month.
///
/// Used as `resets_at` for the monthly Developer tier quota.
fn first_of_next_month_utc() -> chrono::DateTime<chrono::Utc> {
    use chrono::{Datelike, TimeZone, Utc};
    let now = Utc::now();
    let (year, month) = if now.month() == 12 {
        (now.year() + 1, 1u32)
    } else {
        (now.year(), now.month() + 1)
    };
    Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0).unwrap()
}

/// POST /api/v1/scans - Create a new scan
pub async fn create_scan(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(request_id): Extension<RequestId>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateScanRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    // 1. Validate input
    if req.url.trim().is_empty() {
        return Err(ApiError::ValidationError(
            "URL is required and cannot be empty".to_string(),
        ));
    }

    if req.email.trim().is_empty() || !req.email.contains('@') || !req.email.contains('.') {
        return Err(ApiError::ValidationError(
            "Valid email address is required".to_string(),
        ));
    }

    // 2. Validate SSRF
    let validated_url = ssrf::validate_scan_target(&req.url).await?;

    // 3. Extract optional Clerk JWT — anonymous users get None
    let clerk_user_id = crate::api::results::extract_optional_clerk_user(&state, &headers).await;

    // 4. Compute tier from auth state
    let tier = match &clerk_user_id {
        None => "free",
        Some(_) => "authenticated",
    };

    // 5. Domain verification gate for authenticated users
    if let Some(ref user_id) = clerk_user_id {
        let domain = crate::api::results::extract_domain_from_url(&validated_url)
            .ok_or_else(|| ApiError::ValidationError("Could not parse domain from URL".to_string()))?;
        let verified = db::domains::is_domain_verified(&state.pool, user_id, &domain)
            .await
            .unwrap_or(false);
        if !verified {
            return Err(ApiError::Custom {
                status: StatusCode::FORBIDDEN,
                error_type: "https://shipsecure.ai/errors/domain-not-verified".to_string(),
                title: "Domain Not Verified".to_string(),
                detail: "You must verify ownership of this domain before scanning. Visit /verify-domain to get started.".to_string(),
            });
        }
    }

    // 6. Rate limit check — anonymous: 1/email+domain/day; authenticated: 5/user/month
    let target_domain = crate::api::results::extract_domain_from_url(&validated_url)
        .unwrap_or_else(|| validated_url.clone());
    rate_limit::check_rate_limits(&state.pool, clerk_user_id.as_deref(), &req.email, &target_domain).await?;
    let client_ip = addr.ip().to_string();

    // 7. Create scan in database with tier and clerk_user_id
    let scan = db::scans::create_scan(
        &state.pool,
        &validated_url,
        &req.email,
        Some(&client_ip),
        Some(request_id.0),
        tier,
        clerk_user_id.as_deref(),
    )
    .await?;

    // 8. Spawn scan execution (fire-and-forget) — route to tier-appropriate method
    match tier {
        "authenticated" => state.orchestrator.spawn_authenticated_scan(scan.id, scan.target_url.clone(), Some(request_id.0)),
        _ => state.orchestrator.spawn_scan(scan.id, scan.target_url.clone(), Some(request_id.0)),
    };

    // 9. Return 201 Created
    let response = json!({
        "id": scan.id,
        "status": "pending",
        "url": format!("/api/v1/scans/{}", scan.id)
    });

    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /api/v1/scans/:id - Get scan status and findings
pub async fn get_scan(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // 1. Query scan from database
    let scan = db::scans::get_scan(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    // 2. Query findings
    let findings = db::findings::get_findings_by_scan(&state.pool, id).await?;

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

    // 4. Build findings array for response
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

    // 5. Return JSON response with stage tracking and results_token
    let response = json!({
        "id": scan.id,
        "target_url": scan.target_url,
        "status": format!("{:?}", scan.status).to_lowercase(),
        "score": scan.score,
        "results_token": scan.results_token,
        "stage_detection": scan.stage_detection,
        "stage_headers": scan.stage_headers,
        "stage_tls": scan.stage_tls,
        "stage_files": scan.stage_files,
        "stage_secrets": scan.stage_secrets,
        "stage_vibecode": scan.stage_vibecode,
        "detected_framework": scan.detected_framework,
        "detected_platform": scan.detected_platform,
        "created_at": scan.created_at,
        "started_at": scan.started_at,
        "completed_at": scan.completed_at,
        "findings": findings_json,
        "summary": summary,
    });

    Ok(Json(response))
}

/// GET /api/v1/quota - Get scan quota usage for the authenticated user
///
/// Returns 401 for unauthenticated callers (via Claims<ClerkClaims> extractor).
pub async fn get_quota(
    State(state): State<AppState>,
    Claims { claims, .. }: Claims<ClerkClaims>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let used = db::scans::count_scans_by_user_this_month(&state.pool, &claims.sub).await?;
    // Developer tier limit: 5 scans/month. When Pro tier is added, gate this on user's tier.
    let limit = 5i64;
    let resets_at = first_of_next_month_utc();
    Ok(Json(json!({
        "used": used,
        "limit": limit,
        "resets_at": resets_at,
    })))
}
