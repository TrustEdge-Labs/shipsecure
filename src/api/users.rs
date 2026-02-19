use axum::extract::{Query, State};
use axum::Json;
use axum_jwt_auth::Claims;
use serde::Deserialize;
use serde_json::json;

use crate::api::auth::ClerkClaims;
use crate::api::errors::ApiError;
use crate::api::scans::AppState;
use crate::db;

#[derive(Deserialize)]
pub struct ScanHistoryQuery {
    pub page: Option<u32>,
}

pub async fn get_user_scans(
    State(state): State<AppState>,
    Claims { claims, .. }: Claims<ClerkClaims>,
    Query(params): Query<ScanHistoryQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let page = params.page.unwrap_or(1).max(1) as i64;
    let per_page: i64 = 10;
    let offset = (page - 1) * per_page;

    let (scans, active_scans, total) = tokio::try_join!(
        db::scans::get_user_scan_history(&state.pool, &claims.sub, per_page, offset),
        db::scans::get_user_active_scans(&state.pool, &claims.sub),
        db::scans::count_user_scans_history(&state.pool, &claims.sub),
    )?;

    let total_pages = if total == 0 { 0 } else { (total + per_page - 1) / per_page };

    Ok(Json(json!({
        "scans": scans,
        "active_scans": active_scans,
        "total": total,
        "page": page,
        "per_page": per_page,
        "total_pages": total_pages,
    })))
}
