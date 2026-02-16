use axum::extract::State;
use axum::response::{IntoResponse, Response};

use crate::api::scans::AppState;

pub async fn metrics_handler(State(state): State<AppState>) -> Response {
    // Access control handled by:
    // 1. Docker: port 3000 bound to 127.0.0.1 only
    // 2. Nginx: /metrics restricted to localhost (deny all)
    state.metrics_handle.render().into_response()
}
