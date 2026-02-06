use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Type)]
#[sqlx(type_name = "finding_severity", rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    /// Returns a numeric weight for scoring calculations
    pub fn score_weight(&self) -> i32 {
        match self {
            Severity::Low => 1,
            Severity::Medium => 2,
            Severity::High => 5,
            Severity::Critical => 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Finding {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub scanner_name: String,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub remediation: String,
    pub raw_evidence: Option<String>,
    pub vibe_code: bool,
    pub created_at: NaiveDateTime,
}
