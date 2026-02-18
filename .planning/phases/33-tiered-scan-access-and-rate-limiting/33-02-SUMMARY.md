---
phase: 33-tiered-scan-access-and-rate-limiting
plan: 02
subsystem: api
tags: [rust, axum, sqlx, nextjs, clerk, rate-limiting, tiered-access, typescript]

# Dependency graph
requires:
  - phase: 33-01
    provides: spawn_authenticated_scan, domain verification gate, GET /api/v1/quota, count_scans_by_user_this_month
  - phase: 32-domain-verification
    provides: GET /api/v1/domains endpoint for domain verification pre-check in scan action

provides:
  - Tier-aware rate limiting: anonymous 1/IP/24h, authenticated Developer 5/user/month
  - RateLimitedWithReset error variant with resets_at ISO timestamp in 429 JSON body
  - count_anonymous_scans_by_ip_today (clerk_user_id IS NULL filter prevents cross-tier inflation)
  - Auth-aware scan action forwarding Clerk Bearer token when authenticated
  - Domain verification pre-check in scan server action for authenticated users
  - 429 countdown display in scan form error ("Resets in 18h 23m")
  - Tier badge in results page header (Basic scan / Enhanced scan)
  - Quota badge in dashboard (X/5 scans, color-coded green/yellow/red)
  - QuotaResponse TypeScript interface
affects: [34-scan-history, frontend-scan-form, dashboard-quota-badge]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Option<clerk_user_id> routing in check_rate_limits: None=anonymous IP limit, Some=user monthly limit"
    - "429 body includes resets_at as RFC3339 string alongside standard ProblemDetails fields"
    - "Server action auth-first pattern: extract Clerk token before calling backend API"
    - "Quota badge color coding: ratio < 0.6 = green, 0.6-1.0 = yellow, >= 1.0 = red"

key-files:
  created: []
  modified:
    - src/api/errors.rs
    - src/db/scans.rs
    - src/rate_limit/middleware.rs
    - src/api/scans.rs
    - frontend/app/actions/scan.ts
    - frontend/app/dashboard/page.tsx
    - frontend/app/results/[token]/page.tsx
    - frontend/lib/types.ts

key-decisions:
  - "Scan server action IS the client-side domain check — runs synchronously on button click; no separate onClick handler needed; backend gate (33-01) provides security enforcement"
  - "Tier history card badge deferred to Phase 34 — scan history dashboard UI doesn't exist yet; badge will be added to history cards when Phase 34 creates that UI"
  - "getQuotaStyle inline function in dashboard server component — colocation with usage, no client-side state needed"

patterns-established:
  - "Auth-aware server action: extract Clerk token first, check domain, then forward Bearer header to backend"
  - "429 resets_at countdown: Math.floor diff/hours/minutes from now to resets_at"

requirements-completed: [TIER-03, TIER-04, TIER-05]

# Metrics
duration: 2min
completed: 2026-02-18
---

# Phase 33 Plan 02: Tiered Rate Limiting and Frontend Tier UX Summary

**Tier-aware rate limiter (1/IP/24h anonymous, 5/user/month Developer), 429 responses with resets_at countdown, results tier badges, and color-coded dashboard quota badge with Clerk token forwarding from scan action**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-18T23:09:00Z
- **Completed:** 2026-02-18T23:11:00Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Rewrote `check_rate_limits` with `Option<clerk_user_id>` routing: anonymous path counts only `clerk_user_id IS NULL` scans from same IP (prevents authenticated scans from inflating anonymous quota); authenticated path counts per-user monthly scans
- Added `RateLimitedWithReset` variant to `ApiError` — 429 responses include `resets_at` as RFC3339 in JSON body alongside standard ProblemDetails fields
- Updated `create_scan` server action to extract Clerk JWT via `auth()`, forward as `Authorization: Bearer` header, pre-check domain verification for authenticated users, and format 429 countdown ("Resets in 18h 23m")
- Added tier badge to results page header: "Basic scan" with upsell link for free-tier, "Enhanced scan" badge for authenticated
- Added server-side quota fetch in dashboard from `GET /api/v1/quota`; displays color-coded badge ("X/5 scans this month") with green/yellow/red thresholds at 60% and 100% usage

## Task Commits

Each task was committed atomically:

1. **Task 1: Rewrite rate limiting with tier-aware routing and extend error response with resets_at** - `b1cff3f` (feat)
2. **Task 2: Frontend auth-aware scan submission, tier badges, quota badge, and 429 countdown handling** - `d3ec532` (feat)

**Plan metadata:** (docs commit — see below)

## Files Created/Modified

- `src/api/errors.rs` - Added `RateLimitedWithReset` variant with `resets_at: chrono::DateTime<Utc>`; custom JSON serialization that includes `resets_at` in 429 body
- `src/db/scans.rs` - Added `count_anonymous_scans_by_ip_today` filtering `clerk_user_id IS NULL` to isolate anonymous quota
- `src/rate_limit/middleware.rs` - Rewrote `check_rate_limits(pool, Option<&str>, ip)` with anonymous/authenticated routing; added `next_midnight_utc` and `first_of_next_month_utc` helpers
- `src/api/scans.rs` - Inserted `rate_limit::check_rate_limits` call after domain gate, before `create_scan`
- `frontend/app/actions/scan.ts` - Added Clerk `auth()` import; JWT extraction and Bearer forwarding; domain verification pre-check; 429 resets_at countdown formatting
- `frontend/app/dashboard/page.tsx` - Added quota fetch from `/api/v1/quota`; `getQuotaStyle` color function; quota badge display in dashboard header
- `frontend/app/results/[token]/page.tsx` - Added tier badge ("Basic scan" / "Enhanced scan") after Scanned date in results header
- `frontend/lib/types.ts` - Added `QuotaResponse` interface (`used`, `limit`, `resets_at`)

## Decisions Made

- Scan server action IS the client-side domain check — the `submitScan` server action runs synchronously when the user clicks "Scan", providing immediate feedback without a separate `onClick` handler; the backend gate added in 33-01 enforces security
- Tier history card badge deferred to Phase 34 — the scan history dashboard UI does not yet exist; Phase 34 will add tier badges to history cards when it creates that UI
- `getQuotaStyle` is an inline function in the server component — colocation with the single usage site is clean; no client-side state needed

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

Pre-existing test failure in `scanners::js_secrets::tests::test_false_positive_detection` was present before this plan and is unrelated to our changes (we did not modify `js_secrets.rs`). Logged to deferred items in Phase 30.

## Next Phase Readiness

- Phase 33 is now complete: both backend tier enforcement and frontend tier UX are implemented
- Phase 34 (scan history) can add tier badges to history cards using the `tier` field now persisted on every scan record
- The `QuotaResponse` type is available for any future quota-related UI components

## Self-Check: PASSED

All files exist and committed. Key patterns verified:
- `RateLimitedWithReset` in errors.rs: confirmed
- `count_anonymous_scans_by_ip_today` in db/scans.rs: confirmed
- `check_rate_limits` in middleware.rs: confirmed
- `Bearer` token forwarding in scan.ts: confirmed
- Tier badge in results page: confirmed
- Quota badge in dashboard: confirmed
- Frontend build: PASSED (no TypeScript errors)
- Backend compile: PASSED (cargo check clean)

---
*Phase: 33-tiered-scan-access-and-rate-limiting*
*Completed: 2026-02-18*
