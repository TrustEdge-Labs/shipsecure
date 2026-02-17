# Feature Research

**Domain:** Frontend Testing Infrastructure for Next.js 16 + React 19 SaaS Application
**Researched:** 2026-02-16
**Confidence:** HIGH

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Component Unit Tests** | Standard practice for React components, ensures individual components work in isolation | MEDIUM | Use Vitest + React Testing Library. Next.js officially recommends Vitest for unit testing as of Feb 2026. Test props, rendering, user events. |
| **Form Validation Testing** | ScanForm uses Zod + React Hook Form - validation logic must be tested | MEDIUM | Test submission failures, field validation (URL, email, CFAA consent), successful submission. Use `waitFor` and `find*` queries for async validation. |
| **API Mocking** | Frontend tests shouldn't hit real backend APIs or external services | MEDIUM | Use MSW (Mock Service Worker) to intercept network requests. Reusable mocks across unit, integration, and E2E tests. |
| **Loading State Testing** | App uses loading skeletons for scan progress - must verify loading UI appears correctly | LOW | Test that skeleton components render during async operations. Use `data-testid` attributes on skeleton elements. |
| **Error Boundary Testing** | Next.js error.js files handle runtime errors - must verify fallback UI | MEDIUM | Test that error boundaries catch rendering errors and display fallback. React 19 has improved error boundary testing with DevTools toggle and `reset()` function. |
| **Critical Path E2E Tests** | Payment flows, scan submission, results viewing must work end-to-end | HIGH | Playwright for E2E. Cover: free scan flow (submit → poll → results), paid audit flow (CTA → Stripe → success). Next.js docs recommend E2E for async Server Components. |
| **Stripe Payment Testing** | Stripe checkout integration must be tested without real payments | MEDIUM | Use Stripe Test Mode with test card numbers. Mock Stripe API responses for component tests. Playwright for full checkout flow. Cannot automate actual Stripe Checkout UI (security measures), but can test redirect and success page. |
| **Dark Mode Testing** | App supports dark mode - components must render correctly in both themes | LOW | Test components with different theme contexts. Verify theme-specific CSS classes apply correctly. |
| **Responsive Layout Testing** | Mobile-responsive design must work across viewport sizes | MEDIUM | Playwright supports viewport testing. Test key breakpoints (mobile, tablet, desktop) for forms and dashboards. |
| **Server Action Testing** | Server actions (submitScan, etc.) must be tested independently | MEDIUM | Vitest can test synchronous server actions. For async actions with `redirect()`, use E2E tests per Next.js recommendations. Mock `next/navigation` for unit tests. |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Visual Regression Testing** | Catch unintended UI changes automatically, ensure design consistency across updates | MEDIUM | Use Playwright snapshots or Chromatic. 2-4 snapshots per component is 2026 best practice. Detects CSS/layout regressions before production. |
| **Accessibility Testing** | Automated a11y checks ensure WCAG compliance, improve SEO and user experience | LOW | Use jest-axe (or vitest-axe) with React Testing Library. Catches issues like missing alt text, improper heading structure. Not a replacement for manual testing but catches 30-40% of issues automatically. |
| **CI/CD Test Pipeline** | Automated test runs on every PR prevent regressions from reaching main | MEDIUM | GitHub Actions workflow running Vitest (unit) + Playwright (E2E). Parallel execution for speed. Block merges if tests fail. |
| **Test Coverage Reporting** | Visibility into which code is tested helps identify gaps | LOW | Vitest has built-in coverage via c8/Istanbul. Generate HTML reports, enforce minimum thresholds (e.g., 80% component coverage). |
| **Stripe Test Clocks** | Test subscription billing lifecycles (trial → payment → renewal) in minutes instead of months | MEDIUM | Stripe Test Clocks + Playwright E2E can verify complete billing flows. Provides 99.99% faster billing verification per recent research. |
| **Component Storybook** | Isolated component development and visual testing outside main app | MEDIUM | Storybook for component isolation/documentation. Integrates with Chromatic for cloud-based visual testing. Not strictly testing but enables better component testing workflow. |
| **Performance Testing** | Measure component render times, identify performance regressions | LOW | React DevTools Profiler or Lighthouse CI. Track metrics like Largest Contentful Paint (LCP) for ResultsDashboard. |
| **Multi-Browser Testing** | Verify app works in Chrome, Firefox, Safari | MEDIUM | Playwright supports all three browsers. Critical for catching cross-browser rendering differences. 2026 trend shows even same snapshots render differently across browsers. |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **100% Code Coverage** | Seems like thorough testing | Encourages testing implementation details instead of behavior. Diminishing returns above 80-90%. Creates brittle tests that break on refactors. | **Target 80% coverage for components**, focus on critical paths and user flows. Quality over quantity. |
| **Snapshot Testing Everything** | Easy to generate, catches changes | Snapshots become massive, fail frequently on intentional changes, developers blindly update them without review. Too many = noise. | **Use snapshots sparingly** - only for stable components like static layouts. Prefer visual regression (Playwright screenshots) over DOM snapshots. 2-4 per component max. |
| **Mocking Everything** | Tests run fast, no dependencies | Over-mocking defeats the purpose - you're testing the mock, not the code. Integration tests become pointless. | **Use MSW for API mocking**, but test real component interactions. Mock external services (Stripe), but not internal components. |
| **Testing Implementation Details** | Ensures code works exactly as written | Tests break when refactoring even if behavior is unchanged. Couples tests to implementation. React Testing Library philosophy: test what users see, not how it works. | **Test user behavior** - query by role/label, simulate user events, assert on visible output. Use `screen.getByRole()` not `wrapper.find()`. |
| **E2E Tests for Everything** | Seems most realistic | E2E tests are slow, flaky, expensive to maintain. Running full browser for every test is overkill. | **Test pyramid**: many unit tests (fast, cheap), some integration tests (medium), few E2E tests (slow, expensive). E2E only for critical user flows. |
| **Enzyme-style Shallow Rendering** | Fast, isolated | React deprecated shallow rendering. Enzyme doesn't support React 19. Shallow rendering tests implementation, not user experience. | **React Testing Library** is the standard for React 19. Renders full components, encourages accessibility-friendly queries. |
| **Testing Every Edge Case** | Complete coverage | Infinite edge cases exist. Testing unlikely scenarios wastes time. 80/20 rule: 20% of tests catch 80% of bugs. | **Focus on user scenarios** and common error cases. Use production error monitoring (Sentry) to identify which edge cases actually occur. |

## Feature Dependencies

```
Component Unit Tests (Vitest + RTL)
    └──requires──> API Mocking (MSW)
                       └──enhances──> Integration Tests

E2E Tests (Playwright)
    └──requires──> Stripe Test Mode
    └──requires──> Local Dev Environment
    └──enhances──> Visual Regression Testing

Server Action Testing
    └──requires──> Vitest Setup
    └──conflicts──> Async Server Components (use E2E instead)

CI/CD Pipeline
    └──requires──> Unit Tests
    └──requires──> E2E Tests
    └──enhances──> Coverage Reporting

Visual Regression Testing
    └──requires──> Component Tests or Storybook
    └──enhances──> CI/CD Pipeline

Accessibility Testing (jest-axe)
    └──requires──> Component Unit Tests
    └──integrates──> React Testing Library
```

### Dependency Notes

- **Component Unit Tests require API Mocking:** Can't test ScanForm submission without mocking the server action/API call.
- **E2E Tests require Stripe Test Mode:** Payment flow testing needs Stripe test environment configured with test API keys.
- **Server Action Testing conflicts with Async Server Components:** Per Next.js docs (Feb 2026), Vitest doesn't support async Server Components - must use E2E tests instead.
- **Visual Regression enhances CI/CD:** Screenshots in CI catch visual bugs, but requires stable rendering environment.
- **Accessibility Testing integrates with RTL:** jest-axe works seamlessly with React Testing Library's rendering approach.

## MVP Definition

### Launch With (v1)

Minimum viable testing infrastructure — what's needed to ship with confidence.

- [x] **Vitest + React Testing Library Setup** — Core unit testing framework for components
- [x] **MSW Setup** — API mocking for isolated tests
- [x] **Component Unit Tests** — Test 9 existing components (ScanForm, ResultsDashboard, GradeSummary, FindingAccordion, ProgressChecklist, UpgradeCTA, Header, Footer, Logo)
- [x] **Form Validation Tests** — Critical: ScanForm URL/email/consent validation with Zod
- [x] **Loading State Tests** — Verify skeletons appear during async operations
- [x] **Error Boundary Tests** — Test error.js fallback UI renders correctly
- [x] **Playwright E2E Setup** — End-to-end testing framework
- [x] **Critical Path E2E Tests** — Free scan flow (home → submit → scan/[id] → results/[token]) and payment flow (UpgradeCTA → Stripe → payment/success)
- [x] **Stripe Test Mode Configuration** — Test API keys in .env, test card numbers documented
- [x] **Dark Mode Tests** — Verify components render in both light/dark themes
- [x] **Basic CI Pipeline** — GitHub Actions running Vitest + Playwright on PRs

**Why essential:** These features enable confident shipping. Without unit tests, component changes risk breaking existing functionality. Without E2E tests, critical user flows (scans, payments) could fail in production. Without mocking, tests become flaky and slow.

### Add After Validation (v1.x)

Features to add once core testing is working.

- [ ] **Coverage Reporting** — Track which code is tested, enforce 80% threshold (trigger: after initial test suite written)
- [ ] **Visual Regression Testing** — Playwright snapshots for ResultsDashboard, GradeSummary (trigger: UI bugs reach production)
- [ ] **Accessibility Testing** — jest-axe integration for WCAG compliance (trigger: SEO/compliance becomes priority)
- [ ] **Server Action Unit Tests** — Test submitScan and other server actions independently (trigger: server logic bugs discovered)
- [ ] **Responsive Layout Tests** — Playwright viewport testing for mobile/tablet (trigger: mobile user complaints)
- [ ] **Multi-Browser Testing** — Firefox, Safari in addition to Chrome (trigger: cross-browser bug reports)
- [ ] **Performance Testing** — Lighthouse CI for LCP/FID/CLS tracking (trigger: performance complaints)

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] **Stripe Test Clocks** — Test subscription lifecycle edge cases (trigger: subscription features added)
- [ ] **Storybook Integration** — Component library documentation (trigger: team grows beyond 2 developers)
- [ ] **Mutation Testing** — Verify test quality by injecting code changes (trigger: test suite feels insufficient despite high coverage)
- [ ] **Contract Testing** — Pact/Swagger for frontend-backend API contracts (trigger: API versioning issues)
- [ ] **Load Testing** — Artillery/k6 for concurrent user testing (trigger: scaling beyond 1K concurrent users)

**Why defer:** These features have diminishing returns or solve problems you don't have yet. Test Clocks are only valuable for subscription billing (not in MVP). Storybook adds maintenance overhead for small teams. Mutation testing and contract testing are advanced techniques for mature codebases.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Component Unit Tests (Vitest + RTL) | HIGH | MEDIUM | P1 |
| Form Validation Tests (Zod) | HIGH | MEDIUM | P1 |
| E2E Critical Paths (Playwright) | HIGH | HIGH | P1 |
| API Mocking (MSW) | HIGH | MEDIUM | P1 |
| Stripe Test Mode | HIGH | LOW | P1 |
| Error Boundary Tests | MEDIUM | MEDIUM | P1 |
| Loading State Tests | MEDIUM | LOW | P1 |
| Dark Mode Tests | MEDIUM | LOW | P1 |
| Server Action Tests | MEDIUM | MEDIUM | P1 |
| CI/CD Pipeline | HIGH | MEDIUM | P1 |
| Coverage Reporting | MEDIUM | LOW | P2 |
| Visual Regression | MEDIUM | MEDIUM | P2 |
| Accessibility Testing (jest-axe) | MEDIUM | LOW | P2 |
| Responsive Layout Tests | MEDIUM | MEDIUM | P2 |
| Multi-Browser Testing | MEDIUM | LOW | P2 |
| Performance Testing | LOW | MEDIUM | P2 |
| Stripe Test Clocks | LOW | MEDIUM | P3 |
| Storybook | LOW | HIGH | P3 |
| Mutation Testing | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for launch — enables confident shipping and prevents critical bugs
- P2: Should have, add when possible — improves quality but not blocking
- P3: Nice to have, future consideration — advanced features for mature products

## Test Type Breakdown for ShipSecure

### Unit Tests (Vitest + React Testing Library)

**What to test:**
- **ScanForm**: URL validation, email validation, CFAA consent requirement, form submission (mocked), error display, success state, loading state (pending)
- **ResultsDashboard**: Renders findings by category, displays grade, handles empty findings, error states
- **GradeSummary**: Displays correct grade (A-F), color coding, grade text
- **FindingAccordion**: Expand/collapse behavior, finding details rendering, severity indicators
- **ProgressChecklist**: Stage progression, completed/active/pending states, checkmarks
- **UpgradeCTA**: Renders pricing info, Stripe checkout link, click tracking
- **Header/Footer/Logo**: Render without errors, navigation links, dark mode toggle

**How to test:**
```typescript
// Example: ScanForm validation
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { ScanForm } from './scan-form'
import { submitScan } from '@/app/actions/scan'

// Mock server action
vi.mock('@/app/actions/scan')

test('shows error for invalid URL', async () => {
  render(<ScanForm />)

  const urlInput = screen.getByLabelText(/website url/i)
  const submitButton = screen.getByRole('button', { name: /scan/i })

  await userEvent.type(urlInput, 'not-a-url')
  await userEvent.click(submitButton)

  await waitFor(() => {
    expect(screen.getByText(/invalid url/i)).toBeInTheDocument()
  })
})
```

**Why these tests matter:** Components are the building blocks of the UI. If they don't work independently, integrated features will fail. Form validation is critical — bad URLs/emails cause backend errors.

### Integration Tests (Vitest + MSW)

**What to test:**
- **ScanForm + Server Action**: Full form submission flow with mocked API
- **ResultsDashboard + API Client**: Fetching and displaying results with mocked backend
- **Payment flow components**: UpgradeCTA → Stripe checkout session creation (mocked)

**How to test:**
```typescript
// Example: Form submission integration
import { http, HttpResponse } from 'msw'
import { setupServer } from 'msw/node'

const server = setupServer(
  http.post('/api/scans', () => {
    return HttpResponse.json({ scanId: '123' })
  })
)

beforeAll(() => server.listen())
afterEach(() => server.resetHandlers())
afterAll(() => server.close())

test('submits scan and redirects', async () => {
  // Test form → API → redirect
})
```

**Why these tests matter:** Unit tests verify components in isolation, but integration tests ensure they work together. ScanForm might validate correctly but fail when submitting to the server action.

### E2E Tests (Playwright)

**What to test:**
- **Free Scan Flow**: Home → submit URL/email → scan/[id] polling → results/[token] display
- **Paid Audit Flow**: Click UpgradeCTA → redirect to Stripe Checkout → return to payment/success
- **Error Flows**: Invalid URL submission, scan not found, API errors
- **Authentication**: If auth is added, login/logout flows

**How to test:**
```typescript
// Example: Free scan flow E2E
import { test, expect } from '@playwright/test'

test('complete free scan flow', async ({ page }) => {
  await page.goto('http://localhost:3000')

  // Submit scan
  await page.fill('[name="url"]', 'https://example.com')
  await page.fill('[name="email"]', 'test@example.com')
  await page.check('[name="cfaa_consent"]')
  await page.click('text=Start Free Scan')

  // Wait for redirect to scan/[id]
  await expect(page).toHaveURL(/\/scan\/\w+/)
  await expect(page.locator('text=Scanning')).toBeVisible()

  // Wait for redirect to results/[token]
  await expect(page).toHaveURL(/\/results\/\w+/, { timeout: 30000 })
  await expect(page.locator('text=Security Grade')).toBeVisible()
})
```

**Why these tests matter:** E2E tests verify the entire user journey works. They catch issues that unit/integration tests miss: routing bugs, timing issues, async Server Component problems, external service integration failures.

### Visual Regression Tests (Playwright Snapshots)

**What to test:**
- **ResultsDashboard**: Ensure layout doesn't break on data changes
- **GradeSummary**: Verify grade colors/styling consistent
- **ScanForm**: Catch accidental CSS changes
- **Error pages**: 404, error boundaries

**How to test:**
```typescript
test('results dashboard visual regression', async ({ page }) => {
  await page.goto('/results/test-token')
  await expect(page).toHaveScreenshot('results-dashboard.png')
})
```

**Why these tests matter:** CSS changes can break layouts without breaking functionality. Visual regression catches unintended design changes before users see them.

### Accessibility Tests (jest-axe)

**What to test:**
- **All components**: WCAG compliance (color contrast, alt text, ARIA labels, heading hierarchy)
- **Forms**: Proper label associations, error announcements
- **Navigation**: Keyboard navigation, focus management

**How to test:**
```typescript
import { axe, toHaveNoViolations } from 'jest-axe'
expect.extend(toHaveNoViolations)

test('ScanForm has no a11y violations', async () => {
  const { container } = render(<ScanForm />)
  const results = await axe(container)
  expect(results).toHaveNoViolations()
})
```

**Why these tests matter:** Accessibility improves SEO, legal compliance (WCAG/ADA), and user experience. Automated tests catch 30-40% of a11y issues cheaply.

## Testing Stripe Payments

**Stripe provides three testing approaches:**

1. **Test Mode** - Real Stripe API, fake payment methods
   - Use test API keys (`pk_test_...`, `sk_test_...`)
   - Test card numbers: `4242 4242 4242 4242` (success), `4000 0000 0000 0002` (declined)
   - Best for: E2E tests, manual QA

2. **Mocked Responses** - Mock Stripe API calls in tests
   - Use MSW to intercept Stripe API requests
   - Return fake checkout session IDs, payment intents
   - Best for: Component unit tests, integration tests

3. **Test Clocks** - Simulate time for subscription billing
   - Advance time to test trial → paid → renewal
   - Requires Test Clock API setup
   - Best for: Subscription lifecycle testing (future feature)

**For ShipSecure MVP:**
- **E2E tests**: Use Stripe Test Mode + test card numbers
- **Component tests**: Mock `createCheckoutSession` server action
- **Cannot automate**: Actual Stripe Checkout UI (security prevents Playwright from filling forms). Test redirect to Stripe and return to success page instead.

## Competitor Feature Analysis

| Feature | Vercel (Next.js Creator) | Stripe (Payments) | Our Approach |
|---------|--------------------------|-------------------|--------------|
| Unit Testing | Recommends Jest or Vitest | Not applicable | **Vitest** (faster, better ESM support) |
| E2E Testing | Recommends Playwright or Cypress | Not applicable | **Playwright** (official Next.js docs, multi-browser) |
| API Mocking | No official stance | Recommends mocking for automated tests | **MSW** (industry standard, network-level mocking) |
| Payment Testing | Not applicable | Test Mode + Test Clocks | **Test Mode for MVP**, Test Clocks if subscriptions added |
| Coverage Tools | No official stance | Not applicable | **Vitest built-in coverage** (c8/Istanbul) |
| Visual Regression | No official stance | Not applicable | **Playwright snapshots** (built-in, fast) |
| Accessibility | Includes eslint-plugin-jsx-a11y | Not applicable | **ESLint plugin (static) + jest-axe (runtime)** |

## Sources

### Official Documentation (HIGH Confidence)
- [Next.js Testing Guide](https://nextjs.org/docs/app/guides/testing) - Updated Feb 11, 2026
- [Next.js Vitest Setup](https://nextjs.org/docs/app/guides/testing/vitest) - Official Next.js docs
- [Stripe Automated Testing](https://docs.stripe.com/automated-testing) - Official Stripe docs

### Testing Tools & Best Practices (MEDIUM-HIGH Confidence)
- [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/)
- [Mock Service Worker](https://mswjs.io/)
- [Playwright Documentation](https://playwright.dev/)
- [Testing in 2026: Jest, React Testing Library, and Full Stack Testing Strategies](https://www.nucamp.co/blog/testing-in-2026-jest-react-testing-library-and-full-stack-testing-strategies)
- [Test Strategy in the Next.js App Router Era](https://shinagawa-web.com/en/blogs/nextjs-app-router-testing-setup)

### Specialized Testing (MEDIUM Confidence)
- [No More 'Ship and Pray': Testing SaaS Billing Systems with Playwright & Stripe Test Clocks](https://hackernoon.com/no-more-ship-and-pray-testing-saas-billing-systems-with-playwright-and-stripe-test-clocks)
- [How to Test React Applications for Accessibility with axe-core](https://oneuptime.com/blog/post/2026-01-15-test-react-accessibility-axe-core/view)
- [How to Mock API Calls in React Tests with MSW](https://oneuptime.com/blog/post/2026-01-15-mock-api-calls-react-msw/view)
- [Snapshot Testing with Playwright in 2026](https://www.browserstack.com/guide/playwright-snapshot-testing)
- [Visual Regression Testing in Mobile QA: The 2026 Guide](https://www.getpanto.ai/blog/visual-regression-testing-in-mobile-qa)

### React 19 & Modern Patterns (MEDIUM Confidence)
- [React 19 Upgrade Guide](https://react.dev/blog/2024/04/25/react-19-upgrade-guide)
- [Solving Dependency Conflicts: React 19 and Testing Library Issues](https://medium.com/@dinukakeshan/solving-dependency-conflicts-react-19-and-testing-library-issues-2eb6f773d4ea)
- [Error Handling in React with react-error-boundary](https://certificates.dev/blog/error-handling-in-react-with-react-error-boundary)
- [Next.js Error Boundary Best Practices](https://www.dhiwise.com/post/nextjs-error-boundary-best-practices)

### Form Validation & Zod (MEDIUM Confidence)
- [Zod React Hook Form: Complete Guide 2026](https://practicaldev.online/blog/reactjs/react-hook-form-zod-validation-guide)
- [React Hook Form with Zod Validation: A Complete Guide](https://medium.com/@toukir.ahamed.pigeon/react-hook-form-with-zod-validation-a-complete-guide-with-typescript-aacbcb370a8b)
- [Learn Zod validation with React Hook Form](https://www.contentful.com/blog/react-hook-form-validation-zod/)

---
*Feature research for: ShipSecure Frontend Testing Infrastructure*
*Researched: 2026-02-16*
