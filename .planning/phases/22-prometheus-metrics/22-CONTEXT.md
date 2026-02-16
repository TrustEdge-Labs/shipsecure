# Phase 22: Prometheus Metrics - Context

**Gathered:** 2026-02-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Expose operational metrics at a /metrics endpoint in OpenMetrics format for Prometheus scraping. Covers HTTP request metrics, scan performance metrics, active scan gauges, queue depth, scanner results, and rate limit counters. Nginx restriction of /metrics to localhost is an infrastructure concern (Phase 24) but the app also enforces localhost-only as defense in depth.

</domain>

<decisions>
## Implementation Decisions

### Metric naming & labels
- HTTP endpoint label uses Axum route pattern (e.g., `/api/v1/scans/:id`), not exact request paths — low cardinality
- HTTP status label uses status groups (2xx, 4xx, 5xx), not individual status codes
- Scanner names in snake_case for Prometheus convention (e.g., `ssl_labs`, `security_headers`)
- Scan tier label uses internal values directly (`free`, `paid`) — no mapping layer

### Histogram buckets
- HTTP request duration: standard web app defaults (5ms to 10s) — buckets like 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10
- Scan duration: scanner-tuned range (1s to 5min) — buckets like 1, 5, 10, 30, 60, 120, 300 reflecting external API call times
- Bucket boundaries defined as constants in code, not configurable via env vars

### /metrics endpoint behavior
- Always available in dev and prod — useful for local testing with curl
- Requests to /metrics excluded from HTTP request metrics (no self-referential counting from scrapes)
- Health check routes (/health, /health/ready) also excluded from HTTP request metrics (polling noise)
- App enforces localhost-only access (defense in depth) — returns 403 for non-localhost requests, even though Nginx also blocks

### Rate limit metrics
- Three distinct limiter label values: `scan_email`, `scan_ip`, `ssl_labs`
- Action label tracks `blocked` only — count when requests are actually rejected (429)
- SSL Labs rate limit: count every backoff event (429/529 from external API), not just final outcomes — shows pressure on external API

### Claude's Discretion
- Prometheus client library choice (metrics, prometheus crate, or other)
- Middleware architecture for recording HTTP metrics (layer vs extractor)
- Internal metrics registry design and thread safety approach
- Exact OpenMetrics output formatting details

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. Follow Prometheus naming conventions and Rust ecosystem best practices.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 22-prometheus-metrics*
*Context gathered: 2026-02-16*
