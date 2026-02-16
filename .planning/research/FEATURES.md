# Feature Landscape: Observability in Rust/Axum Applications

**Domain:** Production observability for security scanning SaaS
**Researched:** 2026-02-16
**Confidence:** MEDIUM (based on training data and Rust ecosystem standards; web access restricted)

## Table Stakes

Features users expect from production-grade observability. Missing = operational blindness.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Structured JSON logging** | Production log aggregators (DataDog, Grafana Loki) require JSON. Human-readable text logs are useless at scale. | Low | tracing-subscriber with JSON formatter. MUST include: timestamp, level, target, message, span context, custom fields. |
| **Request correlation IDs** | Trace a single request through logs. Without this, debugging distributed failures is impossible. | Low | tower-http TraceLayer with x-request-id propagation. Auto-generate if missing, return in response headers. |
| **Health check endpoints** | Load balancers, orchestrators (k8s, Docker Swarm) need /health. Basic "ok" is insufficient - must check dependencies. | Low | /health (shallow), /health/ready (deep checks: DB, external services). Return 200 (healthy) or 503 (unhealthy). |
| **Basic metrics endpoint** | Prometheus is industry standard. /metrics endpoint exposing request counts, latencies, errors is expected. | Medium | prometheus crate with axum integration. Histograms for latency, counters for requests/errors, gauges for active connections. |
| **Graceful shutdown** | Prevents data loss, interrupted scans, broken database transactions. K8s sends SIGTERM before killing pod. | Medium | axum::serve with graceful shutdown signal. Must: stop accepting new requests, wait for in-flight requests (timeout), close DB pool, flush metrics. |
| **Error tracking** | Differentiate user errors (400s) from system errors (500s). Track error rates by type. | Low | Structured error types with context. Log at appropriate levels (warn for 4xx, error for 5xx). Include request_id, endpoint, error_type. |
| **Resource metrics** | CPU, memory, open connections, thread pool utilization. Needed for capacity planning and detecting leaks. | Low | Process-level metrics via prometheus (process_cpu_seconds_total, process_resident_memory_bytes). tokio-metrics for async runtime stats. |
| **Log levels** | Dynamic filtering by component (debug DB queries, but only errors from HTTP client). | Low | tracing EnvFilter with RUST_LOG="info,trustedge_audit=debug,sqlx=warn". Already implemented but needs structured output. |

## Differentiators

Features that set observability apart. Not expected, but highly valued for operational excellence.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Distributed tracing (OpenTelemetry)** | See request flow across scanner workers, external API calls (SSL Labs, Stripe). Pinpoint slowdowns in pipeline. | High | tracing-opentelemetry with OTLP exporter. Requires external collector (Jaeger, Tempo). Spans for: API request, scan orchestration, each scanner, DB queries. |
| **Scan-specific metrics** | Business metrics: scan duration by tier, scanner success rate, findings distribution, queue depth. | Medium | Custom Prometheus metrics: scan_duration_seconds (histogram with tier label), scanner_success_total (counter with scanner label), scan_queue_depth (gauge). |
| **Alerting rules** | Proactive notifications: high error rate, scan queue backing up, DB connection exhaustion, webhook failures. | Medium | Prometheus AlertManager rules. Examples: error_rate > 5% for 5min, scan_duration_p99 > 300s, db_connections_available < 2. |
| **Structured error context** | Rich error details: SSRF violation details, scanner timeout context, Stripe webhook signature mismatch. | Low | error_stack or anyhow for error context chains. Log structured fields: error.kind, error.scanner, error.target_url, error.duration. |
| **Real-time scan progress** | WebSocket or SSE updates during long scans. Show which scanner is running, progress %. | High | Not traditional observability, but improves user experience during 60-300s scans. Requires state synchronization. |
| **Background job monitoring** | Track tokio::spawn tasks: active count, completion rate, panic detection. | Medium | tokio-console (dev) or custom metrics tracking spawn/completion events. Helps detect leaked tasks or deadlocks. |
| **Request rate limiting visibility** | Expose rate limit metrics: requests blocked, tokens remaining, bucket fill rate per IP/email. | Low | Augment existing rate_limit module with Prometheus counters: rate_limit_blocked_total{limiter="email"}, rate_limit_allowed_total. |
| **External service health** | Dedicated health checks for SSL Labs API, Stripe API, Resend email, database pool. | Medium | /health/ready with component checks. Return detailed status: {"db": "healthy", "stripe": "degraded", "email": "healthy"}. |
| **Log sampling** | Reduce log volume in high-traffic scenarios without losing critical errors. | Medium | tracing-subscriber with sampling filter: log 100% of errors/warns, 10% of info, 1% of debug. Saves storage costs. |
| **Performance profiling** | Continuous profiling (CPU flamegraphs, memory allocation tracking) in production. | High | pprof-rs for CPU profiling, dhat for memory. Requires debug symbols in release builds, small performance overhead. |

## Anti-Features

Features to explicitly NOT build in early stages.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Custom log aggregation system** | Don't build what exists. DataDog, Grafana, Loki, CloudWatch all consume structured JSON. | Emit structured JSON logs to stdout. Let container orchestrator/log shipper forward to aggregator. |
| **In-app dashboards** | Grafana exists. Building custom visualization is scope creep. | Export Prometheus metrics, point Grafana at them. Use pre-built dashboards from community. |
| **Custom metrics format** | Prometheus is de facto standard. Inventing custom format limits tooling. | Use prometheus crate, expose /metrics in OpenMetrics format. |
| **Synchronous health checks** | Deep health checks (DB query, external API call) in critical path slow down /health. | Shallow /health (always fast), deep /health/ready (allowed to be slow). Cache health check results with TTL. |
| **Logging to files** | Container environments expect stdout/stderr. File-based logs require volume mounts, rotation, cleanup. | Log to stdout (JSON format). Infrastructure captures and routes logs. |
| **Metrics in logs** | "Request completed in 235ms" is not queryable. Metrics belong in Prometheus, not log lines. | Emit request_duration_seconds histogram to Prometheus. Logs are for events, metrics are for measurements. |
| **Excessive span depth** | tracing::instrument on every function creates noise. Too many spans = performance hit and unreadable traces. | Instrument boundaries: API handlers, scanner entry points, DB queries. NOT helper functions or inner loops. |
| **Blocking health checks** | /health that queries DB synchronously can timeout, fail health checks, trigger cascading failures. | Use async checks with short timeouts. Health check failure should not DoS the service. |

## Feature Dependencies

```
Graceful Shutdown → Requires structured logging (log shutdown events)
Distributed Tracing → Requires request correlation IDs (trace context propagation)
Alerting Rules → Requires metrics endpoint (Prometheus as data source)
Background Job Monitoring → Requires structured logging (task lifecycle events)
External Service Health → Requires basic health checks (extend with component checks)
Scan-specific Metrics → Requires metrics endpoint (expose business metrics)
```

## Domain-Specific Observability Signals

### Security Scanning SaaS Operational Priorities

| Signal Category | Key Metrics | Why Critical |
|-----------------|-------------|--------------|
| **Scan Health** | scan_duration_seconds{tier, status}, scan_success_rate{tier}, scan_queue_depth | Detect scanner hangs, queue backlog, tier-specific issues. |
| **Scanner Performance** | scanner_duration_seconds{scanner}, scanner_success_total{scanner}, scanner_timeout_total{scanner} | Identify unreliable scanners (TLS timeout, exposed_files DNS failures). |
| **Payment Flow** | stripe_webhook_received_total{event_type}, stripe_webhook_failed_total{reason}, paid_scan_initiated_total | Revenue-critical: webhook failures = lost conversions. |
| **Email Delivery** | email_sent_total{type}, email_failed_total{reason, smtp_code} | Customer communication: email failure = poor UX. |
| **Database Health** | db_connections_active, db_connections_idle, db_query_duration_seconds{query_type} | Connection exhaustion = service degradation. |
| **API Latency** | http_request_duration_seconds{endpoint, method, status}, http_requests_total{endpoint, status} | Track API performance by endpoint (scans, results, webhooks). |
| **Rate Limiting** | rate_limit_blocked_total{limiter}, rate_limit_allowed_total{limiter} | Detect abuse patterns, tune rate limits. |
| **SSRF Protection** | ssrf_blocked_total{reason}, ssrf_allowed_total | Security telemetry: track blocked internal/localhost scans. |

### Structured Log Field Conventions (Rust tracing ecosystem)

| Field Name | Purpose | Example |
|------------|---------|---------|
| `request_id` | Correlation across logs for single request | "req_a3f9c2b1" |
| `scan_id` | Track scan lifecycle | "550e8400-e29b-41d4-a716-446655440000" |
| `scanner` | Which scanner emitted log | "tls", "js_secrets" |
| `target_url` | Scan target (redact sensitive params) | "https://example.com" |
| `duration_ms` | Operation duration | 1234 |
| `status` | Operation outcome | "success", "timeout", "error" |
| `error_type` | Error category | "DatabaseError", "ScannerTimeout", "SSRFViolation" |
| `tier` | Scan tier | "free", "paid" |
| `endpoint` | API endpoint hit | "/api/v1/scans" |
| `method` | HTTP method | "POST" |
| `status_code` | HTTP status | 200, 500 |
| `client_ip` | Request origin (anonymized if needed) | "203.0.113.42" |
| `user_email` | User identifier (hash if privacy concern) | "user@example.com" |

### Health Check Signal Design

#### Shallow Health Check (`/health`)

**Purpose:** Fast liveness check for load balancer
**Response time:** < 10ms
**Checks:** Process is running, can accept connections

```rust
// Returns 200 OK immediately
async fn health() -> &'static str { "ok" }
```

#### Deep Health Check (`/health/ready`)

**Purpose:** Readiness check for traffic routing
**Response time:** < 1000ms
**Checks:**
- Database: Can execute `SELECT 1` (with 500ms timeout)
- External services: Cached status from periodic background checks (don't hit APIs in /health path)
- Scan queue: Not critically backed up (< 100 pending)
- Semaphore: Available permits (not deadlocked)

**Response format:**
```json
{
  "status": "healthy",
  "components": {
    "database": {"status": "healthy", "latency_ms": 5},
    "scan_queue": {"status": "healthy", "pending": 3, "active": 2},
    "semaphore": {"status": "healthy", "available_permits": 3}
  },
  "uptime_seconds": 86400
}
```

### Graceful Shutdown Pattern for Long-Running Scans

**Challenge:** Scans take 60-300s. SIGTERM gives 30s before SIGKILL. How to avoid killing in-flight scans?

**Pattern:**
1. **Shutdown signal received** (SIGTERM)
   - Stop accepting new scan requests (return 503)
   - Set shutdown flag in AppState
2. **Wait for in-flight scans** (with timeout)
   - Background tokio::spawn tasks hold Arc<Semaphore> permits
   - Track active scan count via Arc<AtomicUsize>
   - Wait up to 60s for count to reach 0
3. **Force shutdown after timeout**
   - Log warning: "Forced shutdown with N scans in-flight"
   - Update DB: set in-progress scans to "interrupted" status
   - Close DB pool gracefully
   - Flush metrics buffer
4. **Exit**

**Implementation approach:**
```rust
// Use tokio::signal::ctrl_c() or tokio::signal::unix::signal(SignalKind::terminate())
// Wrap axum::serve with graceful_shutdown(shutdown_signal)
// Track active scans in Arc<AtomicUsize>, decrement on completion
// Use tokio::select! to wait for scans or timeout
```

## MVP Recommendation

Prioritize for first observability milestone (estimated 2-3 days):

### Phase 1: Foundation (Day 1)
1. **Structured JSON logging**
   - Replace tracing_subscriber::fmt() with JSON formatter
   - Add request_id, scan_id fields to all logs
   - Test with jq: `cargo run | jq .`
2. **Request correlation middleware**
   - tower-http TraceLayer with x-request-id
   - Propagate to tracing spans
3. **Basic Prometheus metrics**
   - Add prometheus crate
   - Expose /metrics endpoint
   - Instrument: http_requests_total, http_request_duration_seconds

### Phase 2: Production Readiness (Day 2)
4. **Deep health checks**
   - Keep /health shallow
   - Add /health/ready with DB check
   - Return component status JSON
5. **Graceful shutdown**
   - Handle SIGTERM
   - Wait for in-flight scans (60s timeout)
   - Log shutdown events
6. **Scan-specific metrics**
   - scan_duration_seconds histogram
   - scanner_success_total counter
   - scan_queue_depth gauge

### Phase 3: Operational Visibility (Day 3)
7. **Error context enrichment**
   - Add scanner, target_url to error logs
   - Structured error types
8. **Background job tracking**
   - Log task spawn/completion with scan_id
   - Detect orphaned tasks
9. **Rate limit metrics**
   - Augment existing rate limiter with counters

### Defer to Later Milestones
- **Distributed tracing (OpenTelemetry)**: Requires external infrastructure (Jaeger/Tempo). Add when multi-service architecture emerges.
- **Alerting rules**: Requires metrics to stabilize, baseline to establish. Add after 1-2 weeks of production data.
- **Log sampling**: Premature optimization. Add if log volume becomes cost concern.
- **Performance profiling**: Complex setup, specialized use case. Add if performance issues emerge.

## Complexity Analysis

| Feature | Implementation Effort | Operational Benefit | Priority |
|---------|----------------------|---------------------|----------|
| Structured JSON logging | 2 hours | High (enables all downstream analysis) | P0 |
| Request correlation IDs | 1 hour | High (debugging critical) | P0 |
| Basic metrics endpoint | 4 hours | High (visibility into system health) | P0 |
| Deep health checks | 3 hours | High (production deployment requirement) | P0 |
| Graceful shutdown | 4 hours | High (prevents data loss) | P0 |
| Scan metrics | 2 hours | Medium (business visibility) | P1 |
| Error context | 2 hours | Medium (faster debugging) | P1 |
| Distributed tracing | 2-3 days | Medium (nice-to-have, not critical) | P2 |
| Alerting rules | 1 day | Low initially (need baseline first) | P2 |

## Sources

**Confidence Assessment:**
- Structured logging patterns: HIGH (tracing ecosystem convention, widely documented)
- Prometheus metrics: HIGH (industry standard, well-established patterns)
- Health check patterns: HIGH (k8s probe patterns, documented extensively)
- Graceful shutdown: MEDIUM (tokio patterns documented, specific scan handling is custom)
- OpenTelemetry: MEDIUM (integration complexity varies, ecosystem still maturing)

**Training data limitations:**
- Web access restricted (cannot verify latest crate versions or 2026 best practices)
- Rust observability ecosystem evolves rapidly (tracing-opentelemetry, tokio-console)
- Specific Axum 0.8.8 observability middleware may have changed since training cutoff

**Recommendation:** Verify latest crate compatibility and best practices from:
- docs.rs for tracing, tracing-subscriber, prometheus, tower-http
- Axum examples repository for observability patterns
- tokio docs for graceful shutdown examples
