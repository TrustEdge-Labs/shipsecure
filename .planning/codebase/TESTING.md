# Testing Patterns

**Analysis Date:** 2026-02-21

## Test Framework

**Runner:**
- Vitest v4.0.18
- Config: `frontend/vitest.config.ts`

**Assertion Library:**
- Vitest built-in expect() assertions
- `@testing-library/react` for DOM queries and assertions
- `@testing-library/jest-dom` for extended matchers (`.toBeInTheDocument()`, `.toBeChecked()`, etc.)

**Run Commands:**
```bash
npm run test              # Run all tests with coverage and dot reporter
npm run test:ci           # CI mode (no watch, dot reporter)
npm run test:e2e          # Run Playwright E2E tests on Chromium
npm run test:e2e:ui       # Run E2E tests with Playwright UI
```

**Environment:**
- Test environment: `happy-dom` (lightweight DOM implementation)
- Setup file: `frontend/vitest.setup.ts` runs before all tests
- Node env automatically set to `test` by Next.js during Vitest runs

## Test File Organization

**Location:**
- Co-located in `__tests__/` directory mirroring source structure
- Component tests: `__tests__/components/*.test.tsx`
- Integration tests: `__tests__/integration/*.test.tsx`
- Test helpers: `__tests__/helpers/`
- MSW handlers/server: `__tests__/helpers/msw/`
- Fixtures: `__tests__/helpers/fixtures/`

**Naming:**
- Test files: `{ComponentName}.test.tsx` (e.g., `ScanForm.test.tsx`, `ResultsDashboard.test.tsx`)
- Fixtures: descriptive names with `.ts` suffix (e.g., `scan.ts`, `results.ts`)
- Handlers: `handlers.ts`, `server.ts` in MSW directory

**Structure:**
```
frontend/
├── __tests__/
│   ├── components/
│   │   ├── ScanForm.test.tsx
│   │   ├── ResultsDashboard.test.tsx
│   │   ├── Logo.test.tsx
│   │   ├── Header.test.tsx
│   │   ├── error-boundary.test.tsx
│   │   └── ...
│   ├── integration/
│   │   └── scan-status.test.tsx
│   └── helpers/
│       ├── test-utils.tsx
│       ├── msw/
│       │   ├── server.ts
│       │   └── handlers.ts
│       └── fixtures/
│           ├── scan.ts
│           └── results.ts
```

## Test Structure

**Suite Organization:**
```typescript
import { describe, test, expect, beforeEach, afterEach } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import { MyComponent } from '@/components/my-component'

describe('MyComponent', () => {
  describe('Category 1', () => {
    test('specific behavior', () => {
      // Test implementation
    })
  })

  describe('Category 2', () => {
    test('another behavior', () => {
      // Test implementation
    })
  })
})
```

**Patterns from actual codebase:**

Component tests use nested `describe()` blocks for logical grouping:
- `describe('ScanForm')` → `describe('Form Fields')`, `describe('Validation Errors')`, `describe('Loading State')`, etc.
- `describe('ResultsDashboard')` → `describe('Empty State')`, `describe('Findings Rendering')`, `describe('Grouping Toggle')`
- `describe('Logo')` → `describe('Size Variants')`, `describe('Common Attributes')`

Test naming uses `test()` function and describes specific behavior:
```typescript
test('renders URL input with label "Website URL"', () => { ... })
test('displays email validation error when present in state', () => { ... })
test('user can type in URL field', async () => { ... })
```

**Setup/Teardown:**

Global setup in `frontend/vitest.setup.ts`:
```typescript
import { beforeAll, afterEach, afterAll, vi } from 'vitest'
import { cleanup } from '@testing-library/react'
import { server } from './__tests__/helpers/msw/server'

beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }))
afterEach(() => {
  cleanup()
  server.resetHandlers()
})
afterAll(() => server.close())
```

Per-test setup:
```typescript
beforeEach(() => {
  mockState = {}
  mockFormAction = vi.fn()
  mockPending = false
})

afterEach(() => {
  vi.restoreAllMocks()
})
```

**Assertion Patterns:**

DOM presence assertions:
```typescript
expect(screen.getByText('Loading...')).toBeInTheDocument()
expect(urlInput).toBeInTheDocument()
```

Attribute/property assertions:
```typescript
expect(urlInput).toHaveAttribute('type', 'url')
expect(urlInput).toHaveAttribute('name', 'url')
expect(button).toBeDisabled()
expect(checkbox).toBeChecked()
```

Text/content assertions:
```typescript
expect(screen.getByTestId('status')).toHaveTextContent('completed')
expect(screen.getByRole('alert')).toHaveTextContent('error message')
```

Function call verification:
```typescript
expect(mockReset).toHaveBeenCalledOnce()
expect(mockFormAction).toHaveBeenCalled()
```

## Mocking

**Framework:** Vitest `vi` module for spying and mocking

**Patterns from codebase:**

Global mocks in `frontend/vitest.setup.ts`:
```typescript
vi.mock('next/navigation', () => ({
  useRouter: vi.fn(() => ({ push: vi.fn(), ... })),
  usePathname: vi.fn(() => '/'),
  useSearchParams: vi.fn(() => new URLSearchParams()),
}))

vi.mock('next/image', () => ({
  default: ({ src, alt, ...props }) => {
    return React.createElement('img', { src, alt, ...props })
  },
}))

vi.mock('@clerk/nextjs', () => ({
  useClerk: vi.fn(() => ({ openSignUp: vi.fn() })),
  useUser: vi.fn(() => ({ isSignedIn: false, user: null })),
  useAuth: vi.fn(() => ({ isSignedIn: false, userId: null })),
  SignedIn: ({ children }) => null,
  SignedOut: ({ children }) => children,
  UserButton: () => null,
  ClerkProvider: ({ children }) => children,
}))
```

Local mocks in component tests:
```typescript
vi.mock('react', async () => {
  const actual = await vi.importActual('react')
  return {
    ...actual,
    useActionState: vi.fn(() => [mockState, mockFormAction, mockPending])
  }
})
```

Mock spy setup:
```typescript
beforeEach(() => {
  vi.spyOn(console, 'error').mockImplementation(() => {})
})
```

**What to Mock:**
- Next.js hooks (`useRouter`, `usePathname`, `useSearchParams`)
- External authentication libraries (`@clerk/nextjs`)
- Image components (Next.js `Image` doesn't work in happy-dom)
- React hooks when testing state behavior (e.g., `useActionState`)
- Console methods during testing to avoid output pollution

**What NOT to Mock:**
- Testing library hooks (`screen`, `render`, etc.)
- Core React functionality unless specifically testing hook behavior
- User event interactions (use `@testing-library/user-event` instead)
- API handlers (use MSW instead)

## Fixtures and Factories

**Test Data:**

Fixtures stored in `__tests__/helpers/fixtures/`:

Example from `scan.ts`:
```typescript
export const scanFixtures = {
  created: {
    id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    status: 'pending',
    url: '/api/v1/scans/a1b2c3d4-e5f6-7890-abcd-ef1234567890',
  },

  inProgress: {
    id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    target_url: 'https://example-vibe-app.vercel.app',
    status: 'in_progress',
    // ... more fields
  },

  completed: {
    id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    // ... full data with findings
  },

  failed: {
    id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    status: 'failed',
    // ... error data
  },
} as const
```

Usage in tests:
```typescript
server.use(
  http.get(`${BACKEND_URL}/api/v1/scans/:id`, () => {
    return HttpResponse.json(scanFixtures.completed)
  })
)
```

Component test fixtures (inline):
```typescript
const testFindings: Finding[] = [
  {
    id: '1',
    title: 'Missing CSP header',
    severity: 'high',
    // ... more fields
  },
  // ... more findings
]
```

**Location:**
- Reusable fixtures: `__tests__/helpers/fixtures/`
- Component-specific test data: Defined inline in test file

## Coverage

**Requirements:**
- Lines: 80%
- Functions: 80%
- Branches: 75%
- Configured in `frontend/vitest.config.ts`

**Included in Coverage:**
- `components/**` only (targeted coverage)

**Excluded from Coverage:**
- `node_modules/`
- `__tests__/`
- Config files (`*.config.*`)
- Next.js built-ins (`.next/`, layouts, loading, error, globals, special files)
- Specific v1.6 components awaiting coverage:
  - `components/domain-badge.tsx`
  - `components/meta-tag-snippet.tsx`
  - `components/scan-history-table.tsx`

**View Coverage:**
```bash
npm run test              # Generates coverage report in coverage/ directory
```

Coverage reporters: text, html, lcov (for CI tools)

## Test Types

**Unit Tests:**
- Scope: Individual components with isolated behavior
- Approach: Test component rendering, user interactions, prop handling, state changes
- Location: `__tests__/components/*.test.tsx`
- Examples: `ScanForm.test.tsx` tests form field rendering, validation errors, loading states, user typing
- Use `renderWithProviders()` from test-utils to wrap components with required providers

**Integration Tests:**
- Scope: Component behavior with simulated API calls via MSW
- Approach: Render component, trigger actions, verify API calls and state updates
- Location: `__tests__/integration/*.test.tsx`
- Example: `scan-status.test.tsx` tests fetching scan status, displaying data, error handling
- Use MSW handlers (`server.use()`) to override default API responses
- Verify data flows through component correctly with real async patterns

**E2E Tests:**
- Framework: Playwright v1.58.2
- Command: `npm run test:e2e` (Chromium only)
- Location: Not visible in provided files (typically `e2e/` or `tests/` directory)
- Scope: Full user workflows in real browser

## Common Patterns

**Async Testing:**

Using `waitFor()` to wait for async operations:
```typescript
test('fetches and displays scan status from MSW handler', async () => {
  renderWithProviders(<ScanStatusTestComponent scanId="test-scan-123" />)

  // Initially shows loading
  expect(screen.getByText('Loading...')).toBeInTheDocument()

  // Wait for async fetch to complete
  await waitFor(() => {
    expect(screen.getByTestId('scan-status')).toHaveTextContent('in_progress')
  })
})
```

User interactions with async behavior:
```typescript
test('user can type in URL field', async () => {
  const user = userEvent.setup()
  renderWithProviders(<ScanForm />)

  const urlInput = screen.getByLabelText(/website url/i)
  await user.type(urlInput, 'https://example.com')

  expect(urlInput).toHaveValue('https://example.com')
})
```

**Error Testing:**

Testing error states returned from server actions:
```typescript
test('displays URL validation error when present in state', () => {
  mockState = { errors: { url: ['Please enter a valid URL'] } }
  renderWithProviders(<ScanForm />)

  expect(screen.getByText(/please enter a valid url/i)).toBeInTheDocument()
})
```

Testing API error responses with MSW:
```typescript
test('displays error when scan API returns 404', async () => {
  server.use(errorHandlers.scanNotFound)

  renderWithProviders(<ScanStatusTestComponent scanId="nonexistent" />)

  await waitFor(() => {
    expect(screen.getByRole('alert')).toHaveTextContent('Scan not found')
  })
})
```

Pre-built error handlers in `__tests__/helpers/msw/handlers.ts`:
```typescript
export const errorHandlers = {
  scanNotFound: http.get(`${BASE_URL}/api/v1/scans/:id`, () => {
    return HttpResponse.json({...}, { status: 404 })
  }),
  scanServerError: http.get(`${BASE_URL}/api/v1/scans/:id`, () => {
    return HttpResponse.json({...}, { status: 500 })
  }),
  // ... more error handlers
}
```

**Query Selection:**

Always prefer semantic queries from testing-library:
```typescript
screen.getByRole('button', { name: /scan now/i })        // Role-based
screen.getByLabelText(/website url/i)                     // Label association
screen.getByText(/something went wrong/i)                 // Visible text
screen.getByTestId('scan-status')                          // Last resort
screen.getByAltText('ShipSecure')                         // Image alt text
```

Avoid: `screen.getByClassName()`, direct DOM manipulation

---

*Testing analysis: 2026-02-21*
