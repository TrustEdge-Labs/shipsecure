use chrono::{Datelike, TimeZone, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::errors::ApiError;
use crate::db::scans;

/// Hard cap on anonymous scans from a single IP per day, regardless of email.
/// Prevents abuse via rotating email addresses.
const ANONYMOUS_IP_DAILY_HARD_CAP: i64 = 3;

/// Per-target hourly cap: if the same domain is scanned this many times in an hour,
/// return cached results for the most recent completed scan transparently.
const PER_TARGET_HOURLY_CAP: i64 = 5;

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
/// Returns `Ok(None)` to proceed with a new scan, or `Ok(Some(scan_id))` to return
/// cached results for an existing scan (per-target cache hit).
///
/// Two layers for anonymous callers:
/// 1. Per-target: 5 scans of the same domain per hour — returns cached results when exceeded
/// 2. Per-IP hard cap: 3 scans per IP per day (abuse prevention, stops email rotation)
///
/// Authenticated: per-target cache also applies; 5 scans per calendar month quota.
pub async fn check_rate_limits(
    pool: &PgPool,
    clerk_user_id: Option<&str>,
    _email: &str,
    target_domain: &str,
    ip: &str,
) -> Result<Option<Uuid>, ApiError> {
    // Per-target rate limit: 5 scans of same domain in last hour (applies to all callers)
    let target_count = scans::count_scans_by_domain_last_hour(pool, target_domain).await?;
    if target_count >= PER_TARGET_HOURLY_CAP
        && let Some(cached_scan) =
            scans::get_recent_completed_scan_for_domain(pool, target_domain).await?
    {
        metrics::counter!(
            "rate_limit_total",
            "limiter" => "scan_per_target",
            "action" => "cached"
        )
        .increment(1);
        return Ok(Some(cached_scan.id));
    }

    match clerk_user_id {
        None => {
            // IP hard cap (abuse prevention)
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
                    message: "You've used your 3 free scans today. Sign up for 5 scans/month and scan history."
                        .to_string(),
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
    Ok(None)
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
