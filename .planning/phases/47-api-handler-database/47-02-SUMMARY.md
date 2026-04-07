---
phase: 47-api-handler-database
plan: 02
subsystem: api
tags: [axum, multipart, reqwest, supply-chain, github-api, base64, tokio-timeout]

# Dependency graph
requires:
  - phase: 46-backend-parsing
    provides: "scan_lockfile(), lockfile_parser::parse(), SupplyChainError, SupplyChainScanResult"
  - phase: 47-api-handler-database (plan 01)
    provides: "create_supply_chain_scan, complete_supply_chain_scan, fail_supply_chain_scan DB functions"
provides:
  - "POST /api/v1/scans/supply-chain endpoint with 3 input modes"
  - "SupplyChainError -> ApiError HTTP status mapping"
  - "GitHub URL lockfile fetcher with main/master fallback"
  - "Token-based result sharing with 30-day expiry"
affects: [48-frontend-pages, 49-testing]

# Tech tracking
tech-stack:
  added: [multer (via axum multipart feature)]
  patterns: [graceful-db-fallback, multipart-from-raw-body, nested-router-body-limit]

key-files:
  created: [src/api/supply_chain.rs]
  modified: [src/api/mod.rs, src/main.rs, Cargo.toml]

key-decisions:
  - "Used axum multipart feature (adds multer dep) for file upload parsing"
  - "Nested Router with DefaultBodyLimit for 5MB limit scoped to supply chain route only"
  - "Pre-check dep count via lockfile_parser::parse() before scan_lockfile() — double parse acceptable (~ms)"
  - "DB row creation failure is non-fatal — results returned inline with share_url: null"
  - "Used r#gen() for rand::Rng in Rust 2024 edition (gen is reserved keyword)"

patterns-established:
  - "Graceful DB fallback: scan results always returned even if DB write fails"
  - "Nested router for per-route body limits in axum"
  - "Strict GitHub URL regex for SSRF mitigation (only github.com domain allowed)"

requirements-completed: [API-01, API-02, API-03, API-04, API-05, API-06, RES-03, RES-04]

# Metrics
duration: 5min
completed: 2026-04-07
---

# Phase 47 Plan 02: Supply Chain HTTP Handler Summary

**Supply chain scan endpoint with JSON/multipart/GitHub URL input modes, error mapping to HTTP status codes, and graceful DB persistence with token sharing**

## Performance

- **Duration:** 5 min
- **Started:** 2026-04-07T02:32:40Z
- **Completed:** 2026-04-07T02:37:36Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- POST /api/v1/scans/supply-chain handler accepting 3 input modes (GitHub URL, lockfile paste, multipart file upload)
- GitHub URL fetcher with strict regex SSRF mitigation, main/master branch fallback, optional GITHUB_TOKEN auth
- 6 SupplyChainError variants mapped to correct HTTP status codes (400/502/504)
- Abuse controls: 5000 dep cap, 5MB body limit, 30s scan timeout
- Token-based result sharing with 30-day expiry and graceful DB write fallback
- Route registered in main.rs without JWT middleware (anonymous access)

## Task Commits

Each task was committed atomically:

1. **Task 1: Supply chain handler with 3 input modes, GitHub fetch, and error mapping** - `12b39be` (feat)
2. **Task 2: Route registration and body size limit** - `99ba0ca` (feat)

## Files Created/Modified
- `src/api/supply_chain.rs` - Supply chain scan handler with 3 input modes, GitHub fetcher, error mapping, token generation, DB persistence
- `src/api/mod.rs` - Added `pub mod supply_chain` registration
- `src/main.rs` - Route registration with nested router for 5MB body limit, supply_chain import
- `Cargo.toml` - Enabled axum `multipart` feature (adds multer dependency)

## Decisions Made
- Used axum's built-in multipart feature rather than a separate crate -- adds multer as transitive dep
- Scoped 5MB body limit to supply chain route via nested Router::new().merge() pattern -- avoids affecting existing 2MB default on other routes
- Pre-check dep count by calling lockfile_parser::parse() before scan_lockfile() -- double parse is negligible (~ms) and avoids modifying shipped Phase 46 code
- DB row creation failure treated as non-fatal -- results always returned to user with share_url: null

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Enabled axum multipart Cargo feature**
- **Found during:** Task 1 (handler compilation)
- **Issue:** axum::extract::Multipart requires the `multipart` feature flag which was not enabled
- **Fix:** Changed `axum = "0.8.8"` to `axum = { version = "0.8.8", features = ["multipart"] }` in Cargo.toml
- **Files modified:** Cargo.toml
- **Verification:** cargo check passes
- **Committed in:** 12b39be (Task 1 commit)

**2. [Rule 1 - Bug] Fixed reserved keyword `gen` in Rust 2024 edition**
- **Found during:** Task 1 (handler compilation)
- **Issue:** `rng.gen()` fails because `gen` is a reserved keyword in Rust 2024 edition
- **Fix:** Changed to `rng.r#gen()` (raw identifier syntax)
- **Files modified:** src/api/supply_chain.rs
- **Verification:** cargo check passes
- **Committed in:** 12b39be (Task 1 commit)

**3. [Rule 1 - Bug] Fixed clippy collapsible_if warning**
- **Found during:** Task 1 (clippy check)
- **Issue:** Nested if with content_length check flagged by clippy -D warnings
- **Fix:** Combined into `if let Some(len) = resp.content_length() && len as usize > MAX_BODY_SIZE`
- **Files modified:** src/api/supply_chain.rs
- **Verification:** cargo clippy -- -D warnings passes
- **Committed in:** 12b39be (Task 1 commit)

---

**Total deviations:** 3 auto-fixed (2 bug fixes, 1 blocking)
**Impact on plan:** All auto-fixes necessary for compilation. No scope creep.

## Issues Encountered
None beyond the auto-fixed items above.

## User Setup Required
None - no external service configuration required. GITHUB_TOKEN env var is optional (improves GitHub API rate limits but not required).

## Next Phase Readiness
- Supply chain scan endpoint is wired and compiles
- Ready for frontend pages (Phase 48) to call POST /api/v1/scans/supply-chain
- Ready for testing (Phase 49) to add integration/E2E tests
- All 116 existing tests continue to pass

---
*Phase: 47-api-handler-database*
*Completed: 2026-04-07*

## Self-Check: PASSED

- src/api/supply_chain.rs: FOUND
- Commit 12b39be: FOUND
- Commit 99ba0ca: FOUND
- SUMMARY.md: FOUND
