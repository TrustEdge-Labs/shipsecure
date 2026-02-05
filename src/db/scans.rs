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
) -> Result<Scan, sqlx::Error> {
    let scan = sqlx::query_as::<_, Scan>(
        "INSERT INTO scans (target_url, email, submitter_ip)
         VALUES ($1, $2, $3::inet)
         RETURNING id, target_url, email, submitter_ip::text, status, score, error_message,
                   started_at::timestamp, completed_at::timestamp, created_at::timestamp"
    )
    .bind(target_url)
    .bind(email)
    .bind(submitter_ip)
    .fetch_one(pool)
    .await?;

    Ok(scan)
}

/// Get a scan by ID
#[allow(dead_code)]
pub async fn get_scan(pool: &PgPool, id: Uuid) -> Result<Option<Scan>, sqlx::Error> {
    let scan = sqlx::query_as::<_, Scan>(
        "SELECT id, target_url, email, submitter_ip::text, status, score, error_message,
                started_at::timestamp, completed_at::timestamp, created_at::timestamp
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
         RETURNING id, target_url, email, submitter_ip::text, status, score, error_message,
                   started_at::timestamp, completed_at::timestamp, created_at::timestamp"
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
