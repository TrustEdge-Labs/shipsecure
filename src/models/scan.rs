use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "scan_status", rename_all = "snake_case")]
pub enum ScanStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Expired,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Scan {
    pub id: Uuid,
    pub target_url: String,
    pub email: String,
    pub submitter_ip: Option<String>,
    pub request_id: Option<Uuid>,
    pub status: ScanStatus,
    pub score: Option<String>,
    pub results_token: Option<String>,
    pub expires_at: Option<NaiveDateTime>,
    pub detected_framework: Option<String>,
    pub detected_platform: Option<String>,
    pub stage_headers: bool,
    pub stage_tls: bool,
    pub stage_files: bool,
    pub stage_secrets: bool,
    pub stage_detection: bool,
    pub stage_vibecode: bool,
    pub tier: String,
    pub error_message: Option<String>,
    pub started_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub clerk_user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateScanRequest {
    pub url: String,
    pub email: String,
}
