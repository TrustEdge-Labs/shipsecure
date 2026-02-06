use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::api::errors::ApiError;
use crate::api::scans::AppState;
use crate::db;

#[derive(Debug, Deserialize)]
pub struct CreateCheckoutRequest {
    pub scan_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct CreateCheckoutResponse {
    pub checkout_url: String,
}

/// POST /api/v1/checkout - Create a Stripe Checkout Session for paid audit
pub async fn create_checkout(
    State(state): State<AppState>,
    Json(req): Json<CreateCheckoutRequest>,
) -> Result<(StatusCode, Json<CreateCheckoutResponse>), ApiError> {
    // 1. Load scan from database
    let scan = db::scans::get_scan(&state.pool, req.scan_id)
        .await?
        .ok_or_else(|| {
            ApiError::ValidationError(format!("Scan {} not found", req.scan_id))
        })?;

    // 2. Validate scan is completed
    if format!("{:?}", scan.status).to_lowercase() != "completed" {
        return Err(ApiError::ValidationError(
            "Scan must be completed before purchasing paid audit".to_string(),
        ));
    }

    // 3. Check if paid audit already exists
    if let Some(paid_audit) = db::paid_audits::get_paid_audit_by_scan_id(&state.pool, req.scan_id).await? {
        if paid_audit.status == "completed" {
            return Err(ApiError::Custom {
                status: StatusCode::CONFLICT,
                error_type: "https://trustedge.dev/errors/already-purchased".to_string(),
                title: "Paid Audit Already Purchased".to_string(),
                detail: "A paid audit has already been purchased for this scan.".to_string(),
            });
        }
    }

    // 4. Create Stripe client
    let stripe_secret_key = std::env::var("STRIPE_SECRET_KEY")
        .unwrap_or_default();

    if stripe_secret_key.is_empty() {
        return Err(ApiError::InternalError(
            "Stripe not configured".to_string(),
        ));
    }

    let client = stripe::Client::new(stripe_secret_key);

    // 5. Get frontend URL for redirect URLs
    let frontend_url = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:3001".to_string());

    // 6. Create CheckoutSession
    let mut params = stripe::CreateCheckoutSession::new();
    params.mode = Some(stripe::CheckoutSessionMode::Payment);
    params.customer_email = Some(&scan.email);

    // Set up line items
    let price_data = stripe::CreateCheckoutSessionLineItemsPriceData {
        currency: stripe::Currency::USD,
        product_data: Some(stripe::CreateCheckoutSessionLineItemsPriceDataProductData {
            name: "TrustEdge Deep Security Audit".to_string(),
            ..Default::default()
        }),
        unit_amount: Some(4900), // $49.00
        ..Default::default()
    };

    let line_item = stripe::CreateCheckoutSessionLineItems {
        price_data: Some(price_data),
        quantity: Some(1),
        ..Default::default()
    };

    params.line_items = Some(vec![line_item]);

    // Set redirect URLs (need owned strings for lifetime)
    let success_url = format!("{}/payment/success?session_id={{CHECKOUT_SESSION_ID}}", frontend_url);
    let cancel_url = format!("{}/results/{}", frontend_url, scan.results_token.as_deref().unwrap_or(""));
    params.success_url = Some(&success_url);
    params.cancel_url = Some(&cancel_url);

    // Set metadata
    let mut metadata = HashMap::new();
    metadata.insert("scan_id".to_string(), req.scan_id.to_string());
    metadata.insert("email".to_string(), scan.email.clone());
    params.metadata = Some(metadata);

    let session = stripe::CheckoutSession::create(&client, params)
        .await
        .map_err(|e| {
            tracing::error!("Stripe checkout session creation failed: {:?}", e);
            ApiError::InternalError(format!("Failed to create checkout session: {}", e))
        })?;

    // 7. Create paid_audit record with pending status
    db::paid_audits::create_paid_audit(
        &state.pool,
        req.scan_id,
        &session.id.to_string(),
        4900,
        &scan.email,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create paid_audit record: {:?}", e);
        ApiError::InternalError("Failed to record checkout session".to_string())
    })?;

    // 8. Return checkout URL
    let checkout_url = session.url.ok_or_else(|| {
        tracing::error!("Stripe session created but no URL returned");
        ApiError::InternalError("Checkout session created but no URL available".to_string())
    })?;

    Ok((
        StatusCode::OK,
        Json(CreateCheckoutResponse { checkout_url }),
    ))
}
