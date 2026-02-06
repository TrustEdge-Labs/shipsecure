use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::api::errors::ApiError;
use crate::api::scans::AppState;
use crate::db;

type HmacSha256 = Hmac<Sha256>;

/// POST /api/v1/webhooks/stripe - Handle Stripe webhook events
pub async fn handle_stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    // 1. Extract Stripe signature header
    let signature_header = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            ApiError::ValidationError("Missing stripe-signature header".to_string())
        })?;

    // 2. Parse signature header to extract timestamp and signature
    let mut timestamp: Option<i64> = None;
    let mut signature: Option<&str> = None;

    for part in signature_header.split(',') {
        let kv: Vec<&str> = part.splitn(2, '=').collect();
        if kv.len() == 2 {
            match kv[0] {
                "t" => timestamp = kv[1].parse().ok(),
                "v1" => signature = Some(kv[1]),
                _ => {}
            }
        }
    }

    let timestamp = timestamp.ok_or_else(|| {
        ApiError::ValidationError("Invalid signature: missing timestamp".to_string())
    })?;

    let signature = signature.ok_or_else(|| {
        ApiError::ValidationError("Invalid signature: missing v1 signature".to_string())
    })?;

    // 3. Verify timestamp is recent (within 5 minutes)
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| ApiError::InternalError(format!("System time error: {}", e)))?
        .as_secs() as i64;

    if (now - timestamp).abs() > 300 {
        return Err(ApiError::ValidationError(
            "Webhook timestamp too old (replay protection)".to_string(),
        ));
    }

    // 4. Verify signature
    let webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET")
        .unwrap_or_default();

    if webhook_secret.is_empty() {
        return Err(ApiError::InternalError(
            "Stripe webhook secret not configured".to_string(),
        ));
    }

    // Compute expected signature
    let payload = format!("{}.{}", timestamp, std::str::from_utf8(&body)
        .map_err(|e| ApiError::ValidationError(format!("Invalid UTF-8 in body: {}", e)))?);

    let mut mac = HmacSha256::new_from_slice(webhook_secret.as_bytes())
        .map_err(|e| ApiError::InternalError(format!("HMAC initialization failed: {}", e)))?;
    mac.update(payload.as_bytes());

    let computed_signature = hex::encode(mac.finalize().into_bytes());

    // Constant-time comparison
    if computed_signature != signature {
        tracing::warn!("Webhook signature verification failed");
        return Err(ApiError::ValidationError(
            "Invalid webhook signature".to_string(),
        ));
    }

    // 5. Parse event body
    let event: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| ApiError::ValidationError(format!("Invalid JSON: {}", e)))?;

    // 6. Extract event ID and type
    let event_id = event["id"]
        .as_str()
        .ok_or_else(|| ApiError::ValidationError("Missing event id".to_string()))?;

    let event_type = event["type"]
        .as_str()
        .ok_or_else(|| ApiError::ValidationError("Missing event type".to_string()))?;

    // 7. Check idempotency - if we've seen this event before, return 200 immediately
    let is_new_event = db::paid_audits::check_and_mark_event(&state.pool, event_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check event idempotency: {:?}", e);
            ApiError::InternalError("Event idempotency check failed".to_string())
        })?;

    if !is_new_event {
        tracing::info!("Duplicate webhook event {} ignored", event_id);
        return Ok(StatusCode::OK);
    }

    // 8. Process event based on type
    match event_type {
        "checkout.session.completed" => {
            handle_checkout_completed(state, event).await?;
        }
        _ => {
            tracing::info!("Ignoring unhandled event type: {}", event_type);
        }
    }

    // 9. Return 200 OK immediately (Stripe best practice)
    Ok(StatusCode::OK)
}

/// Handle checkout.session.completed event
async fn handle_checkout_completed(
    state: AppState,
    event: serde_json::Value,
) -> Result<(), ApiError> {
    // Extract session object
    let session = &event["data"]["object"];

    let checkout_session_id = session["id"]
        .as_str()
        .ok_or_else(|| ApiError::ValidationError("Missing checkout session id".to_string()))?;

    // Extract metadata
    let scan_id_str = session["metadata"]["scan_id"]
        .as_str()
        .ok_or_else(|| ApiError::ValidationError("Missing scan_id in metadata".to_string()))?;

    let scan_id = Uuid::parse_str(scan_id_str)
        .map_err(|e| ApiError::ValidationError(format!("Invalid scan_id UUID: {}", e)))?;

    let email = session["metadata"]["email"]
        .as_str()
        .ok_or_else(|| ApiError::ValidationError("Missing email in metadata".to_string()))?;

    // Extract payment_intent
    let payment_intent_id = session["payment_intent"].as_str();

    tracing::info!(
        "Processing checkout.session.completed for scan_id={}, email={}, session={}",
        scan_id,
        email,
        checkout_session_id
    );

    // Update paid_audit status to completed
    db::paid_audits::update_paid_audit_status(
        &state.pool,
        checkout_session_id,
        "completed",
        payment_intent_id,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update paid_audit status: {:?}", e);
        ApiError::InternalError("Failed to update payment status".to_string())
    })?;

    // Update scan tier to "paid"
    db::paid_audits::update_scan_tier(&state.pool, scan_id, "paid")
        .await
        .map_err(|e| {
            tracing::error!("Failed to update scan tier: {:?}", e);
            ApiError::InternalError("Failed to update scan tier".to_string())
        })?;

    // Spawn paid scan asynchronously (placeholder for Plan 03)
    // This will be wired to the orchestrator's paid scan method in Plan 03
    let _pool = state.pool.clone();
    tokio::spawn(async move {
        tracing::info!("Paid scan triggered for scan_id={}", scan_id);
        // TODO (Plan 03): Call orchestrator.spawn_paid_scan(scan_id, _pool)
        // For now, just log as placeholder
    });

    Ok(())
}
