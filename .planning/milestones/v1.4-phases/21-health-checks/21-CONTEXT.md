# Phase 21: Health Checks - Context

**Gathered:** 2026-02-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Liveness and readiness endpoints for load balancers and monitoring systems. GET /health for shallow liveness, GET /health/ready for deep readiness validation with DB checks. Prometheus metrics integration and infrastructure deployment are separate phases (22, 24).

</domain>

<decisions>
## Implementation Decisions

### Readiness response shape
- scan_capacity is an object: `{ active: N, max: N }` showing in-flight scans against limit
- Response body is exactly three fields: `db_connected`, `scan_capacity`, `status` — no extras
- Status field is a string enum: "healthy", "degraded", or "unhealthy"
- Liveness endpoint (/health) returns JSON `{ "status": "ok" }` for consistency across both endpoints

### Degraded state handling
- Three states: healthy (200), degraded (429), unhealthy (503)
- Degraded triggers on DB latency — DB responds but slowly (above threshold)
- Unhealthy triggers when DB is unreachable
- DB latency threshold configurable via `HEALTH_DB_LATENCY_THRESHOLD_MS` env var, default 50ms

### Endpoint access control
- /health (liveness) is publicly accessible — external monitoring tools can reach it
- /health/ready (readiness) restricted to localhost via Nginx (same pattern as /metrics — 403 for external)
- Health check requests bypass logging/tracing entirely — consistent with Phase 20 decision
- Readiness check result cached for 5 seconds to protect DB from aggressive polling

### Claude's Discretion
- Exact caching implementation (in-memory timestamp check, tokio mutex, etc.)
- DB connectivity check query (SELECT 1 or equivalent)
- How to measure DB latency for degraded threshold
- Error response body structure when unhealthy

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 21-health-checks*
*Context gathered: 2026-02-16*
