# Domain Pitfalls: Observability in Rust/Axum

**Domain:** Adding observability to existing production Rust/Axum security scanning SaaS
**Researched:** 2026-02-16
**Confidence:** MEDIUM (based on Rust/Axum ecosystem knowledge, specific integration patterns)

## Critical Pitfalls

Mistakes that cause security breaches, data loss, or production outages.

### Pitfall 1: Exposing /metrics Endpoint to Public Internet

**What goes wrong:** Prometheus metrics endpoint exposed without authentication reveals sensitive business metrics, user counts, scan volumes, database query patterns, internal architecture details, and potentially even customer identifiers in label values.

**Why it happens:**
- Default Axum router examples don't show access control for /metrics
- Nginx misconfiguration forwards all traffic including /metrics
- Assumption that "metrics are just numbers" overlooks business intelligence leakage
- Copy-paste from tutorials designed for internal-only deployments

**Consequences:**
- Competitors scrape user counts, scan volumes, growth metrics
- Attackers discover database connection pool exhaustion timing for DoS
- Error rate metrics reveal when new vulnerabilities are deployed
- Customer identifiers in labels (scan_id, user_id) leak data
- Stack traces in error metrics expose internal implementation

**Prevention:**
```rust
// WRONG: Public metrics endpoint
let app = Router::new()
    .route("/metrics", get(metrics_handler))  // Exposed to internet!
    .route("/api/v1/scans", post(scan_handler));

// RIGHT: Internal-only metrics with IP restriction
let metrics_app = Router::new()
    .route("/metrics", get(metrics_handler))
    .layer(RequireAuthorizationLayer::custom(|req: &Request<_>| {
        // Only allow localhost or monitoring server
        req.connection_info().remote_addr() == Some("127.0.0.1")
    }));

let api_app = Router::new()
    .route("/api/v1/scans", post(scan_handler));

// Bind to different ports or use path-based routing with Nginx filtering
```

**Nginx configuration:**
```nginx
# WRONG: Forwards everything
location / {
    proxy_pass http://localhost:3000;
}

# RIGHT: Block /metrics from external access
location /metrics {
    allow 127.0.0.1;      # Localhost only
    allow 10.0.0.0/8;     # Internal network if needed
    deny all;
    proxy_pass http://localhost:3000;
}

location / {
    proxy_pass http://localhost:3000;
}
```

**Detection:**
- Curl https://yourdomain.com/metrics from external network
- Monitor access logs for /metrics requests from non-monitoring IPs
- Security scan for exposed management endpoints

**Security checklist for /metrics:**
- [ ] IP whitelist in Nginx (127.0.0.1 only)
- [ ] Separate internal port binding for metrics (3001) vs API (3000)
- [ ] No customer PII in metric labels (use IDs only if necessary, prefer aggregates)
- [ ] Review all custom metrics for business intelligence leakage
- [ ] Document what metrics expose and justify each one
- [ ] Consider mTLS for DigitalOcean metrics agent communication

---

### Pitfall 2: Health Check Cascading Failures (Thundering Herd)

**What goes wrong:** Health check endpoint queries database and external services (Stripe, scan engines). When external service is slow (not down, just slow), health check times out, load balancer marks instance unhealthy, orchestration restarts container, new instance starts making health check calls, overwhelming the already-struggling service.

**Why it happens:**
- "Deep" health checks that verify all dependencies seem thorough
- No timeout or circuit breaker on health check dependencies
- Health check interval shorter than slowest dependency timeout
- Restart loops when external service degraded but not completely down

**Consequences:**
- Self-induced DoS during external service degradation
- Container restart loops prevent recovery
- Database connection pool exhaustion from health check queries
- Lost in-flight scan jobs during unnecessary restarts
- Stripe API rate limiting from health check webhook validation attempts

**Prevention:**

```rust
// WRONG: Deep health check with no timeouts
async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    // Blocks on slow database
    let db_ok = state.db.execute("SELECT 1").await.is_ok();

    // Blocks on slow Stripe API
    let stripe_ok = state.stripe_client.check_connection().await.is_ok();

    if db_ok && stripe_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

// RIGHT: Shallow health check with separate readiness check
async fn liveness_check() -> impl IntoResponse {
    // Just checks if process is alive and can respond
    StatusCode::OK
}

async fn readiness_check(State(state): State<AppState>) -> impl IntoResponse {
    // Local checks only with timeouts
    let db_ok = timeout(
        Duration::from_millis(100),
        state.db.execute("SELECT 1")
    ).await.is_ok();

    // Don't check external services in health check!
    // Use separate monitoring with circuit breakers instead

    if db_ok {
        Json(json!({
            "status": "ready",
            "database": "connected"
        }))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(json!({
            "status": "not_ready",
            "database": "unavailable"
        })))
    }
}
```

**Health check best practices:**
- **Liveness** (/health): Just return 200, proves process responsive
- **Readiness** (/ready): Check local dependencies only (database, redis)
- **Timeouts**: 100ms max for readiness checks
- **No external services**: Don't check Stripe, external scan engines in health
- **Monitor separately**: Use Prometheus metrics + alerting for external dependencies

**systemd configuration:**
```ini
[Service]
# WRONG: Restarts on any failure including slow health checks
Restart=always
RestartSec=5s

# RIGHT: Rate limit restarts, use liveness probe
Restart=on-failure
RestartSec=10s
StartLimitInterval=300s
StartLimitBurst=5
```

**Detection:**
- Sudden restart loops during external service degradation
- Health check duration metrics show >500ms response times
- Database connection pool metrics spike during health checks
- Log correlation: health check failures coincide with external API slowness

---

### Pitfall 3: Graceful Shutdown Data Loss (Fire-and-Forget Tasks)

**What goes wrong:** Background tokio::spawn tasks processing security scans don't receive shutdown signal. SIGTERM arrives, Axum server shuts down immediately, in-flight scan tasks killed mid-execution, partial results written to database, customer charged for incomplete scan.

**Why it happens:**
- tokio::spawn creates detached tasks without shutdown coordination
- Axum graceful shutdown only waits for HTTP request handlers
- Docker STOPSIGNAL or systemd TimeoutStopSec too short
- No JoinHandle tracking for background tasks
- Assumption that "tokio handles this automatically"

**Consequences:**
- Lost scan results (customer experience degradation)
- Database corruption from partial writes
- Inconsistent state: payment recorded, scan incomplete
- No retry mechanism for interrupted tasks
- Customer support burden from "lost" scans

**Prevention:**

```rust
// WRONG: Fire-and-forget background task
pub async fn create_scan(
    State(state): State<AppState>,
    Json(payload): Json<CreateScanRequest>,
) -> impl IntoResponse {
    tokio::spawn(async move {
        // No way to cancel or await this!
        process_scan(payload).await;
    });

    StatusCode::ACCEPTED
}

// RIGHT: Track background tasks with shutdown coordination
pub struct AppState {
    db: PgPool,
    shutdown_tx: broadcast::Sender<()>,
    task_tracker: TaskTracker,  // From tokio-util
}

pub async fn create_scan(
    State(state): State<AppState>,
    Json(payload): Json<CreateScanRequest>,
) -> impl IntoResponse {
    let mut shutdown_rx = state.shutdown_tx.subscribe();

    state.task_tracker.spawn(async move {
        tokio::select! {
            result = process_scan(payload) => {
                // Scan completed normally
                result
            }
            _ = shutdown_rx.recv() => {
                // Shutdown signal received, save progress
                save_scan_checkpoint(payload.scan_id).await;
                tracing::warn!("Scan {} interrupted by shutdown", payload.scan_id);
            }
        }
    });

    StatusCode::ACCEPTED
}

// Main server with graceful shutdown
#[tokio::main]
async fn main() {
    let (shutdown_tx, _) = broadcast::channel(1);
    let task_tracker = TaskTracker::new();

    let state = AppState {
        db: create_pool().await,
        shutdown_tx: shutdown_tx.clone(),
        task_tracker: task_tracker.clone(),
    };

    let app = create_router(state);
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    // Wait for background tasks to complete
    task_tracker.close();
    task_tracker.wait().await;

    tracing::info!("All background tasks completed, shutting down");
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
    tracing::info!("Shutdown signal received, starting graceful shutdown");
}
```

**Docker/systemd configuration:**

```dockerfile
# Dockerfile
FROM rust:1.83 AS builder
# ... build steps ...

FROM debian:bookworm-slim
# WRONG: Default SIGKILL after 10s
# STOPSIGNAL SIGTERM

# RIGHT: Send SIGTERM and wait longer
STOPSIGNAL SIGTERM
# Note: Use docker stop --time=60 or systemd TimeoutStopSec
```

```ini
# systemd service
[Service]
Type=notify  # If using sd-notify
ExecStart=/usr/local/bin/trustedge-audit
ExecStop=/bin/kill -SIGTERM $MAINPID

# WRONG: Only 90s for shutdown
# TimeoutStopSec=90s

# RIGHT: Generous timeout for long-running scans
TimeoutStopSec=300s
KillMode=mixed
KillSignal=SIGTERM
FinalKillSignal=SIGKILL
```

**Detection:**
- Monitor scan completion rates before/after deployments
- Alert on scans in "processing" state for >expected duration after deploy
- Log correlation: "scan started" without "scan completed" around restart times
- Customer reports of "stuck" or "lost" scans after maintenance windows

**Testing graceful shutdown:**
```bash
# Test locally
cargo run &
PID=$!
sleep 5
kill -SIGTERM $PID  # Should see graceful shutdown logs
wait $PID

# Test in Docker
docker run -d --name test-shutdown myapp
docker exec test-shutdown curl localhost:3000/scans -d '{...}'  # Start scan
docker stop --time=60 test-shutdown  # Watch logs for graceful shutdown
```

---

### Pitfall 4: Structured Logging Performance Cliff (Large Payload Serialization)

**What goes wrong:** Switching from println! to tracing::info! with structured fields. Developer logs entire request/response bodies, scan results, or large JSON payloads. JSON serialization happens synchronously in request handler, adding 50-200ms latency to every request. Production performance degrades 10x overnight.

**Why it happens:**
- tracing! macro looks "free" syntactically
- No obvious serialization cost at call site
- Development testing with small payloads doesn't reveal issue
- Copying patterns from examples that log entire structs
- Not understanding difference between Display vs Debug vs Serialize

**Consequences:**
- Request latency increases 50-200ms per logged item
- CPU usage spikes on production deployment
- Logs fill disk rapidly (scan results can be megabytes)
- Log aggregation costs spike (CloudWatch, Datadog pricing)
- Difficult to rollback because "just logging changes"

**Prevention:**

```rust
// WRONG: Logs entire request body
#[tracing::instrument(fields(body = ?payload))]  // Serializes entire payload!
pub async fn create_scan(
    Json(payload): Json<CreateScanRequest>,
) -> impl IntoResponse {
    tracing::info!("Processing scan request");
    // ...
}

// ALSO WRONG: Logs large scan results
tracing::info!(
    scan_results = ?results,  // Could be megabytes!
    "Scan completed"
);

// RIGHT: Log only identifiers and metadata
#[tracing::instrument(
    skip(payload),
    fields(
        scan_id = %payload.scan_id,
        target = %payload.target_url,
        scan_type = ?payload.scan_type
    )
)]
pub async fn create_scan(
    Json(payload): Json<CreateScanRequest>,
) -> impl IntoResponse {
    tracing::info!("Processing scan request");
    // ...
}

// RIGHT: Log summary stats, not full results
tracing::info!(
    scan_id = %scan_id,
    findings_count = results.findings.len(),
    duration_ms = elapsed.as_millis(),
    "Scan completed"
);

// For debugging, use span instead of fields
let span = tracing::debug_span!(
    "process_scan",
    scan_id = %scan_id,
);
async move {
    // Detailed logging only at debug level
    tracing::debug!(?payload, "Full scan payload");
    // ...
}.instrument(span).await
```

**Log level discipline:**
```rust
// Production should run at INFO level
// ERROR: Service-level failures (database down, panic recovery)
// WARN:  Recoverable issues (rate limit hit, retry attempted)
// INFO:  Business events (scan started, payment processed)
// DEBUG: Request/response details (controlled in dev only)
// TRACE: Internal implementation details (never in production)

// WRONG: Log at INFO with debug-level detail
tracing::info!(?request_headers, ?request_body, "Request received");

// RIGHT: Log business event at INFO, details at DEBUG
tracing::info!(
    request_id = %request_id,
    method = %request.method(),
    path = %request.uri().path(),
    "Request received"
);
tracing::debug!(?request_headers, "Request headers");
```

**Environment configuration:**
```rust
// WRONG: Same log level everywhere
let subscriber = tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)  // Too verbose for prod!
    .json()
    .init();

// RIGHT: Environment-specific levels with targeted debug
let subscriber = tracing_subscriber::fmt()
    .with_max_level(
        env::var("RUST_LOG")
            .unwrap_or_else(|_| "info,trustedge_audit=info".into())
    )
    .json()
    .with_target(true)  // Include module path
    .with_thread_ids(false)  // Don't need in prod
    .with_thread_names(false)
    .init();

// Allows runtime control:
// RUST_LOG=info,trustedge_audit::scan=debug  (debug only scan module)
```

**Detection:**
- Request latency P99 increases after "logging improvements"
- CPU profiling shows significant time in serde_json::to_string
- Log volume metrics spike (bytes/minute)
- Disk I/O wait increases on log writes
- Compare request latency before/after tracing adoption

**Performance testing:**
```rust
#[cfg(test)]
mod tests {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn bench_logging(c: &mut Criterion) {
        let large_payload = create_test_scan_request();

        c.bench_function("log_full_payload", |b| {
            b.iter(|| {
                tracing::info!(payload = ?large_payload, "test");
            });
        });

        c.bench_function("log_identifiers_only", |b| {
            b.iter(|| {
                tracing::info!(
                    scan_id = %large_payload.scan_id,
                    "test"
                );
            });
        });
    }
}
```

---

### Pitfall 5: Request Correlation ID Propagation Gaps

**What goes wrong:** tower-http TraceLayer adds request_id to logs for HTTP requests, but background tasks, database queries, external API calls don't include it. When debugging production issue, can't correlate logs across async boundaries. "Scan failed" log has no request_id to trace back to originating API call.

**Why it happens:**
- TraceLayer only instruments HTTP layer
- tokio::spawn loses tracing context by default
- Database queries happen outside HTTP request span
- External API client not instrumented
- Copy current span manually forgotten

**Consequences:**
- Cannot trace request through full lifecycle
- Debug sessions require manual timestamp correlation
- Lost context when troubleshooting async workflows
- Separate systems (logs, metrics, traces) can't be correlated

**Prevention:**

```rust
// WRONG: Background task loses request context
pub async fn create_scan(
    request_id: Extension<RequestId>,
    Json(payload): Json<CreateScanRequest>,
) -> impl IntoResponse {
    tokio::spawn(async move {
        // request_id not available here!
        tracing::info!("Processing scan");  // Missing correlation
        process_scan(payload).await;
    });

    StatusCode::ACCEPTED
}

// RIGHT: Propagate span to background task
pub async fn create_scan(
    request_id: Extension<RequestId>,
    Json(payload): Json<CreateScanRequest>,
) -> impl IntoResponse {
    let span = tracing::info_span!(
        "background_scan",
        request_id = %request_id.0,
        scan_id = %payload.scan_id
    );

    tokio::spawn(
        async move {
            tracing::info!("Processing scan");  // Now has request_id!
            process_scan(payload).await;
        }.instrument(span)  // Attach span to future
    );

    StatusCode::ACCEPTED
}

// RIGHT: Database queries include context
pub async fn save_scan_results(
    pool: &PgPool,
    results: ScanResults,
) -> Result<()> {
    let span = tracing::Span::current();

    sqlx::query("INSERT INTO scan_results ...")
        .bind(results.scan_id)
        .bind(results.findings)
        .execute(pool)
        .instrument(span.clone())  // Propagate to query
        .await?;

    Ok(())
}

// RIGHT: External API clients propagate correlation
pub struct StripeClient {
    client: reqwest::Client,
}

impl StripeClient {
    pub async fn create_payment_intent(&self, amount: i64) -> Result<PaymentIntent> {
        let request_id = tracing::Span::current()
            .field("request_id")
            .map(|f| f.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let response = self.client
            .post("https://api.stripe.com/v1/payment_intents")
            .header("X-Request-Id", request_id)  // Send to Stripe
            .header("Idempotency-Key", uuid::Uuid::new_v4().to_string())
            .send()
            .instrument(tracing::info_span!("stripe_api"))
            .await?;

        Ok(response.json().await?)
    }
}
```

**tower-http configuration:**

```rust
use tower_http::trace::TraceLayer;
use uuid::Uuid;

// Comprehensive request tracing
let trace_layer = TraceLayer::new_for_http()
    .make_span_with(|request: &Request<_>| {
        let request_id = request
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        tracing::info_span!(
            "http_request",
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            request_id = %request_id,
        )
    })
    .on_request(|_request: &Request<_>, _span: &Span| {
        tracing::info!("Request started");
    })
    .on_response(|response: &Response<_>, latency: Duration, _span: &Span| {
        tracing::info!(
            status = %response.status(),
            latency_ms = latency.as_millis(),
            "Request completed"
        );
    })
    .on_failure(
        |error: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
            tracing::error!(
                error = %error,
                latency_ms = latency.as_millis(),
                "Request failed"
            );
        },
    );

let app = Router::new()
    .route("/api/v1/scans", post(create_scan))
    .layer(trace_layer);
```

**Detection:**
- Grep logs for request_id, find gaps where it's missing
- Try to trace single request end-to-end, identify missing links
- Background task logs lack request_id field
- Database query logs show query but not originating request

---

### Pitfall 6: Metrics Cardinality Explosion

**What goes wrong:** Adding customer_id, scan_id, or other high-cardinality values as Prometheus metric labels. Each unique label combination creates a new time series. With 10K customers and 100 scan types, that's 1M time series. Prometheus OOMs, queries timeout, metrics scraping fails.

**Why it happens:**
- Natural instinct to track "per customer" metrics
- Copy labels pattern from counter/gauge examples
- Not understanding cardinality constraints in Prometheus
- Wanting detailed breakdown without aggregation strategy

**Consequences:**
- Prometheus server OOM (crash or eviction by systemd)
- Scrape timeouts (>30s to serialize metrics)
- Query timeouts in Grafana dashboards
- High memory usage on application (storing all series)
- Lost metrics during overload period

**Prevention:**

```rust
// WRONG: High-cardinality labels
lazy_static! {
    static ref SCAN_DURATION: HistogramVec = register_histogram_vec!(
        "scan_duration_seconds",
        "Duration of security scans",
        &["customer_id", "scan_id", "target_url"]  // Explosion!
        // With 1000 customers × 100 scans × 50 URLs = 5M time series
    ).unwrap();
}

// RIGHT: Low-cardinality labels with aggregation
lazy_static! {
    static ref SCAN_DURATION: HistogramVec = register_histogram_vec!(
        "scan_duration_seconds",
        "Duration of security scans",
        &["scan_type", "status"]  // ~10 scan types × 3 statuses = 30 series
    ).unwrap();

    static ref SCANS_TOTAL: CounterVec = register_counter_vec!(
        "scans_total",
        "Total number of scans",
        &["scan_type", "status"]
    ).unwrap();
}

// Track per-customer metrics in database, not Prometheus
pub async fn record_scan_complete(
    scan_id: Uuid,
    customer_id: Uuid,
    duration: Duration,
    status: ScanStatus,
) {
    // Prometheus: Aggregated metrics
    SCAN_DURATION
        .with_label_values(&[scan_type.as_str(), status.as_str()])
        .observe(duration.as_secs_f64());

    // Database: Per-customer analytics
    sqlx::query(
        "INSERT INTO scan_metrics (scan_id, customer_id, duration_ms, status)
         VALUES ($1, $2, $3, $4)"
    )
    .bind(scan_id)
    .bind(customer_id)
    .bind(duration.as_millis() as i64)
    .bind(status)
    .execute(pool)
    .await?;
}
```

**Cardinality guidelines:**
- **Low (<10 values):** scan_type, method (GET/POST), status (success/error)
- **Medium (10-100):** endpoint, region, instance_id
- **High (>100):** customer_id, user_id, scan_id, IP addresses
- **Never use:** UUIDs, timestamps, free-form text

**Good label choices:**
```rust
// Environment labels (static, very low cardinality)
&["environment"]  // prod/staging/dev (3 values)

// API endpoint labels (low cardinality)
&["method", "endpoint"]  // GET/POST × 20 endpoints = 40 series

// Business domain labels (low cardinality)
&["scan_type", "status"]  // 10 types × 3 statuses = 30 series

// Error tracking (medium cardinality)
&["error_type"]  // DatabaseError, NetworkError, etc. (< 50 types)
```

**Detection:**
- Prometheus metrics scrape duration >5s
- Application memory usage grows with number of customers
- `prometheus_tsdb_symbol_table_size_bytes` metric increases rapidly
- Grafana queries timeout or return "too many series"
- Check cardinality: `curl localhost:9090/api/v1/status/tsdb`

---

## Moderate Pitfalls

### Pitfall 7: JSON Logging Format Breaking Log Parsers

**What goes wrong:** Switching from text to JSON logging, but some log messages contain unescaped quotes or newlines. Log aggregator (journald, CloudWatch) parses each line as separate JSON object, multiline stack traces split across multiple malformed entries.

**Prevention:**
- Use tracing_subscriber::fmt().json() not manual JSON formatting
- Configure panic handler to output JSON-compatible stack traces
- Test with intentional panics and errors that include quotes

```rust
// RIGHT: Structured panic handler
std::panic::set_hook(Box::new(|panic_info| {
    tracing::error!(
        panic_message = %panic_info,
        backtrace = ?std::backtrace::Backtrace::capture(),
        "Panic occurred"
    );
}));
```

**Detection:**
- Log aggregator shows malformed JSON errors
- Cannot parse logs with jq locally
- Stack traces split across multiple log entries

---

### Pitfall 8: DigitalOcean Metrics Agent Port Conflicts

**What goes wrong:** DO metrics agent (prometheus-node-exporter) binds to port 9100. Application also tries to bind metrics to 9100. Silent failure or container restart loop.

**Prevention:**
- Application metrics on port 9091 (or 3001 if internal to app)
- Document port assignments in Ansible inventory
- Nginx proxies internal ports to /metrics path

```ini
# ports.md
9090 - Prometheus (if self-hosted)
9091 - Application metrics
9100 - Node exporter (DO agent)
9187 - PostgreSQL exporter (if added)
```

**Detection:**
- Application startup fails with "address already in use"
- Check `ss -tlnp | grep 9100` to see what's bound

---

### Pitfall 9: Database Connection Pool Exhaustion from Metrics

**What goes wrong:** Each metrics scrape queries database for health stats. Prometheus scrapes every 15s. Connection acquired for each scrape never released properly, pool exhausted within minutes.

**Prevention:**
- Don't query database in metrics handler
- Use in-memory counters updated by application logic
- If database stats needed, cache with 60s TTL

```rust
// WRONG: Database query on every scrape
async fn metrics_handler(State(pool): State<PgPool>) -> impl IntoResponse {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scans")
        .fetch_one(&pool)  // Every 15s!
        .await
        .unwrap();
    // ...
}

// RIGHT: In-memory metrics updated by application
lazy_static! {
    static ref SCANS_TOTAL: Counter = register_counter!(
        "scans_total",
        "Total scans processed"
    ).unwrap();
}

// Increment when scan completes (in application logic)
SCANS_TOTAL.inc();

// Metrics handler just serializes in-memory state
async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    Response::builder()
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(Body::from(buffer))
        .unwrap()
}
```

**Detection:**
- Database connection pool metrics show exhaustion
- "connection pool timeout" errors in logs
- Metrics scrape intervals correlate with connection spikes

---

### Pitfall 10: Nginx Buffer Size Limits for /metrics Response

**What goes wrong:** Prometheus metrics response grows to 100KB+. Nginx default proxy_buffer_size is 4KB. Response truncated, Prometheus scrape fails with parse error.

**Prevention:**

```nginx
location /metrics {
    allow 127.0.0.1;
    deny all;

    proxy_pass http://localhost:9091;
    proxy_buffering on;
    proxy_buffer_size 128k;      # Increase from default 4k
    proxy_buffers 4 256k;        # 4 buffers of 256k each
    proxy_busy_buffers_size 256k;

    # Longer timeout for metrics generation
    proxy_read_timeout 10s;
}
```

**Detection:**
- Prometheus scrape errors: "unexpected end of input"
- Nginx error log: "upstream sent too big header"
- curl /metrics returns truncated response

---

## Minor Pitfalls

### Pitfall 11: Log Timestamp Timezone Confusion

**What goes wrong:** Application logs in local time, Nginx logs in UTC, database timestamps in UTC, debugging requires mental timezone conversion.

**Prevention:**
- **Always use UTC for logs** (tracing_subscriber default)
- Configure Nginx to log in UTC
- Display conversions happen in UI, never in logs

**Detection:**
- Timestamps don't align when correlating app logs with Nginx logs
- Off-by-N-hours errors when debugging

---

### Pitfall 12: Forgetting to Rotate Logs (Disk Full)

**What goes wrong:** Structured JSON logs are verbose. Application logs to stdout, Docker json-file driver captures to disk, no rotation configured, disk fills in 3 days, application crashes.

**Prevention:**

```yaml
# docker-compose.yml
services:
  app:
    image: trustedge-audit:latest
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

```ini
# systemd service
[Service]
StandardOutput=journal
StandardError=journal

# journald.conf
[Journal]
SystemMaxUse=1G
SystemMaxFileSize=100M
```

**Detection:**
- Disk usage alerts
- Application crash with "no space left on device"
- Check log sizes: `du -sh /var/lib/docker/containers/*/`

---

### Pitfall 13: Prometheus Histogram Bucket Misconfiguration

**What goes wrong:** Default histogram buckets are [0.005, 0.01, 0.025, ..., 10]. Scan durations are 30-300 seconds. All observations fall into "+Inf" bucket, no useful percentile data.

**Prevention:**

```rust
// WRONG: Default buckets
static ref SCAN_DURATION: Histogram = register_histogram!(
    "scan_duration_seconds",
    "Scan duration"
).unwrap();

// RIGHT: Custom buckets for your domain
static ref SCAN_DURATION: Histogram = register_histogram!(
    "scan_duration_seconds",
    "Scan duration",
    vec![1.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0]
    //    5s   10s  30s   1m    2m    5m     10m
).unwrap();
```

**Detection:**
- Grafana dashboard shows all requests in highest bucket
- Percentile queries return identical values
- Check `histogram_quantile(0.99, ...)` vs `histogram_quantile(0.50, ...)`

---

### Pitfall 14: Request ID Format Breaking Downstream Systems

**What goes wrong:** Generate request ID as UUID without hyphens. Stripe webhook verification expects hyphenated UUID. Correlation breaks.

**Prevention:**
- Use standard UUID v4 format with hyphens
- Document format in API standards
- Validate format in middleware

```rust
use uuid::Uuid;

// RIGHT: Standard UUID format
let request_id = Uuid::new_v4().to_string();  // "550e8400-e29b-41d4-a716-446655440000"
```

---

### Pitfall 15: Over-instrumenting Hot Paths

**What goes wrong:** Add tracing::instrument to every function. Hot path function called 1000 times per request now creates 1000 spans, overwhelming tracing subscriber.

**Prevention:**
- Instrument at service boundaries (HTTP handlers, DB queries, external APIs)
- Don't instrument pure functions or tight loops
- Use #[instrument(skip_all)] for hot paths if needed

```rust
// WRONG: Over-instrumentation
#[tracing::instrument]
fn calculate_risk_score(findings: &[Finding]) -> u8 {
    findings.iter()
        .map(|f| calculate_finding_score(f))  // Called 100s of times
        .sum()
}

// RIGHT: Instrument at higher level only
#[tracing::instrument(skip(findings), fields(findings_count = findings.len()))]
fn analyze_scan_results(findings: &[Finding]) -> AnalysisResult {
    let risk_score = calculate_risk_score(findings);  // Not instrumented
    // ...
}
```

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Structured logging setup | Logging large payloads (Pitfall 4) | Start with identifiers only, add detail incrementally at DEBUG level |
| Prometheus metrics | Exposing /metrics publicly (Pitfall 1) | Configure Nginx IP restrictions before deploying metrics endpoint |
| Health checks | Cascading failures (Pitfall 2) | Implement shallow liveness check first, deep readiness check later with timeouts |
| Graceful shutdown | Lost background tasks (Pitfall 3) | Use TaskTracker from tokio-util, test with docker stop before production |
| Request tracing | Context propagation gaps (Pitfall 5) | Use .instrument(span) on all tokio::spawn calls |
| Metrics cardinality | Label explosion (Pitfall 6) | Review every label, limit to <10 distinct values, use database for high-cardinality tracking |
| Nginx integration | /metrics buffer size (Pitfall 10) | Increase proxy_buffer_size to 128k for metrics endpoint |
| DO metrics agent | Port conflicts (Pitfall 8) | Document port assignments, use 9091 for app metrics |
| Database health checks | Connection pool exhaustion (Pitfall 9) | Never query database in metrics handler, use in-memory counters only |

---

## Security Implications Summary

**Critical security issues with observability:**

1. **Metrics exposure** (Pitfall 1): Most critical, directly leaks business intelligence
   - Mitigation: IP whitelist in Nginx, separate internal port
   - Test: curl from external IP should 403

2. **PII in logs**: Customer data in structured log fields
   - Mitigation: Log IDs only, never log email/name/payment details
   - Test: Grep logs for email patterns, should find zero matches

3. **Log aggregation access**: CloudWatch logs contain business logic
   - Mitigation: IAM policies, log retention policies
   - Test: Principle of least privilege for log access

4. **Metrics agent authentication**: DigitalOcean agent scrapes metrics
   - Mitigation: mTLS or IP restriction for agent communication
   - Test: Verify agent certificate validation

5. **Health check information disclosure**: Error messages reveal internals
   - Mitigation: Generic "unavailable" for external /health, detailed for internal /ready
   - Test: External health check should not reveal "postgres connection failed"

**Security checklist before going to production:**
- [ ] /metrics blocked from public internet (Nginx config)
- [ ] No PII in log fields (code review + log grep)
- [ ] Health check returns generic errors externally
- [ ] Request IDs don't contain sequential numbers (use UUIDs)
- [ ] Metrics don't include sensitive label values
- [ ] Log aggregation access restricted via IAM
- [ ] Prometheus retention limited (don't store forever)
- [ ] No secrets in logs (API keys, tokens) - check with grep

---

## Research Confidence Assessment

**Overall confidence:** MEDIUM

| Area | Confidence | Notes |
|------|------------|-------|
| Rust/Axum patterns | HIGH | Direct ecosystem knowledge, common patterns documented |
| Prometheus security | HIGH | Well-established best practices, documented vulnerabilities |
| Tokio graceful shutdown | HIGH | Common pitfall with clear solutions in tokio docs |
| Nginx integration | MEDIUM | Standard reverse proxy patterns, may need DigitalOcean-specific testing |
| DigitalOcean metrics agent | LOW | Specific to DO implementation, need to verify port/config in actual environment |

**Sources:**
- Rust tracing documentation (training data, January 2025)
- Prometheus best practices (training data, well-established)
- Tokio documentation on graceful shutdown patterns
- Axum examples and common middleware patterns
- General observability pitfalls from production systems

**Gaps to validate:**
- DigitalOcean metrics agent specific port and configuration (need environment-specific research)
- Current state of systemd configuration in existing deployment
- Existing Nginx configuration for reverse proxy setup
- Database connection pool configuration and current utilization
- Actual scan duration ranges (affects histogram bucket configuration)

**Recommendations for phase-specific research:**
- Phase addressing Nginx integration: Research actual DO droplet Nginx config, test /metrics routing
- Phase addressing graceful shutdown: Profile actual scan job durations to set TimeoutStopSec appropriately
- Phase addressing metrics: Audit all planned metrics for cardinality before implementation

---

## Key Takeaways for Roadmap

**Highest risk pitfalls to address early:**
1. /metrics endpoint security (Pitfall 1) - Security breach risk
2. Graceful shutdown (Pitfall 3) - Data loss risk
3. Structured logging performance (Pitfall 4) - Production performance risk

**Suggested phase ordering based on risk:**
1. **Structured logging** - Lower risk, good foundation
2. **Health checks** - Medium risk, enables better monitoring
3. **Prometheus metrics** - HIGH RISK, requires security configuration
4. **Graceful shutdown** - HIGH RISK, requires careful coordination
5. **Request tracing** - Lower risk, builds on logging foundation
6. **DO metrics agent** - Environment-specific, test in staging first

**Research flags:**
- Phase implementing /metrics: Requires security review and penetration testing
- Phase implementing graceful shutdown: Requires load testing with real scan workloads
- Phase implementing DO agent: Requires staging environment validation first

**Testing requirements:**
- Security testing: Attempt to access /metrics from external IP (should fail)
- Load testing: Structured logging with production-like payloads
- Graceful shutdown testing: Kill process during active scan, verify no data loss
- Integration testing: All observability components working together in staging before production
