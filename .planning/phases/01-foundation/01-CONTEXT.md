# Phase 1: Foundation - Context

**Gathered:** 2026-02-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Backend infrastructure operational with core scanning capability. Delivers: scan orchestrator, findings aggregator, rate limiting, SSRF protection, containerized scanner execution, and security headers analysis. Users interact via API only — no frontend in this phase.

</domain>

<decisions>
## Implementation Decisions

### API contract design
- Path-based versioning from day one: `/api/v1/scans`
- Scan submission requires URL + email (both required)
- Submission response pattern: Claude's discretion (polling vs SSE)
- Error format: RFC 7807 Problem Details (type, title, status, detail)

### Job orchestration model
- In-process worker pool using Tokio tasks within the Axum process
- Low concurrency: 3-5 concurrent scans max (Render starter tier constraints)
- 60-second timeout per scanner
- Failure strategy: retry failed scanner once, then return partial results with failure noted
- Partial results returned for any scanners that completed successfully

### Findings data model
- 4-level severity: Critical, High, Medium, Low
- Include remediation guidance from Phase 1 (plain-language explanation + fix suggestion per finding)
- Deduplication by finding type: merge findings from multiple scanners into one, noting all sources
- Compute A-F security score in Phase 1 aggregator (based on findings severity/count)

### Rate limiting & SSRF rules
- Rate limiting scoped to email AND IP: 3 scans/day per email, 10 scans/day per IP
- Rate limit response: 429 with clear message ("You've reached your daily scan limit"), no specific retry time
- SSRF protection: block RFC 1918 private ranges, localhost, link-local, and cloud metadata endpoints (169.254.169.254)
- DNS rebinding protection deferred (not in Phase 1)
- Consent model: TOS acceptance on scan submission (checkbox or implied by clicking "Scan")

### Claude's Discretion
- Scan submission response pattern (polling URL vs SSE stream)
- Database schema design for scans and findings tables
- Security headers scanner implementation approach (in-process HTTP client vs containerized)
- Score computation algorithm (weighting of severity levels)
- Exact SSRF blocklist implementation (regex vs IP parsing library)

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. Key constraint: must work on Render hosting with conservative resource usage.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-foundation*
*Context gathered: 2026-02-04*
