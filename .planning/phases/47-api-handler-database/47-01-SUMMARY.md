---
phase: 47-api-handler-database
plan: 01
subsystem: database
tags: [postgres, sqlx, migration, jsonb, supply-chain]

requires:
  - phase: none
    provides: existing scans table schema and queries
provides:
  - "kind VARCHAR(20) column on scans table discriminating web_app vs supply_chain"
  - "supply_chain_results JSONB column for storing scan output"
  - "Kind-aware queries: 7 web_app-filtered queries ensuring dashboard/stats/quota correctness"
  - "3 new DB functions: create_supply_chain_scan, complete_supply_chain_scan, fail_supply_chain_scan"
  - "Updated Scan model with kind and supply_chain_results fields"
affects: [48-api-handler-endpoint, 49-frontend-supply-chain]

tech-stack:
  added: []
  patterns: ["kind discriminator column for multi-scan-type table", "JSONB for denormalized scan results"]

key-files:
  created:
    - "migrations/20260407000001_add_supply_chain_columns.sql"
  modified:
    - "src/models/scan.rs"
    - "src/db/scans.rs"

key-decisions:
  - "7 queries filtered by kind='web_app' (5 critical + history count + active scans) -- more than the 5 minimum in the plan"
  - "soft_expire_scans_by_tier left unfiltered -- expires both web_app and supply_chain scans by tier, which is correct behavior"
  - "UUID-based lookups (get_scan, get_scan_by_token, get_scan_by_token_including_expired) remain unfiltered per plan design"

patterns-established:
  - "kind discriminator: all aggregate/list queries on scans table must include AND kind = 'web_app' or explicit kind filter"
  - "supply chain DB functions: create sets kind='supply_chain' + 30-day expiry; complete sets results + token; fail sets error"

requirements-completed: [DB-01, DB-02, DB-03, DB-04]

duration: 3min
completed: 2026-04-07
---

# Phase 47 Plan 01: Database Migration and Query Audit Summary

**Schema migration adding kind discriminator + JSONB results column, with 7 kind-aware query filters and 3 new supply chain DB functions**

## Performance

- **Duration:** 3 min
- **Started:** 2026-04-07T02:28:08Z
- **Completed:** 2026-04-07T02:30:59Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Migration adds kind VARCHAR(20) DEFAULT 'web_app' (backwards-compatible, all existing rows auto-tagged) and supply_chain_results JSONB
- Scan model updated with kind and supply_chain_results fields; all 6 Scan-returning queries include new columns
- 7 queries filtered by kind='web_app': scan history, history count, monthly quota, domain rate limit, domain cache, completed scan stats, active scans
- 3 new supply chain DB functions: create (with tier detection + 30-day expiry), complete (results + token), fail (error recording)

## Task Commits

Each task was committed atomically:

1. **Task 1: Migration file + Scan model update** - `96f9f31` (feat)
2. **Task 2: Query audit + supply chain DB functions** - `97ce39f` (feat)

## Files Created/Modified
- `migrations/20260407000001_add_supply_chain_columns.sql` - Schema migration adding kind + supply_chain_results columns
- `src/models/scan.rs` - Scan struct with kind (String) and supply_chain_results (Option<serde_json::Value>) fields
- `src/db/scans.rs` - 7 kind='web_app' filters on aggregate/list queries + 3 new supply chain CRUD functions

## Decisions Made
- Added kind filter to `get_user_active_scans()` beyond the 5 critical queries in the plan -- active scan list is dashboard-specific and should only show web_app scans
- Added kind filter to `count_user_scans_history()` to match the history query filter -- pagination count must agree with filtered results
- Left `soft_expire_scans_by_tier()` and `delete_expired_scans_by_tier()` unfiltered -- both scan kinds should be expired/deleted by their tier, which is correct behavior

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required. Migration will run automatically via `sqlx migrate run` on next deploy.

## Next Phase Readiness
- Schema and queries ready for the API handler endpoint plan (47-02)
- Scan model supports both web_app and supply_chain kinds
- Supply chain DB functions ready for the scan orchestrator to call

---
*Phase: 47-api-handler-database*
*Completed: 2026-04-07*
