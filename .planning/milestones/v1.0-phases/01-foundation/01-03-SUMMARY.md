---
phase: 01-foundation
plan: 03
subsystem: orchestrator
tags: [rust, sqlx, tokio, semaphore, worker-pool, database-layer]

# Dependency graph
requires:
  - phase: 01-01
    provides: Database schema with scans and findings tables, models with enums
provides:
  - Database access layer (db::scans, db::findings) with rate limiting queries
  - Scan orchestrator with semaphore-based concurrency control
  - Worker pool pattern for background scan execution
  - Scanner integration with timeout and retry logic
  - Finding deduplication and letter-grade scoring
affects: [01-04-api-handlers, 01-05-deployment, 02-free-tier]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Worker pool with semaphore for concurrency control (5 workers default)
    - Database-as-queue with SELECT FOR UPDATE SKIP LOCKED
    - Fire-and-forget spawn pattern for background tasks
    - Scanner retry logic with timeout (60s, retry once)
    - Deduplication by (scanner_name, title) tuple

key-files:
  created:
    - src/db/mod.rs
    - src/db/scans.rs
    - src/db/findings.rs
    - src/orchestrator/mod.rs
    - src/orchestrator/worker_pool.rs
    - migrations/20260204_003_add_ip_to_scans.sql
  modified:
    - src/models/scan.rs
    - src/lib.rs
    - Cargo.toml

key-decisions:
  - "Use sqlx query_as (not query_as!) to avoid DATABASE_URL requirement at compile time"
  - "Semaphore-based concurrency control with 5 workers default"
  - "60-second scanner timeout with single retry on failure"
  - "Letter-grade scoring (A+ to F) based on severity weights"
  - "Allow partial success (some scanners fail but scan completes)"

patterns-established:
  - "Database queries use &PgPool parameter pattern"
  - "Rate limiting via count_scans_by_{email,ip}_today queries"
  - "Scanner results collected into ScannerResult struct before processing"
  - "Findings deduplicated by (scanner_name, title) before persisting"

# Metrics
duration: 3m
completed: 2026-02-05
---

# Phase 01 Plan 03: Orchestrator and DB Layer Summary

**Database access layer with IP-based rate limiting, worker pool orchestrator with semaphore concurrency control, and scanner integration with retry logic**

## Performance

- **Duration:** 3m 8s
- **Started:** 2026-02-05T03:04:53Z
- **Completed:** 2026-02-05T03:08:01Z
- **Tasks:** 2
- **Files modified:** 9 created, 3 modified

## Accomplishments
- Database access layer with CRUD operations for scans and findings
- IP-based rate limiting queries (count_scans_by_email_today, count_scans_by_ip_today)
- Worker pool with semaphore-based concurrency (5 workers default)
- Scanner integration with 60s timeout and single retry on failure
- Finding deduplication and letter-grade scoring algorithm
- Database-as-queue with SELECT FOR UPDATE SKIP LOCKED for worker-safe claiming

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement database access layer** - `7fe90dc` (feat)
2. **Task 2: Implement scan orchestrator with worker pool** - `469dcdb` (feat)

## Files Created/Modified

### Created
- `src/db/mod.rs` - Database module exports
- `src/db/scans.rs` - Scan CRUD operations and rate limiting queries
- `src/db/findings.rs` - Bulk finding insertion and retrieval by scan
- `src/orchestrator/mod.rs` - Orchestrator module exports
- `src/orchestrator/worker_pool.rs` - ScanOrchestrator with semaphore concurrency
- `migrations/20260204_003_add_ip_to_scans.sql` - Add submitter_ip INET column with index

### Modified
- `src/models/scan.rs` - Added submitter_ip field for IP-based rate limiting
- `src/lib.rs` - Added db and orchestrator modules
- `Cargo.toml` - Added chrono feature to sqlx for NaiveDateTime support

## Decisions Made

**1. Use sqlx::query_as (not query_as! macro) for runtime flexibility**
- Rationale: Avoids DATABASE_URL requirement at compile time, enables building without running PostgreSQL
- Trade-off: Less compile-time type checking, but better developer experience

**2. Semaphore-based concurrency control (5 workers)**
- Rationale: Simple, effective throttling for parallel scan execution
- Trade-off: Fixed concurrency vs. dynamic scaling, but sufficient for MVP

**3. 60-second scanner timeout with single retry**
- Rationale: Balance between giving scanners time to complete and preventing hangs
- Trade-off: May timeout slow targets, but prevents resource exhaustion

**4. Letter-grade scoring (A+ to F) based on severity weights**
- Rationale: User-friendly scoring more intuitive than numeric scores
- Formula: Critical=10, High=5, Medium=2, Low=1 → weighted sum mapped to grades

**5. Allow partial success (some scanners fail)**
- Rationale: If security_headers succeeds but future scanners fail, still provide value
- Trade-off: Results may be incomplete, but better than total failure

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added chrono feature to sqlx**
- **Found during:** Task 1 (Database access layer compilation)
- **Issue:** NaiveDateTime not implementing sqlx::Decode/Type - compile errors on all db functions
- **Fix:** Added "chrono" to sqlx features in Cargo.toml
- **Files modified:** Cargo.toml
- **Verification:** cargo check passes
- **Committed in:** 7fe90dc (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Critical dependency fix for database layer. No scope creep.

## Issues Encountered

None - plan executed smoothly after chrono feature addition.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

**Ready for Plan 01-04 (API handlers and rate limiting):**
- Database layer complete with rate limiting queries
- Orchestrator ready to accept scan requests
- spawn_scan method provides fire-and-forget API for handlers

**Ready for integration:**
- Plan 01-02 (scanners module) completed in parallel - security_headers scanner integrated
- Plan 01-04 will wire API endpoints to orchestrator
- Plan 01-05 will deploy complete system

**Blockers:** None

**Concerns:** None - both wave-2 plans (01-02 and 01-03) completed successfully

---
*Phase: 01-foundation*
*Completed: 2026-02-05*
