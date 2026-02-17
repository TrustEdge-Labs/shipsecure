# Phase 27: E2E Tests - Research

**Researched:** 2026-02-16
**Domain:** Playwright E2E testing for Next.js 16 App Router with mocked API responses
**Confidence:** HIGH

## Summary

Phase 27 adds Playwright E2E tests for three critical user journeys: free scan flow, paid audit flow, and error recovery. Tests run against a production build (`npm run build && npm run start`) using Playwright route interception to mock all API responses — no real backend required.

The most significant architectural challenge is that the application mixes request types. The scan progress page (`/scan/[id]`) makes **client-side fetches** that Playwright `page.route()` can intercept directly. The results page (`/results/[token]`) makes **server-side fetches** in an async Server Component, which `page.route()` cannot intercept — these require the Next.js `experimental.testProxy` feature to reach the server process. The ScanForm uses a **Server Action** that calls the backend; the action itself cannot be mocked by `page.route()`, but the outbound HTTP call from the server to the backend can be intercepted via `next.onFetch()` with `experimental.testProxy`.

The correct approach: enable `experimental.testProxy: true` in `next.config.ts`, import `defineConfig` from `next/experimental/testmode/playwright`, and use `next.onFetch()` in tests to intercept all server-side requests. Client-side requests from the scan progress page still use `page.route()`. This is the official Next.js-maintained mechanism (README last updated Feb 6, 2025).

**Primary recommendation:** Use `next/experimental/testmode/playwright` for the config + `next.onFetch()` for server-side mocking, combined with `page.route()` for client-side mocking. Place all E2E fixtures in `frontend/e2e/fixtures/` (separate from MSW component test fixtures). Use `webServer` pointing to `npm run start` (the production `next start` server).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Backend approach
- Mock all API responses using Playwright route interception — no real backend needed
- Separate E2E-specific fixtures (not shared with MSW component test fixtures)
- Short delays (100-500ms) on mocked responses to simulate real timing and catch race conditions
- Mock responses must match real API exactly — correct status codes, headers, and response body shapes

#### Stripe payment boundary
- UpgradeCTA click test: intercept the redirect and verify it targets a Stripe Checkout URL pattern
- Success return: navigate directly to /payment/success?session_id=mock_123, verify success page renders
- Cancel return: navigate to cancel/failure return URL, verify UI handles cancellation gracefully
- After payment success: navigate to results page and verify paid-tier content appears (PDF link, deeper findings)

#### Error scenario coverage
- Invalid URL: test both client-side form validation AND server-rejection of unreachable domains
- 404 missing scan: test via direct URL navigation AND via a previously-valid scan link becoming invalid
- Network timeout: simulate API not responding, verify timeout/connection error message
- Server 500: API returns 500, verify error boundary or error message displays
- Recovery: all error tests verify the user can retry or navigate away successfully (not just error display)

#### Scan progress flow
- Verify scan starts (progress UI shown) and completes (results page) — skip verifying intermediate stage transitions
- Results page: full content verification — grade, severity badges, finding details, and UpgradeCTA for free tier
- CFAA consent: test that submitting without consent fails, then check it and submit successfully
- Email input: tested as part of the main scan submission flow, no separate validation test

### Claude's Discretion
- Playwright configuration details (browsers, viewport sizes, timeouts)
- Exact delay values for mocked responses
- Test file organization and naming conventions
- How to structure route interception helpers

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| E2E-01 | Free scan flow E2E test: home page → URL/email submission → scan progress page (polling) → results page with grade and findings | `next/experimental/testmode/playwright` for server action mocking; `page.route()` for client-side polling; fixture polling sequence using stateful counter |
| E2E-02 | Paid audit flow E2E test: UpgradeCTA click → Stripe Checkout redirect → return to payment success page | `page.route()` intercepts `/api/v1/checkout`; `page.waitForURL()` verifies redirect target pattern; direct navigation to `/payment/success?session_id=mock_123` |
| E2E-03 | Error flow E2E tests: invalid URL submission, scan not found (404), API error states | `next.onFetch()` returns 500/404; `page.route().abort()` for timeout simulation; verify recovery navigation |
| E2E-04 | Playwright configured to run against production build (`npm run build && npm run start`), not dev server | `webServer.command: 'npm run start'`; `reuseExistingServer: !process.env.CI`; `experimental.testProxy: true` in next.config.ts |
| E2E-05 | Stripe Test Mode configured with test API keys and documented test card numbers | Test Stripe keys in `.env.test.local` or `.env.e2e`; Stripe Checkout is not automatable — test only up to redirect URL pattern |
</phase_requirements>

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| @playwright/test | ^1.58.2 | E2E test runner, browser automation | Official cross-browser E2E framework; built-in route interception, assertions, parallelism |
| next (testProxy) | 16.x (built-in) | Server-side fetch interception via proxy | Official Next.js mechanism to intercept server-side fetches including Server Actions |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @types/node | ^20 | TypeScript types | Already in project; needed for playwright.config.ts |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `next/experimental/testmode/playwright` | `playwright-ssr` (community) | testProxy is officially maintained by Next.js team; playwright-ssr is a third-party package with lower maintenance guarantees |
| `page.route()` only | HAR file replay | HAR files need recording from a real server; route interception is more flexible for controlled scenarios |
| Separate test server | `webServer` in playwright.config.ts | `webServer` automates server lifecycle; separate server requires manual management |

**Installation:**
```bash
cd frontend && npm install -D @playwright/test
npx playwright install --with-deps chromium
```

## Architecture Patterns

### Recommended Project Structure

```
frontend/
├── e2e/
│   ├── fixtures/            # E2E-specific API response fixtures (separate from MSW)
│   │   ├── scan.ts          # Scan API response shapes
│   │   ├── results.ts       # Results API response shapes
│   │   └── checkout.ts      # Checkout API response shapes
│   ├── helpers/
│   │   ├── route-mocks.ts   # Reusable page.route() interceptors
│   │   └── fetch-mocks.ts   # Reusable next.onFetch() interceptors
│   ├── free-scan.spec.ts    # E2E-01: Free scan flow
│   ├── paid-audit.spec.ts   # E2E-02: Paid audit flow
│   └── error-flows.spec.ts  # E2E-03: Error scenarios
├── playwright.config.ts
└── next.config.ts           # Add experimental.testProxy: true
```

### Pattern 1: Production Build webServer

**What:** Playwright starts Next.js in production mode via `webServer`, reusing an existing server locally but always starting fresh on CI.

**When to use:** Required per E2E-04. Production builds behave differently from dev (no hot reload, minification, static optimization active).

**Example:**
```typescript
// Source: https://playwright.dev/docs/test-webserver
// playwright.config.ts
import { defineConfig } from 'next/experimental/testmode/playwright'

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,      // Sequential helps with server reuse
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,                // Single worker to avoid port conflicts
  reporter: process.env.CI ? 'dot' : 'html',
  use: {
    baseURL: 'http://localhost:3000',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { viewport: { width: 1280, height: 720 } },
    },
  ],
  webServer: {
    command: 'npm run start',
    cwd: process.cwd(),      // frontend/ directory
    url: 'http://localhost:3000',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,     // Production builds need time to start
    stdout: 'ignore',
    stderr: 'pipe',
  },
})
```

### Pattern 2: Server-Side Fetch Mocking with next.onFetch()

**What:** Intercepts `fetch()` calls made by Server Components, Server Actions, and other server-side Next.js code. Requires `experimental.testProxy: true` in `next.config.ts`.

**When to use:** Results page (`/results/[token]`) is an async Server Component that fetches from the backend on the server. The scan form's Server Action (`submitScan`) also calls the backend server-side. `page.route()` cannot intercept these.

**Critical note from official docs:** `next.onFetch` only intercepts external `fetch` requests. If a client fetches a relative URL (e.g. `/api/hello`) handled by a Next.js route handler, it won't be intercepted by `next.onFetch`.

**Example:**
```typescript
// Source: https://github.com/vercel/next.js/blob/canary/packages/next/src/experimental/testmode/playwright/README.md
import { test, expect } from 'next/experimental/testmode/playwright'
import { resultsFixtures } from './fixtures/results'

test('results page shows grade and findings', async ({ page, next }) => {
  // Intercept the server-side fetch to the backend
  next.onFetch((request) => {
    if (request.url.includes('/api/v1/results/')) {
      return new Response(
        JSON.stringify(resultsFixtures.gradeB),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }
      )
    }
    return 'abort'
  })

  await page.goto('/results/tok_abc123def456')
  await expect(page.locator('[data-testid="grade"]')).toContainText('B')
})
```

### Pattern 3: Client-Side Route Interception with page.route()

**What:** Intercepts fetch/XHR requests made by the browser. Works for the scan progress page which polls `/api/v1/scans/:id` using `NEXT_PUBLIC_BACKEND_URL`.

**When to use:** Any fetch made from client components (`'use client'`) — specifically `/scan/[id]/page.tsx` and `UpgradeCTA`.

**Example:**
```typescript
// Source: https://playwright.dev/docs/mock
import { test, expect } from 'next/experimental/testmode/playwright'
import { scanFixtures } from './fixtures/scan'

// Stateful mock for polling: returns in_progress first, then completed
test('scan progress polls and redirects to results', async ({ page, next }) => {
  let pollCount = 0

  // Mock the server action's backend call (server-side)
  next.onFetch((request) => {
    if (request.url.includes('/api/v1/scans') && request.method === 'POST') {
      return new Response(
        JSON.stringify(scanFixtures.created),
        { status: 201, headers: { 'Content-Type': 'application/json' } }
      )
    }
    return 'abort'
  })

  // Mock the client-side polling (client-side, intercepted by page.route)
  await page.route('**/api/v1/scans/**', async (route) => {
    await new Promise(r => setTimeout(r, 200)) // 200ms delay per locked decision
    pollCount++
    if (pollCount < 3) {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(scanFixtures.inProgress),
      })
    } else {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(scanFixtures.completed),
      })
    }
  })

  await page.goto('/')
  // fill and submit the form...
})
```

### Pattern 4: Simulating Response Delays

**What:** Add artificial delays to route handlers to simulate real API timing and catch race conditions.

**When to use:** All mock responses — locked decision requires 100-500ms delays.

**Example:**
```typescript
// Source: https://playwright.dev/docs/network
await page.route('**/api/v1/scans/**', async (route) => {
  await new Promise(r => setTimeout(r, 200)) // Simulate 200ms backend latency
  await route.fulfill({ ... })
})
```

### Pattern 5: Simulating Timeout / Network Failure

**What:** Use `route.abort()` to simulate a connection failure/timeout. Do not use `setTimeout` for actual timeout simulation — abort immediately and let the app's own timeout mechanism trigger.

**When to use:** Error flow tests for "network timeout" scenario.

**Example:**
```typescript
// Source: https://playwright.dev/docs/network#abort-requests
await page.route('**/api/v1/scans/**', async (route) => {
  await route.abort('failed')
})
// Then verify the app shows connection error message
```

### Anti-Patterns to Avoid

- **Using `page.route()` to mock server-side fetches:** This only intercepts browser-originated requests. Results page and Server Actions run on the Next.js server process — they are invisible to `page.route()` without `testProxy`.
- **Running against dev server:** Dev server has hot-reload and different behavior than production. E2E-04 explicitly requires production build.
- **Sharing MSW fixtures with E2E fixtures:** The phase context explicitly requires separate E2E fixtures. MSW fixtures (in `__tests__/helpers/fixtures/`) are for unit/integration tests.
- **Using `workers > 1` with `reuseExistingServer: true`:** Multiple workers trying to use the same server port causes flaky failures.
- **Not adding response delays:** Tests without delays won't catch race conditions in polling logic.
- **Using `test.only` in committed tests:** `forbidOnly: !!process.env.CI` guards this in CI.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Server-side fetch interception | Custom proxy server or env var hacks | `experimental.testProxy` + `next.onFetch()` | Official solution maintained by Next.js team; works with production builds |
| Polling simulation | Complex timer manipulation | Stateful `page.route()` counter | Simple counter returning different responses on successive calls is sufficient |
| Stripe payment testing | Attempting to automate Stripe Checkout UI | Test up to redirect URL pattern only | Stripe Checkout is a 3rd-party URL — Playwright cannot reach it; this is documented as a known blocker in STATE.md |
| Browser setup | Manual browser download scripts | `npx playwright install --with-deps` | Handles platform-specific browser binaries and system dependencies |

**Key insight:** The `next/experimental/testmode/playwright` package eliminates the need for complex proxy setups or env var tricks. Import `defineConfig` from it and gain `next.onFetch()` automatically.

## Common Pitfalls

### Pitfall 1: Server Actions Cannot Be Intercepted by page.route()

**What goes wrong:** You set up `page.route('**/api/v1/scans', ...)` to mock the scan creation, but the Server Action (`submitScan` in `app/actions/scan.ts`) still calls the real backend because it runs in the Next.js server process, not the browser.

**Why it happens:** `page.route()` intercepts network traffic from the browser process. Server Actions execute on the Next.js server process — a different OS process entirely.

**How to avoid:** Use `next.onFetch()` (from `next/experimental/testmode/playwright`) to intercept server-process fetches. Also requires `experimental.testProxy: true` in `next.config.ts`.

**Warning signs:** Route intercept is set up but form submission behavior is inconsistent or the scan creation mock never fires.

### Pitfall 2: Results Page Server-Side Fetch Not Intercepted

**What goes wrong:** The `/results/[token]` page fetches data in an async Server Component. `page.route()` returns mock data for browser requests, but the initial HTML render shows a 404 or error state because the server-side fetch hit a non-existent backend.

**Why it happens:** Same root cause as Pitfall 1 — server component fetches happen on the Next.js server process.

**How to avoid:** Use `next.onFetch()` for the results page token fetch. Pattern the URL match against `request.url.includes('/api/v1/results/')`.

**Warning signs:** Navigation to `/results/some-token` shows 404 or "scan not found" even though you set up a `page.route()` mock.

### Pitfall 3: Production Build Requires Different Port Start Sequence

**What goes wrong:** `webServer.command: 'npm run start'` fails because no build exists yet. Or the production build listens on a different port.

**Why it happens:** `next start` requires a completed `next build`. The standalone output (`output: 'standalone'` in next.config.ts) means `npm run start` runs `next start`, not `node .next/standalone/server.js`.

**How to avoid:** Run `npm run build` before running E2E tests. For CI, add a build step before the playwright test step. The `webServer.command` should only be `npm run start` (which runs `next start` in the project's package.json).

**Warning signs:** "Could not find a production build in the '.next' directory" error when Playwright starts the webServer.

### Pitfall 4: Polling Mock Returns Same Response Forever

**What goes wrong:** The scan progress page polls every 2 seconds. If the mock always returns `inProgress`, the test waits forever for the redirect to results.

**Why it happens:** Stateless route handler returns the same fixture on every call.

**How to avoid:** Use a closure-scoped counter in the route handler. Return `inProgress` for the first N calls, then return `completed` with a `results_token`.

**Warning signs:** Test times out; the scan progress spinner never stops spinning.

### Pitfall 5: Stripe Redirect Verification Approach

**What goes wrong:** Test tries to `page.goto()` to `checkout.stripe.com` to verify the redirect worked, or waits for Stripe UI to load.

**Why it happens:** Misunderstanding the testing boundary. Stripe Checkout lives on Stripe's infrastructure.

**How to avoid:** Use `page.waitForURL()` with a pattern to verify the redirect target, or intercept the `/api/v1/checkout` call and assert the response contains a Stripe URL. Then use `page.route()` to prevent the actual navigation to Stripe and redirect to the success page instead.

**Example approach:**
```typescript
// Intercept checkout API to return mock Stripe URL
await page.route('**/api/v1/checkout', async (route) => {
  await route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ checkout_url: 'https://checkout.stripe.com/c/pay/cs_test_mock123' }),
  })
})

// Intercept the Stripe redirect itself and redirect to success page
await page.route('https://checkout.stripe.com/**', async (route) => {
  await route.fulfill({ status: 302, headers: { location: 'http://localhost:3000/payment/success?session_id=mock_123' } })
})
```

**Warning signs:** Test hangs at Stripe URL or fails with CORS/navigation errors on checkout.stripe.com.

### Pitfall 6: next.onFetch Only Intercepts External Fetches

**What goes wrong:** You use `next.onFetch()` to try to intercept a call like `fetch('/api/v1/scans')` (relative URL to a Next.js API route), but it doesn't get intercepted.

**Why it happens:** Official docs state: `next.onFetch` only intercepts external `fetch` requests. Relative URLs handled by Next.js route handlers are not intercepted.

**How to avoid:** The backend in this project is the Rust API running at `BACKEND_URL` or `NEXT_PUBLIC_BACKEND_URL` — an absolute URL (e.g., `http://localhost:3000` in tests). Since all backend calls use the `BACKEND_URL` env var with an absolute URL, `next.onFetch()` will intercept them correctly.

**Warning signs:** `next.onFetch` handler never fires for calls you expected it to catch.

## Code Examples

Verified patterns from official sources:

### Playwright Config with testProxy and Production Build

```typescript
// Source: https://github.com/vercel/next.js/blob/canary/packages/next/src/experimental/testmode/playwright/README.md
// frontend/playwright.config.ts
import { defineConfig } from 'next/experimental/testmode/playwright'

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: process.env.CI ? 'dot' : 'html',
  use: {
    baseURL: 'http://localhost:3000',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { viewport: { width: 1280, height: 720 } },
    },
  ],
  webServer: {
    command: 'npm run start',
    url: 'http://localhost:3000',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
    stdout: 'ignore',
    stderr: 'pipe',
  },
})
```

### next.config.ts Change Required

```typescript
// Source: https://github.com/vercel/next.js/blob/canary/packages/next/src/experimental/testmode/playwright/README.md
// frontend/next.config.ts — add experimental.testProxy
import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: 'standalone',
  experimental: {
    testProxy: process.env.PLAYWRIGHT_TEST === '1',  // Only enable during E2E tests
  },
};

export default nextConfig;
```

**Note:** `testProxy` can be conditionally enabled via env var to avoid production overhead. The webServer config passes `env: { PLAYWRIGHT_TEST: '1' }` to activate it.

### Route Fulfillment with Delay

```typescript
// Source: https://playwright.dev/docs/mock + decision: 100-500ms delays
await page.route('**/api/v1/scans/*', async (route) => {
  await new Promise(r => setTimeout(r, 200)) // 200ms delay
  await route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify(scanFixtures.inProgress),
  })
})
```

### Stateful Polling Simulation

```typescript
// Pattern: stateful counter for polling progression
test('scan progresses from in_progress to completed', async ({ page, next }) => {
  let pollCount = 0

  await page.route('**/api/v1/scans/*', async (route) => {
    await new Promise(r => setTimeout(r, 150))
    pollCount++
    const fixture = pollCount >= 2 ? scanFixtures.completed : scanFixtures.inProgress
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(fixture),
    })
  })

  await page.goto(`/scan/${scanFixtures.created.id}`)
  await expect(page.locator('text=Scan Complete!')).toBeVisible({ timeout: 15000 })
  // scan page auto-redirects to results after 1s when completed
  await page.waitForURL('**/results/**', { timeout: 10000 })
})
```

### Server-Side Fetch Mock for Results Page

```typescript
// Source: https://github.com/vercel/next.js/blob/canary/packages/next/src/experimental/testmode/playwright/README.md
import { test, expect } from 'next/experimental/testmode/playwright'
import { resultsFixtures } from './fixtures/results'
import { scanFixtures } from './fixtures/scan'

test('results page shows grade, findings, and UpgradeCTA', async ({ page, next }) => {
  const token = scanFixtures.completed.results_token

  next.onFetch((request) => {
    if (request.url.includes(`/api/v1/results/${token}`)) {
      return new Response(
        JSON.stringify(resultsFixtures.gradeA),
        { status: 200, headers: { 'Content-Type': 'application/json' } }
      )
    }
    return 'abort'
  })

  await page.goto(`/results/${token}`)
  await expect(page.locator('h1')).toContainText('Security Scan Results')
  await expect(page.locator('text=A')).toBeVisible() // Grade
})
```

### Form Submission with CFAA Consent Check

```typescript
// Pattern derived from ScanForm implementation in components/scan-form.tsx
test('form requires authorization checkbox before submitting', async ({ page }) => {
  await page.goto('/')

  // Fill form without checking authorization
  await page.fill('input[name="url"]', 'https://example.com')
  await page.fill('input[name="email"]', 'test@example.com')
  await page.click('button[type="submit"]')

  // Form should show validation error for authorization
  await expect(page.locator('text=You must confirm you have authorization')).toBeVisible()

  // Now check the authorization checkbox and resubmit
  await page.check('input[name="authorization"]')
  await page.click('button[type="submit"]')
  // ... verify scan starts
})
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual proxy server for SSR mocking | `next/experimental/testmode/playwright` | Next.js 15 (2024), updated Feb 2025 | Eliminates need for custom proxy infrastructure |
| Dev server for E2E tests | Production build (`next build && next start`) | Best practice documented 2024+ | More realistic test environment |
| MSW in E2E tests (browser worker) | Playwright `page.route()` + `next.onFetch()` | 2024 (MSW+testProxy bug noted) | Native Playwright interception more reliable for E2E; MSW remains for unit tests |

**Deprecated/outdated:**
- Running E2E against `npm run dev`: Not recommended; dev server differs from production.
- Using `page.route()` alone for Next.js SSR apps: Insufficient — misses server-side fetches.

## Open Questions

1. **testProxy stability with production builds**
   - What we know: `experimental.testProxy` README was updated Feb 6, 2025; works with dev server examples
   - What's unclear: Whether production builds (`next start`) fully support `testProxy`; all documented examples use `npm run dev` in webServer command
   - Recommendation: Test `next start` + `testProxy` in the first plan task. If unstable, fall back to: (a) conditionally enabling testProxy only with dev server, or (b) accepting that results page tests navigate directly with server running against real backend pointing to mock data via env vars.

2. **Conditional testProxy activation**
   - What we know: `testProxy` adds overhead to Next.js request handling
   - What's unclear: Whether to enable it unconditionally or via env var check
   - Recommendation: Use `experimental: { testProxy: process.env.PLAYWRIGHT_TEST === '1' }` and pass this env var via `webServer.env` in playwright config to avoid enabling in production builds deployed to Docker.

3. **CI E2E workflow**
   - What we know: The existing `build-push.yml` runs on push to main; no test step exists
   - What's unclear: Whether E2E should block the build-push workflow or run separately
   - Recommendation: Add a separate `e2e.yml` workflow or add a `test-frontend` job before `build-frontend` in `build-push.yml`. Must run `npm run build` before `npm test:e2e`.

## Sources

### Primary (HIGH confidence)
- `https://playwright.dev/docs/mock` — Mock APIs official docs (Playwright stable, Docusaurus v3.8.1)
- `https://playwright.dev/docs/network` — Network interception, `route.abort()`, glob patterns
- `https://playwright.dev/docs/test-configuration` — `webServer`, `baseURL`, `forbidOnly`, `retries`
- `https://playwright.dev/docs/test-webserver` — Production webServer setup, `reuseExistingServer`
- `https://playwright.dev/docs/intro` — Version 1.58.2 confirmed; install instructions
- `https://github.com/vercel/next.js/blob/canary/packages/next/src/experimental/testmode/playwright/README.md` — `testProxy`, `next.onFetch()`, MSW integration (updated Feb 6, 2025)
- `https://nextjs.org/docs/pages/guides/testing/playwright` — Official Next.js + Playwright guide (updated Feb 11, 2026)

### Secondary (MEDIUM confidence)
- `https://www.melkstam.com/blog/next-proxy-playwright` — Practical `testProxy` setup with code examples (Aug 2024); notes MSW+testProxy bug
- `https://maxschmitt.me/posts/nextjs-ssr-request-mocking-playwright` — SSR mocking patterns (Mar 2024)

### Tertiary (LOW confidence)
- `https://github.com/vercel/next.js/discussions/67136` — Server Action mocking discussion (Jun 2024); unresolved as of Jan 2025 — confirms `page.route()` cannot intercept Server Actions

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — Playwright 1.58.2 confirmed on npm; `next/experimental/testmode` documented in official Next.js canary README
- Architecture: HIGH — `page.route()` vs `next.onFetch()` distinction confirmed by official docs and multiple sources
- Pitfalls: HIGH — Server Action interception limitation confirmed by GitHub discussion; polling pattern is standard JavaScript
- testProxy with production build: MEDIUM — Examples show dev server; production build compatibility not explicitly confirmed

**Research date:** 2026-02-16
**Valid until:** 2026-03-16 (Playwright updates frequently; `experimental.testProxy` may graduate from experimental)
