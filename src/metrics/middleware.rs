use axum::extract::{MatchedPath, Request};
use axum::middleware::Next;
use axum::response::IntoResponse;
use std::time::Instant;

pub async fn track_http_metrics(req: Request, next: Next) -> impl IntoResponse {
    // Extract route pattern, fall back to path if not available
    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|mp| mp.as_str().to_owned())
        .unwrap_or_else(|| req.uri().path().to_string());

    // Exclude metrics and health endpoints from tracking
    if path == "/metrics" || path.starts_with("/health") {
        return next.run(req).await;
    }

    // Clone method and record start time
    let method = req.method().clone();
    let start = Instant::now();

    // Execute the request
    let response = next.run(req).await;

    // Compute latency and status group
    let latency = start.elapsed().as_secs_f64();
    let status_group = format!("{}xx", response.status().as_u16() / 100);

    // Record metrics
    metrics::counter!(
        "http_requests_total",
        "method" => method.as_str().to_owned(),
        "endpoint" => path.clone(),
        "status" => status_group.clone()
    )
    .increment(1);

    metrics::histogram!(
        "http_request_duration_seconds",
        "method" => method.as_str().to_owned(),
        "endpoint" => path
    )
    .record(latency);

    response
}
