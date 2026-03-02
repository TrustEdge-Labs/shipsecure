use chrono::{Datelike, TimeZone, Utc};
use sqlx::PgPool;

use crate::api::errors::ApiError;
use crate::db::scans;

/// Hard cap on anonymous scans from a single IP per day, regardless of email.
/// Prevents abuse via rotating email addresses.
const ANONYMOUS_IP_DAILY_HARD_CAP: i64 = 10;

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
/// Anonymous scans have two layers:
/// 1. Fair-use: 1 scan per email+domain per day (so users can scan multiple apps)
/// 2. Abuse prevention: hard cap of 10 scans per IP per day (stops email rotation attacks)
///
/// Authenticated: 5 scans per calendar month
pub async fn check_rate_limits(
    pool: &PgPool,
    clerk_user_id: Option<&str>,
    email: &str,
    target_domain: &str,
    ip: &str,
) -> Result<(), ApiError> {
    match clerk_user_id {
        None => {
            // Layer 1: IP-based hard cap (abuse prevention)
            let ip_count = scans::count_anonymous_scans_by_ip_today(pool, ip).await?;
            if ip_count >= ANONYMOUS_IP_DAILY_HARD_CAP {
                let resets_at = next_midnight_utc();
                metrics::counter!(
                    "rate_limit_total",
                    "limiter" => "scan_ip_hard_cap",
                    "action" => "blocked"
                )
                .increment(1);
                return Err(ApiError::RateLimitedWithReset {
                    message: "Too many scans from this network today. Please try again tomorrow."
                        .to_string(),
                    resets_at,
                });
            }

            // Layer 2: email+domain fair-use limit
            let count =
                scans::count_anonymous_scans_by_email_and_domain_today(pool, email, target_domain)
                    .await?;
            if count >= 1 {
                let resets_at = next_midnight_utc();
                metrics::counter!(
                    "rate_limit_total",
                    "limiter" => "scan_email_domain",
                    "action" => "blocked"
                )
                .increment(1);
                return Err(ApiError::RateLimitedWithReset {
                    message: format!(
                        "You've already scanned {} today. Try again tomorrow, or use a different email to scan again now.",
                        target_domain
                    ),
                    resets_at,
                });
            }
        }
        Some(user_id) => {
            // Authenticated Developer: 5 scans per calendar month
            let count = scans::count_scans_by_user_this_month(pool, user_id).await?;
            if count >= 5 {
                let resets_at = first_of_next_month_utc();
                metrics::counter!(
                    "rate_limit_total",
                    "limiter" => "scan_user",
                    "action" => "blocked"
                )
                .increment(1);
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
