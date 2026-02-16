# Technology Stack — Observability Features

**Project:** ShipSecure
**Domain:** Structured logging, Prometheus metrics, health checks, graceful shutdown, request tracing
**Researched:** 2026-02-16
**Confidence:** HIGH (established Rust ecosystem patterns)

## Context

Stack additions for observability on an existing Rust/Axum backend. The application already has:
- Axum 0.8.8, tokio 1.x, tower-http 0.6.8 (trace feature available but unused)
- tracing 0.1 + tracing-subscriber 0.3 (env-filter enabled)
- PostgreSQL via sqlx 0.8.6
- DigitalOcean single-droplet deployment with Docker, Nginx, systemd, Ansible

## Recommended Stack Additions

### Structured JSON Logging

| Technology | Feature/Version | Purpose | Why Recommended |
|------------|----------------|---------|-----------------|
| **tracing-subscriber** | `json` feature | JSON log output | Already a dependency — just enable the `json` feature flag. Zero new crates. |

**Change:** Update `Cargo.toml` from `features = ["env-filter"]` to `features = ["env-filter", "json"]`.

**Toggle pattern:** Environment variable controls format (text for dev, JSON for prod):
```rust
if std::env::var("LOG_FORMAT").unwrap_or_default() == "json" {
    tracing_subscriber::fmt().json().with_env_filter(filter).init();
} else {
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
```

### Request Tracing / Correlation IDs

| Technology | Feature/Version | Purpose | Why Recommended |
|------------|----------------|---------|-----------------|
| **tower-http** | `trace` feature (already in deps) | Request/response tracing | Already a dependency with trace feature. TraceLayer adds spans with method, URI, status, latency. |
| **uuid** | Already in deps | Generate request IDs | Already used for scan IDs. Reuse for request correlation. |

**No new dependencies.** tower-http 0.6.8 trace feature is already declared. Just configure TraceLayer and add to middleware stack.

### Prometheus Metrics

| Technology | Feature/Version | Purpose | Why Recommended |
|------------|----------------|---------|-----------------|
| **prometheus** | 0.13.x | Metrics registry, counters, histograms, gauges | De facto standard Rust Prometheus client. Global registry pattern. |
| **lazy_static** | 1.x | Static metric declarations | Standard pattern for prometheus crate metric registration. |

**Why prometheus over alternatives:**
- `metrics` + `metrics-exporter-prometheus`: More abstract but adds indirection. prometheus crate is simpler for direct Prometheus exposition.
- `opentelemetry-prometheus`: Overkill without full OpenTelemetry — add later if needed.

### Health Checks

| Technology | Feature/Version | Purpose | Why Recommended |
|------------|----------------|---------|-----------------|
| **serde_json** | Already in deps | JSON health response | Already used throughout the API. |
| **sqlx** | Already in deps | DB connectivity check | Use `sqlx::query("SELECT 1")` with timeout. |

**No new dependencies.** Health checks use existing Axum handlers + sqlx.

### Graceful Shutdown

| Technology | Feature/Version | Purpose | Why Recommended |
|------------|----------------|---------|-----------------|
| **tokio** | signal feature (already in `full`) | SIGTERM/SIGINT handling | Already available — tokio `full` feature includes signal handling. |
| **tokio-util** | 0.7.x | TaskTracker for background tasks | Provides `TaskTracker` to track and await spawned tasks. Small, focused crate from tokio team. |

**Why tokio-util TaskTracker:**
- Current code uses `tokio::spawn` fire-and-forget. TaskTracker wraps spawns to track active tasks and wait for completion during shutdown.
- Alternative: manual `Arc<AtomicUsize>` counter — works but TaskTracker is cleaner.

### DigitalOcean Metrics Agent

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **do-agent** | System package | Infrastructure metrics (CPU, memory, disk, network) | Free with DigitalOcean. System-level monitoring. Install via Ansible. |

**Ansible integration:**
```yaml
- name: Install DigitalOcean metrics agent
  shell: curl -sSL https://repos.insights.digitalocean.com/install.sh | bash
  args:
    creates: /opt/digitalocean/bin/do-agent
```

## What NOT to Add

| Library | Why Avoid |
|---------|-----------|
| `opentelemetry` / `tracing-opentelemetry` | No external collector (Jaeger/Tempo) to receive traces. Add when multi-service or need distributed tracing. |
| `metrics` + `metrics-exporter-prometheus` | Adds abstraction layer over prometheus. Direct prometheus crate is simpler for our needs. |
| `tokio-console` | Dev-only tool. Not production observability. |
| `sentry` / `datadog-apm` | Vendor lock-in. Structured logs + Prometheus is vendor-neutral. |
| `actix-web-prom` | Wrong framework (Actix, not Axum). |
| Self-hosted Prometheus server | Overkill for single droplet. Expose /metrics, add Grafana Cloud or DO monitoring later. |

## New Dependencies Summary

```toml
# Cargo.toml changes

# MODIFY existing:
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }  # Add "json"

# ADD new:
prometheus = "0.13"
lazy_static = "1"
tokio-util = { version = "0.7", features = ["rt"] }  # TaskTracker
```

**Total new crates:** 3 (prometheus, lazy_static, tokio-util)
**Modified features:** 1 (tracing-subscriber json)

## Integration Points

| Addition | Integrates With | How |
|----------|----------------|-----|
| JSON logging | tracing_subscriber init in main.rs | Conditional formatter based on env var |
| TraceLayer | Axum middleware stack | `.layer(TraceLayer::new_for_http())` |
| Prometheus /metrics | Axum router | New route handler, Nginx proxy config |
| Health checks | Axum router, sqlx pool | New route handlers using AppState |
| Graceful shutdown | tokio runtime, axum::serve | `axum::serve(...).with_graceful_shutdown(signal)` |
| TaskTracker | ScanOrchestrator | Replace `tokio::spawn` with `tracker.spawn` |
| DO metrics agent | Ansible provisioning | New play/task in infrastructure playbook |

## Performance Considerations

| Addition | Impact | Notes |
|----------|--------|-------|
| JSON logging | ~1-5% CPU overhead | JSON serialization vs text formatting. Negligible. |
| TraceLayer | ~0.5% per request | Span creation/recording overhead. Minimal. |
| Prometheus metrics | ~0.1% per request | Atomic counter increments. Near-zero. |
| /metrics endpoint | ~10ms per scrape | Serialize all metrics to text. Prometheus scrapes every 15-60s. |
| Health checks | ~5ms (shallow), ~50ms (deep) | SELECT 1 with timeout for deep check. |
| TaskTracker | ~0% | Wraps existing spawn, negligible overhead. |

## Sources

- tracing-subscriber docs: https://docs.rs/tracing-subscriber
- prometheus crate: https://docs.rs/prometheus
- tower-http TraceLayer: https://docs.rs/tower-http/latest/tower_http/trace
- tokio-util TaskTracker: https://docs.rs/tokio-util/latest/tokio_util/task/struct.TaskTracker.html
- DigitalOcean metrics agent: https://docs.digitalocean.com/products/monitoring/how-to/install-metrics-agent/
