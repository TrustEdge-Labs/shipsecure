use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::models::VerifiedDomain;

/// Get a verified_domains record by (clerk_user_id, domain).
///
/// Returns None if no record exists for this user+domain combination.
pub async fn get_verified_domain(
    pool: &PgPool,
    clerk_user_id: &str,
    domain: &str,
) -> Result<Option<VerifiedDomain>, sqlx::Error> {
    let record = sqlx::query_as::<_, VerifiedDomain>(
        "SELECT id, clerk_user_id, domain, verification_token, status, \
         verified_at, expires_at, created_at, updated_at \
         FROM verified_domains \
         WHERE clerk_user_id = $1 AND domain = $2",
    )
    .bind(clerk_user_id)
    .bind(domain)
    .fetch_optional(pool)
    .await?;

    Ok(record)
}

/// Upsert a pending domain verification record.
///
/// On conflict (clerk_user_id, domain), resets the token, status, and timestamps.
/// Callers must check for already-verified non-expired domains BEFORE calling this.
pub async fn upsert_pending_domain(
    pool: &PgPool,
    clerk_user_id: &str,
    domain: &str,
    token: &str,
) -> Result<VerifiedDomain, sqlx::Error> {
    let record = sqlx::query_as::<_, VerifiedDomain>(
        "INSERT INTO verified_domains \
         (clerk_user_id, domain, verification_token, status, verified_at, expires_at) \
         VALUES ($1, $2, $3, 'pending', NULL, NULL) \
         ON CONFLICT (clerk_user_id, domain) DO UPDATE SET \
           verification_token = EXCLUDED.verification_token, \
           status = 'pending', \
           verified_at = NULL, \
           expires_at = NULL, \
           updated_at = NOW() \
         RETURNING id, clerk_user_id, domain, verification_token, status, \
                   verified_at, expires_at, created_at, updated_at",
    )
    .bind(clerk_user_id)
    .bind(domain)
    .bind(token)
    .fetch_one(pool)
    .await?;

    Ok(record)
}

/// Mark a domain as verified with the given expiry timestamp.
///
/// Sets status = 'verified', verified_at = NOW(), and expires_at = provided value.
pub async fn mark_verified(
    pool: &PgPool,
    clerk_user_id: &str,
    domain: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE verified_domains \
         SET status = 'verified', verified_at = NOW(), expires_at = $3, updated_at = NOW() \
         WHERE clerk_user_id = $1 AND domain = $2",
    )
    .bind(clerk_user_id)
    .bind(domain)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// List all verified_domains records for a user, ordered by creation time descending.
pub async fn list_user_domains(
    pool: &PgPool,
    clerk_user_id: &str,
) -> Result<Vec<VerifiedDomain>, sqlx::Error> {
    let records = sqlx::query_as::<_, VerifiedDomain>(
        "SELECT id, clerk_user_id, domain, verification_token, status, \
         verified_at, expires_at, created_at, updated_at \
         FROM verified_domains \
         WHERE clerk_user_id = $1 \
         ORDER BY created_at DESC",
    )
    .bind(clerk_user_id)
    .fetch_all(pool)
    .await?;

    Ok(records)
}

/// Check whether a user has an active (non-expired, verified) domain record.
///
/// This is the performance-critical function called from results.rs on every
/// authenticated result fetch. Uses an EXISTS query for minimal overhead.
pub async fn is_domain_verified(
    pool: &PgPool,
    clerk_user_id: &str,
    domain: &str,
) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(\
           SELECT 1 FROM verified_domains \
           WHERE clerk_user_id = $1 \
             AND domain = $2 \
             AND status = 'verified' \
             AND expires_at > NOW()\
         )",
    )
    .bind(clerk_user_id)
    .bind(domain)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}
