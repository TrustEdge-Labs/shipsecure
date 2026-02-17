# Phase 27: E2E Tests - Context

**Gathered:** 2026-02-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Playwright E2E tests verifying critical user journeys: free scan flow, paid audit flow, and error recovery. Tests run against a production build with mocked API responses. Playwright setup, test infrastructure, and CI integration are in scope. Adding new application features or modifying existing behavior is not.

</domain>

<decisions>
## Implementation Decisions

### Backend approach
- Mock all API responses using Playwright route interception — no real backend needed
- Separate E2E-specific fixtures (not shared with MSW component test fixtures)
- Short delays (100-500ms) on mocked responses to simulate real timing and catch race conditions
- Mock responses must match real API exactly — correct status codes, headers, and response body shapes

### Stripe payment boundary
- UpgradeCTA click test: intercept the redirect and verify it targets a Stripe Checkout URL pattern
- Success return: navigate directly to /payment/success?session_id=mock_123, verify success page renders
- Cancel return: navigate to cancel/failure return URL, verify UI handles cancellation gracefully
- After payment success: navigate to results page and verify paid-tier content appears (PDF link, deeper findings)

### Error scenario coverage
- Invalid URL: test both client-side form validation AND server-rejection of unreachable domains
- 404 missing scan: test via direct URL navigation AND via a previously-valid scan link becoming invalid
- Network timeout: simulate API not responding, verify timeout/connection error message
- Server 500: API returns 500, verify error boundary or error message displays
- Recovery: all error tests verify the user can retry or navigate away successfully (not just error display)

### Scan progress flow
- Verify scan starts (progress UI shown) and completes (results page) — skip verifying intermediate stage transitions
- Results page: full content verification — grade, severity badges, finding details, and UpgradeCTA for free tier
- CFAA consent: test that submitting without consent fails, then check it and submit successfully
- Email input: tested as part of the main scan submission flow, no separate validation test

### Claude's Discretion
- Playwright configuration details (browsers, viewport sizes, timeouts)
- Exact delay values for mocked responses
- Test file organization and naming conventions
- How to structure route interception helpers

</decisions>

<specifics>
## Specific Ideas

- Tests run against production build (npm run build && npm run start), not dev server
- Stripe Checkout UI cannot be automated — test up to redirect and return page only (noted blocker from STATE.md)
- Mock response fidelity matters: response shapes should mirror actual backend so tests catch frontend assumptions about API contracts

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 27-e2e-tests*
*Context gathered: 2026-02-16*
