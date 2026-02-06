use sqlx::PgPool;
use uuid::Uuid;

use crate::models::paid_audit::PaidAudit;

/// Create a new paid audit record
#[allow(dead_code)]
pub async fn create_paid_audit(
    pool: &PgPool,
    scan_id: Uuid,
    checkout_session_id: &str,
    amount_cents: i32,
    customer_email: &str,
) -> Result<PaidAudit, sqlx::Error> {
    let paid_audit = sqlx::query_as::<_, PaidAudit>(
        "INSERT INTO paid_audits (scan_id, stripe_checkout_session_id, amount_cents, customer_email)
         VALUES ($1, $2, $3, $4)
         RETURNING id, scan_id, stripe_checkout_session_id, stripe_payment_intent_id,
                   amount_cents, currency, customer_email, status,
                   pdf_generated_at::timestamp, created_at::timestamp, updated_at::timestamp"
    )
    .bind(scan_id)
    .bind(checkout_session_id)
    .bind(amount_cents)
    .bind(customer_email)
    .fetch_one(pool)
    .await?;

    Ok(paid_audit)
}

/// Update paid audit status and optionally set payment intent ID
#[allow(dead_code)]
pub async fn update_paid_audit_status(
    pool: &PgPool,
    checkout_session_id: &str,
    status: &str,
    payment_intent_id: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE paid_audits
         SET status = $1, stripe_payment_intent_id = $2, updated_at = NOW()
         WHERE stripe_checkout_session_id = $3"
    )
    .bind(status)
    .bind(payment_intent_id)
    .bind(checkout_session_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get paid audit by scan ID
#[allow(dead_code)]
pub async fn get_paid_audit_by_scan_id(
    pool: &PgPool,
    scan_id: Uuid,
) -> Result<Option<PaidAudit>, sqlx::Error> {
    let paid_audit = sqlx::query_as::<_, PaidAudit>(
        "SELECT id, scan_id, stripe_checkout_session_id, stripe_payment_intent_id,
                amount_cents, currency, customer_email, status,
                pdf_generated_at::timestamp, created_at::timestamp, updated_at::timestamp
         FROM paid_audits
         WHERE scan_id = $1"
    )
    .bind(scan_id)
    .fetch_optional(pool)
    .await?;

    Ok(paid_audit)
}

/// Check if a Stripe event has been processed and mark it if new
/// Returns true if this is a new event (not duplicate), false if already processed
#[allow(dead_code)]
pub async fn check_and_mark_event(
    pool: &PgPool,
    event_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO stripe_events (event_id)
         VALUES ($1)
         ON CONFLICT (event_id) DO NOTHING"
    )
    .bind(event_id)
    .execute(pool)
    .await?;

    // If rows_affected is 1, this was a new event
    // If rows_affected is 0, this was a duplicate
    Ok(result.rows_affected() == 1)
}

/// Mark a PDF as generated for a paid audit
#[allow(dead_code)]
pub async fn mark_pdf_generated(
    pool: &PgPool,
    scan_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE paid_audits
         SET pdf_generated_at = NOW(), updated_at = NOW()
         WHERE scan_id = $1"
    )
    .bind(scan_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Update scan tier (free or paid)
#[allow(dead_code)]
pub async fn update_scan_tier(
    pool: &PgPool,
    scan_id: Uuid,
    tier: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE scans
         SET tier = $1
         WHERE id = $2"
    )
    .bind(tier)
    .bind(scan_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Clear all findings for a scan (used before paid rescan)
#[allow(dead_code)]
pub async fn clear_findings_by_scan(
    pool: &PgPool,
    scan_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM findings WHERE scan_id = $1")
        .bind(scan_id)
        .execute(pool)
        .await?;

    Ok(())
}
