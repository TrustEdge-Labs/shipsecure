# Project Research Summary: Frontend Testing Infrastructure

**Project:** ShipSecure v1.5
**Domain:** Frontend Testing (Unit, Component, E2E) for Next.js 16 + React 19 SaaS Application
**Researched:** 2026-02-16
**Confidence:** HIGH

## Executive Summary

ShipSecure is a SaaS security scanning platform with a Rust/Axum backend and a Next.js 16.1.6 frontend using React 19.2.3's App Router architecture. The application currently has zero test infrastructure across 9 client components, 5 pages, server actions, and a Stripe payment integration. The industry-standard approach for testing this type of application in 2026 is a layered strategy: Vitest with React Testing Library for fast unit and component tests, Playwright for E2E browser automation of critical user flows, and MSW (Mock Service Worker) for network-level API mocking. This is the stack officially recommended by Next.js documentation as of February 2026.

The recommended approach is to build testing infrastructure in a strict dependency order: Vitest configuration and mocking foundations first, then component tests for all 9 existing components, then Playwright E2E tests for the two critical user journeys (free scan flow and paid audit flow), and finally CI/CD integration with GitHub Actions. This ordering is driven by the fact that component tests require the mocking infrastructure to exist, E2E tests require a working application but not component tests, and CI integration requires both test suites to be working. The key architectural insight is that async Server Components cannot be unit tested with Vitest -- they must be covered by Playwright E2E tests. All 9 existing components are client components, so they are fully testable with Vitest + RTL.

The primary risks are: (1) attempting to unit test async Server Components, which will silently fail or hang; (2) incorrect server action mocking that causes forms to pass tests but break in production; (3) TypeScript path alias resolution failures that block the entire test suite from running; and (4) running Playwright against the dev server instead of a production build, which masks real bugs. All four risks are well-documented and have straightforward prevention strategies that should be implemented during initial setup, not retrofitted later.

## Key Findings

### Recommended Stack

The testing stack consists of 10 core dev dependencies and 2 optional packages, adding zero impact to production bundle size. All packages are verified compatible with the existing React 19.2.3 and Next.js 16.1.6 versions.

**Core technologies:**
- **Vitest 4.0.18**: Unit and component test runner -- native ESM support, built on Vite, Next.js officially recommended. 50-200ms per test file.
- **@testing-library/react 16.3.2**: Component testing utilities -- version 16+ required for React 19, handles `act()` changes automatically. Encourages testing user behavior over implementation details.
- **@playwright/test 1.58.2**: E2E browser automation -- supports Chromium, Firefox, WebKit. Official Next.js recommendation. Includes trace support and visual regression.
- **happy-dom 20.6.1**: DOM environment for Vitest -- 2-4x faster than jsdom with sufficient API coverage for RTL. Use jsdom only as fallback.
- **MSW (Mock Service Worker)**: Network-level API mocking -- works consistently across unit, integration, and E2E tests. Intercepts fetch calls with type-safe handlers.
- **@vitejs/plugin-react 5.1.4**: Vite React plugin -- required for JSX/TSX transforms in tests. Must use this (not SWC variant) for testing environment compatibility.
- **vite-tsconfig-paths 6.1.1**: TypeScript path resolution -- resolves Next.js `@/*` import aliases in test environment.

**Critical version requirement:** @testing-library/react must be 16+ for React 19 compatibility. Earlier versions expect React 18.

**What to avoid:** Jest (poor ESM support, slow), Enzyme (unmaintained, no React 19), Cypress (slower CI, more complex), @vitejs/plugin-react-swc (breaks test environments).

### Expected Features

**Must have (table stakes -- v1.5 launch):**
- Component unit tests for all 9 client components (ScanForm, ResultsDashboard, GradeSummary, FindingAccordion, ProgressChecklist, UpgradeCTA, Header, Footer, Logo)
- Form validation tests for ScanForm (URL, email, CFAA consent via Zod)
- API mocking with MSW for isolated tests
- Loading state and error boundary testing
- Critical path E2E tests: free scan flow (home -> submit -> scan/[id] -> results/[token]) and paid audit flow (UpgradeCTA -> Stripe -> payment/success)
- Stripe Test Mode configuration with test card numbers
- Dark mode rendering verification
- Server action testing for submitScan
- Basic CI pipeline in GitHub Actions running both Vitest and Playwright on PRs

**Should have (add after core testing works -- v1.5.x):**
- Coverage reporting with 80% threshold enforcement
- Visual regression testing via Playwright snapshots
- Accessibility testing with jest-axe (catches 30-40% of a11y issues automatically)
- Responsive layout tests at mobile/tablet/desktop breakpoints
- Multi-browser testing (Firefox, Safari alongside Chrome)

**Defer (v2+):**
- Stripe Test Clocks for subscription lifecycle testing
- Storybook for component isolation/documentation
- Mutation testing, contract testing, load testing

### Architecture Approach

The architecture follows a three-layer testing pyramid: many fast unit/component tests (Vitest + RTL in happy-dom), a smaller set of integration tests (Vitest + MSW), and a focused set of E2E tests for critical user flows (Playwright against production build). Tests are colocated with source code in `__tests__/` directories for components and server actions, while E2E tests live in a separate `e2e/` directory since they span multiple components. Shared test utilities (MSW handlers, custom render wrapper, mock factories) live in `test-utils/` to prevent duplication.

**Major components:**
1. **vitest.config.ts** -- Test runner configuration with happy-dom, React plugin, path aliases, coverage settings
2. **test-utils/mocks/** -- MSW server and handlers for API mocking, reset between tests
3. **test-utils/custom-render.tsx** -- RTL render wrapper with providers (theme, state)
4. **playwright.config.ts** -- E2E configuration with production build webServer, multi-browser projects, trace on retry
5. **e2e/** -- Playwright specs for scan flow, results flow, payment flow
6. **.github/workflows/test.yml** -- Parallel CI jobs for unit+component tests and E2E tests

**Key patterns:**
- Server actions tested in Node environment (`@vitest-environment node` directive)
- Client components tested in happy-dom environment
- Async Server Components tested only via Playwright E2E
- MSW handlers reset via `server.resetHandlers()` in `afterEach` to prevent test coupling
- Playwright runs against `npm run build && npm run start` (production build), never dev server

### Critical Pitfalls

1. **Async Server Component unit testing** -- Vitest cannot render async Server Components. Tests will hang or fail with cryptic errors. Prevention: classify all components as sync/async upfront, route async components to E2E tests only.

2. **Server action mocking failures** -- Server actions use `'use server'` directive and cannot run in jsdom. Mocking must happen at the module level with `vi.mock()` and return production-matching type shapes (`ScanFormState`). Incorrect mocks cause tests to pass while production forms break.

3. **TypeScript path alias resolution** -- Vite does not read `tsconfig.json` paths automatically. Without `vite-tsconfig-paths` plugin, every `@/` import fails in tests despite TypeScript compiling fine. This blocks the entire test suite.

4. **next/navigation mock configuration** -- App Router uses `next/navigation` (not `next/router`). All hooks (`useRouter`, `usePathname`, `useSearchParams`) must be mocked before component imports. Missing any one causes "invariant expected app router" errors.

5. **Playwright running against dev server** -- Testing against `next dev` masks production-only bugs, is slower due to hot reload overhead, and produces flaky results. The webServer config must use `npm run build && npm run start`.

6. **Environment variables not loading** -- Vitest does not auto-load `.env` files like Next.js does. Must call `loadEnvConfig(process.cwd())` from `@next/env` in vitest.config.ts or all `process.env` values will be undefined.

## Implications for Roadmap

Based on research, the milestone naturally decomposes into 5 phases with a strict dependency chain. The first phase is the most critical -- if the Vitest foundation is misconfigured, every subsequent phase fails.

### Phase 1: Vitest Foundation and Test Infrastructure

**Rationale:** Every other phase depends on a working Vitest configuration. The six most critical pitfalls (path aliases, env vars, navigation mocks, server action mocks, happy-dom setup, React plugin) all manifest during initial setup. Getting this right prevents cascading failures.
**Delivers:** Working vitest.config.ts, test setup files, MSW mock infrastructure, custom render wrapper, first passing component test (Logo or Footer as proof-of-concept).
**Addresses:** Vitest + RTL setup, MSW setup, path alias resolution, environment variable loading.
**Avoids:** Pitfalls 3 (path aliases), 4 (navigation mocks), 5 (env vars), 7 (client/server awareness).
**Stack:** vitest, @testing-library/react, @testing-library/jest-dom, @testing-library/user-event, happy-dom, @vitejs/plugin-react, vite-tsconfig-paths, MSW.

### Phase 2: Component Unit Tests

**Rationale:** With infrastructure in place, write tests for all 9 components. ScanForm is the most complex (Zod validation, server action integration, multiple form states) and should be tested first. Simpler display components (Logo, Footer, Header) provide quick wins and validate the setup.
**Delivers:** Unit tests for all 9 client components covering rendering, user interactions, validation, loading states, error states, dark mode.
**Addresses:** Component unit tests, form validation tests, loading state tests, error boundary tests, dark mode tests, server action tests.
**Avoids:** Pitfalls 1 (async Server Components -- all 9 are client components, but document the boundary), 2 (server action mocking -- use established patterns from Phase 1), 6 (React 19 Suspense -- use `findBy*` queries for async content).

### Phase 3: Playwright E2E Setup and Critical Path Tests

**Rationale:** E2E tests are independent of unit tests but require a working application. They cover what unit tests cannot: async Server Components, full page routing, actual Stripe redirects, and cross-component user flows. The two critical paths (scan flow, payment flow) represent the core business value.
**Delivers:** Working playwright.config.ts, E2E tests for free scan flow and paid audit flow, Stripe Test Mode configuration, error flow coverage.
**Addresses:** Playwright E2E setup, critical path E2E tests, Stripe test mode, responsive layout tests (if included).
**Avoids:** Pitfalls 8 (dev server testing -- configure production build from day one), 9 (external API mocking -- mock backend responses via route interception), 10 (auth state reuse -- implement storageState if auth exists).

### Phase 4: CI/CD Integration

**Rationale:** Once both test suites work locally, integrate into GitHub Actions. Run unit/component tests and E2E tests as parallel jobs for speed. This is the gatekeeper that prevents regressions from reaching production.
**Delivers:** GitHub Actions test workflow, parallel job execution (Vitest + Playwright), dependency caching, Playwright browser caching, artifact upload on failure, merge blocking on test failure.
**Addresses:** CI/CD pipeline, coverage reporting, merge gating.
**Avoids:** Performance traps (cache node_modules, cache Playwright browsers, run E2E with limited workers in CI).

### Phase 5: Quality Gates and Coverage Enforcement

**Rationale:** After the test suite is established and producing results, add coverage thresholds and quality gates. Setting thresholds too early creates friction; setting them after the initial suite is written creates a meaningful baseline.
**Delivers:** Coverage thresholds (80% lines/functions, 75% branches), HTML coverage reports, coverage badge, documented testing patterns and PR guidelines.
**Addresses:** Coverage reporting, coverage enforcement, testing documentation.
**Avoids:** Pitfall of no coverage enforcement leading to gradual degradation.

### Phase Ordering Rationale

- **Phases 1-2 must be sequential:** Component tests depend on Vitest configuration, MSW setup, and mock patterns established in Phase 1.
- **Phase 3 is partially parallel with Phase 2:** Playwright setup is independent of Vitest. However, it logically follows because the team should understand the testing patterns before writing E2E tests.
- **Phase 4 depends on Phases 2 and 3:** CI integration requires both test suites to exist and pass locally.
- **Phase 5 comes last:** Coverage thresholds are meaningless without tests to measure. Setting the baseline after the suite is written prevents arbitrary targets.
- **The grouping reflects the dependency graph:** `vitest.config.ts -> test-utils/ -> component tests -> playwright.config.ts -> e2e tests -> CI workflow -> coverage thresholds`.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2 (Component Tests):** ScanForm testing is complex due to Zod validation + useActionState + server action mocking. The mock pattern for `useActionState` is not well-documented and may require experimentation. Research the exact mock structure needed.
- **Phase 3 (Playwright E2E):** Stripe Checkout cannot be automated (security measures prevent Playwright from filling the hosted checkout form). Research the exact boundary: test up to the redirect to Stripe, then test the return to payment/success page. May need Playwright route interception to simulate Stripe return.

Phases with standard patterns (skip research-phase):
- **Phase 1 (Vitest Foundation):** Extremely well-documented in Next.js official docs (updated Feb 11, 2026). Follow the official setup guide directly.
- **Phase 4 (CI/CD Integration):** Standard GitHub Actions patterns. Playwright provides official CI documentation. Use the parallel job pattern from ARCHITECTURE.md directly.
- **Phase 5 (Quality Gates):** Standard Vitest coverage configuration. No research needed.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All recommendations from official Next.js docs (Feb 2026), verified npm package versions, confirmed React 19 compatibility |
| Features | HIGH | Feature prioritization based on official testing guides, Stripe docs, and established SaaS testing patterns |
| Architecture | HIGH | Layered testing pyramid is industry standard; project structure follows Next.js conventions; CI patterns from Playwright official docs |
| Pitfalls | HIGH | Pitfalls sourced from official docs, verified GitHub discussions, and React Testing Library issue tracker; all have documented solutions |

**Overall confidence:** HIGH

All four research files drew from official Next.js 16.1.6 documentation (updated February 11, 2026), official React 19 documentation, verified npm package registries, and established community patterns with multiple corroborating sources. The testing stack (Vitest + RTL + Playwright) is the canonical recommendation from the Next.js team, not a community preference.

### Gaps to Address

- **Stripe Checkout automation boundary:** Cannot automate the actual Stripe Checkout hosted page. Need to determine during Phase 3 planning exactly how to test the payment flow -- likely test redirect URL generation and success page rendering, skipping the Stripe-hosted portion.
- **useActionState mock pattern:** The exact mock structure for React 19's `useActionState` hook in Vitest is not comprehensively documented. Phase 2 may require experimentation to get ScanForm tests working correctly with form state management.
- **Backend test environment for E2E:** Research assumes E2E tests will mock the backend API via Playwright route interception. If a real backend test environment is preferred, additional configuration (test database, seed data, backend startup in CI) would be needed. Recommend mocking for v1.5, real backend integration for v2.
- **MSW v2 with Next.js App Router:** MSW integration with Next.js App Router server-side is still evolving. For client components this is straightforward, but if future Server Components need API mocking, the MSW setup may need adjustment.

## Sources

### Primary (HIGH confidence)
- [Next.js Testing: Vitest](https://nextjs.org/docs/app/guides/testing/vitest) -- Official setup guide, updated 2026-02-11
- [Next.js Testing: Playwright](https://nextjs.org/docs/pages/guides/testing/playwright) -- Official E2E guide, updated 2026-02-11
- [React 19 Upgrade Guide](https://react.dev/blog/2024/04/25/react-19-upgrade-guide) -- Testing migration (act, testing-library)
- [Vitest 4.0 Release](https://vitest.dev/blog/vitest-4) -- React 19 support, Browser Mode stable
- [Playwright Release Notes v1.58](https://playwright.dev/docs/release-notes) -- Trace support, CI patterns
- [Stripe Automated Testing](https://docs.stripe.com/automated-testing) -- Test Mode, test cards, Test Clocks
- [Testing Library FAQ](https://testing-library.com/docs/react-testing-library/faq/) -- React 19 patterns

### Secondary (MEDIUM confidence)
- [Test Strategy in the Next.js App Router Era](https://shinagawa-web.com/en/blogs/nextjs-app-router-testing-setup) -- Layered testing approach
- [Strapi: NextJs Unit Testing and E2E Testing](https://strapi.io/blog/nextjs-testing-guide-unit-and-e2e-tests-with-vitest-and-playwright) -- Vitest + Playwright integration
- [App Router pitfalls: common Next.js mistakes](https://imidef.com/en/2026-02-11-app-router-pitfalls) -- Navigation mocking, Server Component issues
- [MSW with Next.js](https://www.gimbap.dev/blog/setting-msw-in-next) -- Mock Service Worker setup patterns
- [Testing in 2026: Full Stack Testing Strategies](https://www.nucamp.co/blog/testing-in-2026-jest-react-testing-library-and-full-stack-testing-strategies) -- Industry trends

### Tertiary (needs validation during implementation)
- [React Testing Library Suspense behavior in React 19 -- Issue #1375](https://github.com/testing-library/react-testing-library/issues/1375) -- Suspense rendering changes may be resolved by RTL update
- [How to unit test server actions? -- vercel/next.js #69036](https://github.com/vercel/next.js/discussions/69036) -- Community patterns, not officially documented

---
*Research completed: 2026-02-16*
*Ready for roadmap: yes*
