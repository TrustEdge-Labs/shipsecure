---
phase: 33-tiered-scan-access-and-rate-limiting
plan: 01
subsystem: api
tags: [rust, axum, jwt, sqlx, postgres, tiered-access, domain-verification, orchestrator]

# Dependency graph
requires:
  - phase: 32-domain-verification
    provides: db::domains::is_domain_verified, extract_optional_clerk_user pattern
  - phase: 29-auth
    provides: ClerkClaims, Decoder<ClerkClaims>, axum-jwt-auth pattern
provides:
  - spawn_authenticated_scan method with enhanced 30-file/300s config
  - 3-arm tier match (free/authenticated/paid) in run_scanners
  - create_scan with tier + clerk_user_id DB persistence
  - Domain verification gate in create_scan (HTTP 403 for unverified domains)
  - count_scans_by_user_this_month for Developer monthly quota
  - GET /api/v1/quota endpoint returning used/limit/resets_at
affects: [33-02-rate-limiting, frontend-scan-form, dashboard-quota-badge]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "pub(crate) fn/async fn for crate-internal reuse of helpers across API modules"
    - "spawn_scan_with_tier(&'static str) private method with thin pub wrappers for tier dispatch"
    - "Mandatory Claims<ClerkClaims> extractor for auth-required endpoints (auto-401)"
    - "Optional auth + domain gate: extract_optional_clerk_user then is_domain_verified"

key-files:
  created: []
  modified:
    - src/orchestrator/worker_pool.rs
    - src/db/scans.rs
    - src/api/scans.rs
    - src/api/results.rs
    - src/main.rs

key-decisions:
  - "extract_optional_clerk_user and extract_domain_from_url made pub(crate) in results.rs — avoids duplication, single normalization source"
  - "spawn_scan_with_tier takes &'static str tier — required because tier value is embedded in span/metrics labels which need &'static lifetime"
  - "Domain gate returns 403 with /verify-domain link — 403 is semantically correct (identity confirmed, ownership not)"
  - "rate_limit::check_rate_limits call removed from create_scan — old email-based signature incompatible with new Optional<clerk_user_id> interface being built in 33-02"
  - "first_of_next_month_utc lives in scans.rs not rate_limit — quota endpoint is in scans.rs, rate limit will import it when needed in 33-02"

patterns-established:
  - "Tier dispatch: match tier { 'authenticated' => spawn_authenticated_scan, _ => spawn_scan }"
  - "Enhanced config activation: match tier { 'authenticated' | 'paid' => (30, true, 300s), _ => (20, false, 180s) }"

requirements-completed: [TIER-01, TIER-02, TIER-06]

# Metrics
duration: 4min
completed: 2026-02-18
---

# Phase 33 Plan 01: Tiered Scan Orchestration Backend Summary

**3-arm tier dispatch in scan orchestrator: authenticated users get 30-file/300s enhanced config, domain verification gate returns 403, and GET /api/v1/quota delivers monthly usage for dashboard**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-18T21:28:34Z
- **Completed:** 2026-02-18T21:31:56Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Refactored `spawn_scan` into `spawn_scan_with_tier` private method; added `spawn_authenticated_scan` public method; anonymous scans still use `spawn_scan`
- Activated tier config match in `run_scanners`: authenticated/paid tier gets 30 JS files, 300s vibecode timeout, extended exposed files (was hardcoded free-tier for all)
- Extended `db::scans::create_scan` to persist `tier` and `clerk_user_id` on every scan record
- Added `count_scans_by_user_this_month` DB function (calendar-month window via `DATE_TRUNC`)
- Wired optional JWT extraction into `create_scan` via `extract_optional_clerk_user`; domain verification gate blocks authenticated users on unverified domains with HTTP 403
- Added `GET /api/v1/quota` endpoint returning `{used, limit, resets_at}` with mandatory `Claims<ClerkClaims>` extractor (auto-401 for anonymous)
- Removed old `rate_limit::check_rate_limits` call (email-based signature, replaced in plan 33-02)

## Task Commits

Each task was committed atomically:

1. **Task 1: Refactor orchestrator for tier-aware scanning and extend DB create_scan** - `1e09add` (feat)
2. **Task 2: Wire JWT extraction, domain gate, tier routing and add quota endpoint** - `94376b1` (feat)

**Plan metadata:** (docs commit — see below)

## Files Created/Modified

- `src/orchestrator/worker_pool.rs` - Added `spawn_scan_with_tier` private method, `spawn_authenticated_scan` public method; activated tier config match in `run_scanners`
- `src/db/scans.rs` - Extended `create_scan` with `tier`/`clerk_user_id` params; added `count_scans_by_user_this_month`
- `src/api/scans.rs` - Added JWT extraction, tier computation, domain gate, updated DB call, tier-routing spawn, `get_quota` handler; removed old rate limit call
- `src/api/results.rs` - Made `extract_optional_clerk_user` and `extract_domain_from_url` `pub(crate)` for use in `scans.rs`
- `src/main.rs` - Registered `/api/v1/quota` route

## Decisions Made

- `extract_optional_clerk_user` and `extract_domain_from_url` made `pub(crate)` in `results.rs` rather than duplicated — single normalization source prevents the www-stripping mismatch pitfall documented in research
- `spawn_scan_with_tier` takes `&'static str` tier — required because the tracing span and metrics labels capture `tier` as `&'static str`; runtime strings would not compile
- Domain gate returns HTTP 403 — semantically correct: the authenticated identity is confirmed, but ownership of the domain is not
- `rate_limit::check_rate_limits` call removed entirely from `create_scan` — the old `(pool, email, ip)` signature is incompatible with the new `Option<clerk_user_id>` routing logic in plan 33-02; leaving both in place would double-rate-limit users
- `first_of_next_month_utc` placed in `scans.rs` — the quota handler owns this calculation; rate limit module will import or duplicate as needed in 33-02

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

Pre-existing test failure in `scanners::js_secrets::tests::test_false_positive_detection` was present before this plan and is unrelated to our changes (we did not modify `js_secrets.rs`). Logged to deferred items.

## Next Phase Readiness

- 33-02 can now call `db::scans::count_scans_by_user_this_month` (added this plan)
- 33-02 must implement new `check_rate_limits(pool, Option<clerk_user_id>, ip)` signature and call it from `create_scan` (the old call was removed this plan)
- Frontend can begin forwarding Clerk tokens in scan form — backend now reads and acts on them
- Dashboard can call `GET /api/v1/quota` with Authorization header to display quota badge

## Self-Check: PASSED

All files exist. All commits verified. All key patterns confirmed present.

---
*Phase: 33-tiered-scan-access-and-rate-limiting*
*Completed: 2026-02-18*
