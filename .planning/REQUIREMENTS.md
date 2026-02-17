# Requirements: ShipSecure v1.5 Frontend Testing

**Defined:** 2026-02-16
**Core Value:** Catch security flaws in vibe-coded apps before they become breaches, with remediation guidance anyone can follow.

## v1.5 Requirements

Requirements for the Frontend Testing milestone. Each maps to roadmap phases.

### Test Infrastructure

- [ ] **INFRA-01**: Vitest configured with happy-dom environment, React plugin, and TypeScript path alias resolution for `@/*` imports
- [ ] **INFRA-02**: MSW (Mock Service Worker) configured with reusable API handlers for scan, results, checkout, and webhook endpoints
- [ ] **INFRA-03**: Custom RTL render wrapper with provider support for consistent component test setup
- [ ] **INFRA-04**: Environment variable loading from `.env.test` via `@next/env` in Vitest config
- [ ] **INFRA-05**: `next/navigation` hooks (useRouter, usePathname, useSearchParams) mocked globally for component tests
- [ ] **INFRA-06**: Test scripts added to package.json (`test`, `test:unit`, `test:e2e`, `test:coverage`)

### Component Unit Tests

- [ ] **COMP-01**: ScanForm tests covering URL validation, email validation, CFAA consent requirement, form submission (mocked server action), loading state, and error display
- [ ] **COMP-02**: ResultsDashboard tests covering findings rendering by category, grade display, empty state, and error state
- [ ] **COMP-03**: GradeSummary tests covering grade display (A-F), color coding, and grade text
- [ ] **COMP-04**: FindingAccordion tests covering expand/collapse behavior, finding details rendering, and severity indicators
- [ ] **COMP-05**: ProgressChecklist tests covering stage progression, completed/active/pending states, and checkmarks
- [ ] **COMP-06**: UpgradeCTA tests covering pricing display, checkout link rendering, and click behavior
- [ ] **COMP-07**: Header tests covering navigation links, logo rendering, "Scan Now" CTA, and responsive behavior
- [ ] **COMP-08**: Footer tests covering legal links, OSS attribution, and rendering without errors
- [ ] **COMP-09**: Logo tests covering icon/compact/full variant rendering and dark mode
- [ ] **COMP-10**: Dark mode rendering verified for all components using `prefers-color-scheme` media query
- [ ] **COMP-11**: Loading skeleton components tested for correct rendering during async operations
- [ ] **COMP-12**: Error boundary (error.tsx) tested for fallback UI rendering on component errors

### E2E Tests

- [ ] **E2E-01**: Free scan flow E2E test covering home page → URL/email submission → scan progress page (polling) → results page with grade and findings
- [ ] **E2E-02**: Paid audit flow E2E test covering UpgradeCTA click → Stripe Checkout redirect → return to payment success page
- [ ] **E2E-03**: Error flow E2E tests covering invalid URL submission, scan not found (404), and API error states
- [ ] **E2E-04**: Playwright configured to run against production build (`npm run build && npm run start`), not dev server
- [ ] **E2E-05**: Stripe Test Mode configured with test API keys and documented test card numbers

### CI/CD Integration

- [ ] **CI-01**: GitHub Actions workflow running Vitest unit/component tests on every PR and push to main
- [ ] **CI-02**: GitHub Actions workflow running Playwright E2E tests on every PR and push to main
- [ ] **CI-03**: Vitest and Playwright jobs run as separate CI jobs with E2E gated on unit test success for resource efficiency
- [ ] **CI-04**: npm dependency caching (node_modules) configured for CI performance; Playwright browsers installed fresh each run per official recommendation
- [ ] **CI-05**: Playwright test artifacts (screenshots, traces) uploaded on test failure for debugging
- [ ] **CI-06**: PR merges blocked when any test job fails

### Quality Gates

- [ ] **QUAL-01**: Code coverage thresholds enforced: 80% lines, 80% functions, 75% branches
- [ ] **QUAL-02**: Coverage reports generated in HTML and lcov formats
- [ ] **QUAL-03**: CI fails when coverage drops below configured thresholds

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Extended Testing

- **EXT-01**: Visual regression testing via Playwright snapshots for ResultsDashboard and GradeSummary
- **EXT-02**: Accessibility testing with jest-axe for WCAG compliance across all components
- **EXT-03**: Responsive layout tests at mobile (375px), tablet (768px), and desktop (1024px) breakpoints
- **EXT-04**: Multi-browser testing (Firefox, Safari in addition to Chromium)
- **EXT-05**: Performance testing with Lighthouse CI for LCP/FID/CLS tracking

### Advanced Testing

- **ADV-01**: Stripe Test Clocks for subscription lifecycle testing (when subscriptions added)
- **ADV-02**: Storybook integration for component isolation and documentation
- **ADV-03**: Mutation testing to verify test quality
- **ADV-04**: Contract testing for frontend-backend API contracts

## Out of Scope

| Feature | Reason |
|---------|--------|
| Backend/Rust testing | Separate milestone; this is frontend-only |
| 100% code coverage | Diminishing returns; encourages testing implementation details |
| Enzyme/shallow rendering | Deprecated; incompatible with React 19 |
| Jest test runner | Poor ESM support; Vitest is Next.js recommended |
| Cypress E2E | Slower CI, more complex than Playwright |
| Stripe Checkout UI automation | Stripe security measures prevent Playwright from filling hosted checkout form |
| Snapshot testing for all components | Brittle, high maintenance; prefer visual regression in v2 |
| Real backend integration in E2E | Use Playwright route mocking for v1.5; real backend in v2 |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| INFRA-01 | Phase 25 | Pending |
| INFRA-02 | Phase 25 | Pending |
| INFRA-03 | Phase 25 | Pending |
| INFRA-04 | Phase 25 | Pending |
| INFRA-05 | Phase 25 | Pending |
| INFRA-06 | Phase 25 | Pending |
| COMP-01 | Phase 26 | Pending |
| COMP-02 | Phase 26 | Pending |
| COMP-03 | Phase 26 | Pending |
| COMP-04 | Phase 26 | Pending |
| COMP-05 | Phase 26 | Pending |
| COMP-06 | Phase 26 | Pending |
| COMP-07 | Phase 26 | Pending |
| COMP-08 | Phase 26 | Pending |
| COMP-09 | Phase 26 | Pending |
| COMP-10 | Phase 26 | Pending |
| COMP-11 | Phase 26 | Pending |
| COMP-12 | Phase 26 | Pending |
| E2E-01 | Phase 27 | Pending |
| E2E-02 | Phase 27 | Pending |
| E2E-03 | Phase 27 | Pending |
| E2E-04 | Phase 27 | Pending |
| E2E-05 | Phase 27 | Pending |
| CI-01 | Phase 28 | Pending |
| CI-02 | Phase 28 | Pending |
| CI-03 | Phase 28 | Pending |
| CI-04 | Phase 28 | Pending |
| CI-05 | Phase 28 | Pending |
| CI-06 | Phase 28 | Pending |
| QUAL-01 | Phase 28 | Pending |
| QUAL-02 | Phase 28 | Pending |
| QUAL-03 | Phase 28 | Pending |

**Coverage:**
- v1.5 requirements: 32 total
- Mapped to phases: 32
- Unmapped: 0

---
*Requirements defined: 2026-02-16*
*Last updated: 2026-02-16 after roadmap creation*
