use chrono::{Datelike, TimeZone, Utc};
use sqlx::PgPool;

use crate::api::errors::ApiError;
use crate::db::scans;

fn next_midnight_utc() -> chrono::DateTime<chrono::Utc> {
    let now = Utc::now();
    let tomorrow = now.date_naive().succ_opt().unwrap();
    let tomorrow_midnight = tomorrow.and_hms_opt(0, 0, 0).unwrap();
    chrono::DateTime::from_naive_utc_and_offset(tomorrow_midnight, chrono::Utc)
}

fn first_of_next_month_utc() -> chrono::DateTime<chrono::Utc> {
    let now = Utc::now();
    let (year, month) = if now.month() == 12 {
        (now.year() + 1, 1u32)
    } else {
        (now.year(), now.month() + 1)
    };
    Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0).unwrap()
}

/// Check rate limits based on caller identity.
///
/// - Anonymous (clerk_user_id is None): 1 scan per IP per 24 hours
/// - Authenticated Developer: 5 scans per calendar month
pub async fn check_rate_limits(
    pool: &PgPool,
    clerk_user_id: Option<&str>,
    ip: &str,
) -> Result<(), ApiError> {
    match clerk_user_id {
        None => {
            // Anonymous: 1 scan per IP per 24h
            let count = scans::count_anonymous_scans_by_ip_today(pool, ip).await?;
            if count >= 1 {
                let resets_at = next_midnight_utc();
                metrics::counter!(
                    "rate_limit_total",
                    "limiter" => "scan_ip",
                    "action" => "blocked"
                ).increment(1);
                return Err(ApiError::RateLimitedWithReset {
                    message: "You've used your free scan for today (1 per day per IP address).".to_string(),
                    resets_at,
                });
            }
        }
        Some(user_id) => {
            // Authenticated Developer: 5 scans per calendar month
            let count = scans::count_scans_by_user_this_month(pool, user_id).await?;
            // TODO: Developer-tier limit. When Pro tier is added, gate this on the user's tier.
            if count >= 5 {
                let resets_at = first_of_next_month_utc();
                metrics::counter!(
                    "rate_limit_total",
                    "limiter" => "scan_user",
                    "action" => "blocked"
                ).increment(1);
                return Err(ApiError::RateLimitedWithReset {
                    message: format!("You've used all 5 scans for this month ({} of 5).", count),
                    resets_at,
                });
            }
        }
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
