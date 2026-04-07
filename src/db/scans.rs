use sqlx::PgPool;
use uuid::Uuid;

use crate::models::scan::{Scan, ScanStatus};

/// Projection row for scan history queries — lives here (not models/) as it is a query result type.
#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct ScanHistoryRow {
    pub id: uuid::Uuid,
    pub target_url: String,
    pub status: String,
    pub results_token: Option<String>,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub tier: String,
    pub created_at: chrono::NaiveDateTime,
    pub critical_count: i64,
    pub high_count: i64,
    pub medium_count: i64,
    pub low_count: i64,
}

/// Create a new scan record
#[allow(dead_code)]
pub async fn create_scan(
    pool: &PgPool,
    target_url: &str,
    email: &str,
    submitter_ip: Option<&str>,
    request_id: Option<Uuid>,
    tier: &str,
    clerk_user_id: Option<&str>,
) -> Result<Scan, sqlx::Error> {
    let scan = sqlx::query_as::<_, Scan>(
        "INSERT INTO scans (target_url, email, submitter_ip, request_id, tier, clerk_user_id)
         VALUES ($1, $2, $3::inet, $4, $5, $6)
         RETURNING id, target_url, email, submitter_ip::text, request_id, status, score, results_token,
                   expires_at::timestamp, detected_framework, detected_platform,
                   stage_headers, stage_tls, stage_files, stage_secrets, stage_detection, stage_vibecode,
                   tier, error_message, started_at::timestamp, completed_at::timestamp, created_at::timestamp,
                   clerk_user_id, kind, supply_chain_results"
    )
    .bind(target_url)
    .bind(email)
    .bind(submitter_ip)
    .bind(request_id)
    .bind(tier)
    .bind(clerk_user_id)
    .fetch_one(pool)
    .await?;

    Ok(scan)
}

/// Get a scan by ID
#[allow(dead_code)]
pub async fn get_scan(pool: &PgPool, id: Uuid) -> Result<Option<Scan>, sqlx::Error> {
    let scan = sqlx::query_as::<_, Scan>(
        "SELECT id, target_url, email, submitter_ip::text, request_id, status, score, results_token,
                expires_at::timestamp, detected_framework, detected_platform,
                stage_headers, stage_tls, stage_files, stage_secrets, stage_detection, stage_vibecode,
                tier, error_message, started_at::timestamp, completed_at::timestamp, created_at::timestamp,
                clerk_user_id, kind, supply_chain_results
         FROM scans WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(scan)
}

/// Claim a pending scan for processing using SELECT FOR UPDATE SKIP LOCKED
/// This ensures only one worker processes each scan
#[allow(dead_code)]
pub async fn claim_pending_scan(pool: &PgPool) -> Result<Option<Scan>, sqlx::Error> {
    let scan = sqlx::query_as::<_, Scan>(
        "UPDATE scans
         SET status = 'in_progress', started_at = NOW()
         WHERE id = (
             SELECT id FROM scans
             WHERE status = 'pending'
             ORDER BY created_at ASC
             FOR UPDATE SKIP LOCKED
             LIMIT 1
         )
         RETURNING id, target_url, email, submitter_ip::text, request_id, status, score, results_token,
                   expires_at::timestamp, detected_framework, detected_platform,
                   stage_headers, stage_tls, stage_files, stage_secrets, stage_detection, stage_vibecode,
                   tier, error_message, started_at::timestamp, completed_at::timestamp, created_at::timestamp,
                   clerk_user_id, kind, supply_chain_results"
    )
    .fetch_optional(pool)
    .await?;

    Ok(scan)
}

/// Update scan status and optionally set score or error message
#[allow(dead_code)]
pub async fn update_scan_status(
    pool: &PgPool,
    id: Uuid,
    status: ScanStatus,
    score: Option<String>,
    error_message: Option<String>,
) -> Result<(), sqlx::Error> {
    // Set completed_at when scan is finished (completed or failed)
    let completed_at = match status {
        ScanStatus::Completed | ScanStatus::Failed => Some("NOW()"),
        _ => None,
    };

    if completed_at.is_some() {
        sqlx::query(
            "UPDATE scans
             SET status = $1, score = $2, error_message = $3, completed_at = NOW()
             WHERE id = $4",
        )
        .bind(status)
        .bind(score)
        .bind(error_message)
        .bind(id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            "UPDATE scans
             SET status = $1, score = $2, error_message = $3
             WHERE id = $4",
        )
        .bind(status)
        .bind(score)
        .bind(error_message)
        .bind(id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Count scans submitted by email address today (UTC)
#[allow(dead_code)]
pub async fn count_scans_by_email_today(pool: &PgPool, email: &str) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)
         FROM scans
         WHERE email = $1 AND created_at >= CURRENT_DATE",
    )
    .bind(email)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

/// Count anonymous scans by email + target domain today (UTC).
///
/// Allows the same email to scan different domains once each per day.
/// Domain is matched via LIKE on the target_url column (scheme://domain/...).
#[allow(dead_code)]
pub async fn count_anonymous_scans_by_email_and_domain_today(
    pool: &PgPool,
    email: &str,
    domain: &str,
) -> Result<i64, sqlx::Error> {
    // Match URLs containing the domain after the scheme (e.g., https://example.com/...)
    let pattern = format!("%://{domain}%");
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)
         FROM scans
         WHERE email = $1
           AND target_url LIKE $2
           AND clerk_user_id IS NULL
           AND created_at >= CURRENT_DATE",
    )
    .bind(email)
    .bind(&pattern)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

/// Count scans submitted by IP address today (UTC)
#[allow(dead_code)]
pub async fn count_scans_by_ip_today(pool: &PgPool, ip: &str) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)
         FROM scans
         WHERE submitter_ip = $1::inet AND created_at >= CURRENT_DATE",
    )
    .bind(ip)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

/// Get a scan by results token (checks expiry)
#[allow(dead_code)]
pub async fn get_scan_by_token(pool: &PgPool, token: &str) -> Result<Option<Scan>, sqlx::Error> {
    let scan = sqlx::query_as::<_, Scan>(
        "SELECT id, target_url, email, submitter_ip::text, request_id, status, score, results_token,
                expires_at::timestamp, detected_framework, detected_platform,
                stage_headers, stage_tls, stage_files, stage_secrets, stage_detection, stage_vibecode,
                tier, error_message, started_at::timestamp, completed_at::timestamp, created_at::timestamp,
                clerk_user_id, kind, supply_chain_results
         FROM scans
         WHERE results_token = $1 AND (expires_at IS NULL OR expires_at > NOW())"
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(scan)
}

/// Update a specific scan stage completion status
#[allow(dead_code)]
pub async fn update_scan_stage(
    pool: &PgPool,
    scan_id: Uuid,
    stage_name: &str,
    completed: bool,
) -> Result<(), sqlx::Error> {
    let query = match stage_name {
        "headers" => "UPDATE scans SET stage_headers = $1 WHERE id = $2",
        "tls" => "UPDATE scans SET stage_tls = $1 WHERE id = $2",
        "files" => "UPDATE scans SET stage_files = $1 WHERE id = $2",
        "secrets" => "UPDATE scans SET stage_secrets = $1 WHERE id = $2",
        "detection" => "UPDATE scans SET stage_detection = $1 WHERE id = $2",
        "vibecode" => "UPDATE scans SET stage_vibecode = $1 WHERE id = $2",
        _ => return Err(sqlx::Error::RowNotFound),
    };

    sqlx::query(query)
        .bind(completed)
        .bind(scan_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Set the results token and expiry for a scan
#[allow(dead_code)]
pub async fn set_results_token(
    pool: &PgPool,
    scan_id: Uuid,
    token: &str,
    expires_at: chrono::NaiveDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE scans
         SET results_token = $1, expires_at = $2
         WHERE id = $3",
    )
    .bind(token)
    .bind(expires_at)
    .bind(scan_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Count total completed scans (for social proof counter)
#[allow(dead_code)]
pub async fn count_completed_scans(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)
         FROM scans
         WHERE status = 'completed'
           AND kind = 'web_app'",
    )
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

/// Update detected framework for a scan
#[allow(dead_code)]
pub async fn update_detected_framework(
    pool: &PgPool,
    scan_id: Uuid,
    framework: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE scans
         SET detected_framework = $1
         WHERE id = $2",
    )
    .bind(framework)
    .bind(scan_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Update detected platform for a scan
#[allow(dead_code)]
pub async fn update_detected_platform(
    pool: &PgPool,
    scan_id: Uuid,
    platform: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE scans
         SET detected_platform = $1
         WHERE id = $2",
    )
    .bind(platform)
    .bind(scan_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Count anonymous scans from a specific IP address today (UTC).
///
/// Filters to only anonymous scans (clerk_user_id IS NULL) so authenticated scans from the
/// same IP do not inflate the anonymous rate limit.
#[allow(dead_code)]
pub async fn count_anonymous_scans_by_ip_today(
    pool: &PgPool,
    ip: &str,
) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)
         FROM scans
         WHERE submitter_ip = $1::inet
           AND clerk_user_id IS NULL
           AND created_at >= CURRENT_DATE",
    )
    .bind(ip)
    .fetch_one(pool)
    .await?;
    Ok(count.0)
}

/// Count all scans (anonymous + authenticated) targeting a given domain in the last hour.
///
/// Used for per-target rate limiting: if the same domain is scanned 5+ times in an hour,
/// return cached results instead of re-scanning.
#[allow(dead_code)]
pub async fn count_scans_by_domain_last_hour(
    pool: &PgPool,
    domain: &str,
) -> Result<i64, sqlx::Error> {
    let pattern = format!("%://{domain}%");
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)
         FROM scans
         WHERE target_url LIKE $1
           AND kind = 'web_app'
           AND created_at >= NOW() - INTERVAL '1 hour'",
    )
    .bind(&pattern)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

/// Get the most recent non-expired completed scan for a given domain.
///
/// Used to return cached results when the per-target hourly cap is exceeded.
/// Returns None if no completed scan exists or all are expired.
#[allow(dead_code)]
pub async fn get_recent_completed_scan_for_domain(
    pool: &PgPool,
    domain: &str,
) -> Result<Option<Scan>, sqlx::Error> {
    let pattern = format!("%://{domain}%");
    let scan = sqlx::query_as::<_, Scan>(
        "SELECT id, target_url, email, submitter_ip::text, request_id, status, score, results_token,
                expires_at::timestamp, detected_framework, detected_platform,
                stage_headers, stage_tls, stage_files, stage_secrets, stage_detection, stage_vibecode,
                tier, error_message, started_at::timestamp, completed_at::timestamp, created_at::timestamp,
                clerk_user_id, kind, supply_chain_results
         FROM scans
         WHERE target_url LIKE $1
           AND kind = 'web_app'
           AND status = 'completed'
           AND (expires_at IS NULL OR expires_at > NOW())
         ORDER BY completed_at DESC
         LIMIT 1",
    )
    .bind(&pattern)
    .fetch_optional(pool)
    .await?;

    Ok(scan)
}

/// Count scans submitted by a Clerk user in the current calendar month (UTC).
///
/// Used for the Developer tier monthly quota (5 scans/month).
#[allow(dead_code)]
pub async fn count_scans_by_user_this_month(
    pool: &PgPool,
    clerk_user_id: &str,
) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)
         FROM scans
         WHERE clerk_user_id = $1
           AND kind = 'web_app'
           AND created_at >= DATE_TRUNC('month', NOW() AT TIME ZONE 'UTC')",
    )
    .bind(clerk_user_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

/// Paginated query for completed/failed scans for a given user, with severity counts via LEFT JOIN.
///
/// Sorted by expiring soonest first (per locked decision): non-null expires_at ASC, then created_at DESC.
#[allow(dead_code)]
pub async fn get_user_scan_history(
    pool: &PgPool,
    clerk_user_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<ScanHistoryRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ScanHistoryRow>(
        "SELECT s.id,
                s.target_url,
                s.status::text AS status,
                s.results_token,
                s.expires_at::timestamp AS expires_at,
                s.tier,
                s.created_at::timestamp AS created_at,
                COUNT(CASE WHEN f.severity = 'critical' THEN 1 END) AS critical_count,
                COUNT(CASE WHEN f.severity = 'high'     THEN 1 END) AS high_count,
                COUNT(CASE WHEN f.severity = 'medium'   THEN 1 END) AS medium_count,
                COUNT(CASE WHEN f.severity = 'low'      THEN 1 END) AS low_count
         FROM scans s
         LEFT JOIN findings f ON f.scan_id = s.id
         WHERE s.clerk_user_id = $1
           AND s.kind = 'web_app'
           AND s.status NOT IN ('pending', 'in_progress')
         GROUP BY s.id, s.target_url, s.status, s.results_token, s.expires_at, s.tier, s.created_at
         ORDER BY CASE WHEN s.expires_at IS NULL THEN 1 ELSE 0 END ASC,
                  s.expires_at ASC NULLS LAST,
                  s.created_at DESC
         LIMIT $2 OFFSET $3",
    )
    .bind(clerk_user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Total count of completed/failed scans for a user — used for pagination metadata.
#[allow(dead_code)]
pub async fn count_user_scans_history(
    pool: &PgPool,
    clerk_user_id: &str,
) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)
         FROM scans
         WHERE clerk_user_id = $1
           AND kind = 'web_app'
           AND status NOT IN ('pending', 'in_progress')",
    )
    .bind(clerk_user_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

/// Delete expired completed/failed scans for a specific tier.
///
/// Returns the number of rows deleted.
/// Only targets status IN ('completed', 'failed') — never deletes pending/in_progress.
/// Applies 24-hour grace period: expires_at + INTERVAL '24 hours' < NOW().
/// findings rows are CASCADE deleted by PostgreSQL automatically.
/// paid_audits.scan_id is SET NULL by PostgreSQL automatically.
pub async fn delete_expired_scans_by_tier(pool: &PgPool, tier: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM scans
         WHERE tier = $1
           AND status IN ('completed', 'failed')
           AND expires_at + INTERVAL '24 hours' < NOW()",
    )
    .bind(tier)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Soft-expire completed/failed scans for a specific tier by setting status to 'expired'.
///
/// Returns the number of rows updated.
/// Marks expired as soon as expires_at passes — no grace period (soft-expire is non-destructive).
/// Only targets status IN ('completed', 'failed') — never modifies pending/in_progress.
pub async fn soft_expire_scans_by_tier(pool: &PgPool, tier: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE scans
         SET status = 'expired'
         WHERE tier = $1
           AND status IN ('completed', 'failed')
           AND expires_at IS NOT NULL
           AND expires_at < NOW()",
    )
    .bind(tier)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Get a scan by results token regardless of expiry status (including expired scans).
///
/// Used as a fallback in the results endpoint when the primary token lookup returns None,
/// allowing the frontend to render a dedicated "results expired" page with the original
/// target_url pre-filled in a scan-again CTA.
#[allow(dead_code)]
pub async fn get_scan_by_token_including_expired(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Scan>, sqlx::Error> {
    let scan = sqlx::query_as::<_, Scan>(
        "SELECT id, target_url, email, submitter_ip::text, request_id, status, score, results_token,
                expires_at::timestamp, detected_framework, detected_platform,
                stage_headers, stage_tls, stage_files, stage_secrets, stage_detection, stage_vibecode,
                tier, error_message, started_at::timestamp, completed_at::timestamp, created_at::timestamp,
                clerk_user_id, kind, supply_chain_results
         FROM scans
         WHERE results_token = $1",
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(scan)
}

/// Non-paginated list of in-progress/pending scans for a user.
///
/// Active scans have no findings yet; severity counts are hardcoded to zero.
#[allow(dead_code)]
pub async fn get_user_active_scans(
    pool: &PgPool,
    clerk_user_id: &str,
) -> Result<Vec<ScanHistoryRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ScanHistoryRow>(
        "SELECT s.id,
                s.target_url,
                s.status::text AS status,
                s.results_token,
                s.expires_at::timestamp AS expires_at,
                s.tier,
                s.created_at::timestamp AS created_at,
                0::bigint AS critical_count,
                0::bigint AS high_count,
                0::bigint AS medium_count,
                0::bigint AS low_count
         FROM scans s
         WHERE s.clerk_user_id = $1
           AND s.kind = 'web_app'
           AND s.status IN ('pending', 'in_progress')
         ORDER BY s.created_at DESC",
    )
    .bind(clerk_user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Insert a supply chain scan row.
///
/// Sets kind='supply_chain', expires_at=NOW()+30 days.
/// Returns the created Scan (with results_token and supply_chain_results NULL initially).
#[allow(dead_code)]
pub async fn create_supply_chain_scan(
    pool: &PgPool,
    target_url: &str,
    submitter_ip: Option<&str>,
    request_id: Option<Uuid>,
    clerk_user_id: Option<&str>,
) -> Result<Scan, sqlx::Error> {
    let tier = if clerk_user_id.is_some() {
        "authenticated"
    } else {
        "free"
    };

    let scan = sqlx::query_as::<_, Scan>(
        "INSERT INTO scans (target_url, email, submitter_ip, request_id, tier, clerk_user_id, kind, expires_at)
         VALUES ($1, '', $2::inet, $3, $4, $5, 'supply_chain', NOW() + INTERVAL '30 days')
         RETURNING id, target_url, email, submitter_ip::text, request_id, status, score, results_token,
                   expires_at::timestamp, detected_framework, detected_platform,
                   stage_headers, stage_tls, stage_files, stage_secrets, stage_detection, stage_vibecode,
                   tier, error_message, started_at::timestamp, completed_at::timestamp, created_at::timestamp,
                   clerk_user_id, kind, supply_chain_results"
    )
    .bind(target_url)
    .bind(submitter_ip)
    .bind(request_id)
    .bind(tier)
    .bind(clerk_user_id)
    .fetch_one(pool)
    .await?;

    Ok(scan)
}

/// Update a supply chain scan with results and token after successful scan.
#[allow(dead_code)]
pub async fn complete_supply_chain_scan(
    pool: &PgPool,
    scan_id: Uuid,
    results: &serde_json::Value,
    token: &str,
    expires_at: chrono::NaiveDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE scans
         SET status = 'completed',
             supply_chain_results = $1,
             results_token = $2,
             expires_at = $3,
             completed_at = NOW()
         WHERE id = $4",
    )
    .bind(results)
    .bind(token)
    .bind(expires_at)
    .bind(scan_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Mark a supply chain scan as failed.
#[allow(dead_code)]
pub async fn fail_supply_chain_scan(
    pool: &PgPool,
    scan_id: Uuid,
    error_message: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE scans
         SET status = 'failed',
             error_message = $1,
             completed_at = NOW()
         WHERE id = $2",
    )
    .bind(error_message)
    .bind(scan_id)
    .execute(pool)
    .await?;

    Ok(())
}
