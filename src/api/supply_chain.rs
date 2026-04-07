// Supply chain scan HTTP handler.
//
// POST /api/v1/scans/supply-chain
// Accepts 3 input modes: GitHub URL, lockfile content (paste), multipart file upload.
// Runs scan synchronously, persists results with 30-day expiry token.
// Anonymous access allowed (no Clerk JWT required).

use axum::Json;
use axum::body::Body;
use axum::extract::{ConnectInfo, Extension, FromRequest, Multipart, State};
use axum::http::{HeaderMap, StatusCode};
use base64::Engine;
use chrono::Utc;
use rand::Rng;
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use std::time::Duration;

use crate::api::errors::ApiError;
use crate::api::results::extract_optional_clerk_user;
use crate::api::scans::AppState;
use crate::scanners::lockfile_parser::{self, SupplyChainError};
use crate::scanners::supply_chain;
use crate::{RequestId, db};

/// Maximum body/file size: 5 MB.
const MAX_BODY_SIZE: usize = 5 * 1024 * 1024;

/// Maximum number of dependencies before rejecting.
const MAX_DEP_COUNT: usize = 5000;

/// Scan timeout in seconds.
const SCAN_TIMEOUT_SECS: u64 = 30;

/// GitHub URL fetch timeout in seconds.
const GITHUB_FETCH_TIMEOUT_SECS: u64 = 5;

/// Result expiry: 30 days.
const RESULT_EXPIRY_DAYS: i64 = 30;

// -- Input dispatch --

enum SupplyChainInput {
    GitHubUrl(String),
    LockfileContent(String),
    FileUpload(String),
}

#[derive(Deserialize)]
struct JsonInput {
    github_url: Option<String>,
    lockfile_content: Option<String>,
}

// -- Error mapping --

impl From<SupplyChainError> for ApiError {
    fn from(err: SupplyChainError) -> Self {
        match err {
            SupplyChainError::LockfileParse(msg) => ApiError::Custom {
                status: StatusCode::BAD_REQUEST,
                error_type: "https://shipsecure.ai/errors/invalid-lockfile".to_string(),
                title: "Invalid Lockfile".to_string(),
                detail: format!("Invalid lockfile format: {msg}"),
            },
            SupplyChainError::OsvQuery(ref msg) => {
                tracing::error!(error = %msg, "OSV query failed");
                ApiError::Custom {
                    status: StatusCode::BAD_GATEWAY,
                    error_type: "https://shipsecure.ai/errors/osv-unavailable".to_string(),
                    title: "Vulnerability Database Unavailable".to_string(),
                    detail: "Vulnerability database unavailable, please try again".to_string(),
                }
            }
            SupplyChainError::GitHubFetch(msg) => ApiError::Custom {
                status: StatusCode::BAD_GATEWAY,
                error_type: "https://shipsecure.ai/errors/github-fetch".to_string(),
                title: "GitHub Fetch Failed".to_string(),
                detail: msg,
            },
            SupplyChainError::ChunkFailure(ref msg) => {
                tracing::error!(error = %msg, "Vulnerability check chunk failed");
                ApiError::Custom {
                    status: StatusCode::BAD_GATEWAY,
                    error_type: "https://shipsecure.ai/errors/vuln-check-failed".to_string(),
                    title: "Vulnerability Check Failed".to_string(),
                    detail: "Vulnerability check failed, please try again".to_string(),
                }
            }
            SupplyChainError::DepCountExceeded(n) => ApiError::Custom {
                status: StatusCode::BAD_REQUEST,
                error_type: "https://shipsecure.ai/errors/too-many-deps".to_string(),
                title: "Too Many Dependencies".to_string(),
                detail: format!("Too many dependencies ({n}, max {MAX_DEP_COUNT})"),
            },
            SupplyChainError::Timeout => ApiError::Custom {
                status: StatusCode::GATEWAY_TIMEOUT,
                error_type: "https://shipsecure.ai/errors/scan-timeout".to_string(),
                title: "Scan Timed Out".to_string(),
                detail: "Scan timed out, try a smaller lockfile".to_string(),
            },
        }
    }
}

// -- GitHub URL fetching --

/// Parse owner/repo from a GitHub URL using strict regex (SSRF mitigation: T-47-03).
fn parse_github_url(url: &str) -> Result<(String, String), SupplyChainError> {
    let re = Regex::new(r"^https?://github\.com/([^/]+)/([^/]+?)(?:\.git)?(?:/.*)?$").unwrap();
    let caps = re.captures(url).ok_or_else(|| {
        SupplyChainError::GitHubFetch(
            "Invalid GitHub URL. Expected format: https://github.com/owner/repo".to_string(),
        )
    })?;
    Ok((caps[1].to_string(), caps[2].to_string()))
}

/// Fetch package-lock.json from a GitHub repository (main branch, fallback to master).
async fn fetch_lockfile_from_github(url: &str) -> Result<String, SupplyChainError> {
    let (owner, repo) = parse_github_url(url)?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(GITHUB_FETCH_TIMEOUT_SECS))
        .build()
        .map_err(|e| SupplyChainError::GitHubFetch(format!("HTTP client error: {e}")))?;

    let github_token = std::env::var("GITHUB_TOKEN").ok();

    for branch in &["main", "master"] {
        let raw_url =
            format!("https://raw.githubusercontent.com/{owner}/{repo}/{branch}/package-lock.json");

        let mut req = client.get(&raw_url);
        if let Some(ref token) = github_token {
            req = req.header("Authorization", format!("token {token}"));
        }

        let resp = req.send().await.map_err(|e| {
            SupplyChainError::GitHubFetch(format!("Failed to fetch from GitHub: {e}"))
        })?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            continue;
        }

        if !resp.status().is_success() {
            return Err(SupplyChainError::GitHubFetch(format!(
                "GitHub returned HTTP {}",
                resp.status()
            )));
        }

        // Check content length if available
        if let Some(len) = resp.content_length()
            && len as usize > MAX_BODY_SIZE
        {
            return Err(SupplyChainError::GitHubFetch(
                "Lockfile exceeds 5MB size limit".to_string(),
            ));
        }

        let body = resp.text().await.map_err(|e| {
            SupplyChainError::GitHubFetch(format!("Failed to read response body: {e}"))
        })?;

        if body.len() > MAX_BODY_SIZE {
            return Err(SupplyChainError::GitHubFetch(
                "Lockfile exceeds 5MB size limit".to_string(),
            ));
        }

        return Ok(body);
    }

    Err(SupplyChainError::GitHubFetch(
        "No package-lock.json found on main or master branch".to_string(),
    ))
}

/// Generate a URL-safe base64 token for result sharing.
fn generate_results_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.r#gen();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

// -- Handler --

/// POST /api/v1/scans/supply-chain
///
/// Accepts 3 input modes:
/// - JSON `{ "github_url": "https://github.com/owner/repo" }`
/// - JSON `{ "lockfile_content": "..." }`
/// - multipart/form-data with field "lockfile"
///
/// Returns scan results with optional share URL. Anonymous access allowed.
pub async fn create_supply_chain_scan(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(request_id): Extension<RequestId>,
    headers: HeaderMap,
    body: Body,
) -> Result<Json<serde_json::Value>, ApiError> {
    // 1. Extract optional Clerk user (anonymous access allowed)
    let clerk_user_id = extract_optional_clerk_user(&state, &headers).await;

    // 2. Parse input by content type
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let input = if content_type.starts_with("multipart/form-data") {
        parse_multipart_input(headers.clone(), body).await?
    } else if content_type.starts_with("application/json") {
        parse_json_input(body).await?
    } else {
        return Err(ApiError::ValidationError(
            "Unsupported content type. Use application/json or multipart/form-data".to_string(),
        ));
    };

    // 3. Resolve lockfile content from input
    let (lockfile_content, target_url) = match input {
        SupplyChainInput::GitHubUrl(ref url) => {
            let content = fetch_lockfile_from_github(url).await?;
            (content, url.clone())
        }
        SupplyChainInput::LockfileContent(content) => {
            if content.trim().is_empty() {
                return Err(ApiError::ValidationError(
                    "lockfile_content cannot be empty".to_string(),
                ));
            }
            (content, "paste".to_string())
        }
        SupplyChainInput::FileUpload(content) => {
            if content.trim().is_empty() {
                return Err(ApiError::ValidationError(
                    "Uploaded lockfile is empty".to_string(),
                ));
            }
            (content, "upload".to_string())
        }
    };

    // 4. Pre-check dep count (before expensive scan)
    let deps = lockfile_parser::parse(&lockfile_content)?;
    if deps.len() > MAX_DEP_COUNT {
        return Err(SupplyChainError::DepCountExceeded(deps.len()).into());
    }

    // 5. Create DB row
    let client_ip = addr.ip().to_string();
    let db_result = db::scans::create_supply_chain_scan(
        &state.pool,
        &target_url,
        Some(&client_ip),
        Some(request_id.0),
        clerk_user_id.as_deref(),
    )
    .await;

    let scan_row = match db_result {
        Ok(scan) => Some(scan),
        Err(e) => {
            tracing::error!(error = %e, "Failed to create supply chain scan DB row");
            None
        }
    };

    // 6. Run scan with 30s timeout
    let scan_result = match tokio::time::timeout(
        Duration::from_secs(SCAN_TIMEOUT_SECS),
        supply_chain::scan_lockfile(&lockfile_content),
    )
    .await
    {
        Ok(Ok(result)) => result,
        Ok(Err(scan_err)) => {
            // Mark scan failed in DB if we have a row
            if let Some(ref scan) = scan_row {
                let _ =
                    db::scans::fail_supply_chain_scan(&state.pool, scan.id, &scan_err.to_string())
                        .await;
            }
            return Err(scan_err.into());
        }
        Err(_timeout) => {
            // Mark scan failed in DB if we have a row
            if let Some(ref scan) = scan_row {
                let _ = db::scans::fail_supply_chain_scan(
                    &state.pool,
                    scan.id,
                    "Scan timed out after 30 seconds",
                )
                .await;
            }
            return Err(SupplyChainError::Timeout.into());
        }
    };

    // 7. Serialize results
    let results_json = serde_json::to_value(&scan_result).unwrap_or_else(|e| {
        tracing::error!(error = %e, "Failed to serialize scan results");
        json!({})
    });

    // 8. Persist results with token (graceful fallback per D-13)
    let (results_token, share_url, share_unavailable) = if let Some(ref scan) = scan_row {
        let token = generate_results_token();
        let expires_at = (Utc::now() + chrono::Duration::days(RESULT_EXPIRY_DAYS)).naive_utc();

        match db::scans::complete_supply_chain_scan(
            &state.pool,
            scan.id,
            &results_json,
            &token,
            expires_at,
        )
        .await
        {
            Ok(()) => (
                Some(token.clone()),
                Some(format!("/supply-chain/results/{token}")),
                false,
            ),
            Err(e) => {
                tracing::error!(error = %e, scan_id = %scan.id, "Failed to persist supply chain results — returning inline");
                (None, None, true)
            }
        }
    } else {
        // No DB row — return results inline
        (None, None, true)
    };

    // 9. Build response
    let response = json!({
        "status": "completed",
        "results_token": results_token,
        "share_url": share_url,
        "share_unavailable": share_unavailable,
        "results": results_json,
    });

    Ok(Json(response))
}

// -- Input parsing helpers --

async fn parse_multipart_input(
    headers: HeaderMap,
    body: Body,
) -> Result<SupplyChainInput, ApiError> {
    let ct = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let request = axum::http::Request::builder()
        .header("content-type", ct)
        .body(body)
        .unwrap();

    let mut multipart = <Multipart as FromRequest<()>>::from_request(request, &())
        .await
        .map_err(|e| ApiError::ValidationError(format!("Invalid multipart request: {e}")))?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::ValidationError(format!("Failed to read multipart field: {e}")))?
    {
        if field.name() == Some("lockfile") {
            let bytes = field
                .bytes()
                .await
                .map_err(|e| ApiError::ValidationError(format!("Failed to read file: {e}")))?;

            if bytes.len() > MAX_BODY_SIZE {
                return Err(ApiError::ValidationError(format!(
                    "File exceeds {}MB size limit",
                    MAX_BODY_SIZE / (1024 * 1024)
                )));
            }

            let content = String::from_utf8(bytes.to_vec()).map_err(|_| {
                ApiError::ValidationError("Lockfile must be valid UTF-8 text".to_string())
            })?;

            return Ok(SupplyChainInput::FileUpload(content));
        }
    }

    Err(ApiError::ValidationError(
        "Multipart request must include a 'lockfile' field".to_string(),
    ))
}

async fn parse_json_input(body: Body) -> Result<SupplyChainInput, ApiError> {
    let bytes = axum::body::to_bytes(body, MAX_BODY_SIZE)
        .await
        .map_err(|_| {
            ApiError::ValidationError(format!(
                "Request body exceeds {}MB size limit",
                MAX_BODY_SIZE / (1024 * 1024)
            ))
        })?;

    let input: JsonInput = serde_json::from_slice(&bytes)
        .map_err(|e| ApiError::ValidationError(format!("Invalid JSON: {e}")))?;

    if let Some(url) = input.github_url {
        if url.trim().is_empty() {
            return Err(ApiError::ValidationError(
                "github_url cannot be empty".to_string(),
            ));
        }
        Ok(SupplyChainInput::GitHubUrl(url.trim().to_string()))
    } else if let Some(content) = input.lockfile_content {
        Ok(SupplyChainInput::LockfileContent(content))
    } else {
        Err(ApiError::ValidationError(
            "Must provide github_url, lockfile_content, or upload a file".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_github_url_valid() {
        let (owner, repo) = parse_github_url("https://github.com/vercel/next.js").unwrap();
        assert_eq!(owner, "vercel");
        assert_eq!(repo, "next.js");
    }

    #[test]
    fn parse_github_url_with_git_suffix() {
        let (owner, repo) = parse_github_url("https://github.com/vercel/next.js.git").unwrap();
        assert_eq!(owner, "vercel");
        assert_eq!(repo, "next.js");
    }

    #[test]
    fn parse_github_url_with_path() {
        let (owner, repo) =
            parse_github_url("https://github.com/vercel/next.js/tree/main/packages").unwrap();
        assert_eq!(owner, "vercel");
        assert_eq!(repo, "next.js");
    }

    #[test]
    fn parse_github_url_http() {
        let (owner, repo) = parse_github_url("http://github.com/user/repo").unwrap();
        assert_eq!(owner, "user");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn parse_github_url_invalid_domain() {
        let result = parse_github_url("https://gitlab.com/user/repo");
        assert!(result.is_err());
    }

    #[test]
    fn parse_github_url_missing_repo() {
        let result = parse_github_url("https://github.com/user");
        assert!(result.is_err());
    }

    #[test]
    fn parse_github_url_empty() {
        let result = parse_github_url("");
        assert!(result.is_err());
    }

    #[test]
    fn error_mapping_lockfile_parse() {
        let err: ApiError = SupplyChainError::LockfileParse("bad json".to_string()).into();
        match err {
            ApiError::Custom { status, .. } => assert_eq!(status, StatusCode::BAD_REQUEST),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn error_mapping_dep_count_exceeded() {
        let err: ApiError = SupplyChainError::DepCountExceeded(6000).into();
        match err {
            ApiError::Custom { status, detail, .. } => {
                assert_eq!(status, StatusCode::BAD_REQUEST);
                assert!(detail.contains("6000"));
            }
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn error_mapping_timeout() {
        let err: ApiError = SupplyChainError::Timeout.into();
        match err {
            ApiError::Custom { status, .. } => assert_eq!(status, StatusCode::GATEWAY_TIMEOUT),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn error_mapping_github_fetch() {
        let err: ApiError = SupplyChainError::GitHubFetch("Not found".to_string()).into();
        match err {
            ApiError::Custom { status, .. } => assert_eq!(status, StatusCode::BAD_GATEWAY),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn error_mapping_osv_query() {
        let err: ApiError = SupplyChainError::OsvQuery("timeout".to_string()).into();
        match err {
            ApiError::Custom { status, .. } => assert_eq!(status, StatusCode::BAD_GATEWAY),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn error_mapping_chunk_failure() {
        let err: ApiError = SupplyChainError::ChunkFailure("network error".to_string()).into();
        match err {
            ApiError::Custom { status, .. } => assert_eq!(status, StatusCode::BAD_GATEWAY),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn token_generation_length() {
        let token = generate_results_token();
        // 32 bytes base64 = 43 chars (URL_SAFE_NO_PAD)
        assert_eq!(token.len(), 43);
    }

    #[test]
    fn token_generation_uniqueness() {
        let t1 = generate_results_token();
        let t2 = generate_results_token();
        assert_ne!(t1, t2);
    }
}
