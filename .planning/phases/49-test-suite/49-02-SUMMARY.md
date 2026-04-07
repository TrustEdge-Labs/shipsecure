---
phase: 49-test-suite
plan: "02"
subsystem: frontend-e2e
tags: [playwright, e2e, supply-chain, testing]
dependency_graph:
  requires:
    - 49-01 (supply chain components must exist)
    - 48-01 (supply chain pages and actions)
  provides:
    - TEST-04 (2 E2E tests for supply chain flow)
  affects:
    - frontend/e2e/supply-chain.spec.ts
    - frontend/e2e/fixtures/supply-chain.ts
    - frontend/e2e/helpers/fetch-mocks.ts
tech_stack:
  added: []
  patterns:
    - "page.route() for client-side fetch interception (no 'use server' action)"
    - "next.onFetch() for Server Component fetch interception"
    - ".first() on locators to avoid strict mode violations with duplicate text"
key_files:
  created:
    - frontend/e2e/supply-chain.spec.ts
    - frontend/e2e/fixtures/supply-chain.ts
  modified:
    - frontend/e2e/helpers/fetch-mocks.ts
decisions:
  - "Use page.route() not next.onFetch() for supply chain scan submission: submitSupplyChainScan has no 'use server' directive so runs client-side"
  - "Use .first() on 'Vulnerable' locator: appears in both summary card label and findings badge causing strict mode violation"
metrics:
  duration: ~25 min
  completed: "2026-04-07"
  tasks_completed: 2
  files_modified: 3
requirements:
  - TEST-04
---

# Phase 49 Plan 02: Supply Chain E2E Tests Summary

2 Playwright E2E tests covering supply chain paste-to-results happy path and empty-paste validation error, using page.route() for client-side mock and next.onFetch() for server-side results mock.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create supply chain E2E fixtures and fetch mock | e633025 | frontend/e2e/fixtures/supply-chain.ts, frontend/e2e/helpers/fetch-mocks.ts |
| 2 | Create 2 Playwright E2E tests for supply chain flow | 0675c80 | frontend/e2e/supply-chain.spec.ts |

## What Was Built

### Task 1: Fixtures and Mock Helpers

`frontend/e2e/fixtures/supply-chain.ts` exports `supplyChainFixtures` with:
- `scanResponse`: POST `/api/v1/scans/supply-chain` response with 1 vulnerable lodash finding (GHSA-jf85-cpcp-j695)
- `resultsPageResponse`: GET `/api/v1/results/{token}` response matching `SupplyChainResultsPageData` shape

`frontend/e2e/helpers/fetch-mocks.ts` gained two new exports:
- `mockSupplyChainScan()`: `next.onFetch()` interceptor for POST `/api/v1/scans/supply-chain`
- `mockSupplyChainResults()`: `next.onFetch()` interceptor for GET `/api/v1/results/`

### Task 2: E2E Spec

`frontend/e2e/supply-chain.spec.ts` with 2 tests:

1. **Happy path** — navigates to `/supply-chain`, clicks Paste Content tab, fills textarea with minimal lockfile JSON, submits, waits for `**/supply-chain/results/**` URL, verifies Vulnerable count card, No Known Issues card, lodash finding, and GHSA advisory ID are all visible.

2. **Error state** — navigates to `/supply-chain`, clicks Paste Content tab, submits without content, verifies "Please paste your package-lock.json content" validation message.

Both chromium tests pass. Firefox/webkit are not installed locally (matching CI behavior — CI only installs chromium via `npx playwright install --with-deps chromium`).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Used page.route() instead of next.onFetch() for scan submission**
- **Found during:** Task 2 execution (first chromium test failed)
- **Issue:** Plan specified `mockSupplyChainScan` (next.onFetch) for the POST, but `supply-chain-scan.ts` has no `'use server'` directive — it runs client-side from `supply-chain-form.tsx` (a `'use client'` component). `next.onFetch()` only intercepts server-side fetches.
- **Fix:** Used inline `page.route('**/api/v1/scans/supply-chain', ...)` in the test instead of `mockSupplyChainScan`. The `mockSupplyChainScan` helper remains in fetch-mocks.ts (useful if the action is ever promoted to server-side).
- **Files modified:** frontend/e2e/supply-chain.spec.ts
- **Commit:** 0675c80

**2. [Rule 1 - Bug] Added .first() to locators with multiple matches**
- **Found during:** Task 2 execution (strict mode violation on `text=Vulnerable`)
- **Issue:** `page.locator('text=Vulnerable')` matched 2 elements — the summary card label div AND the findings badge span. Playwright's `toBeVisible()` uses strict mode and rejects multi-element locators.
- **Fix:** Added `.first()` to `text=Vulnerable`, `text=No Known Issues`, `text=lodash`, and `text=GHSA-jf85-cpcp-j695` locators.
- **Files modified:** frontend/e2e/supply-chain.spec.ts
- **Commit:** 0675c80

## Known Stubs

None. All fixtures contain real-shape data and the tests verify actual rendered content.

## Threat Flags

None. Test-only code — no production surface changes.

## Self-Check: PASSED

Files exist:
- frontend/e2e/supply-chain.spec.ts: FOUND
- frontend/e2e/fixtures/supply-chain.ts: FOUND
- frontend/e2e/helpers/fetch-mocks.ts: FOUND (mockSupplyChainScan + mockSupplyChainResults added)

Commits exist:
- e633025: FOUND (feat(49-02): add supply chain E2E fixtures and fetch mock helpers)
- 0675c80: FOUND (feat(49-02): add 2 Playwright E2E tests for supply chain flow)

Tests: 2 chromium tests pass (happy path + error state).
