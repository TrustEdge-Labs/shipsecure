use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::net::SocketAddr;

use crate::api::scans::AppState;

pub async fn metrics_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> Response {
    // Defense-in-depth: only allow localhost access
    if !addr.ip().is_loopback() {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }

    state.metrics_handle.render().into_response()
}
