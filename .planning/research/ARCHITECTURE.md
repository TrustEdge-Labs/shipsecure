# Architecture Research: Observability Integration

**Domain:** Structured logging, Prometheus metrics, health checks, graceful shutdown, request tracing
**Researched:** 2026-02-16
**Confidence:** HIGH (established Axum/tower patterns)

## Existing Architecture (Relevant Components)

```
src/
  main.rs              <- MODIFY: tracing init, middleware stack, graceful shutdown, routes
  api/
    scans.rs           <- MODIFY: Add tracing spans with scan_id
    payments.rs        <- MODIFY: Add tracing spans
    errors.rs          <- MODIFY: Structured error logging fields
  orchestrator/
    worker_pool.rs     <- MODIFY: TaskTracker instead of raw tokio::spawn, scan metrics
  scanners/
    *.rs               <- MODIFY: Add scanner-specific metrics and spans

infrastructure/        <- MODIFY: Ansible playbooks
  playbooks/
    provision.yml      <- MODIFY: Add DO metrics agent installation
  templates/
    nginx.conf.j2      <- MODIFY: /metrics endpoint, /health/ready proxy
    docker-compose.yml  <- MODIFY: STOPSIGNAL, log rotation
```

## Integration Architecture

### 1. Structured JSON Logging (main.rs)

**Current:**
```rust
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::try_from_default_env().expect("..."))
    .init();
```

**New:**
```rust
let env_filter = EnvFilter::try_from_default_env()
    .expect("RUST_LOG must be set");

if std::env::var("LOG_FORMAT").unwrap_or_default() == "json" {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_current_span(true)
        .init();
} else {
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();
}
```

**Data flow:** All `tracing::info!()`, `tracing::error!()` calls throughout codebase automatically emit JSON when `LOG_FORMAT=json`. No changes needed at call sites.

### 2. Request Tracing Middleware (main.rs)

**Integration point:** Add TraceLayer to Axum middleware stack, before CORS.

```rust
use tower_http::trace::TraceLayer;

let trace_layer = TraceLayer::new_for_http()
    .make_span_with(|request: &Request<_>| {
        let request_id = Uuid::new_v4().to_string();
        tracing::info_span!(
            "http_request",
            method = %request.method(),
            uri = %request.uri(),
            request_id = %request_id,
        )
    })
    .on_response(|response, latency, _span| {
        tracing::info!(status = %response.status(), latency_ms = latency.as_millis(), "Response");
    });

let app = Router::new()
    .route("/api/v1/scans", post(create_scan))
    // ... routes
    .layer(trace_layer)  // Add before CORS
    .layer(cors_layer);
```

**Key:** All downstream handlers and spawned tasks inherit the request span if properly instrumented with `.instrument()`.

### 3. Prometheus Metrics (new module + route)

**New file:** `src/metrics.rs`

```rust
use prometheus::{Registry, Counter, Histogram, HistogramVec, CounterVec, Gauge, Encoder, TextEncoder};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref HTTP_REQUESTS_TOTAL: CounterVec = register_counter_vec!(
        "http_requests_total", "Total HTTP requests", &["method", "endpoint", "status"]
    ).unwrap();

    pub static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds", "HTTP request duration",
        &["method", "endpoint"],
        vec![0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]
    ).unwrap();

    pub static ref SCAN_DURATION: HistogramVec = register_histogram_vec!(
        "scan_duration_seconds", "Scan duration",
        &["tier", "status"],
        vec![1.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0]
    ).unwrap();

    pub static ref SCAN_QUEUE_DEPTH: Gauge = register_gauge!(
        "scan_queue_depth", "Number of scans waiting in queue"
    ).unwrap();

    pub static ref ACTIVE_SCANS: Gauge = register_gauge!(
        "active_scans", "Number of scans currently running"
    ).unwrap();
}

pub async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    ([(header::CONTENT_TYPE, "text/plain; version=0.0.4")], buffer)
}
```

**Router integration:** Add `GET /metrics` route in main.rs.

**Metrics increment points:**
- HTTP middleware (custom layer or on_response callback) → HTTP_REQUESTS_TOTAL, HTTP_REQUEST_DURATION
- ScanOrchestrator.run_scan() → SCAN_DURATION, ACTIVE_SCANS
- Worker pool queue → SCAN_QUEUE_DEPTH

### 4. Health Checks (expand existing)

**Current:** `GET /health` → `"ok"`

**New:** Keep `/health` shallow (liveness), add `/health/ready` (readiness).

```rust
// Shallow - unchanged
.route("/health", get(|| async { "ok" }))

// Deep - new
.route("/health/ready", get(health_ready))

async fn health_ready(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.pool)
        .await
        .is_ok();

    let permits = state.semaphore.available_permits();
    let status = if db_ok { "healthy" } else { "degraded" };
    let http_status = if db_ok { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };

    (http_status, Json(serde_json::json!({
        "status": status,
        "components": {
            "database": { "status": if db_ok { "healthy" } else { "unhealthy" } },
            "scan_capacity": { "available_permits": permits }
        }
    })))
}
```

### 5. Graceful Shutdown (main.rs)

**Current:** `axum::serve(listener, app).await.expect("Server error");`

**New:**
```rust
use tokio::signal;
use tokio_util::task::TaskTracker;

let tracker = TaskTracker::new();
// Pass tracker into AppState for use in ScanOrchestrator

let shutdown = async {
    let ctrl_c = signal::ctrl_c();
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
    tokio::select! {
        _ = ctrl_c => tracing::info!("Received SIGINT"),
        _ = sigterm.recv() => tracing::info!("Received SIGTERM"),
    }
    tracing::info!("Shutting down gracefully...");
    tracker.close();  // Stop accepting new tasks
    tracker.wait().await;  // Wait for in-flight tasks
    tracing::info!("All tasks completed, shutting down");
};

axum::serve(listener, app)
    .with_graceful_shutdown(shutdown)
    .await
    .expect("Server error");
```

**ScanOrchestrator change:** Replace `tokio::spawn(future)` with `tracker.spawn(future.instrument(span))`.

### 6. Infrastructure Changes (Ansible + Nginx + Docker)

**Nginx:** Protect /metrics from public access, proxy /health/ready.
```nginx
location /metrics {
    allow 127.0.0.1;
    deny all;
    proxy_pass http://localhost:3000;
    proxy_buffer_size 128k;
    proxy_buffers 4 256k;
}

location /health/ready {
    proxy_pass http://localhost:3000;
    proxy_read_timeout 5s;
}
```

**Docker Compose:** Add STOPSIGNAL and log rotation.
```yaml
services:
  app:
    stop_signal: SIGTERM
    stop_grace_period: 90s  # Longer than max scan duration
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
    environment:
      - LOG_FORMAT=json
```

**systemd:** Extend stop timeout.
```ini
[Service]
TimeoutStopSec=90
```

**Ansible:** Install DO metrics agent.
```yaml
- name: Install DigitalOcean metrics agent
  shell: curl -sSL https://repos.insights.digitalocean.com/install.sh | bash
  args:
    creates: /opt/digitalocean/bin/do-agent
```

## Suggested Build Order

Dependencies determine order:

1. **Structured JSON logging** — Foundation. All other features emit logs.
2. **Request tracing middleware** — Depends on logging. Adds correlation IDs.
3. **Prometheus metrics endpoint** — Independent of logging but benefits from it.
4. **Health checks** — Independent. Can be built in parallel with metrics.
5. **Graceful shutdown** — Requires TaskTracker integration with orchestrator. Most complex.
6. **Infrastructure (Ansible/Nginx/Docker)** — Deploy all observability together. DO metrics agent, Nginx /metrics protection, Docker log rotation.

```
[1] Structured Logging ──→ [2] Request Tracing ──→ [5] Graceful Shutdown
                                                         ↑
[3] Prometheus Metrics ─────────────────────────────────┤
                                                         ↑
[4] Health Checks ──────────────────────────────────────┤
                                                         ↓
                                                    [6] Infrastructure
```

## New vs Modified Files

### New Files
| File | Purpose |
|------|---------|
| `src/metrics.rs` | Prometheus metric definitions and handler |
| `src/health.rs` | Deep health check handler |

### Modified Files
| File | Change | Risk |
|------|--------|------|
| `Cargo.toml` | Add prometheus, lazy_static, tokio-util; add json feature | LOW |
| `src/main.rs` | Logging init, TraceLayer, routes, graceful shutdown | MEDIUM |
| `src/orchestrator/worker_pool.rs` | TaskTracker, scan metrics | MEDIUM |
| `src/api/scans.rs` | Span instrumentation | LOW |
| `src/api/errors.rs` | Structured error fields | LOW |
| Nginx config template | /metrics protection, /health/ready | LOW |
| Docker Compose | STOPSIGNAL, log rotation, LOG_FORMAT | LOW |
| systemd service | TimeoutStopSec | LOW |
| Ansible playbook | DO metrics agent task | LOW |

## Sources

- Axum examples (graceful shutdown): https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown
- tower-http TraceLayer: https://docs.rs/tower-http/latest/tower_http/trace
- prometheus crate patterns: https://docs.rs/prometheus
- tokio-util TaskTracker: https://docs.rs/tokio-util/latest/tokio_util/task/struct.TaskTracker.html
- DigitalOcean monitoring: https://docs.digitalocean.com/products/monitoring/
