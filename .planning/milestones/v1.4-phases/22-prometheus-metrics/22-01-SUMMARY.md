---
phase: 22-prometheus-metrics
plan: 01
subsystem: observability
tags: [prometheus, metrics, axum, middleware, openmetrics]

# Dependency graph
requires:
  - phase: 21-health-checks
    provides: Health check endpoints and HealthCache pattern for state management
provides:
  - PrometheusBuilder with custom histogram buckets for HTTP and scan metrics
  - HTTP metrics tracking middleware with path exclusion and status grouping
  - /metrics endpoint with localhost-only access control
  - PrometheusHandle in AppState for metrics state management
affects: [23-graceful-shutdown, 24-infrastructure, observability, monitoring]

# Tech tracking
tech-stack:
  added: [metrics 0.24, metrics-exporter-prometheus 0.17]
  patterns:
    - Separate router pattern for metrics endpoint (bypasses tracing like health)
    - Histogram bucket constants for domain-specific latency distributions
    - ConnectInfo for IP-based access control in handlers

key-files:
  created:
    - src/metrics/mod.rs
    - src/metrics/middleware.rs
    - src/api/metrics.rs
  modified:
    - src/main.rs
    - src/lib.rs
    - src/api/mod.rs
    - src/api/scans.rs
    - Cargo.toml

key-decisions:
  - "Histogram buckets defined as constants (not env vars): Chosen for stability - metrics bucket changes invalidate historical data"
  - "Status grouping (2xx, 4xx, 5xx) instead of individual codes: Reduces cardinality for better Prometheus performance"
  - "Path exclusion for /metrics and /health: Prevents recursive metric generation and noise from health checks"
  - "Localhost-only /metrics access: Defense-in-depth - Nginx also blocks in production but handler validates at application layer"

patterns-established:
  - "Metrics middleware layer closest to routes: Ensures MatchedPath extension is available after route matching"
  - "Separate metrics_router merged after layers: Follows health_router pattern to bypass tracing and metrics middleware"
  - "PrometheusHandle in AppState: Consistent with existing state management pattern (pool, orchestrator, health_cache)"

# Metrics
duration: 3min
completed: 2026-02-16
---

# Phase 22 Plan 01: Prometheus Metrics Infrastructure Summary

**Prometheus metrics endpoint with HTTP request tracking middleware, custom histogram buckets for latency distributions, and localhost-only access control**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-16T18:49:03Z
- **Completed:** 2026-02-16T18:52:17Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- Metrics infrastructure with PrometheusBuilder configuring HTTP request (11 buckets) and scan duration (7 buckets) histograms
- HTTP metrics tracking middleware recording http_requests_total counter and http_request_duration_seconds histogram with method, endpoint, and status labels
- /metrics endpoint serving OpenMetrics format text with is_loopback() access control
- PrometheusHandle integrated into AppState and routed via separate metrics_router pattern

## Task Commits

Each task was committed atomically:

1. **Task 1: Add dependencies and create metrics module with PrometheusBuilder setup and /metrics endpoint** - `1a9767e` (feat)
2. **Task 2: Wire metrics into Axum router — PrometheusHandle in state, middleware layer, /metrics route** - `f8ba909` (feat)

## Files Created/Modified
- `src/metrics/mod.rs` - Metrics recorder installation with custom histogram buckets for HTTP requests (0.005-10s) and scan durations (1-300s)
- `src/metrics/middleware.rs` - HTTP metrics tracking middleware with MatchedPath extraction, path exclusion (/metrics, /health), and status grouping
- `src/api/metrics.rs` - Metrics handler with ConnectInfo-based localhost check and PrometheusHandle rendering
- `src/main.rs` - Metrics recorder installation at startup, middleware layer wiring, metrics_router creation
- `src/api/scans.rs` - AppState updated with PrometheusHandle field
- `src/lib.rs` - metrics module registration
- `src/api/mod.rs` - metrics handler module registration
- `Cargo.toml` - Added metrics 0.24 and metrics-exporter-prometheus 0.17

## Decisions Made
- **Histogram buckets as constants:** User decision from research - bucket boundaries defined in code (not env vars) because changing them invalidates historical Prometheus data
- **Status grouping (Xxx format):** User decision from research - reduces label cardinality from ~50 status codes to 5 groups (2xx, 3xx, 4xx, 5xx) for better Prometheus performance
- **Path exclusion in middleware:** Excludes /metrics and /health from HTTP metrics to prevent recursive metric generation and health check noise
- **Localhost-only access:** is_loopback() check in handler provides defense-in-depth (Nginx also restricts in production)
- **Separate metrics_router:** Follows existing health_router pattern to bypass tracing and metrics middleware

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added PrometheusHandle to AppState in Task 1**
- **Found during:** Task 1 (Creating metrics_handler)
- **Issue:** Task 1 created metrics_handler that references state.metrics_handle, but AppState didn't have this field yet (planned for Task 2). Caused compilation failure.
- **Fix:** Added metrics_handle: PrometheusHandle field to AppState struct and imported PrometheusHandle in src/api/scans.rs during Task 1
- **Files modified:** src/api/scans.rs
- **Verification:** cargo check compiles cleanly, library builds successfully
- **Committed in:** 1a9767e (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking issue)
**Impact on plan:** Necessary to unblock Task 1 compilation. Plan intended this change for Task 2, but Task 1 dependencies required it earlier. No scope creep.

## Issues Encountered
None - plan executed smoothly after resolving AppState dependency order.

## User Setup Required
None - no external service configuration required. Prometheus metrics are exposed but not yet scraped (Phase 24 will configure Prometheus server).

## Next Phase Readiness
- Metrics infrastructure complete and operational
- Ready for Phase 22 Plan 02 (domain-specific scan and scanner metrics)
- All HTTP requests now tracked with latency and status metrics
- /metrics endpoint serves OpenMetrics format text for Prometheus scraping
- Test suite passes (62 pass, 1 pre-existing js_secrets failure as expected)

## Self-Check: PASSED

All files and commits verified:
- FOUND: src/metrics/mod.rs
- FOUND: src/metrics/middleware.rs
- FOUND: src/api/metrics.rs
- FOUND: commit 1a9767e (Task 1)
- FOUND: commit f8ba909 (Task 2)

---
*Phase: 22-prometheus-metrics*
*Completed: 2026-02-16*
