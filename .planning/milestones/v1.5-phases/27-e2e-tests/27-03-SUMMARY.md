---
phase: 27-e2e-tests
plan: 03
subsystem: testing
tags: [playwright, e2e, next.js, error-handling, browser-validation, page-route, testProxy]

# Dependency graph
requires:
  - phase: 27-e2e-tests
    plan: 01
    provides: Playwright infrastructure, fixtures, route/fetch mock helpers
provides:
  - 6 error flow E2E tests covering all E2E-03 error scenarios with recovery verification
  - Full E2E suite (12 tests) passing: error-flows + free-scan + paid-audit
  - playwright.config.ts fixed: testMatch override, port 3001, reuseExistingServer: false
affects: [27-e2e-tests, 28-ci-integration]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "page.route() with stateful call counter: first call succeeds, subsequent calls abort — triggers errorCount warning"
    - "Use [class*='grade-X-bg'] selectors for grade badges to avoid strict mode violations"
    - "Capture appOrigin from page.url() after goto() for dynamic port-aware redirects"
    - "Browser validity.valid check (el.validity.valid) for HTML5 constraint validation assertions"
    - "next/experimental/testmode/playwright defineConfig requires explicit testMatch override to find e2e/ directory"

key-files:
  created:
    - frontend/e2e/error-flows.spec.ts
  modified:
    - frontend/playwright.config.ts
    - frontend/e2e/free-scan.spec.ts
    - frontend/e2e/paid-audit.spec.ts

key-decisions:
  - "Network timeout test: first fetch returns in-progress scan, subsequent requests abort — allows errorCount >= 3 while isScanning=true to trigger 'Having trouble connecting' warning"
  - "Port 3001 for E2E tests to avoid conflict with other services on port 3000; reuseExistingServer: false for deterministic runs"
  - "CFAA consent test uses browser validity.valid assertion instead of Zod error path — HTML required attribute on checkbox prevents form submission client-side"
  - "testMatch: '**/*.spec.ts' override needed because next/experimental/testmode/playwright defineConfig defaults to {app,pages}/**/*.spec.ts"
  - "Stripe redirect Location built from appOrigin captured after goto() — makes tests port-agnostic"

patterns-established:
  - "Stateful route mock for progressive failure: return success on first call, abort on subsequent — triggers error accumulation UI"
  - "Grade badge selectors: [class*='grade-X-bg'] not text='X' to avoid strict mode violations from ambiguous single-character text"

requirements-completed: [E2E-03]

# Metrics
duration: 13min
completed: 2026-02-17
---

# Phase 27 Plan 03: Error Flow E2E Tests Summary

**6 error flow E2E tests with recovery verification covering client-side validation, server rejection, 404 not-found, network timeout, and server 500 — full 12-test suite passing against production build on port 3001**

## Performance

- **Duration:** 13 min
- **Started:** 2026-02-17T04:58:21Z
- **Completed:** 2026-02-17T05:11:01Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Created `frontend/e2e/error-flows.spec.ts` with 6 error scenario tests: invalid URL (browser + Zod validation), server rejection (422 response), scan 404 (Scan Not Found UI), results 404 (Next.js notFound), network timeout (connection warning after 3 abort failures), server 500 (notFound via error boundary)
- Fixed `playwright.config.ts`: added `testMatch: '**/*.spec.ts'` override, switched to port 3001, set `reuseExistingServer: false` to avoid connecting to OpenWebUI on port 3000
- Fixed `free-scan.spec.ts` and `paid-audit.spec.ts` selector issues discovered during full suite run (strict mode violations on grade letters, Stripe redirect port, CFAA consent approach)
- Full E2E suite: 12 tests across 3 spec files all pass against production build

## Task Commits

Each task was committed atomically:

1. **Task 1: Write error flow E2E tests** - `5da41c0` (feat)
2. **Task 2: Run full E2E suite and fix any issues** - `29380f8` (fix)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `frontend/e2e/error-flows.spec.ts` - 6 error scenario tests with recovery verification; imports from fetch-mocks and scan fixtures
- `frontend/playwright.config.ts` - Added testMatch override, port 3001, reuseExistingServer: false, BACKEND_URL env var
- `frontend/e2e/free-scan.spec.ts` - Fixed grade selector (grade-b-bg class), severity count labels (1 High/1 Medium), CFAA consent approach
- `frontend/e2e/paid-audit.spec.ts` - Fixed grade selector (grade-a-bg class), dynamic appOrigin for Stripe redirect Location header

## Decisions Made
- Used stateful call counter in network timeout test: first request returns in-progress scan so `scan` state is populated, subsequent requests abort so `errorCount` builds up — this is required because the "Having trouble connecting" warning only shows when `errorCount >= 3 && isScanning` (scan.status must be set)
- Port 3001 for test server to avoid conflict with OpenWebUI service running on port 3000 in dev environment
- CFAA test asserts `validity.valid = false` on authorization checkbox (browser HTML5 constraint validation) rather than attempting to show Zod error — simpler and tests the actual user experience

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed playwright.config.ts testMatch default overriding testDir**
- **Found during:** Task 1 (running tests after creation)
- **Issue:** `defineConfig` from `next/experimental/testmode/playwright` sets `testMatch: "{app,pages}/**/*.spec.{t,j}s"` by default, ignoring `testDir: './e2e'` — no tests found
- **Fix:** Added explicit `testMatch: '**/*.spec.ts'` to override the default
- **Files modified:** frontend/playwright.config.ts
- **Verification:** `npx playwright test --list` shows all 12 tests
- **Committed in:** 5da41c0 (Task 1 commit)

**2. [Rule 3 - Blocking] Fixed port conflict: switched from port 3000 to port 3001**
- **Found during:** Task 1 (tests failing with timeout on input locator)
- **Issue:** OpenWebUI service occupies port 3000; `reuseExistingServer: true` caused Playwright to connect to OpenWebUI instead of starting the shipsecure Next.js app
- **Fix:** Changed webServer to use port 3001, set `reuseExistingServer: false`, added `BACKEND_URL` and `NEXT_PUBLIC_BACKEND_URL` env vars pointing to port 3001
- **Files modified:** frontend/playwright.config.ts
- **Verification:** Tests connect to the correct Next.js app and find form elements
- **Committed in:** 5da41c0 (Task 1 commit)

**3. [Rule 1 - Bug] Fixed free-scan.spec.ts selector strict mode violations**
- **Found during:** Task 2 (full suite run)
- **Issue:** `text=B` and `text=high/medium` selectors matched multiple elements (strict mode violation); Stripe redirect Location used port 3000
- **Fix:** Use `[class*="grade-b-bg"]` for grade badge, `text=1 High`/`text=1 Medium` for severity counts, `appOrigin` captured dynamically from `page.url()` for Stripe redirect
- **Files modified:** frontend/e2e/free-scan.spec.ts, frontend/e2e/paid-audit.spec.ts
- **Committed in:** 29380f8 (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (2 blocking, 1 bug)
**Impact on plan:** All auto-fixes required for tests to run and pass. No scope creep. The testMatch and port issues are infrastructure setup problems that arise from the development environment; the selector fixes are standard test maintenance.

## Issues Encountered
- `next/experimental/testmode/playwright`'s `defineConfig` overwrites `testMatch` with a pattern that only looks in `app/` and `pages/` directories — not documented prominently. Adding explicit `testMatch: '**/*.spec.ts'` resolves this.
- Port 3000 occupied by OpenWebUI in dev environment; all E2E tests now use port 3001.
- "Having trouble connecting" warning requires `scan != null && errorCount >= 3` — a pure network abort from the start never shows this because the loading state persists. Fixed by mocking first call to return in-progress scan.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Full E2E suite: 12 tests across 3 spec files pass against production build on port 3001
- Phase 28 (CI Integration) can reference this infrastructure — use `E2E_PORT=3001` (or override with `E2E_PORT` env var) and `npm run build && npm run test:e2e` to run the full suite

---
*Phase: 27-e2e-tests*
*Completed: 2026-02-17*

## Self-Check: PASSED

- `frontend/e2e/error-flows.spec.ts` exists: FOUND
- `frontend/playwright.config.ts` exists: FOUND
- `.planning/phases/27-e2e-tests/27-03-SUMMARY.md` exists: FOUND
- Task 1 commit `5da41c0` verified in git log: FOUND
- Task 2 commit `29380f8` verified in git log: FOUND
- Full E2E suite: 12/12 tests pass against production build
