use sqlx::PgPool;
use uuid::Uuid;

use crate::models::scan::{Scan, ScanStatus};

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
                   clerk_user_id"
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
                clerk_user_id
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
                   clerk_user_id"
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

    if let Some(_) = completed_at {
        sqlx::query(
            "UPDATE scans
             SET status = $1, score = $2, error_message = $3, completed_at = NOW()
             WHERE id = $4"
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
             WHERE id = $4"
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
         WHERE email = $1 AND created_at >= CURRENT_DATE"
    )
    .bind(email)
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
         WHERE submitter_ip = $1::inet AND created_at >= CURRENT_DATE"
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
                clerk_user_id
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
         WHERE id = $3"
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
         WHERE status = 'completed'"
    )
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

/// Update detected framework for a scan
#[allow(dead_code)]
pub async fn update_detected_framework(pool: &PgPool, scan_id: Uuid, framework: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE scans
         SET detected_framework = $1
         WHERE id = $2"
    )
    .bind(framework)
    .bind(scan_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Update detected platform for a scan
#[allow(dead_code)]
pub async fn update_detected_platform(pool: &PgPool, scan_id: Uuid, platform: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE scans
         SET detected_platform = $1
         WHERE id = $2"
    )
    .bind(platform)
    .bind(scan_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Count scans submitted by a Clerk user in the current calendar month (UTC).
///
/// Used for the Developer tier monthly quota (5 scans/month).
#[allow(dead_code)]
pub async fn count_scans_by_user_this_month(pool: &PgPool, clerk_user_id: &str) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)
         FROM scans
         WHERE clerk_user_id = $1
           AND created_at >= DATE_TRUNC('month', NOW() AT TIME ZONE 'UTC')"
    )
    .bind(clerk_user_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}
