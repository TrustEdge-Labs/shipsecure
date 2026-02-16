# Project Research Summary

**Project:** ShipSecure (TrustEdge Audit) v1.4 Observability
**Domain:** Production observability for Rust/Axum security scanning SaaS
**Researched:** 2026-02-16
**Confidence:** HIGH

## Executive Summary

This research covers adding production-grade observability to an existing Rust/Axum security scanning SaaS deployed on DigitalOcean. The recommended approach leverages existing dependencies (tracing-subscriber, tower-http, tokio) with minimal new crates (prometheus, lazy_static, tokio-util). The strategy is to enable structured JSON logging, add Prometheus metrics, implement deep health checks, ensure graceful shutdown coordination for long-running scans (60-300s), and integrate request tracing across async boundaries.

The key technical recommendation is to use the JSON feature flag in tracing-subscriber (already a dependency), tower-http TraceLayer for request correlation, prometheus crate for metrics exposition, and tokio-util TaskTracker for graceful shutdown coordination. This approach adds only 3 new dependencies and requires no external infrastructure (Jaeger, Prometheus server) for the initial milestone.

Critical risks center on security exposure of /metrics endpoint (must restrict to localhost only), graceful shutdown coordination with long-running background scan tasks (SIGTERM gives 30s but scans take 60-300s), and structured logging performance (avoid serializing full payloads). The mitigation strategy is to implement Nginx IP restrictions for /metrics before deployment, use tokio-util TaskTracker with extended stop timeout (90s+ via systemd), and log identifiers only (scan_id, request_id) rather than full payloads. Testing requirements include security validation that /metrics is unreachable externally and graceful shutdown testing with docker stop during active scans.

## Key Findings

### Recommended Stack

The stack additions leverage existing dependencies to minimize integration risk. tracing-subscriber already exists with env-filter — just enable the json feature flag for structured logging. tower-http 0.6.8 already has the trace feature enabled — just configure TraceLayer middleware. uuid is already used for scan IDs — reuse for request correlation. sqlx is already used — reuse for health check SELECT 1 queries. Only 3 new crates required: prometheus 0.13.x for metrics, lazy_static 1.x for metric declarations, and tokio-util 0.7.x for TaskTracker to coordinate graceful shutdown.

**Core technologies:**
- **tracing-subscriber (json feature)**: Structured JSON logging — zero new dependencies, just enable feature flag
- **tower-http (TraceLayer)**: Request tracing and correlation IDs — already in dependencies with trace feature
- **prometheus crate**: Metrics registry and exposition — de facto standard Rust Prometheus client
- **tokio-util (TaskTracker)**: Graceful shutdown coordination — tracks background tasks for awaiting during shutdown
- **DigitalOcean metrics agent**: Infrastructure metrics (CPU, memory, disk) — free with DO, install via Ansible

**What NOT to add:**
- OpenTelemetry (no external collector, overkill for single-service)
- metrics + metrics-exporter-prometheus (abstraction layer adds complexity)
- Self-hosted Prometheus server (premature for single droplet)
- Sentry/DataDog APM (vendor lock-in, structured logs + Prometheus is vendor-neutral)

### Expected Features

Research identifies clear tiers of observability features. Table stakes are production deployment requirements: structured JSON logging (log aggregators require JSON), request correlation IDs (trace requests through async execution), health checks (load balancers require /health endpoint), basic Prometheus metrics (industry standard), graceful shutdown (prevents data loss on SIGTERM), and error tracking (differentiate 4xx from 5xx). These are P0 features without which production operations are blind.

**Must have (table stakes):**
- Structured JSON logging — log aggregators (Grafana Loki, CloudWatch) require JSON format
- Request correlation IDs — trace single request through logs via x-request-id header
- Health check endpoints — /health (shallow liveness), /health/ready (deep readiness with DB check)
- Basic Prometheus metrics — http_requests_total, http_request_duration_seconds, process metrics
- Graceful shutdown — handle SIGTERM, wait for in-flight scans, prevent data loss
- Error tracking — structured error types with context, differentiate user errors (4xx) from system errors (5xx)
- Resource metrics — CPU, memory, DB connections, thread pool utilization

**Should have (competitive):**
- Scan-specific metrics — scan_duration_seconds histogram by tier, scanner_success_total counter
- Structured error context — rich error details with scanner, target_url, duration context
- Background job tracking — track tokio::spawn task lifecycle, detect orphaned tasks
- Rate limit visibility — expose rate_limit_blocked_total, rate_limit_allowed_total metrics
- External service health — component health checks in /health/ready (DB, semaphore status)

**Defer (v2+):**
- Distributed tracing (OpenTelemetry) — requires external infrastructure (Jaeger/Tempo), add when multi-service
- Alerting rules — requires metrics baseline, add after 1-2 weeks production data
- Log sampling — premature optimization, add if log volume becomes cost concern
- Performance profiling — complex setup, add if performance issues emerge

### Architecture Approach

Integration points span main.rs (logging init, TraceLayer middleware, graceful shutdown), new src/metrics.rs module (Prometheus metric declarations with lazy_static), modified ScanOrchestrator (replace tokio::spawn with TaskTracker.spawn), and infrastructure (Ansible for DO metrics agent, Nginx IP restrictions for /metrics, Docker stop_grace_period for extended shutdown). The build order follows dependencies: structured logging first (foundation), request tracing second (depends on logging), metrics and health checks in parallel (independent), graceful shutdown last (most complex, integrates with orchestrator).

**Major components:**
1. **Logging initialization (main.rs)** — Conditional JSON formatter based on LOG_FORMAT env var, keeps text format for development
2. **TraceLayer middleware (main.rs)** — Generate/propagate request_id via tower-http, add before CORS layer
3. **Metrics module (src/metrics.rs)** — lazy_static declarations for HTTP, scan, and resource metrics, /metrics handler endpoint
4. **Health checks (src/health.rs or inline)** — Shallow /health (fast liveness), deep /health/ready (DB check with timeout)
5. **Graceful shutdown (main.rs + orchestrator)** — SIGTERM/SIGINT signal handling, TaskTracker for background tasks, extended stop timeout
6. **Infrastructure integration** — Nginx /metrics IP restriction, Docker log rotation + stop_grace_period, systemd TimeoutStopSec=90s, Ansible DO agent

**Key patterns:**
- Environment-driven logging format (text in dev, JSON in prod) via LOG_FORMAT env var
- Request span propagation to background tasks via .instrument(span) on tokio::spawn
- In-memory metrics incremented by application (never query DB in /metrics handler)
- Shallow vs deep health checks (liveness vs readiness with different timeout expectations)
- TaskTracker wraps all background scan tasks to await during graceful shutdown

### Critical Pitfalls

Five critical pitfalls dominate the risk profile. Most severe is exposing /metrics endpoint to public internet (Pitfall 1) — reveals business metrics, user counts, scan volumes, database patterns, and potentially customer identifiers in labels. Prevention requires Nginx IP whitelist (127.0.0.1 only) and security testing that curl from external IP fails. Second is health check cascading failures (Pitfall 2) — deep health checks querying database and external services without timeouts cause restart loops when services degrade. Mitigation is shallow /health for liveness (just return 200), deep /health/ready for readiness with 100ms timeout on local dependencies only. Third is graceful shutdown data loss (Pitfall 3) — background tokio::spawn tasks killed mid-scan when SIGTERM arrives because Axum only waits for HTTP handlers. Solution is tokio-util TaskTracker tracking all background tasks with extended systemd TimeoutStopSec=90s+. Fourth is structured logging performance cliff (Pitfall 4) — logging entire request/response payloads adds 50-200ms per request due to JSON serialization. Prevention is log identifiers only (scan_id, request_id) not full structs, use skip() in #[instrument]. Fifth is request correlation ID propagation gaps (Pitfall 5) — TraceLayer adds request_id to HTTP logs but background tasks lose context. Fix with .instrument(span) on all tokio::spawn calls.

**Top 5 critical pitfalls:**

1. **Exposing /metrics endpoint publicly** — Business intelligence leak, security breach risk
   - Prevention: Nginx IP whitelist (127.0.0.1), separate internal port, no customer PII in labels
   - Test: curl https://domain.com/metrics from external IP should 403

2. **Health check cascading failures** — Self-induced DoS during external service degradation
   - Prevention: Shallow /health (liveness), deep /ready (readiness with 100ms timeout), no external service checks
   - Test: Health check duration <100ms, restart loops don't occur during DB slowness

3. **Graceful shutdown data loss** — In-flight scans killed mid-execution on SIGTERM
   - Prevention: TaskTracker wraps all background tasks, systemd TimeoutStopSec=90s+, docker stop_grace_period=90s
   - Test: docker stop during active scan, verify scan completion or checkpoint saved

4. **Structured logging performance cliff** — Full payload serialization adds 50-200ms per request
   - Prevention: Log identifiers only (scan_id, target_url), use skip() in #[instrument], full payloads at DEBUG level only
   - Test: Request latency P99 should not increase after logging changes

5. **Request correlation ID propagation gaps** — Background tasks lose tracing context across async boundaries
   - Prevention: .instrument(span) on all tokio::spawn, propagate request_id to external API calls
   - Test: Grep logs for request_id across full scan lifecycle, verify no gaps

**Moderate pitfalls:**
- Metrics cardinality explosion (Pitfall 6) — customer_id labels create 1M+ time series, Prometheus OOMs
- JSON logging format breaking log parsers (Pitfall 7) — multiline stack traces split across entries
- DigitalOcean metrics agent port conflicts (Pitfall 8) — do-agent binds 9100, app also tries 9100
- Database connection pool exhaustion from metrics (Pitfall 9) — /metrics queries DB every 15s scrape
- Nginx buffer size limits for /metrics response (Pitfall 10) — default 4KB truncates 100KB+ metrics

## Implications for Roadmap

Based on research, suggested phase structure prioritizes low-risk foundation before high-risk security-sensitive components:

### Phase 1: Structured JSON Logging
**Rationale:** Foundation for all observability. Low risk (just enable feature flag), no infrastructure changes. Delivers immediate debugging value in development. All other phases depend on structured logs.
**Delivers:** Environment-driven JSON logging (LOG_FORMAT=json in prod), request_id and scan_id fields in all logs, human-readable text in development.
**Addresses:** Table stakes structured logging (FEATURES.md), avoids logging performance cliff (Pitfall 4) by logging identifiers only.
**Avoids:** Pitfall 4 (performance cliff) by establishing discipline of logging identifiers not full payloads upfront.
**Research flag:** Standard pattern, skip research-phase.

### Phase 2: Request Tracing Middleware
**Rationale:** Builds on structured logging foundation. Low risk (tower-http already in dependencies). Enables request correlation before adding complexity. Small, focused change.
**Delivers:** tower-http TraceLayer with x-request-id generation/propagation, request/response logging with method/URI/status/latency.
**Uses:** tower-http (already in Cargo.toml with trace feature), uuid (already used for scan IDs).
**Addresses:** Table stakes request correlation IDs (FEATURES.md).
**Avoids:** Pitfall 5 (propagation gaps) by establishing span instrumentation pattern early.
**Research flag:** Standard pattern, skip research-phase.

### Phase 3: Basic Health Checks
**Rationale:** Independent of metrics, enables load balancer readiness checks. Medium risk (requires DB timeout handling). Delivers operational value quickly.
**Delivers:** GET /health (shallow liveness), GET /health/ready (deep readiness with DB check + timeout), component status JSON.
**Addresses:** Table stakes health checks (FEATURES.md).
**Avoids:** Pitfall 2 (cascading failures) by implementing shallow vs deep distinction upfront with timeouts.
**Research flag:** Standard pattern, skip research-phase.

### Phase 4: Prometheus Metrics Endpoint
**Rationale:** HIGH RISK due to security exposure. Requires Nginx configuration and security testing before deployment. Delivers operational visibility.
**Delivers:** src/metrics.rs module with lazy_static declarations, /metrics endpoint handler, HTTP metrics (requests_total, request_duration_seconds), scan metrics (scan_duration_seconds, scan_queue_depth).
**Uses:** prometheus 0.13.x (new), lazy_static 1.x (new), Nginx IP restriction (infrastructure).
**Addresses:** Table stakes basic metrics (FEATURES.md), should-have scan-specific metrics (FEATURES.md).
**Avoids:** Pitfall 1 (public exposure) via Nginx IP whitelist, Pitfall 6 (cardinality explosion) via label discipline, Pitfall 9 (connection exhaustion) via in-memory metrics.
**Research flag:** NEEDS SECURITY REVIEW — verify Nginx IP restriction works, test from external IP before production.

### Phase 5: Graceful Shutdown
**Rationale:** Most complex integration, touches orchestrator and main.rs. HIGH RISK due to data loss potential. Requires careful testing. Depends on logging for shutdown event visibility.
**Delivers:** SIGTERM/SIGINT signal handling, tokio-util TaskTracker integration in ScanOrchestrator, extended systemd TimeoutStopSec=90s, docker stop_grace_period=90s.
**Uses:** tokio-util 0.7.x TaskTracker (new), tokio signal handling (already in full feature).
**Addresses:** Table stakes graceful shutdown (FEATURES.md).
**Avoids:** Pitfall 3 (data loss) via TaskTracker coordination and extended shutdown timeout.
**Research flag:** NEEDS LOAD TESTING — test with docker stop during active scans, verify no data loss or incomplete scan state.

### Phase 6: Infrastructure Integration
**Rationale:** Deploys all observability components together. Requires staging validation. Environment-specific (DigitalOcean droplet).
**Delivers:** Nginx /metrics IP restriction + buffer size config, Docker Compose LOG_FORMAT=json + log rotation + stop_grace_period, systemd TimeoutStopSec=90s, Ansible playbook for DigitalOcean metrics agent.
**Addresses:** Production deployment configuration for all observability features.
**Avoids:** Pitfall 1 (metrics exposure) via Nginx IP restriction, Pitfall 8 (port conflicts) via documented port assignments, Pitfall 10 (Nginx buffer) via increased proxy_buffer_size, Pitfall 12 (disk full) via log rotation.
**Research flag:** NEEDS STAGING VALIDATION — test DO metrics agent installation, verify port 9100 not conflicting, test Nginx IP restriction from external network.

### Phase Ordering Rationale

Phases 1-2 are low-risk foundation work that delivers immediate debugging value without infrastructure changes. Phase 3 (health checks) is independent and can proceed in parallel with metrics planning. Phase 4 (metrics) is HIGH RISK and requires security review before deployment — must not proceed without Nginx IP restriction testing. Phase 5 (graceful shutdown) is most complex and depends on structured logging for visibility into shutdown events, so it comes after logging/tracing/health/metrics are stable. Phase 6 (infrastructure) deploys everything together and requires staging validation to catch environment-specific issues (DO agent, Nginx config, systemd).

Dependency flow:
```
[1] Structured Logging → [2] Request Tracing → [5] Graceful Shutdown → [6] Infrastructure
                              ↓
                         [3] Health Checks ───────────────────────────→ [6] Infrastructure
                              ↓
                         [4] Prometheus Metrics (HIGH RISK) ──────────→ [6] Infrastructure
```

Key risk mitigation ordering:
- Establish logging discipline (Phase 1) before adding metrics (Phase 4) to avoid performance pitfalls
- Implement health check patterns (Phase 3) before graceful shutdown (Phase 5) to validate timeout handling
- Security test metrics endpoint (Phase 4) in isolation before deploying infrastructure (Phase 6)
- Defer infrastructure integration (Phase 6) until all code changes are stable

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 4 (Prometheus Metrics):** Security review required — verify Nginx IP restriction config, test /metrics unreachable from external IP, audit metric labels for business intelligence leakage. Load test metrics endpoint response time with production-like metric cardinality.
- **Phase 5 (Graceful Shutdown):** Load testing required — test docker stop during active scans (60-300s duration), verify TaskTracker awaits completion, check scan state consistency after forced shutdown (timeout). Profile actual scan duration distribution to tune TimeoutStopSec.
- **Phase 6 (Infrastructure Integration):** Staging validation required — test DO metrics agent installation, verify port 9100 available, test Nginx IP whitelist from external network, validate systemd service restart behavior.

Phases with standard patterns (skip research-phase):
- **Phase 1 (Structured Logging):** Well-documented tracing-subscriber pattern, JSON feature flag is straightforward, environment variable toggle is standard practice.
- **Phase 2 (Request Tracing):** tower-http TraceLayer is established Axum middleware pattern, Axum examples cover this extensively.
- **Phase 3 (Health Checks):** Standard Axum handler pattern, sqlx timeout usage is documented, shallow vs deep health check is industry convention.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Existing dependencies cover most needs, only 3 new crates required, all are established ecosystem standards |
| Features | MEDIUM | Table stakes features well-defined from industry standards, scan-specific metrics based on domain knowledge, but actual production requirements may surface additional needs |
| Architecture | HIGH | Integration points clearly defined, tracing-subscriber and tower-http patterns well-documented, graceful shutdown pattern established in Axum examples |
| Pitfalls | MEDIUM | Security pitfalls (metrics exposure) and performance pitfalls (logging serialization) based on ecosystem knowledge, but environment-specific pitfalls (DO agent, Nginx buffer sizes) may need staging validation |

**Overall confidence:** HIGH

The recommended stack is conservative and leverages existing dependencies. The feature set is informed by production observability best practices in Rust/Axum applications. The architecture follows established patterns from Axum examples and tracing ecosystem conventions. The critical pitfalls are well-documented issues in Rust observability implementations.

### Gaps to Address

Gaps requiring validation during planning/execution:

- **Actual scan duration distribution:** Research assumes 60-300s scans based on domain knowledge, but actual P50/P90/P99 durations should be profiled to tune histogram buckets and graceful shutdown timeout. Query production database for scan duration stats before implementing Phase 4 and Phase 5.

- **Existing systemd configuration:** Research assumes systemd deployment but doesn't know current TimeoutStopSec value. Check existing service file before planning Phase 5 to understand current shutdown behavior.

- **Nginx reverse proxy configuration:** Research recommends Nginx IP restrictions but doesn't know current proxy config. Review existing Nginx config during Phase 4 planning to understand current routing and security posture.

- **Database connection pool size:** Pitfall 9 warns about connection exhaustion but research doesn't know current pool size or utilization. Check sqlx pool configuration and monitor active connections before implementing Phase 3 health checks to ensure SELECT 1 queries won't exhaust pool.

- **DigitalOcean metrics agent state:** Research recommends DO agent installation but doesn't know if it's already installed or what port it uses. Check droplet during Phase 6 planning to avoid port conflicts.

- **Log volume projections:** Research recommends JSON logging and log rotation but doesn't estimate actual log volume at production scale. Monitor log volume during Phase 1 development to tune rotation policy (max-size, max-file) before production deployment.

## Sources

### Primary (HIGH confidence)
- tracing-subscriber documentation: https://docs.rs/tracing-subscriber (JSON formatter, EnvFilter patterns)
- tower-http TraceLayer documentation: https://docs.rs/tower-http/latest/tower_http/trace (request tracing middleware)
- prometheus crate documentation: https://docs.rs/prometheus (metric types, registry patterns, lazy_static usage)
- tokio-util TaskTracker documentation: https://docs.rs/tokio-util/latest/tokio_util/task/struct.TaskTracker.html (graceful shutdown coordination)
- Axum examples (graceful shutdown): https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown (signal handling patterns)
- DigitalOcean monitoring documentation: https://docs.digitalocean.com/products/monitoring/how-to/install-metrics-agent/ (metrics agent installation)

### Secondary (MEDIUM confidence)
- Rust ecosystem observability conventions (training data through January 2025) — structured logging patterns, span instrumentation discipline
- Prometheus best practices (training data) — cardinality management, label selection, histogram bucket tuning
- Production observability patterns for SaaS applications — health check design, graceful shutdown requirements, security considerations

### Tertiary (LOW confidence)
- DigitalOcean-specific deployment patterns — systemd configuration, Nginx reverse proxy setup, metrics agent port assignments (requires staging validation)
- Scan duration distribution for security scanning workloads — research assumes 60-300s based on domain knowledge, actual distribution may vary (requires profiling)

---
*Research completed: 2026-02-16*
*Ready for roadmap: yes*
