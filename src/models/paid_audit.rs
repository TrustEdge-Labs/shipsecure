use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum PaidAuditStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PaidAudit {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub stripe_checkout_session_id: String,
    pub stripe_payment_intent_id: Option<String>,
    pub amount_cents: i32,
    pub currency: String,
    pub customer_email: String,
    pub status: String,
    pub pdf_generated_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct StripeEvent {
    pub event_id: String,
    pub processed_at: NaiveDateTime,
}
