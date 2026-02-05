use sqlx::PgPool;
use uuid::Uuid;

use crate::models::finding::Finding;

/// Insert multiple findings for a scan in a transaction
#[allow(dead_code)]
pub async fn insert_findings(
    pool: &PgPool,
    scan_id: Uuid,
    findings: &[Finding],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    for finding in findings {
        sqlx::query(
            "INSERT INTO findings (scan_id, scanner_name, title, description, severity, remediation, raw_evidence)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(scan_id)
        .bind(&finding.scanner_name)
        .bind(&finding.title)
        .bind(&finding.description)
        .bind(&finding.severity)
        .bind(&finding.remediation)
        .bind(&finding.raw_evidence)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Get all findings for a scan, ordered by severity (Critical -> High -> Medium -> Low)
#[allow(dead_code)]
pub async fn get_findings_by_scan(
    pool: &PgPool,
    scan_id: Uuid,
) -> Result<Vec<Finding>, sqlx::Error> {
    let findings = sqlx::query_as::<_, Finding>(
        "SELECT id, scan_id, scanner_name, title, description, severity, remediation, raw_evidence, created_at::timestamp
         FROM findings
         WHERE scan_id = $1
         ORDER BY
             CASE severity
                 WHEN 'critical' THEN 1
                 WHEN 'high' THEN 2
                 WHEN 'medium' THEN 3
                 WHEN 'low' THEN 4
             END,
             created_at ASC"
    )
    .bind(scan_id)
    .fetch_all(pool)
    .await?;

    Ok(findings)
}
