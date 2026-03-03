use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum_jwt_auth::Claims;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{Duration, Utc};
use rand::Rng;
use scraper::{Html, Selector};
use serde::Deserialize;

use crate::api::auth::ClerkClaims;
use crate::api::errors::ApiError;
use crate::api::scans::AppState;
use crate::{db, ssrf};

// ---------------------------------------------------------------------------
// Shared-hosting TLD blocklist
// ---------------------------------------------------------------------------

/// Root shared-hosting TLDs that cannot be verified directly.
/// Subdomains of these (e.g., myapp.vercel.app) are allowed — blocking is
/// only for the bare root (e.g., vercel.app).
const BLOCKED_ROOT_TLDS: &[&str] = &["github.io", "vercel.app", "netlify.app", "pages.dev"];

/// Returns true ONLY for an exact root TLD match.
///
/// Examples:
/// - `vercel.app` → true (blocked)
/// - `myapp.vercel.app` → false (allowed — subdomain)
fn is_blocked_root_tld(domain: &str) -> bool {
    BLOCKED_ROOT_TLDS.contains(&domain)
}

// ---------------------------------------------------------------------------
// Domain normalization
// ---------------------------------------------------------------------------

/// Normalize a user-supplied domain string to a bare lowercase domain without www.
///
/// Handles inputs like:
/// - "example.com"
/// - "https://example.com"
/// - "www.example.com/path"
/// - "myapp.vercel.app"
///
/// Returns an error if the input cannot be parsed as a URL.
pub fn normalize_domain(input: &str) -> Result<String, ApiError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ApiError::ValidationError(
            "Domain cannot be empty".to_string(),
        ));
    }

    // If input contains "://" it's already a URL, otherwise prepend https://
    let url_str = if input.contains("://") {
        input.to_string()
    } else {
        format!("https://{}", input)
    };

    let parsed = url::Url::parse(&url_str).map_err(|_| {
        ApiError::ValidationError(format!("'{}' is not a valid domain name", input))
    })?;

    let host = parsed.host_str().ok_or_else(|| {
        ApiError::ValidationError(format!("'{}' is not a valid domain name", input))
    })?;

    let domain = host.strip_prefix("www.").unwrap_or(host);
    Ok(domain.to_lowercase())
}

// ---------------------------------------------------------------------------
// Token generation
// ---------------------------------------------------------------------------

/// Generate a cryptographically random 256-bit opaque verification token.
///
/// Uses the same pattern as `generate_results_token()` in worker_pool.rs.
fn generate_verification_token() -> String {
    let bytes: [u8; 32] = rand::thread_rng().r#gen();
    URL_SAFE_NO_PAD.encode(bytes)
}

// ---------------------------------------------------------------------------
// Meta tag verification
// ---------------------------------------------------------------------------

/// Describes why the meta tag verification failed.
#[derive(Debug)]
enum VerificationFailureReason {
    FetchFailed(String),
    TagNotFound,
    /// Tag was found but only in <body>, not <head>. Verification still succeeds.
    TagInBody,
    WrongContent,
}

/// Fetch the domain's homepage and check for the ShipSecure verification meta tag.
///
/// Returns Ok(()) if the tag is present and the content matches `expected_token`.
/// Returns Err(VerificationFailureReason) on any failure or mismatch.
async fn fetch_and_check_meta_tag(
    target_url: &str,
    expected_token: &str,
    resolved_addrs: &[std::net::SocketAddr],
) -> Result<(), VerificationFailureReason> {
    let hostname = url::Url::parse(target_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_default();

    let client = ssrf::safe_client_builder(&hostname, resolved_addrs)
        .timeout(std::time::Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(5))
        .user_agent("ShipSecure-Verifier/1.0")
        .build()
        .map_err(|e| VerificationFailureReason::FetchFailed(e.to_string()))?;

    let response = client
        .get(target_url)
        .send()
        .await
        .map_err(|e| VerificationFailureReason::FetchFailed(e.to_string()))?;

    // Limit response body to 1MB to prevent memory exhaustion from malicious domains
    let bytes = response
        .bytes()
        .await
        .map_err(|e| VerificationFailureReason::FetchFailed(e.to_string()))?;
    if bytes.len() > 1_048_576 {
        return Err(VerificationFailureReason::FetchFailed(
            "Response body exceeds 1MB limit".to_string(),
        ));
    }
    let html_body = String::from_utf8_lossy(&bytes).to_string();

    let document = Html::parse_document(&html_body);

    let meta_selector = Selector::parse("meta[name='shipsecure-verification']")
        .expect("Static selector is always valid");

    // Check for the tag anywhere in the document first
    let all_tags: Vec<_> = document.select(&meta_selector).collect();

    if all_tags.is_empty() {
        return Err(VerificationFailureReason::TagNotFound);
    }

    // Check if the content matches
    let matching_tag = all_tags.iter().find(|el| {
        el.value()
            .attr("content")
            .map(|c| c == expected_token)
            .unwrap_or(false)
    });

    if matching_tag.is_none() {
        return Err(VerificationFailureReason::WrongContent);
    }

    // Diagnostic: check if the tag is in <body> rather than <head>
    // (verification still succeeds — proving HTML control is sufficient)
    let head_selector = Selector::parse("head meta[name='shipsecure-verification']")
        .expect("Static selector is always valid");
    let in_head: Vec<_> = document.select(&head_selector).collect();

    let tag_in_head = in_head.iter().any(|el| {
        el.value()
            .attr("content")
            .map(|c| c == expected_token)
            .unwrap_or(false)
    });

    if !tag_in_head {
        // Tag found and content matches, but it's in <body>
        return Err(VerificationFailureReason::TagInBody);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct VerifyStartRequest {
    pub domain: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyConfirmRequest {
    pub domain: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyCheckRequest {
    pub domain: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/v1/domains/verify-start
///
/// Starts or resets verification for a domain. Returns a unique token and the
/// meta tag snippet the user needs to place in their page's <head>.
///
/// Returns 200 if the domain is already actively verified (no new token issued).
/// Returns 201 with token + meta_tag for a new or reset verification.
/// Returns 400 if the domain is a blocked shared-hosting root TLD.
pub async fn verify_start(
    State(state): State<AppState>,
    Claims { claims, .. }: Claims<ClerkClaims>,
    Json(req): Json<VerifyStartRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let domain = normalize_domain(&req.domain)?;
    let clerk_user_id = claims.sub;

    // Block bare shared-hosting root TLDs
    if is_blocked_root_tld(&domain) {
        return Err(ApiError::ValidationError(format!(
            "'{}' is a shared hosting platform. Enter your app's subdomain instead (e.g., myapp.{}).",
            domain, domain
        )));
    }

    // Ensure user row exists (Clerk webhook may not have fired yet)
    sqlx::query(
        "INSERT INTO users (clerk_user_id, email) VALUES ($1, '') ON CONFLICT (clerk_user_id) DO NOTHING"
    )
    .bind(&clerk_user_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to ensure user exists: {:?}", e);
        ApiError::InternalError(format!("Database error: {}", e))
    })?;

    // Check if already actively verified — return 200 with status, no new token
    if let Some(existing) =
        db::domains::get_verified_domain(&state.pool, &clerk_user_id, &domain).await?
        && existing.status == "verified"
        && let Some(expires_at) = existing.expires_at
        && expires_at > Utc::now()
    {
        let expires_in_days = (expires_at - Utc::now()).num_days();
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "already_verified": true,
                "domain": domain,
                "expires_in_days": expires_in_days,
            })),
        ));
    }

    // Generate token and meta tag
    let token = generate_verification_token();
    let meta_tag = format!(
        r#"<meta name="shipsecure-verification" content="{}">"#,
        token
    );

    // Upsert pending record (resets any expired or pending records)
    db::domains::upsert_pending_domain(&state.pool, &clerk_user_id, &domain, &token).await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "domain": domain,
            "token": token,
            "meta_tag": meta_tag,
        })),
    ))
}

/// POST /api/v1/domains/verify-confirm
///
/// Fetches the domain's homepage and checks for the meta tag. On success,
/// marks the domain verified with a 30-day expiry.
pub async fn verify_confirm(
    State(state): State<AppState>,
    Claims { claims, .. }: Claims<ClerkClaims>,
    Json(req): Json<VerifyConfirmRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let domain = normalize_domain(&req.domain)?;
    let clerk_user_id = claims.sub;

    // Look up existing verification record
    let record = db::domains::get_verified_domain(&state.pool, &clerk_user_id, &domain)
        .await?
        .ok_or_else(|| {
            ApiError::ValidationError(format!(
                "No verification started for '{}'. Call verify-start first.",
                domain
            ))
        })?;

    let target_url = format!("https://{}", domain);

    // SSRF validation before any outbound fetch — returns resolved IPs to prevent rebinding
    let validated = ssrf::validate_scan_target(&target_url)
        .await
        .map_err(|e| ApiError::SsrfBlocked(e.to_string()))?;

    // Fetch and check meta tag
    match fetch_and_check_meta_tag(
        &target_url,
        &record.verification_token,
        &validated.resolved_addrs,
    )
    .await
    {
        Ok(()) => {
            // Tag found and matches — mark verified with 30-day expiry
            let expires_at = Utc::now() + Duration::days(30);
            db::domains::mark_verified(&state.pool, &clerk_user_id, &domain, expires_at).await?;

            Ok(Json(serde_json::json!({
                "verified": true,
                "domain": domain,
                "expires_at": expires_at,
            })))
        }
        Err(VerificationFailureReason::TagInBody) => {
            // Tag in <body> — soft warning, verification still succeeds
            let expires_at = Utc::now() + Duration::days(30);
            db::domains::mark_verified(&state.pool, &clerk_user_id, &domain, expires_at).await?;

            Ok(Json(serde_json::json!({
                "verified": true,
                "domain": domain,
                "expires_at": expires_at,
                "warning": "We found the meta tag in <body>, not <head>. Move it to the <head> section for best results. Verification still succeeded.",
            })))
        }
        Err(VerificationFailureReason::FetchFailed(e)) => Ok(Json(serde_json::json!({
            "verified": false,
            "domain": domain,
            "failure_reason": format!("We couldn't reach your site: {}. Make sure it's publicly accessible.", e),
        }))),
        Err(VerificationFailureReason::TagNotFound) => Ok(Json(serde_json::json!({
            "verified": false,
            "domain": domain,
            "failure_reason": "We fetched your page but didn't find the meta tag. Check that it's in <head>, not <body>.",
        }))),
        Err(VerificationFailureReason::WrongContent) => Ok(Json(serde_json::json!({
            "verified": false,
            "domain": domain,
            "failure_reason": "We found the meta tag but the content doesn't match the expected token. Copy the snippet again and replace any existing tag.",
        }))),
    }
}

/// POST /api/v1/domains/verify-check
///
/// Pre-check ("Test my tag"): fetches the domain homepage and checks for the
/// meta tag without writing to the database. Returns diagnostic information
/// so users can confirm their tag is live before running verify-confirm.
pub async fn verify_check(
    State(state): State<AppState>,
    Claims { claims, .. }: Claims<ClerkClaims>,
    Json(req): Json<VerifyCheckRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let domain = normalize_domain(&req.domain)?;
    let clerk_user_id = claims.sub;

    // Look up existing verification record for the expected token
    let record = db::domains::get_verified_domain(&state.pool, &clerk_user_id, &domain)
        .await?
        .ok_or_else(|| {
            ApiError::ValidationError(format!(
                "No verification started for '{}'. Call verify-start first.",
                domain
            ))
        })?;

    let target_url = format!("https://{}", domain);

    // SSRF validation before any outbound fetch — returns resolved IPs to prevent rebinding
    let validated = ssrf::validate_scan_target(&target_url)
        .await
        .map_err(|e| ApiError::SsrfBlocked(e.to_string()))?;

    // Fetch and check meta tag (no DB write — pre-check only)
    match fetch_and_check_meta_tag(
        &target_url,
        &record.verification_token,
        &validated.resolved_addrs,
    )
    .await
    {
        Ok(()) => Ok(Json(serde_json::json!({
            "found": true,
            "domain": domain,
            "message": "Meta tag found and content matches. You're ready to verify.",
        }))),
        Err(VerificationFailureReason::TagInBody) => Ok(Json(serde_json::json!({
            "found": true,
            "domain": domain,
            "message": "We found the meta tag in <body>, not <head>. Move it to the <head> section for best results. Verification still succeeded.",
        }))),
        Err(VerificationFailureReason::FetchFailed(e)) => Ok(Json(serde_json::json!({
            "found": false,
            "domain": domain,
            "message": format!("We couldn't reach your site: {}. Make sure it's publicly accessible.", e),
        }))),
        Err(VerificationFailureReason::TagNotFound) => Ok(Json(serde_json::json!({
            "found": false,
            "domain": domain,
            "message": "We fetched your page but didn't find the meta tag. Check that it's in <head>, not <body>.",
        }))),
        Err(VerificationFailureReason::WrongContent) => Ok(Json(serde_json::json!({
            "found": false,
            "domain": domain,
            "message": "We found the meta tag but the content doesn't match the expected token. Copy the snippet again and replace any existing tag.",
        }))),
    }
}

/// GET /api/v1/domains
///
/// List all domain verification records for the authenticated user.
/// Excludes `verification_token` from the response to prevent token leakage.
pub async fn list_domains(
    State(state): State<AppState>,
    Claims { claims, .. }: Claims<ClerkClaims>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let clerk_user_id = claims.sub;
    let domains = db::domains::list_user_domains(&state.pool, &clerk_user_id).await?;

    let response: Vec<serde_json::Value> = domains
        .iter()
        .map(|d| {
            serde_json::json!({
                "id": d.id,
                "domain": d.domain,
                "status": d.status,
                "verified_at": d.verified_at,
                "expires_at": d.expires_at,
                "created_at": d.created_at,
                "updated_at": d.updated_at,
            })
        })
        .collect();

    Ok(Json(serde_json::json!(response)))
}
