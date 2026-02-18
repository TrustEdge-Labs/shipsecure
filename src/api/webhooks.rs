use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use svix::webhooks::Webhook;

use crate::api::errors::ApiError;
use crate::api::scans::AppState;

/// POST /api/v1/webhooks/clerk - Handle Clerk webhook events
///
/// Verifies the svix signature using CLERK_WEBHOOK_SIGNING_SECRET, then processes
/// known event types. On user.created: inserts a row into the users table.
/// Unknown event types are logged and ignored. Returns 204 No Content on success.
///
/// Svix automatic retry handles DB errors (returns 500) and signature failures (returns 401).
pub async fn handle_clerk_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    // 1. Get signing secret from env
    let secret = std::env::var("CLERK_WEBHOOK_SIGNING_SECRET").map_err(|_| {
        tracing::error!("CLERK_WEBHOOK_SIGNING_SECRET not set");
        ApiError::InternalError("CLERK_WEBHOOK_SIGNING_SECRET not set".to_string())
    })?;

    // 2. Create svix verifier — svix::Webhook::new accepts "whsec_..." format directly
    let wh = Webhook::new(&secret).map_err(|e| {
        tracing::error!("Invalid Clerk webhook secret format: {:?}", e);
        ApiError::InternalError("Invalid webhook secret format".to_string())
    })?;

    // 3. Verify signature — axum::http::HeaderMap is http 1.x HeaderMap, which svix supports directly
    wh.verify(&body, &headers).map_err(|_| {
        tracing::warn!("Clerk webhook signature verification failed");
        ApiError::Unauthorized
    })?;

    // 4. Parse JSON event body
    let event: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| ApiError::ValidationError(format!("Invalid JSON: {}", e)))?;

    let event_type = event["type"].as_str().unwrap_or("").to_string();

    // 5. Route by event type
    match event_type.as_str() {
        "user.created" => {
            let clerk_user_id = event["data"]["id"]
                .as_str()
                .ok_or_else(|| ApiError::ValidationError("Missing user id in webhook".to_string()))?;

            let email = event["data"]["email_addresses"][0]["email_address"]
                .as_str()
                .unwrap_or("");

            sqlx::query(
                "INSERT INTO users (clerk_user_id, email) VALUES ($1, $2) ON CONFLICT (clerk_user_id) DO NOTHING"
            )
            .bind(clerk_user_id)
            .bind(email)
            .execute(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to insert user from Clerk webhook: {:?}", e);
                ApiError::InternalError(format!("Database error: {}", e))
            })?;

            tracing::info!(
                clerk_user_id = %clerk_user_id,
                "User created via Clerk webhook"
            );
        }
        _ => {
            tracing::info!(event_type = %event_type, "Unhandled Clerk webhook event");
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
