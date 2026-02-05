---
phase: 02-free-tier-mvp
plan: 05
subsystem: api
tags: [axum, rust, rest-api, cors, markdown]

# Dependency graph
requires:
  - phase: 02-01
    provides: Database schema with results_token, stage tracking, and scan counter functions
provides:
  - Token-based results retrieval endpoint (privacy-preserving)
  - Markdown report download endpoint with formatted findings
  - Scan counter endpoint for social proof
  - Stage progress tracking in scan status responses
  - CORS-enabled API for frontend communication
affects: [02-06-frontend, 02-07-worker-execution]

# Tech tracking
tech-stack:
  added: [tower-http CORS middleware]
  patterns: [RFC 7807 custom error responses, privacy-preserving API design, markdown report generation]

key-files:
  created:
    - src/api/results.rs
    - src/api/stats.rs
  modified:
    - src/api/scans.rs
    - src/api/mod.rs
    - src/api/errors.rs
    - src/main.rs

key-decisions:
  - "Return token instead of scan ID in results endpoint (privacy)"
  - "Exclude email and IP from results responses (no PII)"
  - "Use markdown for downloadable reports (developer-friendly)"
  - "Apply CORS to all routes via layer (simpler than per-endpoint)"

patterns-established:
  - "RFC 7807 Custom variant in ApiError for flexible error responses"
  - "Markdown report format with summary table and findings by severity"
  - "Privacy-first API design excluding PII from public endpoints"

# Metrics
duration: 3min
completed: 2026-02-05
---

# Phase 02 Plan 05: API Endpoints Summary

**Token-based results retrieval, markdown downloads, scan progress stages, and social proof counter - backend API complete for Phase 2**

## Performance

- **Duration:** 2 minutes 47 seconds
- **Started:** 2026-02-05T14:12:24Z
- **Completed:** 2026-02-05T14:15:07Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Token-based results access with automatic expiry checking (privacy-preserving)
- Markdown report generation with severity-grouped findings and remediation guidance
- Scan progress tracking with four stage booleans (headers, TLS, files, secrets)
- Social proof scan counter endpoint for landing page
- CORS-enabled API ready for Next.js frontend integration

## Task Commits

Each task was committed atomically:

1. **Task 1: Token-based results endpoint and markdown download** - `2cbb57e` (feat)
2. **Task 2: Scan progress stages and social proof counter** - `59d22f1` (feat)

## Files Created/Modified
- `src/api/results.rs` - Token-based results retrieval and markdown download handlers
- `src/api/stats.rs` - Scan counter endpoint for social proof
- `src/api/scans.rs` - Added stage_* fields and results_token to scan status response
- `src/api/mod.rs` - Registered results and stats modules
- `src/api/errors.rs` - Extended ApiError with Custom variant for RFC 7807 flexibility
- `src/main.rs` - Registered 3 new routes and added CORS middleware layer

## Decisions Made

**1. Privacy-preserving results endpoint**
- Return token as "id" instead of internal UUID (prevent correlation)
- Exclude email and submitter_ip from results responses
- Rationale: Free tier results are public via token, must not leak PII

**2. Markdown format for downloads**
- Generate structured markdown with severity grouping and remediation
- Include scan grade, timestamp, and summary table
- Rationale: Developer-friendly format, easily viewable and shareable

**3. CORS applied globally via layer**
- Use `tower-http` CorsLayer on entire router
- Allow any origin for development (production restriction noted in comment)
- Rationale: Simpler than per-endpoint CORS, consistent behavior across all routes

**4. RFC 7807 Custom error variant**
- Extended ApiError enum with Custom variant for flexible error responses
- Allows specific error types and details for different failure scenarios
- Rationale: Expired token needs different message than not-found scan

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - implementation proceeded smoothly with clear database functions from Plan 02-01.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

**Ready for frontend integration (Plan 02-06):**
- GET /api/v1/scans/:id returns scan with stage progress and results_token
- GET /api/v1/results/:token returns full results (privacy-preserving)
- GET /api/v1/results/:token/download returns markdown report
- GET /api/v1/stats/scan-count returns total completed scans
- CORS enabled for cross-origin requests from Next.js dev server

**API surface complete:**
- All 6 endpoints implemented and tested with cargo check
- Error handling with RFC 7807 compliance
- Privacy-first design excluding PII from public endpoints

**No blockers** - frontend can begin consuming these endpoints immediately.

---
*Phase: 02-free-tier-mvp*
*Completed: 2026-02-05*
