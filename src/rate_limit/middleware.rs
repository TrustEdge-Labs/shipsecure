use sqlx::PgPool;

use crate::api::errors::ApiError;
use crate::db::scans;

/// Check rate limits for both email and IP address.
///
/// Returns Ok(()) if limits are not exceeded, or ApiError::RateLimited if exceeded.
///
/// Rate limits:
/// - 3 scans per email per day
/// - 10 scans per IP address per day
pub async fn check_rate_limits(
    pool: &PgPool,
    email: &str,
    ip: &str,
) -> Result<(), ApiError> {
    // Check email-based rate limit (3 scans per day)
    let email_count = scans::count_scans_by_email_today(pool, email)
        .await?;

    if email_count >= 3 {
        return Err(ApiError::RateLimited(
            "You've reached your daily scan limit of 3 scans per email. Try again tomorrow.".to_string()
        ));
    }

    // Check IP-based rate limit (10 scans per day)
    let ip_count = scans::count_scans_by_ip_today(pool, ip)
        .await?;

    if ip_count >= 10 {
        return Err(ApiError::RateLimited(
            "You've reached your daily scan limit of 10 scans per IP address. Try again tomorrow.".to_string()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Note: These tests require a running PostgreSQL database
    // For now, they're marked as ignored and can be run manually
    // with `cargo test -- --ignored`

    #[ignore]
    #[tokio::test]
    async fn test_rate_limit_check() {
        // This would require a test database setup
        // Leaving as a placeholder for integration tests
    }
}
