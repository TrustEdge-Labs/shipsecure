use serde::Serialize;

/// A domain that a user has started or completed verification for.
///
/// Maps directly to the `verified_domains` table.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct VerifiedDomain {
    pub id: uuid::Uuid,
    pub clerk_user_id: String,
    pub domain: String,
    pub verification_token: String,
    pub status: String,
    pub verified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
