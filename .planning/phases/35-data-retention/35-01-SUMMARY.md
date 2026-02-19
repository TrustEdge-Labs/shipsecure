---
phase: 35-data-retention
plan: 01
subsystem: database
tags: [tokio, sqlx, postgres, retention, cleanup, chrono, cancellation-token]

# Dependency graph
requires:
  - phase: 34-scan-history-dashboard
    provides: scan history dashboard that displays expires_at countdown — this phase makes those timestamps correct
  - phase: 30-stripe-removal-and-schema-cleanup
    provides: paid_audits.scan_id SET NULL FK — required before retention DELETE runs

provides:
  - Tier-conditional expires_at at scan completion (24h free, 30d authenticated)
  - delete_expired_scans_by_tier() DB function targeting completed/failed scans with 24h grace
  - Hourly retention cleanup task registered with TaskTracker for graceful shutdown
  - Per-tier structured INFO log on every cleanup run

affects:
  - Any future phase adding new tiers (must update match arm in worker_pool.rs)
  - Infrastructure/deployment phases (cleanup task fires 1 hour after deploy)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - interval_at deferred first tick pattern for background tasks (no action on deploy)
    - MissedTickBehavior::Skip for hourly cleanup tasks (prevents burst)
    - tokio::select! with CancellationToken for graceful shutdown of background loops
    - Per-tier DB DELETE callable independently to enable separate log counts

key-files:
  created:
    - src/cleanup.rs
  modified:
    - src/orchestrator/worker_pool.rs
    - src/db/scans.rs
    - src/lib.rs
    - src/main.rs

key-decisions:
  - "Free tier expires_at = NOW() + 24 hours (not 3 days); authenticated tier = NOW() + 30 days"
  - "24-hour grace period in DELETE: expires_at + INTERVAL '24 hours' < NOW() — total retention is expires_at + 24h"
  - "Only completed/failed status deleted — pending/in_progress scans are never touched"
  - "First cleanup tick deferred by 1 hour via interval_at — no DELETE fires on startup or deploy"
  - "MissedTickBehavior::Skip — if DB is slow and a tick is missed, skip to next scheduled hour"
  - "On DB error: log at ERROR, return early, no retry until next tick — tiers are independent deletes"
  - "Always log at INFO even when zero scans deleted — confirms task is running"
  - "spawn_cleanup_task called before task_tracker moves into ScanOrchestrator::new() — ownership order"
  - "No Prometheus counter for cleanup — structured logs only (locked decision)"
  - "No stuck scan detection — deferred idea, must not be present"

patterns-established:
  - "Background cleanup tasks: spawn on &TaskTracker before it moves into orchestrator"
  - "Deferred tick pattern: interval_at(Instant::now() + period, period) for no-op on deploy"
  - "Per-tier DB operations callable independently for granular structured logging"

requirements-completed: [RETN-01, RETN-02, RETN-03]

# Metrics
duration: 2min
completed: 2026-02-19
---

# Phase 35 Plan 01: Data Retention Summary

**Hourly Tokio cleanup task with tier-conditional expires_at (24h free / 30d authenticated) and PostgreSQL hard-delete with 24-hour grace period, registered with TaskTracker for graceful shutdown**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-19T02:06:52Z
- **Completed:** 2026-02-19T02:08:37Z
- **Tasks:** 2
- **Files modified:** 5 (4 modified, 1 created)

## Accomplishments
- Replaced hardcoded 3-day expires_at with tier-conditional: free tier gets 24 hours, authenticated tier gets 30 days
- Added `delete_expired_scans_by_tier()` in `src/db/scans.rs` targeting only completed/failed scans with 24-hour grace period via `expires_at + INTERVAL '24 hours' < NOW()`
- Created `src/cleanup.rs` with `spawn_cleanup_task` using deferred first tick (no DELETE on deploy), `MissedTickBehavior::Skip`, and `CancellationToken`-aware `tokio::select!` loop
- Wired cleanup task into `src/main.rs` before `task_tracker` moves into `ScanOrchestrator::new()`, ensuring graceful shutdown waits for any in-progress DELETE

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix tier-based expires_at and add delete_expired_scans_by_tier** - `bce327d` (feat)
2. **Task 2: Create cleanup module and wire into lib.rs and main.rs** - `d6335a6` (feat)

**Plan metadata:** committed with docs commit

## Files Created/Modified
- `src/cleanup.rs` - New file: hourly retention cleanup task with spawn_cleanup_task and run_cleanup
- `src/orchestrator/worker_pool.rs` - Fixed hardcoded `Duration::days(3)` to tier-conditional match (24h / 30d)
- `src/db/scans.rs` - Added `delete_expired_scans_by_tier()` public async function
- `src/lib.rs` - Added `pub mod cleanup;` declaration (alphabetically between api and db)
- `src/main.rs` - Wired `shipsecure::cleanup::spawn_cleanup_task()` call before orchestrator construction

## Decisions Made
- Free tier expires_at is 24 hours (not 3 days as previously hardcoded)
- Authenticated tier expires_at is 30 days
- `_` wildcard in match covers "free" and any unexpected tier values (short retention — fail safe)
- 24-hour grace period applies beyond expires_at before DELETE fires
- No Prometheus counter for cleanup — structured log fields only
- No stuck scan detection in this plan — deferred idea, not implemented

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Pre-existing test failure in `scanners::js_secrets::tests::test_false_positive_detection` — verified via `git stash` that this existed before any changes; unrelated to data retention work. Not fixed (out of scope per deviation rules).

## User Setup Required

None - no external service configuration required. Cleanup runs automatically on next hourly tick after deploy.

## Next Phase Readiness
- Data retention system is complete and operational
- Cleanup task will fire 1 hour after first deploy, then every hour thereafter
- Per-tier structured logs (`retention_cleanup` event) will appear in production logs confirming operation
- Dashboard's `expires_at` countdown now reflects correct retention periods (24h free, 30d authenticated)

## Self-Check: PASSED

- FOUND: src/cleanup.rs
- FOUND: src/orchestrator/worker_pool.rs
- FOUND: src/db/scans.rs
- FOUND: 35-01-SUMMARY.md
- FOUND: bce327d (Task 1 commit)
- FOUND: d6335a6 (Task 2 commit)

---
*Phase: 35-data-retention*
*Completed: 2026-02-19*
