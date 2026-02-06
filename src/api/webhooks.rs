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

    // Spawn paid scan pipeline asynchronously
    let pool = state.pool.clone();
    let orchestrator = state.orchestrator.clone();
    tokio::spawn(async move {
        tracing::info!("Starting paid scan pipeline for scan_id={}", scan_id);

        // Get scan from database
        let scan = match crate::db::scans::get_scan(&pool, scan_id).await {
            Ok(Some(s)) => s,
            Ok(None) => {
                tracing::error!("Scan {} not found for paid audit", scan_id);
                return;
            }
            Err(e) => {
                tracing::error!("Failed to fetch scan {}: {:?}", scan_id, e);
                return;
            }
        };

        let target_url = scan.target_url.clone();
        let email = scan.email.clone();

        // Trigger paid scan via orchestrator
        tracing::info!("Spawning paid scan for scan_id={}, url={}", scan_id, target_url);
        orchestrator.spawn_paid_scan(scan_id, target_url.clone());

        // Poll database for completion (max 15 minutes)
        let max_attempts = 180; // 15 min / 5s = 180 attempts
        let mut completed = false;

        for attempt in 1..=max_attempts {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            match crate::db::scans::get_scan(&pool, scan_id).await {
                Ok(Some(updated_scan)) => {
                    if updated_scan.status == crate::models::scan::ScanStatus::Completed {
                        tracing::info!("Paid scan {} completed after {} attempts", scan_id, attempt);
                        completed = true;
                        break;
                    } else if updated_scan.status == crate::models::scan::ScanStatus::Failed {
                        tracing::error!("Paid scan {} failed", scan_id);
                        return;
                    }
                }
                Ok(None) => {
                    tracing::error!("Scan {} disappeared during polling", scan_id);
                    return;
                }
                Err(e) => {
                    tracing::warn!("Poll attempt {} for scan {} failed: {:?}", attempt, scan_id, e);
                }
            }
        }

        if !completed {
            tracing::error!("Paid scan {} timed out after 15 minutes", scan_id);
            return;
        }

        // Fetch completed scan and findings
        let scan = match crate::db::scans::get_scan(&pool, scan_id).await {
            Ok(Some(s)) => s,
            Ok(None) => {
                tracing::error!("Scan {} not found after completion", scan_id);
                return;
            }
            Err(e) => {
                tracing::error!("Failed to fetch completed scan {}: {:?}", scan_id, e);
                return;
            }
        };

        let findings = match crate::db::findings::get_findings_by_scan(&pool, scan_id).await {
            Ok(f) => f,
            Err(e) => {
                tracing::error!("Failed to fetch findings for scan {}: {:?}", scan_id, e);
                return;
            }
        };

        // Generate PDF report
        let grade = scan.score.as_deref().unwrap_or("N/A");
        let scan_date = scan.completed_at
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let framework = scan.detected_framework.as_deref();
        let platform = scan.detected_platform.as_deref();

        let pdf_bytes = match crate::pdf::generate_report(
            &target_url,
            grade,
            &scan_date,
            framework,
            platform,
            &findings,
        ) {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::error!("Failed to generate PDF for scan {}: {:?}", scan_id, e);
                return;
            }
        };

        // Mark PDF generated
        if let Err(e) = crate::db::paid_audits::mark_pdf_generated(&pool, scan_id).await {
            tracing::warn!("Failed to mark PDF generated for scan {}: {:?}", scan_id, e);
        }

        // Compute findings summary
        let summary = crate::email::FindingsSummary {
            critical: findings.iter().filter(|f| f.severity == crate::models::Severity::Critical).count() as i64,
            high: findings.iter().filter(|f| f.severity == crate::models::Severity::High).count() as i64,
            medium: findings.iter().filter(|f| f.severity == crate::models::Severity::Medium).count() as i64,
            low: findings.iter().filter(|f| f.severity == crate::models::Severity::Low).count() as i64,
            total: findings.len() as i64,
        };

        // Get results token and base URL
        let results_token = scan.results_token.as_deref().unwrap_or("");
        let base_url = std::env::var("TRUSTEDGE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:3001".to_string());

        // Format scan_id for filename (first 8 chars)
        let scan_id_str = scan_id.to_string();
        let short_scan_id = if scan_id_str.len() >= 8 {
            &scan_id_str[..8]
        } else {
            &scan_id_str
        };

        // Send paid audit email with PDF attachment
        if let Err(e) = crate::email::send_paid_audit_email(
            &email,
            &target_url,
            grade,
            &summary,
            results_token,
            &base_url,
            pdf_bytes,
            short_scan_id,
        ).await {
            tracing::error!("Failed to send paid audit email for scan {}: {:?}", scan_id, e);
            return;
        }

        tracing::info!("Paid scan pipeline completed successfully for scan_id={}", scan_id);
    });

    Ok(())
}
