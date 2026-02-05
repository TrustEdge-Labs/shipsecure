use axum::extract::State;
use axum::http::header;
use axum::Json;
use serde_json::json;

use crate::api::errors::ApiError;
use crate::api::scans::AppState;
use crate::db;

/// GET /api/v1/stats/scan-count - Get total completed scans count for social proof
pub async fn get_scan_count(
    State(state): State<AppState>,
) -> Result<([(header::HeaderName, &'static str); 1], Json<serde_json::Value>), ApiError> {
    let count = db::scans::count_completed_scans(&state.pool).await?;

    // Add CORS header for cross-origin access from Next.js frontend
    let headers = [(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")];

    Ok((headers, Json(json!({ "count": count }))))
}
