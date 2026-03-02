use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::api::scans::AppState;

// Response types
#[derive(Serialize, Clone)]
pub struct LivenessResponse {
    pub status: String,
}

#[derive(Serialize, Clone)]
pub struct ScanCapacity {
    pub active: usize,
    pub max: usize,
}

#[derive(Serialize, Clone)]
pub struct ReadinessResponse {
    pub db_connected: bool,
    pub scan_capacity: ScanCapacity,
    pub status: String,
}

// Health cache for protecting DB from aggressive polling
#[derive(Clone)]
pub struct HealthCache {
    inner: Arc<Mutex<Option<(Instant, ReadinessResponse)>>>,
}

impl Default for HealthCache {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }
}

impl HealthCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_cached(&self, ttl: Duration) -> Option<ReadinessResponse> {
        let guard = self.inner.lock().unwrap();
        if let Some((timestamp, response)) = &*guard
            && timestamp.elapsed() < ttl
        {
            return Some(response.clone());
        }
        None
    }

    pub fn update(&self, response: ReadinessResponse) {
        let mut guard = self.inner.lock().unwrap();
        *guard = Some((Instant::now(), response));
    }
}

/// GET /health - Liveness check (no I/O, always returns ok)
pub async fn health_liveness() -> Json<LivenessResponse> {
    Json(LivenessResponse {
        status: "ok".to_string(),
    })
}

/// GET /health/ready - Readiness check with DB connectivity, scan capacity, and caching
pub async fn health_readiness(
    State(state): State<AppState>,
) -> (StatusCode, Json<ReadinessResponse>) {
    // Check shutdown state first -- per user decision: /health/ready returns unhealthy during shutdown
    if state.orchestrator.is_shutting_down() {
        let (active, max) = state.orchestrator.get_capacity();
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ReadinessResponse {
                db_connected: true, // Assume true during shutdown
                scan_capacity: ScanCapacity { active, max },
                status: "unhealthy".to_string(),
            }),
        );
    }

    // 1. Check cache first
    let cache_ttl = Duration::from_secs(5);
    if let Some(cached_response) = state.health_cache.get_cached(cache_ttl) {
        let status_code = match cached_response.status.as_str() {
            "healthy" => StatusCode::OK,
            "degraded" => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::SERVICE_UNAVAILABLE,
        };
        return (status_code, Json(cached_response));
    }

    // 2. Get scan capacity (non-blocking)
    let (active, max) = state.orchestrator.get_capacity();
    let scan_capacity = ScanCapacity { active, max };

    // 3. Check DB connectivity with latency measurement
    let threshold_ms = std::env::var("HEALTH_DB_LATENCY_THRESHOLD_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(50);
    let threshold = Duration::from_millis(threshold_ms);

    let db_start = Instant::now();
    let db_result = sqlx::query("SELECT 1").fetch_one(&state.pool).await;
    let db_latency = db_start.elapsed();

    // 4. Determine status
    let (db_connected, status) = match db_result {
        Ok(_) => {
            if db_latency > threshold {
                (true, "degraded".to_string())
            } else {
                (true, "healthy".to_string())
            }
        }
        Err(_) => (false, "unhealthy".to_string()),
    };

    // 5. Build response
    let response = ReadinessResponse {
        db_connected,
        scan_capacity,
        status: status.clone(),
    };

    // 6. Update cache
    state.health_cache.update(response.clone());

    // 7. Map status to HTTP status code
    let status_code = match status.as_str() {
        "healthy" => StatusCode::OK,
        "degraded" => StatusCode::TOO_MANY_REQUESTS,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(response))
}
