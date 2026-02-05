---
phase: 01-foundation
plan: 04
subsystem: api
tags: [rust, axum, rest-api, rate-limiting, error-handling, rfc-7807]

# Dependency graph
requires:
  - phase: 01-01
    provides: Database schema, models with ScanStatus and Severity enums
  - phase: 01-02
    provides: SSRF validator and security headers scanner
  - phase: 01-03
    provides: Scan orchestrator worker pool and database layer
provides:
  - REST API endpoints (POST /api/v1/scans, GET /api/v1/scans/{id})
  - RFC 7807 Problem Details error responses
  - Database-backed rate limiting (3/email/day, 10/IP/day)
  - Axum application state with pool and orchestrator
  - ConnectInfo extraction for client IP
affects: [01-05-deployment, 02-free-tier]

# Tech tracking
tech-stack:
  added:
    - http-body-util 0.1 (dev dependency for testing)
  patterns:
    - RFC 7807 Problem Details for all API errors with Content-Type application/problem+json
    - ApiError enum with IntoResponse for ergonomic error handling
    - From<SsrfError> and From<sqlx::Error> conversions for ? operator
    - Database-backed rate limiting (not in-memory) for persistence across restarts
    - Fire-and-forget orchestrator spawn after scan creation
    - Axum State extractor for dependency injection
    - ConnectInfo<SocketAddr> for client IP extraction

key-files:
  created:
    - src/api/mod.rs
    - src/api/errors.rs
    - src/api/scans.rs
    - src/rate_limit/mod.rs
    - src/rate_limit/middleware.rs
  modified:
    - src/lib.rs
    - src/main.rs
    - src/orchestrator/worker_pool.rs
    - Cargo.toml

key-decisions:
  - "RFC 7807 Problem Details manually implemented (not using problem_details crate)"
  - "Rate limiting is database function called in handler (not Tower middleware)"
  - "Email-based limiting (3/day) checked before IP-based (10/day)"
  - "SSRF validator returns normalized URL which is stored in database"
  - "Scan findings summarized by severity in GET response"
  - "into_make_service_with_connect_info for SocketAddr extraction"

patterns-established:
  - "API error responses: type URI, title, status, detail fields"
  - "Rate limit messages specify limit and suggest retry timing"
  - "Internal errors logged with tracing::error! but sanitized for client"
  - "Input validation before SSRF check before rate limit check"
  - "201 Created with Location-style url field for POST responses"
  - "Summary object in scan response with counts by severity"

# Metrics
duration: 3min
completed: 2026-02-05
---

# Phase 01 Plan 04: API Handlers and Rate Limiting Summary

**Complete REST API with RFC 7807 error handling, database-backed rate limiting, and end-to-end scan flow from submission to results**

## Performance

- **Duration:** 2min 53sec
- **Started:** 2026-02-05T03:13:16Z
- **Completed:** 2026-02-05T03:16:09Z
- **Tasks:** 2
- **Files created:** 5
- **Files modified:** 4

## Accomplishments

- REST API with POST /api/v1/scans (create) and GET /api/v1/scans/{id} (query results)
- RFC 7807 Problem Details error responses for all error types with proper Content-Type headers
- Database-backed rate limiting (3 scans/email/day, 10 scans/IP/day) with descriptive 429 responses
- Full end-to-end integration: input validation → SSRF check → rate limiting → scan creation → orchestrator spawn → results query
- Comprehensive unit tests for error response format and HTTP status codes

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement API error handling and rate limiting** - `88a4ce7` (feat)
   - RFC 7807 error responses with ApiError enum (ValidationError, SsrfBlocked, RateLimited, NotFound, InternalError)
   - From impls for SsrfError and sqlx::Error for ergonomic error handling with ? operator
   - Database-backed rate limiting function checking email and IP limits
   - 5 unit tests for error response format, Content-Type, and status codes
   - Fixed orchestrator test imports for Severity enum

2. **Task 2: Implement scan API handlers and wire main.rs** - `ea83378` (feat)
   - POST /api/v1/scans: validate input, check SSRF, enforce rate limits, create scan, spawn orchestrator
   - GET /api/v1/scans/{id}: return scan status, findings array, and summary counts by severity
   - AppState struct with PgPool and Arc<ScanOrchestrator> for shared state
   - Complete main.rs rewrite: database pool, migrations, orchestrator initialization, route registration
   - into_make_service_with_connect_info for ConnectInfo<SocketAddr> extraction

## Files Created/Modified

**Created:**
- `src/api/mod.rs` - Module exports for scans and errors
- `src/api/errors.rs` - ApiError enum with RFC 7807 Problem Details responses, From impls, unit tests
- `src/api/scans.rs` - POST/GET handlers with validation, SSRF checks, rate limiting, AppState
- `src/rate_limit/mod.rs` - Module exports for rate limiting
- `src/rate_limit/middleware.rs` - Database-backed rate limit check function

**Modified:**
- `src/lib.rs` - Added pub mod api and pub mod rate_limit
- `src/main.rs` - Complete rewrite with Axum router, state, routes, ConnectInfo
- `src/orchestrator/worker_pool.rs` - Added Severity import in tests
- `Cargo.toml` - Added http-body-util dev dependency for testing

## Decisions Made

1. **RFC 7807 manually implemented** - Rather than adding a problem_details crate dependency, implemented ProblemDetails struct directly. Gives full control over response format and keeps dependencies minimal.

2. **Rate limiting as handler function, not middleware** - Email-based rate limiting requires reading request body, which can't happen in Tower middleware without buffering. Database-backed approach is simpler and persists across restarts (unlike in-memory tower-governor). MVP scale doesn't need high-performance in-memory limiting.

3. **Email limit checked before IP limit** - Check more restrictive limit first (3/email/day) before looser limit (10/IP/day). Provides better error messages for users hitting their personal limit vs. shared IP limit.

4. **SSRF validator returns normalized URL** - Store the validated/normalized URL from the SSRF validator rather than the raw user input. Ensures consistency and prevents variations of the same URL from bypassing deduplication.

5. **Summary counts in GET response** - Include finding counts by severity (critical/high/medium/low) alongside findings array. Enables UI to show summary statistics without client-side aggregation.

6. **Axum 0.8 path syntax** - Use `{id}` not `:id` for path parameters (changed in Axum 0.8). Use into_make_service_with_connect_info for ConnectInfo extraction.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - all code compiled and tests passed on first attempt after fixing the ProblemDetails Deserialize derive and unused imports.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

**Ready for deployment (Plan 01-05):**
- API handlers operational and tested
- Error handling follows RFC 7807 standard
- Rate limiting prevents abuse
- ConnectInfo configured for IP extraction
- Database migrations included in startup

**Ready for Phase 2 (Free Tier MVP):**
- API contract established for frontend integration
- Rate limits appropriate for free tier (3/email/day)
- SSRF protection prevents abuse of scanner infrastructure
- Findings response format stable for UI development

**No blockers:** All foundational API infrastructure complete.

---
*Phase: 01-foundation*
*Completed: 2026-02-05*
