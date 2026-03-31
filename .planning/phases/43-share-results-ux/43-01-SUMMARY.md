---
phase: 43-share-results-ux
plan: 01
subsystem: api, database
tags: [rust, postgres, axum, soft-delete, scan-retention, expiry]

requires:
  - phase: 42-funnel-unlock
    provides: Per-target rate limiting and domain verification removal
provides:
  - "scan_status 'expired' DB enum value via migration"
  - "ScanStatus::Expired Rust variant"
  - "soft_expire_scans_by_tier: UPDATE-based soft-delete replacing hard DELETE"
  - "get_scan_by_token_including_expired: token lookup with no expiry filter"
  - "Results endpoint returns expired scan data (target_url + status='expired') at HTTP 200"
  - "Download endpoint returns 410 Gone for expired scans"
affects: [43-share-results-ux, frontend-results-page, phase-44, phase-45]

tech-stack:
  added: []
  patterns:
    - "Soft-delete via status='expired' — cleanup is non-destructive, data preserved for UX tombstone"
    - "Two-phase token lookup: active query first, expired fallback second"
    - "410 Gone for binary download endpoints, 200 with tombstone JSON for API endpoints"

key-files:
  created:
    - migrations/20260331000001_add_expired_status.sql
  modified:
    - src/models/scan.rs
    - src/db/scans.rs
    - src/cleanup.rs
    - src/api/results.rs

key-decisions:
  - "Soft-expire has no grace period — marking expired is non-destructive so expires_at is the exact cutoff (unlike hard-delete which had 24h grace)"
  - "delete_expired_scans_by_tier retained for potential future hard-delete pass on very old 'expired' rows"
  - "Expired API response returns HTTP 200 (not 404/410) with minimal JSON — frontend can detect status='expired' and render CTA"
  - "Download endpoint returns 410 Gone for expired scans — binary download has no tombstone equivalent"

patterns-established:
  - "Two-phase lookup pattern: get_scan_by_token (active only) → get_scan_by_token_including_expired (fallback for expired)"
  - "Expired JSON tombstone shape: {id, target_url, status:'expired', score:null, tier, expires_at, created_at, completed_at, findings:[], summary:{zeros}, owner_verified:false}"

requirements-completed: [FUNNEL-06]

duration: 3min
completed: 2026-03-31
---

# Phase 43 Plan 01: Soft-Delete Scan Expiry Summary

**PostgreSQL enum extended with 'expired' status; cleanup task switched to soft-expire; results API returns tombstone JSON with target_url for scan-again CTA instead of 404**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-31T02:34:47Z
- **Completed:** 2026-03-31T02:37:25Z
- **Tasks:** 2
- **Files modified:** 4 (+ 1 migration created)

## Accomplishments

- Migration adds 'expired' enum value to scan_status via `ALTER TYPE ... ADD VALUE IF NOT EXISTS`
- ScanStatus::Expired Rust variant added with sqlx snake_case mapping
- Cleanup task soft-expires instead of deleting — `run_cleanup` now calls `soft_expire_scans_by_tier` which UPDATEs status, preserving data
- Results endpoint falls back to `get_scan_by_token_including_expired` when active lookup returns None; expired scans get HTTP 200 with tombstone JSON including target_url
- Download endpoint returns 410 Gone for expired scans with clear scan-again message

## Task Commits

1. **Task 1: Database migration and Rust model update** - `d9f293e` (feat)
2. **Task 2: Soft-delete cleanup, expired scan query, results endpoint** - `a8249f9` (feat)

## Files Created/Modified

- `migrations/20260331000001_add_expired_status.sql` - ALTER TYPE scan_status ADD VALUE 'expired'
- `src/models/scan.rs` - Expired variant added to ScanStatus enum
- `src/db/scans.rs` - soft_expire_scans_by_tier + get_scan_by_token_including_expired added
- `src/cleanup.rs` - run_cleanup switched to soft_expire; log fields renamed *_expired
- `src/api/results.rs` - Two-phase token lookup; expired tombstone response; 410 for download

## Decisions Made

- Soft-expire has no 24h grace period (unlike old hard-delete). Non-destructive update means expires_at is the exact cutoff — no reason to delay.
- `delete_expired_scans_by_tier` retained (not removed). Future phases may want a hard-delete pass on rows that have been expired for 30+ days.
- Expired results return HTTP 200 with `status: "expired"` (not 404/410). Frontend can branch on status field and show the scan-again UX. A 404 would require the frontend to hit a different endpoint.
- Download endpoint returns 410 Gone — there is no meaningful tombstone for a markdown binary download, and 410 correctly communicates "this resource existed but is permanently gone."

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - `cargo check` and `cargo test` (56 tests) passed on first attempt.

## User Setup Required

The migration `migrations/20260331000001_add_expired_status.sql` must run against production PostgreSQL before deploying. The CI/CD pipeline runs `sqlx migrate run` automatically on deploy.

## Next Phase Readiness

- Backend ready for Phase 43 Plan 02 (frontend expired results page)
- `target_url` is preserved in expired tombstone response — frontend can pre-fill scan-again form
- `status: "expired"` in API response is the branch condition the frontend component needs
- No breaking changes to existing non-expired scan lookups

---
*Phase: 43-share-results-ux*
*Completed: 2026-03-31*
