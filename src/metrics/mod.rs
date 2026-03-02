use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};

pub mod middleware;

// Histogram bucket boundaries for HTTP request latencies (seconds)
const HTTP_REQUEST_BUCKETS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

// Histogram bucket boundaries for scan durations (seconds)
const SCAN_DURATION_BUCKETS: &[f64] = &[1.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0];

/// Initialize the Prometheus metrics recorder with custom histogram buckets
pub fn install_metrics_recorder() -> PrometheusHandle {
    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_request_duration_seconds".to_string()),
            HTTP_REQUEST_BUCKETS,
        )
        .expect("Failed to set HTTP request duration buckets")
        .set_buckets_for_metric(
            Matcher::Full("scan_duration_seconds".to_string()),
            SCAN_DURATION_BUCKETS,
        )
        .expect("Failed to set scan duration buckets")
        .install_recorder()
        .expect("Failed to install Prometheus recorder")
}
