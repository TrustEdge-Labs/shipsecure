use axum::Json;
use axum::extract::{ConnectInfo, Extension, Path, State};
use axum::http::StatusCode;
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
use crate::models::CreateScanRequest;
use crate::orchestrator::ScanOrchestrator;
use crate::{RequestId, db, rate_limit, ssrf};

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

/// Basic email validation: local@domain.tld format with length/character checks.
fn is_valid_email(email: &str) -> bool {
    let parts: Vec<&str> = email.splitn(2, '@').collect();
    if parts.len() != 2 {
        return false;
    }
    let (local, domain) = (parts[0], parts[1]);
    if local.is_empty() || local.len() > 64 {
        return false;
    }
    if domain.len() < 3 || !domain.contains('.') {
        return false;
    }
    // Domain must not start/end with dot or hyphen
    let domain_parts: Vec<&str> = domain.split('.').collect();
    if domain_parts.iter().any(|p| p.is_empty()) {
        return false;
    }
    // TLD must be at least 2 chars
    if domain_parts.last().is_none_or(|tld| tld.len() < 2) {
        return false;
    }
    // Only allow common safe characters
    let valid_chars = |c: char| c.is_ascii_alphanumeric() || "._+-".contains(c);
    if !local.chars().all(valid_chars) {
        return false;
    }
    let valid_domain_chars = |c: char| c.is_ascii_alphanumeric() || ".-".contains(c);
    if !domain.chars().all(valid_domain_chars) {
        return false;
    }
    true
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
    // 1. Validate URL
    let url_trimmed = req.url.trim();
    if url_trimmed.is_empty() {
        return Err(ApiError::ValidationError(
            "URL is required and cannot be empty".to_string(),
        ));
    }
    if url_trimmed.len() > 2048 {
        return Err(ApiError::ValidationError(
            "URL must not exceed 2048 characters".to_string(),
        ));
    }
    if url::Url::parse(url_trimmed).is_err() {
        return Err(ApiError::ValidationError(
            "URL must be a valid HTTP or HTTPS URL".to_string(),
        ));
    }

    // 2. Validate email
    let email_trimmed = req.email.trim();
    if email_trimmed.len() > 254 {
        return Err(ApiError::ValidationError(
            "Email address is too long".to_string(),
        ));
    }
    if !is_valid_email(email_trimmed) {
        return Err(ApiError::ValidationError(
            "Valid email address is required".to_string(),
        ));
    }

    // 3. Validate SSRF — returns URL + pre-resolved IPs to prevent DNS rebinding
    let validated = ssrf::validate_scan_target(&req.url).await?;
    let validated_url = &validated.url;

    // 3. Extract optional Clerk JWT — anonymous users get None
    let clerk_user_id = crate::api::results::extract_optional_clerk_user(&state, &headers).await;

    // 4. Compute tier from auth state
    let tier = match &clerk_user_id {
        None => "free",
        Some(_) => "authenticated",
    };

    // 5. Rate limit check — per-target: 5/domain/hour (cached); anonymous: 3/IP/day; authenticated: 5/user/month
    let client_ip = addr.ip().to_string();
    let target_domain = crate::api::results::extract_domain_from_url(validated_url)
        .unwrap_or_else(|| validated_url.clone());
    let cached_scan_id = rate_limit::check_rate_limits(
        &state.pool,
        clerk_user_id.as_deref(),
        &req.email,
        &target_domain,
        &client_ip,
    )
    .await?;

    // Per-target cache hit — return existing scan results
    if let Some(scan_id) = cached_scan_id {
        let response = json!({
            "id": scan_id,
            "status": "completed",
            "url": format!("/api/v1/scans/{}", scan_id),
            "cached": true
        });
        return Ok((StatusCode::OK, Json(response)));
    }

    // 6. Create scan in database with tier and clerk_user_id
    let scan = db::scans::create_scan(
        &state.pool,
        validated_url,
        &req.email,
        Some(&client_ip),
        Some(request_id.0),
        tier,
        clerk_user_id.as_deref(),
    )
    .await?;

    // 7. Spawn scan execution (fire-and-forget) — route to tier-appropriate method
    match tier {
        "authenticated" => state.orchestrator.spawn_authenticated_scan(
            scan.id,
            scan.target_url.clone(),
            validated.resolved_addrs.clone(),
            Some(request_id.0),
        ),
        _ => state.orchestrator.spawn_scan(
            scan.id,
            scan.target_url.clone(),
            validated.resolved_addrs.clone(),
            Some(request_id.0),
        ),
    };

    // 8. Return 201 Created
    let response = json!({
        "id": scan.id,
        "status": "pending",
        "url": format!("/api/v1/scans/{}", scan.id)
    });

    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /api/v1/scans/:id - Get scan progress (status-only, no findings)
///
/// Returns only the fields needed for the progress polling UI. Full results
/// including findings are available via `GET /api/v1/results/{token}`.
pub async fn get_scan(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let scan = db::scans::get_scan(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let response = json!({
        "id": scan.id,
        "target_url": scan.target_url,
        "status": format!("{:?}", scan.status).to_lowercase(),
        "results_token": scan.results_token,
        "stage_detection": scan.stage_detection,
        "stage_headers": scan.stage_headers,
        "stage_tls": scan.stage_tls,
        "stage_files": scan.stage_files,
        "stage_secrets": scan.stage_secrets,
        "stage_vibecode": scan.stage_vibecode,
        "created_at": scan.created_at,
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
