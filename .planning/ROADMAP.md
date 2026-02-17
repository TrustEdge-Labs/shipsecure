# Roadmap: ShipSecure

## Milestones

- ✅ **v1.0 MVP** — Phases 01-04 (shipped 2026-02-06)
- ✅ **v1.1 DigitalOcean Deployment** — Phases 05-07 (shipped 2026-02-08)
- ✅ **v1.2 Launch Readiness** — Phases 08-12 (shipped 2026-02-10)
- ✅ **v1.3 Brand Identity** — Phases 13-18 (shipped 2026-02-11)
- ✅ **v1.4 Observability** — Phases 19-24 (shipped 2026-02-16)
- 🚧 **v1.5 Frontend Testing** — Phases 25-28 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 01-04) — SHIPPED 2026-02-06</summary>

- [x] Phase 01: Foundation (5/5 plans) — Rust/Axum backend, Next.js frontend, PostgreSQL schema
- [x] Phase 02: Free Tier MVP (8/8 plans) — 5 scanners, email delivery, results dashboard
- [x] Phase 03: Vibe-Code Intelligence (5/5 plans) — Framework detection, Nuclei templates, remediation guidance
- [x] Phase 04: Monetization (5/5 plans) — Stripe checkout, PDF reports, paid audit flow

See: `.planning/milestones/v1.0-ROADMAP.md`

</details>

<details>
<summary>✅ v1.1 DigitalOcean Deployment (Phases 05-07) — SHIPPED 2026-02-08</summary>

- [x] Phase 05: Codebase Preparation (4/4 plans) — Native subprocesses, config externalization, Docker builds
- [x] Phase 06: Deployment Infrastructure (4/4 plans) — Ansible provisioning, Nginx + SSL, systemd, UFW
- [x] Phase 07: Production Validation (2/2 plans) — Scanner validation, email delivery, Stripe flow, resilience

See: `.planning/milestones/v1.1-ROADMAP.md`

</details>

<details>
<summary>✅ v1.2 Launch Readiness (Phases 08-12) — SHIPPED 2026-02-10</summary>

- [x] Phase 08: Analytics & Tracking (1/1 plan) — Plausible analytics, conversion events
- [x] Phase 09: SEO & Discoverability (2/2 plans) — Meta tags, OG image, JSON-LD, sitemap, robots.txt
- [x] Phase 10: Legal Compliance (2/2 plans) — Privacy Policy, TOS, CFAA consent checkbox
- [x] Phase 11: Mobile & UX Polish (3/3 plans) — Mobile responsive, loading states, error boundaries, Lighthouse
- [x] Phase 12: Landing Page Optimization (2/2 plans) — Developer-focused copy, methodology transparency, OSS attribution

See: `.planning/milestones/v1.2-ROADMAP.md`

</details>

<details>
<summary>✅ v1.3 Brand Identity (Phases 13-18) — SHIPPED 2026-02-11</summary>

- [x] Phase 13: Design Token System (3/3 plans) — OKLch primitives, semantic tokens, dark mode, WCAG AA
- [x] Phase 14: Logo Component (2/2 plans) — Shield logo, responsive variants, professional PNG
- [x] Phase 15: Layout Refactor (1/1 plan) — Header-height token, layout preparation
- [x] Phase 16: Header & Navigation (1/1 plan) — Sticky header, responsive logo, CTA, keyboard nav
- [x] Phase 17: Icon System & Migration (1/1 plan) — Lucide React SVG icons replacing emoji
- [x] Phase 18: Favicon & OG Image (2/2 plans) — Branded favicon (ICO+SVG), Apple touch icon, OG image

See: `.planning/milestones/v1.3-ROADMAP.md`

</details>

<details>
<summary>✅ v1.4 Observability (Phases 19-24) — SHIPPED 2026-02-16</summary>

- [x] Phase 19: Structured JSON Logging (2/2 plans) — Environment-driven JSON/text logging, scan lifecycle instrumentation
- [x] Phase 20: Request Tracing (2/2 plans) — UUID correlation IDs, TraceLayer middleware, end-to-end propagation
- [x] Phase 21: Health Checks (1/1 plan) — Liveness + readiness endpoints, DB validation, scan capacity
- [x] Phase 22: Prometheus Metrics (2/2 plans) — HTTP counters, scan histograms, queue depth, rate limit tracking
- [x] Phase 23: Graceful Shutdown (2/2 plans) — TaskTracker, SIGTERM handling, scan draining, 503 rejection
- [x] Phase 24: Infrastructure Integration (2/2 plans) — Nginx /metrics security, Docker grace periods, systemd timeout, DO metrics agent

See: `.planning/milestones/v1.4-ROADMAP.md`

</details>

### 🚧 v1.5 Frontend Testing (In Progress)

**Milestone Goal:** Add comprehensive frontend testing with Vitest + React Testing Library for component tests and Playwright for E2E tests covering scan and payment flows, integrated into CI/CD with coverage enforcement.

**Phase Numbering:**
- Integer phases (25, 26, 27, 28): Planned milestone work
- Decimal phases (25.1, 25.2): Urgent insertions (marked with INSERTED)

- [x] **Phase 25: Test Infrastructure** — Vitest, MSW, RTL setup with working foundation and first passing test (completed 2026-02-17)
- [x] **Phase 26: Component Tests** — Unit tests for all 9 client components plus dark mode, loading, and error states (completed 2026-02-17)
- [ ] **Phase 27: E2E Tests** — Playwright setup with free scan flow, paid audit flow, and error flow coverage
- [ ] **Phase 28: CI/CD and Quality Gates** — GitHub Actions pipeline with parallel test jobs, caching, and coverage enforcement

## Phase Details

### Phase 25: Test Infrastructure
**Goal**: Developers can run `npm test` and see a working test suite with mocking infrastructure ready for component and integration tests
**Depends on**: Nothing (first phase of v1.5)
**Requirements**: INFRA-01, INFRA-02, INFRA-03, INFRA-04, INFRA-05, INFRA-06
**Success Criteria** (what must be TRUE):
  1. Running `npm test` executes Vitest with happy-dom and all `@/*` imports resolve correctly
  2. MSW handlers intercept API calls for scan, results, checkout, and webhook endpoints in tests
  3. Components render in tests using the custom RTL wrapper with providers
  4. Environment variables from `.env.test` are available in the test environment
  5. Components using `useRouter`, `usePathname`, or `useSearchParams` render without errors in tests
**Plans**: 2 plans

Plans:
- [ ] 25-01-PLAN.md — Vitest config, test dependencies, env, custom render wrapper, test scripts
- [ ] 25-02-PLAN.md — MSW fixtures/handlers/server, navigation mocks, Header test, integration test

### Phase 26: Component Tests
**Goal**: Every client component has tests verifying its rendering, interactions, and edge cases from a user's perspective
**Depends on**: Phase 25
**Requirements**: COMP-01, COMP-02, COMP-03, COMP-04, COMP-05, COMP-06, COMP-07, COMP-08, COMP-09, COMP-10, COMP-11, COMP-12
**Success Criteria** (what must be TRUE):
  1. ScanForm tests verify URL validation, email validation, CFAA consent, submission, loading state, and error display
  2. Results-related components (ResultsDashboard, GradeSummary, FindingAccordion, ProgressChecklist) render findings, grades, severity, and state transitions correctly in tests
  3. Layout components (Header, Footer, Logo, UpgradeCTA) render their content and interactions correctly in tests
  4. All components render correctly under both light and dark color schemes in tests
  5. Loading skeletons and error boundary fallback UI render correctly in tests
**Plans**: 4 plans

Plans:
- [ ] 26-01-PLAN.md — ScanForm tests (useActionState mock, validation errors, loading/success states, user interactions)
- [ ] 26-02-PLAN.md — ResultsDashboard, GradeSummary, FindingAccordion tests (findings rendering, grade display, expand/collapse)
- [ ] 26-03-PLAN.md — ProgressChecklist, UpgradeCTA, Footer, Logo tests (stage transitions, checkout flow, legal links, size variants)
- [ ] 26-04-PLAN.md — Dark mode rendering, loading skeletons, error boundary tests (cross-cutting verification)

### Phase 27: E2E Tests
**Goal**: Critical user journeys (free scan, paid audit, error recovery) are verified end-to-end in a production-like browser environment
**Depends on**: Phase 25
**Requirements**: E2E-01, E2E-02, E2E-03, E2E-04, E2E-05
**Success Criteria** (what must be TRUE):
  1. Free scan E2E test navigates from home page through URL submission, scan progress polling, to results page with grade and findings
  2. Paid audit E2E test verifies UpgradeCTA click triggers Stripe Checkout redirect and return to payment success page
  3. Error flow E2E tests verify invalid URL handling, 404 for missing scans, and API error states display correctly
  4. All E2E tests run against a production build (`npm run build && npm run start`), not the dev server
**Plans**: 3 plans

Plans:
- [ ] 27-01-PLAN.md — Playwright infrastructure, config, testProxy, E2E fixtures, route interception helpers
- [ ] 27-02-PLAN.md — Free scan flow E2E test, paid audit flow E2E test with Stripe checkout
- [ ] 27-03-PLAN.md — Error flow E2E tests (validation, 404, timeout, 500, recovery)

### Phase 28: CI/CD and Quality Gates
**Goal**: Every PR and push to main automatically runs both test suites with coverage enforcement, blocking merges on failure
**Depends on**: Phase 26, Phase 27
**Requirements**: CI-01, CI-02, CI-03, CI-04, CI-05, CI-06, QUAL-01, QUAL-02, QUAL-03
**Success Criteria** (what must be TRUE):
  1. Pushing to main or opening a PR triggers Vitest and Playwright jobs running in parallel in GitHub Actions
  2. CI uses cached npm dependencies and Playwright browsers for fast feedback
  3. Failed Playwright tests upload screenshots and traces as artifacts for debugging
  4. PRs cannot merge when any test job fails
  5. CI fails when code coverage drops below 80% lines, 80% functions, or 75% branches
**Plans**: TBD

Plans:
- [ ] 28-01: TBD
- [ ] 28-02: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 01-24 (complete) → 25 → 26 → 27 → 28

| Phase | Milestone | Plans | Status | Completed |
|-------|-----------|-------|--------|-----------|
| 01 - Foundation | v1.0 | 5/5 | Complete | 2026-02-04 |
| 02 - Free Tier MVP | v1.0 | 8/8 | Complete | 2026-02-05 |
| 03 - Vibe-Code Intelligence | v1.0 | 5/5 | Complete | 2026-02-05 |
| 04 - Monetization | v1.0 | 5/5 | Complete | 2026-02-06 |
| 05 - Codebase Preparation | v1.1 | 4/4 | Complete | 2026-02-07 |
| 06 - Deployment Infrastructure | v1.1 | 4/4 | Complete | 2026-02-08 |
| 07 - Production Validation | v1.1 | 2/2 | Complete | 2026-02-08 |
| 08 - Analytics & Tracking | v1.2 | 1/1 | Complete | 2026-02-08 |
| 09 - SEO & Discoverability | v1.2 | 2/2 | Complete | 2026-02-08 |
| 10 - Legal Compliance | v1.2 | 2/2 | Complete | 2026-02-08 |
| 11 - Mobile & UX Polish | v1.2 | 3/3 | Complete | 2026-02-09 |
| 12 - Landing Page Optimization | v1.2 | 2/2 | Complete | 2026-02-09 |
| 13 - Design Token System | v1.3 | 3/3 | Complete | 2026-02-10 |
| 14 - Logo Component | v1.3 | 2/2 | Complete | 2026-02-11 |
| 15 - Layout Refactor | v1.3 | 1/1 | Complete | 2026-02-11 |
| 16 - Header & Navigation | v1.3 | 1/1 | Complete | 2026-02-11 |
| 17 - Icon System & Migration | v1.3 | 1/1 | Complete | 2026-02-11 |
| 18 - Favicon & OG Image | v1.3 | 2/2 | Complete | 2026-02-11 |
| 19 - Structured JSON Logging | v1.4 | 2/2 | Complete | 2026-02-16 |
| 20 - Request Tracing | v1.4 | 2/2 | Complete | 2026-02-16 |
| 21 - Health Checks | v1.4 | 1/1 | Complete | 2026-02-16 |
| 22 - Prometheus Metrics | v1.4 | 2/2 | Complete | 2026-02-16 |
| 23 - Graceful Shutdown | v1.4 | 2/2 | Complete | 2026-02-16 |
| 24 - Infrastructure Integration | v1.4 | 2/2 | Complete | 2026-02-16 |
| 25 - Test Infrastructure | v1.5 | Complete    | 2026-02-17 | - |
| 26 - Component Tests | v1.5 | Complete    | 2026-02-17 | - |
| 27 - E2E Tests | v1.5 | 0/TBD | Not started | - |
| 28 - CI/CD and Quality Gates | v1.5 | 0/TBD | Not started | - |

---
*Last updated: 2026-02-16*
