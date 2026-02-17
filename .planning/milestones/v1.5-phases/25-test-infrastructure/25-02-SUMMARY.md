---
phase: 25-test-infrastructure
plan: 02
subsystem: frontend-testing
tags: [msw, fixtures, integration-tests, component-tests]
dependency-graph:
  requires:
    - 25-01
  provides:
    - msw-mock-infrastructure
    - api-fixtures
    - vitest-global-mocks
    - proof-point-tests
  affects:
    - frontend/vitest.setup.ts
    - frontend/__tests__/helpers/msw/
    - frontend/__tests__/helpers/fixtures/
tech-stack:
  added:
    - "@testing-library/jest-dom": "^6.6.3"
  patterns:
    - MSW request handler pattern with error factories
    - Fixture-based API response mocking
    - Global next/navigation and next/image mocks
    - Test component pattern for integration tests
key-files:
  created:
    - frontend/__tests__/helpers/fixtures/scan.ts
    - frontend/__tests__/helpers/fixtures/results.ts
    - frontend/__tests__/helpers/fixtures/checkout.ts
    - frontend/__tests__/helpers/msw/handlers.ts
    - frontend/__tests__/helpers/msw/server.ts
    - frontend/__tests__/components/Header.test.tsx
    - frontend/__tests__/integration/scan-status.test.tsx
  modified:
    - frontend/vitest.setup.ts
    - frontend/package.json
decisions:
  - "MSW handlers use BASE_URL='http://localhost:3000' matching .env.test NEXT_PUBLIC_BACKEND_URL"
  - "Error handlers exported as factories for server.use() overrides in individual tests"
  - "Fixtures use 'as const' for type safety and immutability"
  - "next/image mock uses React.createElement to avoid JSX parsing issues in .ts setup file"
  - "@testing-library/jest-dom installed and imported in vitest.setup.ts for custom matchers"
  - "Explicit cleanup() added to afterEach for test isolation"
  - "Header test uses getAllByAltText to handle responsive design (desktop/mobile logos)"
  - "Integration test uses minimal test component instead of importing full page component"
metrics:
  duration: "212 seconds"
  completed: "2026-02-17"
---

# Phase 25 Plan 02: MSW Mock Infrastructure Summary

**One-liner:** MSW handlers and realistic fixtures for all 6 API endpoints with global next/navigation mocks, plus Header component test and scan status integration test proving the full stack works.

## What Was Built

Created comprehensive MSW mock infrastructure with realistic API fixtures, global Next.js mocks, and two proof-point tests demonstrating the test stack works end-to-end.

### Components Created

1. **Scan Fixtures (`frontend/__tests__/helpers/fixtures/scan.ts`)**
   - 5 fixtures: created, pending, inProgress, completed, failed
   - Realistic data matching ScanResponse and CreateScanResponse types
   - Includes all scan stages, findings, and summary counts
   - Completed fixture has 3 findings (critical, high, medium severity)

2. **Results Fixtures (`frontend/__tests__/helpers/fixtures/results.ts`)**
   - 2 fixtures: gradeA (free tier), paidTier
   - Distinct from scan fixtures for clear test scenarios
   - gradeA has 1 low severity finding
   - paidTier has 2 findings (critical + high) and no expiration

3. **Checkout Fixtures (`frontend/__tests__/helpers/fixtures/checkout.ts`)**
   - 2 fixtures: success, error
   - Success contains Stripe checkout URL
   - Error follows RFC 7807 problem details format

4. **MSW Handlers (`frontend/__tests__/helpers/msw/handlers.ts`)**
   - 6 happy path handlers (default behavior):
     - POST /api/v1/scans → returns created fixture (201)
     - GET /api/v1/scans/:id → returns inProgress fixture
     - GET /api/v1/results/:token → returns gradeA fixture
     - POST /api/v1/checkout → returns success fixture
     - POST /api/v1/webhooks/stripe → returns 200 OK
     - GET /api/v1/stats/scan-count → returns count
   - 7 error handler factories exported:
     - scanNotFound, scanServerError
     - resultsNotFound, resultsServerError
     - createScanRateLimited, createScanServerError
     - checkoutServerError

5. **MSW Server (`frontend/__tests__/helpers/msw/server.ts`)**
   - setupServer instance with all handlers
   - Exported for use in vitest.setup.ts and individual tests

6. **Vitest Setup (`frontend/vitest.setup.ts`)**
   - MSW server lifecycle (listen/resetHandlers/close)
   - @testing-library/jest-dom matchers imported
   - Explicit cleanup() between tests
   - Global next/navigation mock (useRouter, usePathname, useSearchParams)
   - Global next/image mock using React.createElement

7. **Header Component Test (`frontend/__tests__/components/Header.test.tsx`)**
   - 4 passing tests:
     - Renders logo image (handles multiple logos for responsive design)
     - Renders navigation landmark
     - Renders Scan Now CTA link with correct href
     - Renders logo as link to home
   - Proves component rendering, path alias resolution, provider wrapping work

8. **Scan Status Integration Test (`frontend/__tests__/integration/scan-status.test.tsx`)**
   - 4 passing tests:
     - Fetches and displays scan status from MSW handler
     - Fetches completed scan with server.use() override
     - Displays error when API returns 404 (using error handler)
     - Displays error when API returns 500 (using error handler)
   - Minimal test component that fetches from API and renders data
   - Proves MSW intercepts API calls and handler overrides work

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Missing @testing-library/jest-dom dependency**
- **Found during:** Task 2 - creating tests
- **Issue:** toBeInTheDocument matcher not available, causing test failures
- **Fix:** Installed @testing-library/jest-dom and imported in vitest.setup.ts
- **Files modified:** package.json, vitest.setup.ts
- **Commit:** 00786f7

**2. [Rule 3 - Blocking] JSX parsing error in vitest.setup.ts**
- **Found during:** Task 2 - running tests
- **Issue:** next/image mock used JSX syntax which cannot be parsed in .ts setup file
- **Fix:** Changed from JSX `<img ... />` to React.createElement('img', ...)
- **Files modified:** vitest.setup.ts
- **Commit:** 00786f7

**3. [Rule 3 - Blocking] Test isolation issue with DOM cleanup**
- **Found during:** Task 2 - running integration tests
- **Issue:** Multiple test components rendering simultaneously, causing "multiple elements found" errors
- **Fix:** Added explicit cleanup() import and call in afterEach hook
- **Files modified:** vitest.setup.ts
- **Commit:** 00786f7

**4. [Rule 1 - Bug] Header test expecting single logo**
- **Found during:** Task 2 - running Header test
- **Issue:** getByAltText expected single logo but Header has two (desktop + mobile responsive variants)
- **Fix:** Changed to getAllByAltText and verified length > 0
- **Files modified:** Header.test.tsx
- **Commit:** 00786f7

## Technical Details

### MSW Handler Pattern

The handlers follow a clear pattern:
- **Default handlers** return happy path fixtures (for most common test scenarios)
- **Error handlers** are exported as factories, allowing tests to override with `server.use(errorHandlers.scanNotFound)`
- **BASE_URL** matches .env.test NEXT_PUBLIC_BACKEND_URL for consistency

This pattern allows tests to be concise (default behavior works) while supporting error scenarios without duplication.

### Fixture Design

All fixtures use `as const` for type safety and immutability. Fixtures are realistic, matching production API response shapes from lib/types.ts. The completed scan fixture includes a mix of severity levels (critical, high, medium) to support comprehensive testing.

### Global Mocks Strategy

Two global mocks are essential for testing Next.js components:
1. **next/navigation**: Components using useRouter, usePathname, useSearchParams render without errors
2. **next/image**: Happy-dom doesn't support Next.js Image optimization, so we mock it as a plain img tag

These mocks are global because they're needed by virtually every component.

### Test Component Pattern

The scan-status integration test uses a minimal test component instead of importing the full page component. This approach:
- Avoids complex dependencies (useParams, routing context)
- Tests the specific behavior (fetch + render) in isolation
- Proves the MSW pipeline works without testing unrelated page logic

## Task Breakdown

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create MSW fixtures, handlers, server, and vitest.setup.ts | c1aa7ad | fixtures/scan.ts, fixtures/results.ts, fixtures/checkout.ts, msw/handlers.ts, msw/server.ts, vitest.setup.ts |
| 2 | Create Header test and scan status integration test | 00786f7 | Header.test.tsx, scan-status.test.tsx, vitest.setup.ts, package.json |

## Verification Results

All verification steps passed:

1. ✓ All 8 tests pass (4 Header + 4 integration)
2. ✓ npm test executes Vitest with coverage
3. ✓ Header test renders component with @/components/header import
4. ✓ Header test uses renderWithProviders from @/__tests__/helpers/test-utils
5. ✓ Integration test fetches data intercepted by MSW
6. ✓ Integration test uses server.use() for handler override
7. ✓ No useRouter/usePathname/useSearchParams errors in any test
8. ✓ 13 HTTP handlers (6 happy path + 7 error variants)

## Success Criteria Met

- [x] MSW handlers cover scan (GET/POST), results (GET), checkout (POST), webhook (POST), stats (GET)
- [x] Each endpoint has success handler as default and error handler variants exported
- [x] Fixtures have realistic data matching TypeScript types from lib/types.ts
- [x] vitest.setup.ts manages MSW lifecycle and mocks next/navigation + next/image
- [x] Header.test.tsx passes 4 tests proving component rendering + path alias + providers
- [x] scan-status.test.tsx passes 4 tests proving MSW intercepts API calls + overrides work
- [x] Running npm test executes all tests and shows 8 passing

## Impact

This plan establishes the full mock infrastructure needed for Phase 26 component tests:
- All API endpoints are mocked with realistic responses
- Error scenarios are easy to test with handler factories
- Global mocks eliminate Next.js-specific test setup in individual tests
- Proof-point tests confirm the stack works end-to-end

Phase 26 can now write component tests with confidence that:
- Components will render without Next.js errors
- API calls will be intercepted by MSW
- Both success and error paths can be tested

## Next Steps

Phase 26 will write comprehensive component tests for ScanForm, ResultsDashboard, and PaymentCheckoutButton using this MSW infrastructure.

## Self-Check

Verifying all artifacts exist:

**Files:**
- ✓ FOUND: frontend/__tests__/helpers/fixtures/scan.ts
- ✓ FOUND: frontend/__tests__/helpers/fixtures/results.ts
- ✓ FOUND: frontend/__tests__/helpers/fixtures/checkout.ts
- ✓ FOUND: frontend/__tests__/helpers/msw/handlers.ts
- ✓ FOUND: frontend/__tests__/helpers/msw/server.ts
- ✓ FOUND: frontend/vitest.setup.ts
- ✓ FOUND: frontend/__tests__/components/Header.test.tsx
- ✓ FOUND: frontend/__tests__/integration/scan-status.test.tsx

**Commits:**
- ✓ FOUND: c1aa7ad (Task 1: create MSW fixtures, handlers, server, and vitest.setup.ts)
- ✓ FOUND: 00786f7 (Task 2: create Header test and scan status integration test)

## Self-Check: PASSED
