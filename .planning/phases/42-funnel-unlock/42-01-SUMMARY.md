---
phase: 42-funnel-unlock
plan: 01
subsystem: api
tags: [rate-limiting, rust, axum, postgres, caching]

# Dependency graph
requires:
  - phase: 39-41-ci-quality-hardening
    provides: backend CI pipeline and test infrastructure

provides:
  - Two-layer anonymous rate limiting (per-target 5/domain/hour cached, per-IP 3/day hard cap)
  - Per-target scan caching returning existing scan IDs transparently
  - Domain verification gate removed for authenticated users
  - New DB queries count_scans_by_domain_last_hour and get_recent_completed_scan_for_domain

affects:
  - 42-02-funnel-unlock (scan form unlock depends on backend not enforcing domain gate)
  - 45-analytics-events (rate limit paths affect conversion event triggers)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - check_rate_limits returns Result<Option<Uuid>, ApiError> — None means proceed, Some(id) means return cached
    - Per-target cache check runs before IP cap so cached results bypass the daily counter

key-files:
  created: []
  modified:
    - src/rate_limit/middleware.rs
    - src/db/scans.rs
    - src/api/scans.rs
    - frontend/app/actions/scan.ts

key-decisions:
  - "Per-target rate limit (5/domain/hour) returns cached scan ID rather than a 429 — transparent to caller"
  - "Per-target check applies to all callers (anonymous + authenticated) for fair caching"
  - "Email+domain fair-use layer removed entirely — two layers sufficient (per-target + per-IP)"
  - "Domain verification gate removed for authenticated users — reduces friction without security regression"

patterns-established:
  - "Rate limiter returns Option<Uuid>: None = proceed, Some(id) = use cached results"
  - "Per-target hourly cache check runs before IP hard cap so cached hits don't consume daily quota"

requirements-completed: [FUNNEL-02, FUNNEL-03, FUNNEL-04]

# Metrics
duration: 20min
completed: 2026-03-30
---

# Phase 42 Plan 01: Rate Limiting Overhaul Summary

**Two-layer anonymous rate limiting (3/IP/day hard cap + 5/domain/hour with transparent cached results) and domain verification gate removed from Rust backend and Next.js server action**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-03-30T22:30:00Z
- **Completed:** 2026-03-30T22:50:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Overhauled `check_rate_limits` signature from `Result<(), ApiError>` to `Result<Option<Uuid>, ApiError>` — returns cached scan ID when per-target limit exceeded
- Lowered `ANONYMOUS_IP_DAILY_HARD_CAP` from 10 to 3, added `PER_TARGET_HOURLY_CAP = 5`
- Added `count_scans_by_domain_last_hour` and `get_recent_completed_scan_for_domain` DB queries
- Removed email+domain fair-use layer from rate limiter entirely
- Removed domain verification gate from `src/api/scans.rs` (18 lines deleted)
- Removed domain verification client-side check from `frontend/app/actions/scan.ts` (21 lines deleted)

## Task Commits

Each task was committed atomically:

1. **Task 1: Overhaul rate limiting — new per-target layer, lower IP cap, remove email+domain layer** - `2abf9e7` (feat)
2. **Task 2: Remove domain verification gate from backend and frontend server action** - `29775a0` (feat)

## Files Created/Modified
- `src/rate_limit/middleware.rs` - Rewritten: new return type, per-target layer, 3/day IP cap, removed email+domain layer
- `src/db/scans.rs` - Added count_scans_by_domain_last_hour and get_recent_completed_scan_for_domain
- `src/api/scans.rs` - Domain verification gate removed, rate limit call updated for Option<Uuid> return, cached response added
- `frontend/app/actions/scan.ts` - Domain verification block removed (domainsRes, DOMAIN_VERIFICATION_REQUIRED)

## Decisions Made
- Per-target cache hit returns 200 with `cached: true` in JSON body (not a redirect) so frontend can handle it in the existing `data.id` path
- Per-target check placed before IP cap so cached results bypass the daily anonymous quota (users don't burn their 3/day on repeated lookups of popular domains)
- `_email` parameter kept in `check_rate_limits` signature with underscore prefix to avoid breaking callers in `src/api/scans.rs` while removing the email+domain layer

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Backend rate limiter and domain gate fully overhauled — ready for Plan 42-02 (scan form unlock)
- Plan 42-02 (scan-form Juice Shop unlock) committed by parallel agent — both plans coordinate cleanly on backend-only vs frontend-only changes

---
*Phase: 42-funnel-unlock*
*Completed: 2026-03-30*
