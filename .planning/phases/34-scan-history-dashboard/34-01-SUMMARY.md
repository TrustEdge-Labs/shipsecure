---
phase: 34-scan-history-dashboard
plan: 01
subsystem: api
tags: [axum, sqlx, postgres, jwt, clerk, pagination]

# Dependency graph
requires:
  - phase: 33-tiered-scan-access
    provides: AppState with pool and jwt_decoder; ClerkClaims; tier-aware scan model
  - phase: 29-auth-foundation
    provides: axum-jwt-auth Claims extractor pattern; ClerkClaims struct
provides:
  - GET /api/v1/users/me/scans paginated endpoint with mandatory JWT auth
  - ScanHistoryRow projection struct with severity counts via LEFT JOIN aggregation
  - get_user_scan_history, count_user_scans_history, get_user_active_scans DB functions
affects: [34-02-scan-history-frontend]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - tokio::try_join! for concurrent DB queries in a single handler
    - ScanHistoryRow projection type (query-result struct lives in db/ not models/)
    - Separate active_scans (pending/in_progress) from history (completed/failed) in one API call

key-files:
  created:
    - src/api/users.rs
  modified:
    - src/db/scans.rs
    - src/api/mod.rs
    - src/main.rs

key-decisions:
  - "ScanHistoryRow lives in src/db/scans.rs — it is a query projection type, not a domain model"
  - "Per-page hardcoded at 10 — locked decision from plan"
  - "History sorted expiring-soonest-first: CASE WHEN expires_at IS NULL THEN 1 ELSE 0 END ASC, expires_at ASC NULLS LAST, created_at DESC"
  - "Active scans use hardcoded 0::bigint severity counts — no findings exist yet for in-progress scans"

patterns-established:
  - "tokio::try_join! pattern: run three DB queries concurrently in a single API handler"
  - "Claims { claims, .. } struct destructuring for mandatory JWT auth (rejects anonymous with 401)"

requirements-completed: [DASH-01]

# Metrics
duration: 2min
completed: 2026-02-19
---

# Phase 34 Plan 01: Scan History Backend Summary

**Paginated GET /api/v1/users/me/scans returning scan history with LEFT JOIN severity counts and separate active scan list using concurrent tokio::try_join! DB queries**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-19T00:19:58Z
- **Completed:** 2026-02-19T00:21:15Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- ScanHistoryRow projection struct and three DB query functions added to src/db/scans.rs (history, active, count)
- New src/api/users.rs handler with mandatory JWT auth (Claims extractor returns 401 for anonymous callers)
- Route GET /api/v1/users/me/scans registered in main.rs; three DB queries run concurrently with tokio::try_join!

## Task Commits

Each task was committed atomically:

1. **Task 1: Add ScanHistoryRow and DB query functions** - `d218f76` (feat)
2. **Task 2: Create users API handler and register route** - `fa5d1c6` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `src/db/scans.rs` - Added ScanHistoryRow struct, get_user_scan_history (paginated + LEFT JOIN severity), count_user_scans_history, get_user_active_scans
- `src/api/users.rs` - New file: get_user_scans handler; tokio::try_join! for concurrent DB queries; pagination metadata
- `src/api/mod.rs` - Added pub mod users
- `src/main.rs` - Added users import and /api/v1/users/me/scans route registration

## Decisions Made

- ScanHistoryRow lives in src/db/scans.rs as a query projection type (not a domain model in models/)
- Per-page is hardcoded at 10 per the locked plan decision
- History sorted expiring-soonest-first per locked decision (non-null expires_at before null, then created_at DESC)
- Active scans (pending/in_progress) returned separately with hardcoded zero severity counts — no findings exist yet

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Backend endpoint fully implemented and cargo check passes
- Plan 02 (frontend dashboard) can consume GET /api/v1/users/me/scans with page query param
- Response shape: { scans[], active_scans[], total, page, per_page, total_pages }
- Each scan row includes: id, target_url, status, results_token, expires_at, tier, created_at, critical_count, high_count, medium_count, low_count

## Self-Check: PASSED

- src/api/users.rs: FOUND
- src/db/scans.rs: FOUND
- 34-01-SUMMARY.md: FOUND
- Commit d218f76 (Task 1): FOUND
- Commit fa5d1c6 (Task 2): FOUND

---
*Phase: 34-scan-history-dashboard*
*Completed: 2026-02-19*
