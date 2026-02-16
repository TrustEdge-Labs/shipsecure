# Phase 22: Prometheus Metrics - Research

**Researched:** 2026-02-16
**Domain:** Prometheus metrics with Rust Axum web framework
**Confidence:** HIGH

## Summary

Phase 22 implements operational metrics exposure via a `/metrics` endpoint in OpenMetrics format for Prometheus scraping. The Rust ecosystem offers two mature approaches: the `prometheus` crate (direct, TikV-maintained) and the `metrics` + `metrics-exporter-prometheus` facade pattern (flexible, widely adopted). The `metrics` facade is recommended for this project because it provides an ergonomic API similar to the existing `tracing` crate, supports the same macro-based instrumentation patterns, and integrates naturally with Axum middleware architecture.

The implementation tracks HTTP request metrics (counter + histogram), scan performance metrics (histogram with extended buckets for external API calls), active scans gauge, queue depth gauge, per-scanner result counters, and rate limit counters. All metrics follow official Prometheus naming conventions (snake_case, unit suffixes, `_total` for counters) and use low-cardinality labels (route patterns not exact paths, status groups not individual codes, bounded scanner/tier/limiter values).

**Primary recommendation:** Use `metrics` crate (0.24+) with `metrics-exporter-prometheus` for instrumentation, implement Axum middleware using `MatchedPath` extractor for low-cardinality route labeling, exclude `/metrics` and health check routes from HTTP metrics to avoid noise, and enforce localhost-only access via ConnectInfo IP checking with 403 response for non-localhost requests.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Metric naming & labels:**
- HTTP endpoint label uses Axum route pattern (e.g., `/api/v1/scans/:id`), not exact request paths — low cardinality
- HTTP status label uses status groups (2xx, 4xx, 5xx), not individual status codes
- Scanner names in snake_case for Prometheus convention (e.g., `ssl_labs`, `security_headers`)
- Scan tier label uses internal values directly (`free`, `paid`) — no mapping layer

**Histogram buckets:**
- HTTP request duration: standard web app defaults (5ms to 10s) — buckets like 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10
- Scan duration: scanner-tuned range (1s to 5min) — buckets like 1, 5, 10, 30, 60, 120, 300 reflecting external API call times
- Bucket boundaries defined as constants in code, not configurable via env vars

**/metrics endpoint behavior:**
- Always available in dev and prod — useful for local testing with curl
- Requests to `/metrics` excluded from HTTP request metrics (no self-referential counting from scrapes)
- Health check routes (`/health`, `/health/ready`) also excluded from HTTP request metrics (polling noise)
- App enforces localhost-only access (defense in depth) — returns 403 for non-localhost requests, even though Nginx also blocks

**Rate limit metrics:**
- Three distinct limiter label values: `scan_email`, `scan_ip`, `ssl_labs`
- Action label tracks `blocked` only — count when requests are actually rejected (429)
- SSL Labs rate limit: count every backoff event (429/529 from external API), not just final outcomes — shows pressure on external API

### Claude's Discretion

- Prometheus client library choice (metrics, prometheus crate, or other)
- Middleware architecture for recording HTTP metrics (layer vs extractor)
- Internal metrics registry design and thread safety approach
- Exact OpenMetrics output formatting details

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope
</user_constraints>

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| metrics | 0.24+ | Metrics facade with counters, gauges, histograms | Industry standard facade pattern, similar to tracing crate already in project, ergonomic macros, exporter-agnostic |
| metrics-exporter-prometheus | 0.17+ | Prometheus-specific exporter for metrics facade | Official Prometheus backend for metrics crate, handles OpenMetrics text format, configurable buckets per metric |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| lazy_static | 1.4 | Static metric handle initialization | Already in Cargo.toml, useful for PrometheusHandle singleton |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| metrics + metrics-exporter-prometheus | prometheus crate (TikV) | prometheus crate is more direct but requires different API, less consistent with tracing patterns, MetricVec has locking overhead; choose if already using prometheus ecosystem |
| metrics + metrics-exporter-prometheus | axum-prometheus | Automated middleware but less flexible, opinionated defaults, harder to exclude routes selectively; choose for rapid prototyping only |
| std::sync::Mutex | tokio::sync::Mutex | tokio Mutex is async but more expensive; std::sync::Mutex preferred when lock not held across .await (metrics increments are instant) |

**Installation:**
```bash
cargo add metrics@0.24
cargo add metrics-exporter-prometheus@0.17
```

## Architecture Patterns

### Recommended Project Structure

```
src/
├── api/
│   ├── metrics.rs       # /metrics endpoint handler, localhost check
│   └── mod.rs           # Router setup with metrics middleware
├── metrics/
│   ├── mod.rs           # Metrics registry setup, PrometheusHandle
│   ├── middleware.rs    # HTTP request tracking middleware
│   └── constants.rs     # Bucket definitions, metric names
└── main.rs              # PrometheusBuilder initialization
```

### Pattern 1: Global Metrics Setup with PrometheusBuilder

**What:** Initialize global Prometheus recorder once at startup with custom histogram buckets
**When to use:** Application startup, before router creation

**Example:**
```rust
// Source: https://docs.rs/metrics-exporter-prometheus + https://ellie.wtf/notes/exporting-prometheus-metrics-with-axum
use metrics_exporter_prometheus::{PrometheusBuilder, Matcher};

fn setup_metrics_recorder() -> PrometheusHandle {
    const HTTP_BUCKETS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];
    const SCAN_BUCKETS: &[f64] = &[
        1.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0,
    ];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_request_duration_seconds".to_string()),
            HTTP_BUCKETS,
        )
        .unwrap()
        .set_buckets_for_metric(
            Matcher::Full("scan_duration_seconds".to_string()),
            SCAN_BUCKETS,
        )
        .unwrap()
        .install_recorder()
        .unwrap()
}
```

### Pattern 2: HTTP Metrics Middleware with MatchedPath

**What:** Track HTTP requests using route patterns (not exact paths) for low cardinality
**When to use:** Applied as Axum middleware layer to all API routes

**Example:**
```rust
// Source: https://docs.rs/axum/latest/axum/extract/struct.MatchedPath.html + https://github.com/tokio-rs/axum/blob/main/examples/prometheus-metrics/src/main.rs
use axum::extract::{Request, MatchedPath};
use axum::middleware::Next;
use axum::response::IntoResponse;
use std::time::Instant;

pub async fn track_http_metrics(req: Request, next: Next) -> impl IntoResponse {
    // Extract matched route pattern for low cardinality
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_string()
    } else {
        req.uri().path().to_string()
    };

    // Skip metrics recording for excluded paths
    if path == "/metrics" || path.starts_with("/health") {
        return next.run(req).await;
    }

    let method = req.method().clone();
    let start = Instant::now();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status();

    // Status group (2xx, 4xx, 5xx)
    let status_group = format!("{}xx", status.as_u16() / 100);

    let labels = [
        ("method", method.as_str()),
        ("endpoint", path.as_str()),
        ("status", status_group.as_str()),
    ];

    metrics::counter!("http_requests_total", &labels).increment(1);
    metrics::histogram!("http_request_duration_seconds", &labels).record(latency);

    response
}
```

### Pattern 3: Localhost-Only Endpoint with ConnectInfo

**What:** Enforce localhost-only access by checking client IP address, return 403 for external requests
**When to use:** /metrics endpoint handler

**Example:**
```rust
// Source: https://docs.rs/axum/latest/axum/extract/connect_info/ + https://github.com/tokio-rs/axum/issues/43
use axum::extract::ConnectInfo;
use axum::http::StatusCode;
use std::net::SocketAddr;

pub async fn metrics_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    recorder_handle: axum::extract::State<PrometheusHandle>,
) -> Result<String, StatusCode> {
    // Defense in depth: reject non-localhost even though Nginx blocks
    if !addr.ip().is_loopback() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(recorder_handle.render())
}

// Router setup requires into_make_service_with_connect_info
let app = Router::new()
    .route("/metrics", get(metrics_handler))
    .with_state(recorder_handle)
    .into_make_service_with_connect_info::<SocketAddr>();
```

### Pattern 4: Scan Metrics Recording

**What:** Record scan duration histogram and scanner result counters during orchestrator execution
**When to use:** In ScanOrchestrator after scan completion, after individual scanner execution

**Example:**
```rust
// Source: metrics crate documentation + Prometheus naming conventions
use std::time::Instant;

// At scan start
let start = Instant::now();
metrics::gauge!("active_scans").increment(1.0);

// At scan end
let duration = start.elapsed().as_secs_f64();
metrics::gauge!("active_scans").decrement(1.0);
metrics::histogram!(
    "scan_duration_seconds",
    "tier" => "free",  // or "paid"
    "status" => "success"  // or "failure"
).record(duration);

// Per scanner result
metrics::counter!(
    "scanner_results_total",
    "scanner" => "security_headers",  // snake_case
    "status" => "success"  // or "failure"
).increment(1);
```

### Pattern 5: Queue Depth Gauge

**What:** Track pending scans waiting to execute using Semaphore available permits
**When to use:** Calculated on-demand (e.g., in health check or periodic update)

**Example:**
```rust
// Source: Existing health.rs get_capacity() pattern
impl ScanOrchestrator {
    pub fn update_queue_metrics(&self) {
        let available = self.semaphore.available_permits();
        let active = self.max_concurrent - available;

        metrics::gauge!("active_scans").set(active as f64);
        // Queue depth = total pending in DB minus active
        // Note: requires DB query, consider caching or periodic update
    }
}
```

### Anti-Patterns to Avoid

- **Using exact request paths as labels:** High cardinality explosion (user IDs in paths, etc.) — always use route patterns via MatchedPath
- **Recording metrics for /metrics endpoint:** Creates self-referential noise in scrapes — explicitly skip in middleware
- **Individual status codes as labels:** 40+ status codes create unnecessary time series — group as 2xx/4xx/5xx
- **Holding locks across .await:** tokio::sync::Mutex overhead when std::sync::Mutex works for instant operations — prefer std for counter/gauge increments
- **Missing unit suffixes:** Prometheus best practice violated — always include `_seconds`, `_bytes`, `_total` in metric names
- **Unbounded label values:** User emails, UUIDs, timestamps as labels — only use bounded enums (scanners, tiers, limiters)

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Prometheus text format encoding | Custom OpenMetrics string builder | metrics-exporter-prometheus | Handles HELP/TYPE comments, escaping, ordering, EOF marker, content negotiation |
| Histogram bucket quantile calculation | Manual percentile approximation | Prometheus server-side aggregation | Histograms designed for PromQL histogram_quantile(), aggregatable across instances |
| Thread-safe metric counters | Custom Arc<Mutex<HashMap>> | metrics crate facade | Lock-free TLS (thread-local storage) for hot path, periodic flush, zero-cost in single-threaded |
| Route pattern extraction | URI regex parsing | MatchedPath extractor | Axum provides matched route after routing, reliable and zero-cost |
| IP address locality check | String parsing "127.0.0.1" | SocketAddr.ip().is_loopback() | Handles IPv4 (127.0.0.0/8) and IPv6 (::1) correctly |

**Key insight:** Prometheus exposition format has subtle requirements (metric ordering, TYPE declarations, EOF markers). Custom implementations miss edge cases. The metrics ecosystem handles complexity (lock-free counters, bucket management, text encoding) that appears simple but has performance/correctness traps.

## Common Pitfalls

### Pitfall 1: High Cardinality Label Explosion

**What goes wrong:** Using unbounded values (user IDs, emails, exact paths, timestamps) as labels creates thousands of time series, exhausting Prometheus memory and slowing queries.

**Why it happens:** Labels seem like the natural place to put identifying information, but Prometheus creates a new time series for every unique label combination.

**How to avoid:**
- Use route patterns (`/api/v1/scans/:id`) not exact paths (`/api/v1/scans/550e8400-...`)
- Use status groups (`4xx`) not individual codes (`404`, `422`, `429`)
- Use bounded enums (`ssl_labs`, `security_headers`) not dynamic scanner names
- Never use emails, IPs (except fixed categories), UUIDs, timestamps as label values

**Warning signs:** Prometheus memory usage spiking, query timeouts, `/metrics` endpoint growing beyond 10KB, cardinality warnings in Prometheus logs.

**Reference:** [Prometheus naming best practices](https://prometheus.io/docs/practices/naming/), [High cardinality guide](https://last9.io/blog/how-to-manage-high-cardinality-metrics-in-prometheus/)

### Pitfall 2: Missing or Incorrect Unit Suffixes

**What goes wrong:** Metrics named `request_duration` or `scan_time` without units cause confusion when building dashboards/alerts, operators don't know if values are milliseconds or seconds.

**Why it happens:** Unit suffixes feel redundant when you "know" the units, but configuration files (alerts.yml, dashboards) have no type information.

**How to avoid:**
- Always suffix with unit in plural: `_seconds`, `_bytes`, `_total` (for counts)
- Use base units: seconds (not milliseconds), bytes (not kilobytes), ratio 0-1 (not percent 0-100)
- Counter metrics must end in `_total` per Prometheus convention

**Warning signs:** Dashboard queries using multiplication factors (value * 1000), alert thresholds in comments ("# 50ms = 0.05"), unit mismatch bugs.

**Reference:** [Prometheus metric naming](https://prometheus.io/docs/practices/naming/)

### Pitfall 3: Recording Metrics for the Metrics Endpoint

**What goes wrong:** Prometheus scrapes `/metrics` every 15-60 seconds, creating noise in HTTP request metrics. Every scrape increments `http_requests_total{endpoint="/metrics"}`, making real traffic analysis harder.

**Why it happens:** Middleware runs on all routes by default, /metrics is just another route unless explicitly excluded.

**How to avoid:**
- Check request path in middleware: `if path == "/metrics" { return next.run(req).await; }`
- Also exclude health checks (`/health`, `/health/ready`) which get polled frequently
- Alternative: use separate Router merged after middleware layer (like health_router pattern in main.rs)

**Warning signs:** Top metric path is `/metrics` in Grafana, scrape interval visible in request rate graphs, metrics traffic overwhelming actual API traffic.

**Reference:** [Prometheus exporters best practices](https://prometheus.io/docs/instrumenting/writing_exporters/)

### Pitfall 4: Using tokio::sync::Mutex for Instant Operations

**What goes wrong:** Wrapping PrometheusHandle or metrics state in tokio::sync::Mutex when operations complete instantly (counter increment, gauge set) adds async overhead and task parking.

**Why it happens:** "We're in an async context, so use async Mutex" seems logical, but not all operations need to be held across .await.

**How to avoid:**
- Use std::sync::Mutex when lock held for nanoseconds (counter increments)
- Use tokio::sync::Mutex ONLY when holding lock across .await points (rare for metrics)
- For PrometheusHandle.render(), the handle itself is Sync so can be cloned/shared via State without Mutex

**Warning signs:** High CPU usage in lock contention, tokio tasks yielding unnecessarily, profiler showing Mutex park/unpark overhead.

**Reference:** [Tokio shared state tutorial](https://tokio.rs/tokio/tutorial/shared-state), [std vs tokio Mutex discussion](https://users.rust-lang.org/t/tokio-mutex-std-mutex/88035)

### Pitfall 5: Forgetting into_make_service_with_connect_info for IP Extraction

**What goes wrong:** ConnectInfo<SocketAddr> extractor panics with "missing extension" error because SocketAddr not inserted into request extensions.

**Why it happens:** Default `.into_make_service()` doesn't provide connection info, must explicitly opt-in with `into_make_service_with_connect_info::<SocketAddr>()`.

**How to avoid:**
- Use `into_make_service_with_connect_info::<SocketAddr>()` when any handler/middleware needs ConnectInfo
- Document at top of file if ConnectInfo is used anywhere
- Consider ConnectInfo in tests: use MockConnectInfo or manually insert extension

**Warning signs:** 500 Internal Server Error on /metrics, panic logs mentioning "extension not found", tests failing that pass locally but break in CI.

**Reference:** [Axum ConnectInfo docs](https://docs.rs/axum/latest/axum/extract/connect_info/), [ConnectInfo discussion](https://github.com/tokio-rs/axum/issues/43)

### Pitfall 6: Histogram Buckets Don't Match Actual Latencies

**What goes wrong:** All observations fall in lowest or highest bucket, making percentile calculations inaccurate. p95 shows as "all requests < 5ms" when actual p95 is 50ms, or "all requests > 10s" when actual p95 is 2s.

**Why it happens:** Copied default web app buckets without considering this app calls external APIs (SSL Labs, VibeCode) with 10-60s response times.

**How to avoid:**
- HTTP request buckets: 0.005 to 10s (internal API responses)
- Scan duration buckets: 1 to 300s (external API calls, multiple scanners)
- Use separate histogram configs via `set_buckets_for_metric` with `Matcher::Full`
- Test in dev with real scans, verify buckets span p50-p99 range

**Warning signs:** Histogram queries showing all values in one bucket, percentile calculations at bucket boundaries, Grafana graphs with sudden jumps.

**Reference:** [Prometheus histogram best practices](https://prometheus.io/docs/practices/histograms/), [Histogram bucket guide](https://last9.io/blog/histogram-buckets-in-prometheus/)

## Code Examples

Verified patterns from official sources:

### PrometheusBuilder Setup with Multiple Bucket Configurations

```rust
// Source: https://docs.rs/metrics-exporter-prometheus
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle, Matcher};

const HTTP_REQUEST_BUCKETS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

const SCAN_DURATION_BUCKETS: &[f64] = &[
    1.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0,
];

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
```

### Metrics Endpoint Handler with Localhost Check

```rust
// Source: https://docs.rs/axum/latest/axum/extract/connect_info/ + Prometheus best practices
use axum::extract::{State, ConnectInfo};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use metrics_exporter_prometheus::PrometheusHandle;
use std::net::SocketAddr;

pub async fn metrics_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(handle): State<PrometheusHandle>,
) -> Response {
    // Defense in depth: reject non-localhost (Nginx also blocks)
    if !addr.ip().is_loopback() {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }

    handle.render().into_response()
}
```

### HTTP Middleware with Path Exclusion and Status Grouping

```rust
// Source: https://github.com/tokio-rs/axum/blob/main/examples/prometheus-metrics/src/main.rs + https://docs.rs/axum/latest/axum/extract/struct.MatchedPath.html
use axum::extract::{Request, MatchedPath};
use axum::middleware::Next;
use axum::response::IntoResponse;
use metrics;
use std::time::Instant;

pub async fn track_http_metrics(req: Request, next: Next) -> impl IntoResponse {
    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());

    // Exclude /metrics and health checks from HTTP metrics
    if path == "/metrics" || path.starts_with("/health") {
        return next.run(req).await;
    }

    let method = req.method().clone();
    let start = Instant::now();
    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status();
    let status_group = format!("{}xx", status.as_u16() / 100);

    // Use array syntax for labels (efficient)
    metrics::counter!(
        "http_requests_total",
        "method" => method.as_str(),
        "endpoint" => path.as_str(),
        "status" => status_group.as_str()
    ).increment(1);

    metrics::histogram!(
        "http_request_duration_seconds",
        "method" => method.as_str(),
        "endpoint" => path.as_str()
    ).record(latency);

    response
}
```

### Scan Orchestrator Metrics Integration

```rust
// Source: metrics crate docs + existing orchestrator patterns
use metrics;
use std::time::Instant;

impl ScanOrchestrator {
    pub fn spawn_scan(&self, scan_id: Uuid, target_url: String, tier: &str) {
        // ... existing code ...

        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.expect("Semaphore closed");

            // Track active scans
            metrics::gauge!("active_scans").increment(1.0);
            let start = Instant::now();

            let result = Self::execute_scan_internal(pool, scan_id, target_url, tier).await;

            let duration = start.elapsed().as_secs_f64();
            metrics::gauge!("active_scans").decrement(1.0);

            let status = if result.is_ok() { "success" } else { "failure" };
            metrics::histogram!(
                "scan_duration_seconds",
                "tier" => tier,
                "status" => status
            ).record(duration);

            result
        });
    }
}

// Per-scanner result tracking
fn record_scanner_result(scanner_name: &str, success: bool) {
    let status = if success { "success" } else { "failure" };
    metrics::counter!(
        "scanner_results_total",
        "scanner" => scanner_name,
        "status" => status
    ).increment(1);
}
```

### Rate Limit Metrics Recording

```rust
// Source: Prometheus naming conventions + existing rate_limit/middleware.rs
use metrics;

// When email rate limit blocks request
pub async fn check_rate_limits(pool: &PgPool, email: &str, ip: &str) -> Result<(), ApiError> {
    let email_count = scans::count_scans_by_email_today(pool, email).await?;
    if email_count >= 3 {
        metrics::counter!(
            "rate_limit_total",
            "limiter" => "scan_email",
            "action" => "blocked"
        ).increment(1);
        return Err(ApiError::RateLimited("...".to_string()));
    }

    let ip_count = scans::count_scans_by_ip_today(pool, ip).await?;
    if ip_count >= 10 {
        metrics::counter!(
            "rate_limit_total",
            "limiter" => "scan_ip",
            "action" => "blocked"
        ).increment(1);
        return Err(ApiError::RateLimited("...".to_string()));
    }

    Ok(())
}

// When SSL Labs scanner encounters 429/529
pub async fn check_ssl_labs_ready(pool: &PgPool) -> Result<(), ScannerError> {
    // ... existing backoff logic ...
    if status == 429 || status == 529 {
        metrics::counter!(
            "rate_limit_total",
            "limiter" => "ssl_labs",
            "action" => "blocked"
        ).increment(1);
        // ... retry logic ...
    }
    Ok(())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| prometheus crate directly | metrics facade + exporters | ~2020 | Exporter-agnostic instrumentation, easier to swap backends, consistent with tracing ecosystem |
| Separate HTTP listener for /metrics | Same server with route restriction | 2024+ | Simpler deployment, defense-in-depth access control, localhost check in app code |
| OpenMetrics as separate spec | Merged into Prometheus project | July 2024 | Unified governance, OpenMetrics 2.0 development planned, less ecosystem fragmentation |
| Text format only | OpenMetrics with protobuf negotiation | 2023+ | metrics-exporter-prometheus auto-negotiates, smaller payload for high-cardinality (when unavoidable) |

**Deprecated/outdated:**
- **Summaries for aggregatable percentiles:** Use histograms instead; summaries can't be aggregated across instances (aggregating pre-computed quantiles is statistically invalid). Only use summaries for single-instance apps where accurate quantiles without bucket tuning are critical.
- **axum-prometheus crate:** Last updated 2023, opinionated middleware, less flexible than manual metrics integration. Use for rapid prototyping only, not production.
- **Separate :9090 metrics port:** Modern pattern is same port with access control (localhost check, Nginx restriction). Simpler firewall rules, fewer deployment complications.

## Open Questions

1. **Queue depth metric implementation strategy**
   - What we know: Semaphore provides active count via available_permits(), but total pending count requires DB query (count scans with status=pending)
   - What's unclear: Performance impact of querying DB on every /metrics scrape (15-60s interval), whether to cache queue depth with TTL
   - Recommendation: Implement queue depth as part of health check readiness logic (already queries DB with 5s cache), expose cached value in metrics. Alternative: periodic background task updates gauge every 30s.

2. **Global labels for instance identification**
   - What we know: PrometheusBuilder supports global labels (hostname, environment, region) applied to all metrics
   - What's unclear: Whether deployment needs instance labels (single-instance staging vs multi-instance production)
   - Recommendation: Leave global labels for Phase 24 (deployment/infrastructure). If multi-instance, add via environment variable (INSTANCE_ID) read at metrics setup.

3. **Histogram bucket adjustment post-deployment**
   - What we know: Buckets defined at compile time, can't be changed without restart
   - What's unclear: Process for identifying bucket problems in production, safe way to adjust without losing historical data
   - Recommendation: Start with conservative buckets (wider range), monitor for 1 week, analyze actual p50/p95/p99 from Prometheus queries, adjust buckets in next deployment. Historical data remains valid (Prometheus stores raw bucket counts).

## Sources

### Primary (HIGH confidence)

- [metrics crate documentation](https://docs.rs/metrics) - Core API for counters, gauges, histograms
- [metrics-exporter-prometheus documentation](https://docs.rs/metrics-exporter-prometheus) - PrometheusBuilder, PrometheusHandle, bucket configuration
- [Axum MatchedPath extractor](https://docs.rs/axum/latest/axum/extract/struct.MatchedPath.html) - Route pattern extraction for low cardinality
- [Axum ConnectInfo extractor](https://docs.rs/axum/latest/axum/extract/connect_info/) - SocketAddr extraction for localhost check
- [Prometheus metric naming conventions](https://prometheus.io/docs/practices/naming/) - Official naming rules, unit suffixes, label guidelines
- [Prometheus histograms vs summaries](https://prometheus.io/docs/practices/histograms/) - When to use each, tradeoffs, aggregation
- [Tokio shared state tutorial](https://tokio.rs/tokio/tutorial/shared-state) - std::sync::Mutex vs tokio::sync::Mutex guidance

### Secondary (MEDIUM confidence)

- [Axum Prometheus metrics example](https://github.com/tokio-rs/axum/blob/main/examples/prometheus-metrics/src/main.rs) - Official example with PrometheusBuilder and middleware
- [Exporting Prometheus metrics with Axum](https://ellie.wtf/notes/exporting-prometheus-metrics-with-axum) - Practical implementation guide
- [Prometheus label best practices](https://middleware.io/blog/prometheus-labels/) - Cardinality management, label design
- [High cardinality metrics management](https://last9.io/blog/how-to-manage-high-cardinality-metrics-in-prometheus/) - Detection and optimization strategies
- [Histogram bucket best practices](https://last9.io/blog/histogram-buckets-in-prometheus/) - Bucket selection for web apps
- [OpenMetrics status 2024](https://horovits.medium.com/openmetrics-is-archived-merged-into-prometheus-d555598d2d04) - OpenMetrics/Prometheus merger announcement
- [prometheus crate documentation](https://docs.rs/prometheus) - Alternative direct approach with TikV rust-prometheus

### Tertiary (LOW confidence)

- [axum-prometheus middleware crate](https://docs.rs/axum-prometheus) - Automated middleware (consider but not recommended for production)
- [Prometheus exporters best practices](https://prometheus.io/docs/instrumenting/writing_exporters/) - General exporter guidelines

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - metrics crate widely adopted, official Prometheus exporter, verified API usage
- Architecture: HIGH - Axum examples from official repo, MatchedPath and ConnectInfo documented extractors, validated patterns
- Pitfalls: HIGH - Prometheus official docs on cardinality/naming, Tokio docs on Mutex choice, Axum discussions on ConnectInfo, verified through multiple sources

**Research date:** 2026-02-16
**Valid until:** ~2026-03-16 (30 days) - Stable ecosystem, metrics crate on 0.24.x stable releases, Prometheus conventions unchanged for years
