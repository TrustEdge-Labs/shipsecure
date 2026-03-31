---
phase: 42-funnel-unlock
plan: 02
subsystem: ui
tags: [next.js, react, e2e, playwright, scan-form, anonymous-scan]

# Dependency graph
requires: []
provides:
  - Editable URL input for anonymous users (no Juice Shop lockdown)
  - Removed DOMAIN_VERIFICATION_REQUIRED error handler from scan form
  - Removed demo messaging from scan form
  - Updated anonymous quota text to 3/day with scan history upsell
  - E2E tests validating open anonymous scan flow
affects: [funnel-unlock, content-routes, analytics-events]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - ScanForm URL input is always enabled for all users — no conditional disabled/hidden input logic
    - Error display simplified to two branches: RATE_LIMITED prefix handler + default

key-files:
  created: []
  modified:
    - frontend/components/scan-form.tsx
    - frontend/e2e/error-flows.spec.ts
    - frontend/e2e/free-scan.spec.ts

key-decisions:
  - "Remove DOMAIN_VERIFICATION_REQUIRED branch — domain verification dropped per D-01, makes this dead code"
  - "Keep isAuthenticated prop — still controls rate limit upsell link and quota text"

patterns-established:
  - "All E2E tests that submit the scan form must explicitly fill input#url (no hidden input assumed)"

requirements-completed: [FUNNEL-01]

# Metrics
duration: 3min
completed: 2026-03-30
---

# Phase 42 Plan 02: Funnel Unlock — Open Scan Form Summary

**Removed Juice Shop lockdown from scan form: anonymous users now get a fully editable URL input with updated 3/day quota messaging and no demo restrictions**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-03-30T22:25:31Z
- **Completed:** 2026-03-30T22:27:38Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Deleted `DEMO_TARGET_URL` constant, hidden input, and demo-only messaging from `ScanForm`
- Replaced conditional disabled/locked URL input with a single always-enabled `<input id="url" name="url">` for all users
- Removed `DOMAIN_VERIFICATION_REQUIRED` error handler (dead code after domain verification was dropped)
- Updated anonymous quota text from "1 free scan per app per day" to "3 free scans per day. Sign in for 5 scans/month and scan history."
- Replaced E2E test "anonymous user URL is locked to demo target" with "anonymous user can enter custom URL" — asserts input enabled, editable, no hidden input
- Updated free-scan.spec.ts happy path and CFAA consent tests to explicitly fill `input#url`

## Task Commits

1. **Task 1: Unlock scan form for anonymous users** - `89fd5a4` (feat)
2. **Task 2: Update E2E tests for open anonymous scan flow** - `a974d5a` (test)

## Files Created/Modified

- `frontend/components/scan-form.tsx` - Removed Juice Shop lockdown, demo messaging, DOMAIN_VERIFICATION_REQUIRED handler; unified URL input; updated quota copy
- `frontend/e2e/error-flows.spec.ts` - Replaced locked-URL test with editable-URL test; added URL fill to server rejection test
- `frontend/e2e/free-scan.spec.ts` - Explicitly fill URL field in both free scan tests

## Decisions Made

- `isAuthenticated` prop retained — it still controls rate limit upsell link visibility and the quota text (authenticated vs anonymous copy differs)
- `DOMAIN_VERIFICATION_REQUIRED` error branch removed entirely — domain verification is dropped from the product, not deferred

## Deviations from Plan

None - plan executed exactly as written. The "server rejection" test in error-flows.spec.ts also needed a URL fill step added (since it was submitting the form) — this was a minor extension of Task 2 that the plan implied but didn't spell out explicitly.

## Issues Encountered

None.

## Known Stubs

None — all changes are live wired to the scan form submission.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Anonymous scan form is now fully open — any visitor can paste any URL
- E2E tests updated to reflect new flow
- Ready for Phase 42-03+ (per-target rate limiting, share button, etc.)

---
*Phase: 42-funnel-unlock*
*Completed: 2026-03-30*
