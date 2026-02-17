---
phase: 27-e2e-tests
plan: 01
subsystem: testing
tags: [playwright, e2e, next.js, testProxy, chromium, fixtures]

# Dependency graph
requires:
  - phase: 25-test-infrastructure
    provides: test scripts and test infrastructure foundation
  - phase: 26-component-tests
    provides: MSW fixtures pattern (E2E fixtures kept separate)
provides:
  - Playwright 1.58.2 installed with Chromium browser
  - playwright.config.ts configured for production build testing with Next.js testProxy
  - E2E fixture files matching real API response shapes (scan, results, checkout)
  - Route interception helpers for both client-side (page.route) and server-side (next.onFetch)
affects: [27-e2e-tests, 28-ci-integration]

# Tech tracking
tech-stack:
  added: ["@playwright/test@1.58.2", "chromium browser via playwright install"]
  patterns:
    - "E2E fixtures separate from MSW unit test fixtures"
    - "next/experimental/testmode/playwright defineConfig for production build testing"
    - "next.onFetch() for server-side fetch interception, page.route() for client-side"
    - "PLAYWRIGHT_TEST env var activates testProxy in next.config.ts"

key-files:
  created:
    - frontend/playwright.config.ts
    - frontend/e2e/fixtures/scan.ts
    - frontend/e2e/fixtures/results.ts
    - frontend/e2e/fixtures/checkout.ts
    - frontend/e2e/helpers/route-mocks.ts
    - frontend/e2e/helpers/fetch-mocks.ts
  modified:
    - frontend/next.config.ts
    - frontend/package.json

key-decisions:
  - "Import defineConfig from next/experimental/testmode/playwright (not @playwright/test) to get Next.js test fixture types"
  - "testProxy conditioned on PLAYWRIGHT_TEST=1 so production/dev builds are unaffected"
  - "webServer uses npm run start (production build) — dev server not used for E2E"
  - "Single worker (workers: 1) to avoid port conflicts between tests"
  - "E2E fixtures separate from MSW fixtures — different test layer, different shape requirements"
  - "200ms delays on mocked responses to simulate real timing and catch race conditions"

patterns-established:
  - "Route mocks pattern: page.route() for browser fetch, next.onFetch() for Server Action/Server Component fetch"
  - "Stateful polling mock: closure counter returns inProgress N times then completed"
  - "Fixture files export typed objects with 'as const' for immutability"

requirements-completed: [E2E-04]

# Metrics
duration: 3min
completed: 2026-02-17
---

# Phase 27 Plan 01: E2E Test Infrastructure Summary

**Playwright 1.58.2 installed with Next.js testProxy integration, production-build webServer config, and typed E2E fixtures with route interception helpers for scan/results/checkout flows**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-17T04:52:14Z
- **Completed:** 2026-02-17T04:55:18Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- Installed @playwright/test 1.58.2 and Chromium browser, configured via next/experimental/testmode/playwright for Next.js testProxy integration
- Created playwright.config.ts with production webServer (npm run start), single worker, PLAYWRIGHT_TEST env var activation, and Chromium project
- Created typed E2E fixtures for scan (created/inProgress/completed/failed), results (freeGradeB/paidGradeA), and checkout (success/error) matching real API shapes
- Created reusable route interception helpers: page.route() for client-side mocking and next.onFetch() for server-side Server Action mocking

## Task Commits

Each task was committed atomically:

1. **Task 1: Install Playwright, configure for production build with testProxy** - `b949eef` (chore)
2. **Task 2: Create E2E fixtures and route interception helpers** - `29380ba` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `frontend/playwright.config.ts` - Playwright config importing from next/experimental/testmode/playwright with webServer, single worker, PLAYWRIGHT_TEST env
- `frontend/next.config.ts` - Added experimental.testProxy conditioned on PLAYWRIGHT_TEST=1
- `frontend/package.json` - Updated test:e2e to run playwright test, added test:e2e:ui script
- `frontend/e2e/fixtures/scan.ts` - Scan API response fixtures (created, inProgress, completed, failed)
- `frontend/e2e/fixtures/results.ts` - Results API response fixtures (freeGradeB with 3 findings, paidGradeA with 1 finding)
- `frontend/e2e/fixtures/checkout.ts` - Checkout API response fixtures (success URL, error response)
- `frontend/e2e/helpers/route-mocks.ts` - page.route() interceptors: mockScanPolling (stateful counter), mockCheckout, mockNetworkFailure
- `frontend/e2e/helpers/fetch-mocks.ts` - next.onFetch() interceptors: mockScanSubmission, mockResultsPage, mockScanCount, mockResultsNotFound, mockServerError

## Decisions Made
- Imported `defineConfig` from `next/experimental/testmode/playwright` (not `@playwright/test`) to get the extended `test` fixture with the `next` object for server-side fetch interception
- Conditioned `testProxy` on `PLAYWRIGHT_TEST === '1'` so the experimental flag does not affect production or development builds
- Used `workers: 1` to avoid port conflicts between parallel test runs against the same `npm run start` process
- Kept E2E fixtures entirely separate from MSW fixtures used in unit/component tests — different test layer with different contract verification needs

## Deviations from Plan

None - plan executed exactly as written.

Note: `--with-deps` flag on `npx playwright install` required sudo access and failed. Used `npx playwright install chromium` (without `--with-deps`) which downloaded the browser successfully. System dependencies were already available on the host.

## Issues Encountered
- `npx playwright install --with-deps chromium` failed due to sudo requirement in the execution environment. Used `npx playwright install chromium` instead — browser installed successfully without system dependency installation (deps already present).

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Playwright infrastructure ready for test spec authoring (Plans 02-04)
- Fixtures and helpers provide building blocks for scan flow, results, and error scenario tests
- Production build must be run (`npm run build`) before `npm run test:e2e` — CI will handle this in Phase 28

---
*Phase: 27-e2e-tests*
*Completed: 2026-02-17*

## Self-Check: PASSED

All files exist, both commits verified (b949eef, 29380ba), TypeScript type checking passes with no errors.
