# Architecture Research: Frontend Testing Infrastructure

**Domain:** Frontend Testing for Next.js 16 App Router Application
**Researched:** 2026-02-16
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Next.js 16 Application                   │
├─────────────────────────────────────────────────────────────┤
│  App Router (frontend/app/)                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                   │
│  │  Pages   │  │  Actions │  │  Layout  │                   │
│  │  (7)     │  │  (1)     │  │          │                   │
│  └─────┬────┘  └─────┬────┘  └────┬─────┘                   │
│        │             │            │                          │
├────────┴─────────────┴────────────┴──────────────────────────┤
│  Components Layer (frontend/components/)                     │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
│  │  Scan   │  │ Results │  │  Grade  │  │  Form   │         │
│  │  Form   │  │Dashboard│  │ Summary │  │Elements │         │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘         │
│       │            │            │            │               │
├───────┴────────────┴────────────┴────────────┴───────────────┤
│  Services Layer (frontend/lib/)                              │
│  ┌─────────────────────────────────────────────────────┐     │
│  │  api.ts (Backend API client)                        │     │
│  │  types.ts (TypeScript definitions)                  │     │
│  └─────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
         ↓                              ↑
    [Backend API]                  [Test Mocks]
         ↓                              ↑
```

### Testing Layers Integration

```
┌─────────────────────────────────────────────────────────────┐
│                  E2E Testing Layer (Playwright)              │
│  Tests full user workflows through browser automation        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                   │
│  │ Scan Flow│  │Results   │  │Navigation│                   │
│  │   E2E    │  │  Flow    │  │   Tests  │                   │
│  └─────┬────┘  └─────┬────┘  └────┬─────┘                   │
│        │             │            │                          │
│        └─────────────┴────────────┘                          │
│                      ↓                                       │
├─────────────────────────────────────────────────────────────┤
│            Component Testing Layer (Vitest + RTL)            │
│  Tests individual components and user interactions           │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
│  │Component│  │Component│  │Component│  │Component│         │
│  │  Tests  │  │  Tests  │  │  Tests  │  │  Tests  │         │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘         │
│       │            │            │            │               │
│       └────────────┴────────────┴────────────┘               │
│                      ↓                                       │
├─────────────────────────────────────────────────────────────┤
│         Unit Testing Layer (Vitest)                          │
│  Tests pure functions, utilities, and server actions         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                      │
│  │  Utils  │  │ Server  │  │   API   │                      │
│  │  Tests  │  │ Actions │  │  Client │                      │
│  └─────────┘  └─────────┘  └─────────┘                      │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| **Vitest** | Unit & component test runner | jsdom environment, React Testing Library integration |
| **React Testing Library** | Component rendering & user interaction testing | render(), screen queries, fireEvent/userEvent |
| **Playwright** | E2E browser automation | Chromium/Firefox/WebKit, page navigation, assertions |
| **MSW (Mock Service Worker)** | Network request mocking | Intercepts fetch calls, provides mock responses |
| **@vitest/coverage-v8** | Code coverage measurement | AST-based coverage, lcov/html reports |
| **@testing-library/user-event** | User interaction simulation | Realistic event firing (click, type, keyboard) |

## Recommended Project Structure

```
frontend/
├── app/                      # Next.js App Router
│   ├── page.tsx             # Pages
│   ├── actions/             # Server actions
│   │   ├── scan.ts
│   │   └── __tests__/       # Server action tests
│   │       └── scan.test.ts
│   └── api/                 # API routes (if any)
│       └── __tests__/       # API route tests
├── components/              # React components
│   ├── scan-form.tsx
│   ├── results-dashboard.tsx
│   └── __tests__/           # Component tests
│       ├── scan-form.test.tsx
│       ├── results-dashboard.test.tsx
│       └── grade-summary.test.tsx
├── lib/                     # Utilities and services
│   ├── api.ts
│   ├── types.ts
│   └── __tests__/           # Utility tests
│       └── api.test.ts
├── e2e/                     # Playwright E2E tests (NEW)
│   ├── scan-flow.spec.ts
│   ├── results-flow.spec.ts
│   └── fixtures/            # Test fixtures and helpers
│       └── test-helpers.ts
├── test-utils/              # Shared test utilities (NEW)
│   ├── setup.ts             # Global test setup
│   ├── mocks/               # Mock factories
│   │   ├── handlers.ts      # MSW handlers
│   │   └── server.ts        # MSW server setup
│   └── custom-render.tsx    # Custom RTL render with providers
├── vitest.config.ts         # Vitest configuration (NEW)
├── playwright.config.ts     # Playwright configuration (NEW)
└── package.json             # Updated with test scripts
```

### Structure Rationale

- **`__tests__/` colocated with source:** Keeps tests close to implementation, easier navigation, clear what's tested
- **`e2e/` separate directory:** E2E tests are cross-component workflows, don't belong to single component
- **`test-utils/` shared utilities:** Reusable test setup (MSW, custom renders) prevents duplication
- **Server action tests in app/actions/__tests__/:** Tests async server actions in Node environment
- **Component tests in components/__tests__/:** Tests client components in jsdom environment

## Architectural Patterns

### Pattern 1: Layered Testing Strategy

**What:** Different test types for different system layers - unit tests for logic, component tests for UI, E2E for workflows.

**When to use:** All Next.js App Router applications with both Server and Client Components.

**Trade-offs:**
- **Pros:** Clear test boundaries, faster feedback loop, comprehensive coverage
- **Cons:** More test infrastructure, requires discipline to avoid overlap

**Example:**
```typescript
// Unit test for server action (lib/api.test.ts)
// Environment: Node
import { describe, it, expect, vi } from 'vitest'
import { createScan } from '../api'

describe('createScan', () => {
  it('should call backend API with correct data', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ id: 'scan-123' })
    })

    const result = await createScan('https://example.com', 'test@test.com')
    expect(result.id).toBe('scan-123')
  })
})

// Component test (components/__tests__/scan-form.test.tsx)
// Environment: jsdom
import { describe, it, expect } from 'vitest'
import { render, screen, userEvent } from '@testing-library/react'
import ScanForm from '../scan-form'

describe('ScanForm', () => {
  it('validates URL input', async () => {
    render(<ScanForm />)
    const input = screen.getByLabelText(/URL/i)
    await userEvent.type(input, 'invalid-url')
    await userEvent.click(screen.getByRole('button', { name: /scan/i }))

    expect(screen.getByText(/valid URL/i)).toBeInTheDocument()
  })
})

// E2E test (e2e/scan-flow.spec.ts)
// Environment: Real browser
import { test, expect } from '@playwright/test'

test('complete scan submission flow', async ({ page }) => {
  await page.goto('/')
  await page.fill('[name="url"]', 'https://example.com')
  await page.fill('[name="email"]', 'test@test.com')
  await page.check('[name="authorization"]')
  await page.click('button[type="submit"]')

  await expect(page).toHaveURL(/\/scan\//)
  await expect(page.locator('h1')).toContainText('Scanning')
})
```

### Pattern 2: MSW for API Mocking

**What:** Mock Service Worker intercepts network requests at the network level, working consistently across tests and development.

**When to use:** Testing components/pages that fetch data, testing error states, avoiding backend dependency in tests.

**Trade-offs:**
- **Pros:** Network-level mocking (works in tests and browser), realistic request/response cycle, type-safe with TypeScript
- **Cons:** Extra setup complexity, need to maintain mock handlers alongside backend changes

**Example:**
```typescript
// test-utils/mocks/handlers.ts
import { http, HttpResponse } from 'msw'

export const handlers = [
  http.post('/api/v1/scans', async ({ request }) => {
    const data = await request.json()
    return HttpResponse.json({
      id: 'scan-123',
      url: data.url,
      status: 'pending'
    })
  }),

  http.get('/api/v1/scans/:id', ({ params }) => {
    return HttpResponse.json({
      id: params.id,
      status: 'completed',
      grade: 'A',
      findings: []
    })
  })
]

// test-utils/mocks/server.ts
import { setupServer } from 'msw/node'
import { handlers } from './handlers'

export const server = setupServer(...handlers)

// test-utils/setup.ts
import { beforeAll, afterEach, afterAll } from 'vitest'
import { server } from './mocks/server'

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }))
afterEach(() => server.resetHandlers())
afterAll(() => server.close())

// Usage in test
import { server } from '@/test-utils/mocks/server'
import { http, HttpResponse } from 'msw'

it('handles API error', async () => {
  server.use(
    http.post('/api/v1/scans', () => {
      return HttpResponse.json({ detail: 'Rate limit exceeded' }, { status: 429 })
    })
  )

  // Test error handling
})
```

### Pattern 3: Custom Render for Component Tests

**What:** Wrapper around React Testing Library's render that includes necessary providers (future: state, theme, etc.).

**When to use:** When components require context providers, when testing components with common setup needs.

**Trade-offs:**
- **Pros:** Consistent test setup, reduces boilerplate, easier to add global providers
- **Cons:** Hides some complexity, can make tests less explicit

**Example:**
```typescript
// test-utils/custom-render.tsx
import { render, RenderOptions } from '@testing-library/react'
import { ReactElement, ReactNode } from 'react'

interface AllTheProvidersProps {
  children: ReactNode
}

function AllTheProviders({ children }: AllTheProvidersProps) {
  return (
    <>
      {/* Add providers as needed */}
      {/* <ThemeProvider> */}
      {/* <StateProvider> */}
      {children}
      {/* </StateProvider> */}
      {/* </ThemeProvider> */}
    </>
  )
}

export function customRender(
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) {
  return render(ui, { wrapper: AllTheProviders, ...options })
}

// Re-export everything
export * from '@testing-library/react'
export { customRender as render }

// Usage in tests
import { render, screen } from '@/test-utils/custom-render'
```

### Pattern 4: Parallel Test Execution in CI

**What:** Run unit/component tests and E2E tests as separate, parallel jobs in GitHub Actions.

**When to use:** Always - speeds up CI feedback loop significantly.

**Trade-offs:**
- **Pros:** Faster CI runs, independent failure isolation, better resource utilization
- **Cons:** More complex workflow file, need to manage multiple job outputs

**Example:**
```yaml
# .github/workflows/test.yml
name: Test

on:
  pull_request:
  push:
    branches: [main]

jobs:
  unit-and-component-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5

      - uses: actions/setup-node@v5
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Install dependencies
        working-directory: frontend
        run: npm ci

      - name: Run unit and component tests
        working-directory: frontend
        run: npm run test:unit -- --coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v5
        with:
          files: ./frontend/coverage/coverage-final.json

  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5

      - uses: actions/setup-node@v5
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Install dependencies
        working-directory: frontend
        run: npm ci

      - name: Cache Playwright browsers
        uses: actions/cache@v4
        with:
          path: ~/.cache/ms-playwright
          key: playwright-${{ hashFiles('frontend/package-lock.json') }}

      - name: Install Playwright browsers
        working-directory: frontend
        run: npx playwright install --with-deps

      - name: Run E2E tests
        working-directory: frontend
        run: npm run test:e2e

      - name: Upload Playwright report
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-report
          path: frontend/playwright-report/
          retention-days: 30
```

### Pattern 5: Testing Server Actions in Node Environment

**What:** Server actions with 'use server' directive run only on the server, requiring Node environment for testing.

**When to use:** Testing any server action (app/actions/*.ts files).

**Trade-offs:**
- **Pros:** Tests run in correct environment, can test async operations, validates actual behavior
- **Cons:** Cannot test Server Components directly (use E2E instead)

**Example:**
```typescript
// app/actions/__tests__/scan.test.ts
/**
 * @vitest-environment node
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { submitScan } from '../scan'

describe('submitScan server action', () => {
  beforeEach(() => {
    vi.unstubAllGlobals()
  })

  it('validates form data', async () => {
    const formData = new FormData()
    formData.append('url', 'invalid-url')
    formData.append('email', 'test@test.com')
    formData.append('authorization', 'on')

    const result = await submitScan({}, formData)

    expect(result.errors?.url).toBeDefined()
    expect(result.scanId).toBeUndefined()
  })

  it('returns scan ID on success', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ id: 'scan-123' })
    })

    const formData = new FormData()
    formData.append('url', 'https://example.com')
    formData.append('email', 'test@test.com')
    formData.append('authorization', 'on')

    const result = await submitScan({}, formData)

    expect(result.scanId).toBe('scan-123')
    expect(result.errors).toBeUndefined()
  })

  it('handles 429 rate limit error', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 429,
      json: async () => ({ detail: 'Rate limit exceeded' })
    })

    const formData = new FormData()
    formData.append('url', 'https://example.com')
    formData.append('email', 'test@test.com')
    formData.append('authorization', 'on')

    const result = await submitScan({}, formData)

    expect(result.errors?._form?.[0]).toContain('maximum number of scans')
  })
})
```

## Data Flow

### Test Execution Flow

```
Developer → npm run test
    ↓
Vitest detects changes
    ↓
┌─────────────────────────────────────────┐
│  1. Run setup.ts (MSW server.listen())  │
└─────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────┐
│  2. Execute test files in parallel      │
│     - Unit tests (Node env)             │
│     - Component tests (jsdom env)       │
└─────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────┐
│  3. Collect coverage via v8             │
└─────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────┐
│  4. Generate coverage reports           │
│     - lcov/coverage-final.json          │
│     - HTML report                       │
└─────────────────────────────────────────┘
    ↓
Results displayed + watch mode continues
```

### E2E Test Flow

```
Developer → npm run test:e2e
    ↓
Playwright reads config
    ↓
┌─────────────────────────────────────────┐
│  1. Start Next.js server (webServer)    │
│     - npm run build && npm run start    │
└─────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────┐
│  2. Launch browsers (Chromium/Firefox)  │
└─────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────┐
│  3. Execute spec files in parallel      │
│     - Navigate pages                    │
│     - Interact with elements            │
│     - Assert expectations               │
└─────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────┐
│  4. Capture artifacts (on failure)      │
│     - Screenshots                       │
│     - Videos                            │
│     - Traces                            │
└─────────────────────────────────────────┘
    ↓
Results + HTML report
```

### CI/CD Integration Flow

```
Git Push → GitHub Actions Trigger
    ↓
┌──────────────────────────────────────────────────────────────┐
│                     Parallel Job Execution                    │
├────────────────────────────┬─────────────────────────────────┤
│  Job: unit-component-tests │  Job: e2e-tests                 │
│  1. Checkout code          │  1. Checkout code               │
│  2. Setup Node 20          │  2. Setup Node 20               │
│  3. Cache npm deps         │  3. Cache npm deps              │
│  4. npm ci                 │  4. npm ci                      │
│  5. Run Vitest w/coverage  │  5. Cache Playwright browsers   │
│  6. Upload coverage        │  6. Install Playwright browsers │
│                            │  7. Run Playwright tests        │
│                            │  8. Upload artifacts (failures) │
└────────────────────────────┴─────────────────────────────────┘
    ↓                              ↓
Both jobs pass → Continue pipeline (build-backend, build-frontend, deploy)
Any job fails → Block deployment
```

### Key Data Flows

1. **Component Test Flow:** Component → render() → jsdom → user interactions → assertions
2. **Server Action Test Flow:** FormData → server action → fetch (mocked) → validation → result
3. **E2E Flow:** Browser → Next.js app → user actions → DOM changes → visual/state assertions
4. **Mock Flow:** fetch() → MSW intercept → handler → mock response → test assertions

## Integration Points

### New Components Required

| Component | Type | Purpose |
|-----------|------|---------|
| `vitest.config.ts` | Config | Vitest test runner configuration |
| `playwright.config.ts` | Config | Playwright E2E test configuration |
| `test-utils/setup.ts` | Setup | Global test setup (MSW, cleanup) |
| `test-utils/mocks/handlers.ts` | Mock | MSW request handlers |
| `test-utils/mocks/server.ts` | Mock | MSW server instance |
| `test-utils/custom-render.tsx` | Utility | Custom RTL render with providers |
| `.github/workflows/test.yml` | CI/CD | Test workflow (NEW job) |

### Modified Components

| Component | Modification | Reason |
|-----------|-------------|--------|
| `package.json` | Add test scripts, dev dependencies | Enable test commands |
| `.github/workflows/build-push.yml` | Add test job before build jobs | Test before deployment |
| `.gitignore` | Add coverage/, playwright-report/ | Exclude test artifacts |

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| Backend API | MSW mocking in tests, real calls in E2E | E2E may need backend running or test environment |
| GitHub Actions | Parallel test jobs, coverage upload | Use ubuntu-latest for speed |
| Codecov (optional) | Upload coverage reports | Requires CODECOV_TOKEN secret |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Components ↔ Server Actions | Import and call, tested via mocking | Component tests mock actions, E2E tests full flow |
| Components ↔ API Client | Import and call, MSW intercepts | Unit test api.ts, component tests use MSW |
| E2E Tests ↔ Next.js App | HTTP/browser automation | Requires app running on localhost |
| Unit Tests ↔ Code | Direct imports | Fast, isolated tests |

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 1-10 components | Colocated `__tests__/` directories, single test file per component |
| 10-50 components | Group tests by feature area, add test utilities for common patterns |
| 50+ components | Consider visual regression testing (Chromatic/Percy), component library testing strategy |

### Testing Performance Optimization

1. **First bottleneck (slow test runs):**
   - **Symptom:** Tests take > 30 seconds
   - **Fix:** Use `test.concurrent` for independent tests, limit E2E tests to critical paths

2. **Second bottleneck (flaky E2E tests):**
   - **Symptom:** Intermittent failures in Playwright
   - **Fix:** Add proper waitFor conditions, use Playwright auto-waiting, avoid hard-coded timeouts

3. **Third bottleneck (CI time):**
   - **Symptom:** CI takes > 5 minutes
   - **Fix:** Parallelize tests across multiple workers, cache dependencies aggressively, run E2E only on main branch

## Anti-Patterns

### Anti-Pattern 1: Testing Implementation Details

**What people do:** Test component internal state, private methods, or exact DOM structure.

**Why it's wrong:** Tests break on refactoring even when behavior unchanged, leads to brittle tests.

**Do this instead:** Test user-visible behavior and outcomes. Use queries that match how users find elements (ByRole, ByLabelText, not ByTestId).

```typescript
// BAD
expect(component.state.isLoading).toBe(true)
expect(wrapper.find('.spinner')).toHaveLength(1)

// GOOD
expect(screen.getByRole('status')).toHaveTextContent('Loading')
```

### Anti-Pattern 2: Mixing Test Environments

**What people do:** Try to test Server Components with jsdom environment, or run component tests in Node environment.

**Why it's wrong:** Tests fail with cryptic errors, or worse, pass but don't reflect reality.

**Do this instead:** Use correct environment (Node for server actions, jsdom for client components, E2E for async Server Components).

```typescript
// BAD - trying to test Server Component with Vitest
import { render } from '@testing-library/react'
import ServerComponent from './server-component' // async Server Component

test('renders', () => {
  render(<ServerComponent />) // FAILS - async components not supported
})

// GOOD - use E2E for Server Components
test('renders server component', async ({ page }) => {
  await page.goto('/page-with-server-component')
  await expect(page.getByRole('heading')).toContainText('Expected content')
})
```

### Anti-Pattern 3: Not Resetting Mocks Between Tests

**What people do:** Set up MSW handlers once, let them persist across tests.

**Why it's wrong:** Tests become interdependent, order-dependent failures, hard to debug.

**Do this instead:** Reset handlers in `afterEach`, use `server.use()` for test-specific overrides.

```typescript
// BAD
beforeAll(() => {
  server.use(
    http.get('/api/scans', () => HttpResponse.json({ data: [] }))
  )
})

// GOOD
import { server } from '@/test-utils/mocks/server'

beforeAll(() => server.listen())
afterEach(() => server.resetHandlers()) // Reset to default handlers
afterAll(() => server.close())

test('specific behavior', () => {
  server.use(
    http.get('/api/scans', () => HttpResponse.json({ data: [] }))
  )
  // Test runs with override, gets reset after
})
```

### Anti-Pattern 4: Over-Mocking in E2E Tests

**What people do:** Mock API calls in Playwright tests.

**Why it's wrong:** Defeats purpose of E2E testing - you're not testing the real integration.

**Do this instead:** Run E2E against real backend (test environment), or use test database. Only mock external services (Stripe, email providers).

```typescript
// BAD - mocking backend in E2E
test('scan flow', async ({ page }) => {
  await page.route('**/api/v1/scans', route => {
    route.fulfill({ body: JSON.stringify({ id: '123' }) })
  })
  // Not testing real backend integration
})

// GOOD - real backend call
test('scan flow', async ({ page }) => {
  // Assumes backend test environment is running
  await page.goto('/')
  await page.fill('[name="url"]', 'https://example.com')
  // ... real flow with real backend
})
```

### Anti-Pattern 5: No Coverage Thresholds

**What people do:** Generate coverage reports but don't enforce minimums.

**Why it's wrong:** Coverage slowly decreases, new code added without tests.

**Do this instead:** Set coverage thresholds in vitest.config.ts, fail CI if below threshold.

```typescript
// vitest.config.ts
export default defineConfig({
  test: {
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov'],
      thresholds: {
        lines: 80,
        functions: 80,
        branches: 75,
        statements: 80
      }
    }
  }
})
```

## Build Order and Dependencies

### Recommended Implementation Sequence

**Phase 1: Foundation Setup (Day 1)**
1. Install Vitest dependencies
2. Create vitest.config.ts
3. Add test script to package.json
4. Create test-utils/setup.ts (minimal)
5. Write first component test (simplest component)
6. Verify test runs successfully

**Phase 2: Component Testing Infrastructure (Day 2)**
1. Install MSW
2. Create test-utils/mocks/handlers.ts
3. Create test-utils/mocks/server.ts
4. Update test-utils/setup.ts with MSW
5. Create test-utils/custom-render.tsx
6. Write tests for API-dependent component
7. Add coverage configuration

**Phase 3: Server Action Testing (Day 3)**
1. Create app/actions/__tests__/
2. Write tests for submitScan action
3. Test validation logic
4. Test error handling
5. Test success cases

**Phase 4: E2E Testing Setup (Day 4)**
1. Install Playwright
2. Create playwright.config.ts
3. Create e2e/ directory
4. Write first E2E test (homepage navigation)
5. Add test:e2e script

**Phase 5: Critical Path E2E Tests (Day 5)**
1. Write scan submission E2E test
2. Write results viewing E2E test
3. Test error states
4. Verify on different browsers

**Phase 6: CI/CD Integration (Day 6)**
1. Create .github/workflows/test.yml
2. Add unit/component test job
3. Add E2E test job
4. Configure caching
5. Add coverage upload
6. Update build-push.yml to depend on tests

**Phase 7: Coverage and Quality Gates (Day 7)**
1. Set coverage thresholds
2. Generate baseline coverage report
3. Add coverage badge to README
4. Document testing patterns
5. Create PR testing guidelines

### Dependency Graph

```
vitest.config.ts
    ↓
test-utils/setup.ts
    ↓
test-utils/mocks/server.ts ← test-utils/mocks/handlers.ts
    ↓
test-utils/custom-render.tsx
    ↓
Component tests can begin
    ↓
playwright.config.ts (independent)
    ↓
E2E tests can begin
    ↓
.github/workflows/test.yml (depends on all test types working)
```

## Sources

**Official Next.js Documentation (HIGH confidence):**
- [Testing: Vitest | Next.js](https://nextjs.org/docs/app/guides/testing/vitest) - v16.1.6, updated 2026-02-11
- [Testing: Playwright | Next.js](https://nextjs.org/docs/pages/guides/testing/playwright) - v16.1.6, updated 2026-02-11
- [Guides: Testing | Next.js](https://nextjs.org/docs/app/guides/testing)

**Comprehensive Guides (MEDIUM confidence):**
- [NextJs Unit Testing and End-to-End Testing - Strapi](https://strapi.io/blog/nextjs-testing-guide-unit-and-e2e-tests-with-vitest-and-playwright)
- [Test Strategy in the Next.js App Router Era - Shinagawa Labs](https://shinagawa-web.com/en/blogs/nextjs-app-router-testing-setup)
- [E2E Testing in Next.js with Playwright - Enreina](https://enreina.com/blog/e2e-testing-in-next-js-with-playwright-vercel-and-github-actions-a-guide-with-example/)

**Playwright CI/CD (MEDIUM confidence):**
- [Setting up CI | Playwright](https://playwright.dev/docs/ci-intro)
- [Automating Playwright Tests with GitHub Actions - Andrew Martin](https://medium.com/@andrewmart.in/automating-playwright-tests-with-github-actions-5f9ba3dc06a7)
- [Caching Playwright Binaries in GitHub Actions - Justin Poehnelt](https://justin.poehnelt.com/posts/caching-playwright-in-github-actions/)

**MSW Integration (MEDIUM confidence):**
- [Setting up MSW in a Next.js App Router Project - JGY's Blog](https://www.gimbap.dev/blog/setting-msw-in-next)
- [Setting up MSW and URQL with Next.js 15 - Stackademic](https://blog.stackademic.com/setting-up-msw-and-urql-with-next-js-15-cbfd374e916a)

**Vitest Coverage (MEDIUM confidence):**
- [Coverage | Guide | Vitest](https://vitest.dev/guide/coverage.html)
- [Coverage Config | Vitest](https://vitest.dev/config/coverage)

**Test Organization Patterns (MEDIUM confidence):**
- [Next.js Project Structure](https://nextjs.org/docs/app/getting-started/project-structure)
- [Colocation of Tests: A Cross-Language Perspective - Mario Dias](https://itsmariodias.medium.com/colocation-of-tests-a-cross-language-perspective-982e75c872d8)

---
*Architecture research for: Frontend Testing Infrastructure - Next.js 16 App Router*
*Researched: 2026-02-16*
*Confidence: HIGH - Based on official Next.js 16.1.6 docs and current ecosystem best practices*
