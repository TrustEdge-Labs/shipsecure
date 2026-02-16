# Phase 20: Request Tracing - Context

**Gathered:** 2026-02-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Add correlation IDs to every HTTP request and propagate them to background scan tasks. Enables tracing a user's request through the entire scan lifecycle in production logs. Builds on Phase 19's structured logging foundation.

</domain>

<decisions>
## Implementation Decisions

### Request ID visibility
- Request ID is internal only — NOT exposed in HTTP response headers
- Always generate a fresh UUID v4 server-side — never honor incoming X-Request-Id headers
- Field name: `request_id` (snake_case, matching scan_id from Phase 19)

### Log detail level
- Minimal HTTP details: method, URI path, status code, latency_ms
- Exact paths logged (e.g., /api/scans/550e8400-...), not grouped route patterns
- Filter out health check endpoints (/health, /health/ready) from request logging to reduce noise
- Log levels: INFO for 4xx/5xx errors, DEBUG for 2xx/3xx successes

### Propagation depth
- Shared field approach: add request_id as a field on scan spans, NOT parent-child span linking
- Scan tasks only — emails and other background tasks do NOT get request_id
- Pass request_id as an explicit function parameter to spawn_scan/spawn_paid_scan (not span context extraction)
- Store request_id in database scans table (requires migration) for queryable correlation

### Sensitive data policy
- Strip query parameters from logged URIs — log path portion only
- Safe headers only: log Content-Type, Accept, etc. but NEVER Authorization, Cookie, or Set-Cookie
- Never log request or response bodies
- No client IP addresses in request logs

### Claude's Discretion
- tower-http TraceLayer configuration specifics
- Exact middleware ordering in the Axum router
- Database migration implementation details for request_id column
- How to wire the request_id from middleware through to the scan spawn call

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. Key constraint: must be consistent with Phase 19's structured logging patterns (tracing spans, structured fields, JSON output).

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 20-request-tracing*
*Context gathered: 2026-02-16*
