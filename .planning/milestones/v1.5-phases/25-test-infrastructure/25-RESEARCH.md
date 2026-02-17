# Phase 25: Test Infrastructure - Research

**Researched:** 2026-02-16
**Domain:** Vitest + MSW + React Testing Library for Next.js 16 App Router testing
**Confidence:** HIGH

## Summary

Phase 25 establishes a modern testing stack for Next.js 16 (App Router) using Vitest, Mock Service Worker (MSW) v2, and React Testing Library. The official Next.js documentation (updated 2026-02-11) recommends Vitest over Jest for better ESM support and faster execution. This stack enables component unit tests and integration tests with API mocking while staying aligned with Next.js's architecture.

Key decisions: happy-dom provides faster DOM simulation than jsdom with acceptable API coverage for most Next.js components. MSW v2's Fetch API-first design aligns perfectly with Next.js Server Actions and modern request handling. The `@next/env` package ensures test environment variables load identically to production.

Critical limitations: Vitest does not support async Server Components (Next.js App Router feature). E2E tests with Playwright (Phase 27) handle async RSC scenarios. Path alias resolution requires `vite-tsconfig-paths` plugin to honor Next.js's `@/*` imports.

**Primary recommendation:** Use the official Next.js `with-vitest` example as the foundation, extend with MSW for API mocking, create centralized test utilities in `__tests__/helpers/`, and configure coverage with v8 provider for performance.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **Test file organization**: Tests live in a separate `frontend/__tests__/` directory, NOT colocated with components
- **Internal structure mirrors `src/`**: e.g., `__tests__/components/Header.test.tsx` for `src/components/Header.tsx`
- **Naming convention**: `.test.tsx` (Vitest default)
- **Test utilities location**: Custom render, MSW handlers, fixtures live in `__tests__/helpers/`
- **Mock data strategy**: Realistic fixtures with full API response shapes; shared fixture files in `__tests__/helpers/fixtures/` as single source of truth
- **All API endpoints covered upfront**: scan, results, checkout, webhook handlers all created in Phase 25
- **Error scenarios included from the start**: success + error variants (500s, timeouts, 404s) for each endpoint
- **First passing test**: Header component (navigation links, CTA, logo rendering) with basic assertions
- **MSW integration test**: Include one test that exercises scan status fetch to prove data fetching pipeline works
- **Two proof points**: Component rendering works (Header) AND API mocking works (scan status fetch)
- **Test runner UX**: `npm test` runs in watch mode with coverage by default; minimal (dots) output
- **Scripts**: `test` (watch + coverage), `test:e2e` (Playwright, Phase 27), `test:ci` (single-run for CI)

### Claude's Discretion
- Vitest configuration details and plugin setup
- Path alias resolution approach
- MSW server setup/teardown mechanics
- Custom RTL render wrapper implementation
- Coverage threshold values (Phase 28 enforces, but Phase 25 configures)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| INFRA-01 | Vitest configured with happy-dom environment, React plugin, and TypeScript path alias resolution for `@/*` imports | Vitest + happy-dom (faster than jsdom); `@vitejs/plugin-react` for JSX; `vite-tsconfig-paths` for `@/*` aliases |
| INFRA-02 | MSW (Mock Service Worker) configured with reusable API handlers for scan, results, checkout, and webhook endpoints | MSW v2 with `setupServer` for Node.js; `http.get`/`http.post` handlers; lifecycle in `vitest.setup.ts` |
| INFRA-03 | Custom RTL render wrapper with provider support for consistent component test setup | RTL's `wrapper` option; custom `renderWithProviders` in `__tests__/helpers/test-utils.tsx` |
| INFRA-04 | Environment variable loading from `.env.test` via `@next/env` in Vitest config | `loadEnvConfig(process.cwd())` in `vitest.config.ts`; `.env.test` (NODE_ENV=test skips `.env.local`) |
| INFRA-05 | `next/navigation` hooks (useRouter, usePathname, useSearchParams) mocked globally for component tests | `vi.mock('next/navigation')` in `vitest.setup.ts` with mock implementations |
| INFRA-06 | Test scripts added to package.json (`test`, `test:unit`, `test:e2e`, `test:coverage`) | Scripts configuration with `vitest` CLI flags; `--watch`, `--coverage`, `--run` modes |
</phase_requirements>

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| vitest | ^4.0.18 | Test runner and assertion library | Next.js official recommendation; faster than Jest with better ESM support |
| @vitejs/plugin-react | ^5.1.4 | React JSX transformation for Vitest | Required for React component testing in Vite ecosystem |
| happy-dom | latest | Lightweight DOM environment | 2-3x faster than jsdom; sufficient API coverage for Next.js components |
| @testing-library/react | ^16.3.2 | Component testing utilities | Industry standard for user-centric testing; React 19 compatible (v16.1.0+) |
| @testing-library/dom | latest | DOM query utilities (RTL dependency) | Peer dependency for @testing-library/react |
| msw | ^2.12.10 | API mocking via request interception | MSW v2 uses Fetch API (matches Next.js Server Actions); network-level mocking vs function mocks |
| vite-tsconfig-paths | latest | TypeScript path alias resolution | Honors `tsconfig.json` paths (`@/*`) for Vite/Vitest |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @next/env | latest | Next.js environment variable loader | Load `.env.test` in Vitest with identical behavior to Next.js runtime |
| @testing-library/user-event | latest | User interaction simulation | More realistic than `fireEvent` for click, type, hover events |
| @vitest/ui | latest | Web-based test UI | Optional; useful for debugging tests in browser |
| @vitest/coverage-v8 | latest | V8 coverage provider (default) | Faster than Istanbul; sufficient for most projects |
| eslint-plugin-testing-library | latest | ESLint rules for RTL best practices | Catches common anti-patterns (query misuse, waitFor mistakes) |
| eslint-plugin-vitest | latest | ESLint rules for Vitest | ~70 rules for test code quality |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| happy-dom | jsdom | jsdom is more complete (fuller browser API) but 2-3x slower; choose if using obscure DOM APIs |
| Vitest | Jest | Jest has larger ecosystem but poor ESM support; Next.js recommends Vitest in 2026 docs |
| MSW | Manual fetch mocking | MSW provides network-level mocking reusable across tests, dev, Storybook vs brittle per-test mocks |
| v8 coverage | istanbul coverage | Istanbul supports more reporters but slower; v8 sufficient for standard HTML/lcov reports |

**Installation:**
```bash
npm install -D vitest @vitejs/plugin-react happy-dom @testing-library/react @testing-library/dom @testing-library/user-event msw vite-tsconfig-paths eslint-plugin-testing-library eslint-plugin-vitest
```

## Architecture Patterns

### Recommended Project Structure
```
frontend/
├── __tests__/
│   ├── components/          # Component tests mirroring src structure
│   │   ├── Header.test.tsx
│   │   ├── Footer.test.tsx
│   │   └── Logo.test.tsx
│   ├── helpers/             # Test utilities
│   │   ├── test-utils.tsx   # Custom render with providers
│   │   ├── fixtures/        # Shared test data
│   │   │   ├── scan.ts      # Scan API fixtures
│   │   │   ├── results.ts   # Results API fixtures
│   │   │   └── checkout.ts  # Checkout API fixtures
│   │   └── msw/
│   │       ├── handlers.ts  # MSW request handlers (happy paths)
│   │       └── server.ts    # MSW setupServer instance
│   └── app/                 # Page/route tests (optional)
├── vitest.config.ts         # Vitest configuration
├── vitest.setup.ts          # Global test setup (MSW, mocks)
└── .env.test                # Test environment variables
```

### Pattern 1: Vitest Configuration with Next.js

**What:** Configure Vitest to resolve `@/*` imports, use happy-dom, and load Next.js environment variables

**When to use:** Base configuration for all Next.js + Vitest projects

**Example:**
```typescript
// vitest.config.ts
// Source: https://nextjs.org/docs/app/guides/testing/vitest
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import tsconfigPaths from 'vite-tsconfig-paths'
import { loadEnvConfig } from '@next/env'

// Load Next.js environment variables
loadEnvConfig(process.cwd())

export default defineConfig({
  plugins: [
    tsconfigPaths(), // Resolve @/* imports from tsconfig.json
    react(),         // React JSX transformation
  ],
  test: {
    environment: 'happy-dom',
    setupFiles: ['./vitest.setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov'],
      exclude: [
        'node_modules/',
        '__tests__/',
        '*.config.*',
        '.next/',
      ],
    },
  },
})
```

### Pattern 2: MSW Server Lifecycle Management

**What:** Initialize MSW server before tests, reset handlers between tests, close after all tests

**When to use:** Required for all projects using MSW for API mocking

**Example:**
```typescript
// vitest.setup.ts
// Source: https://mswjs.io/docs/integrations/node/
import { beforeAll, afterEach, afterAll } from 'vitest'
import { server } from './__tests__/helpers/msw/server'

// Enable API mocking before all tests
beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }))

// Reset handlers to initial state between tests (critical for test isolation)
afterEach(() => server.resetHandlers())

// Restore native modules after all tests
afterAll(() => server.close())
```

```typescript
// __tests__/helpers/msw/server.ts
import { setupServer } from 'msw/node'
import { handlers } from './handlers'

export const server = setupServer(...handlers)
```

```typescript
// __tests__/helpers/msw/handlers.ts
// Source: https://mswjs.io/docs/http/intercepting-requests/
import { http, HttpResponse } from 'msw'
import { scanFixture } from '../fixtures/scan'

export const handlers = [
  // Scan status endpoint (happy path)
  http.get('/api/scan/:scanId', ({ params }) => {
    return HttpResponse.json(scanFixture.inProgress)
  }),

  // Results endpoint (happy path)
  http.get('/api/results/:scanId', ({ params }) => {
    return HttpResponse.json(resultsFixture.gradeA)
  }),

  // Error scenarios (use server.use() in tests for overrides)
]
```

### Pattern 3: Custom Render with Providers

**What:** Reusable render function that wraps components in necessary providers (future: theme, context)

**When to use:** Every component test that renders React components

**Example:**
```typescript
// __tests__/helpers/test-utils.tsx
// Source: https://testing-library.com/docs/react-testing-library/api/
import { render, RenderOptions } from '@testing-library/react'
import { ReactElement, ReactNode } from 'react'

interface AllTheProvidersProps {
  children: ReactNode
}

// Wrapper component with all providers
function AllTheProviders({ children }: AllTheProvidersProps) {
  return (
    <>
      {/* Add providers here as needed (e.g., ThemeProvider, QueryClientProvider) */}
      {children}
    </>
  )
}

// Custom render that wraps in providers
export function renderWithProviders(
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) {
  return render(ui, { wrapper: AllTheProviders, ...options })
}

// Re-export everything from RTL
export * from '@testing-library/react'
```

**Usage in tests:**
```typescript
import { renderWithProviders, screen } from '@/__tests__/helpers/test-utils'
import Header from '@/components/Header'

test('Header renders logo and navigation', () => {
  renderWithProviders(<Header />)
  expect(screen.getByRole('banner')).toBeInTheDocument()
})
```

### Pattern 4: Mocking Next.js Navigation Hooks

**What:** Global mocks for `next/navigation` hooks (useRouter, usePathname, useSearchParams)

**When to use:** Components using Next.js App Router navigation hooks

**Example:**
```typescript
// vitest.setup.ts (add to existing file)
import { vi } from 'vitest'

// Mock next/navigation hooks globally
// Source: https://github.com/vercel/next.js/discussions/48937
vi.mock('next/navigation', () => ({
  useRouter: vi.fn(() => ({
    push: vi.fn(),
    replace: vi.fn(),
    prefetch: vi.fn(),
    back: vi.fn(),
    forward: vi.fn(),
  })),
  usePathname: vi.fn(() => '/'),
  useSearchParams: vi.fn(() => new URLSearchParams()),
}))
```

### Pattern 5: Shared Fixtures with TypeScript

**What:** Type-safe, reusable test data for API responses

**When to use:** All tests that need mock API data

**Example:**
```typescript
// __tests__/helpers/fixtures/scan.ts
export const scanFixture = {
  inProgress: {
    scanId: 'scan_test123',
    status: 'in_progress',
    currentStage: 'scanning',
    progress: 50,
  },
  completed: {
    scanId: 'scan_test123',
    status: 'completed',
    currentStage: 'complete',
    progress: 100,
  },
  error: {
    scanId: 'scan_test123',
    status: 'failed',
    error: 'Invalid URL',
  },
} as const
```

### Pattern 6: Runtime MSW Handler Overrides

**What:** Override MSW handlers in individual tests for error scenarios

**When to use:** Testing error states, timeouts, edge cases

**Example:**
```typescript
// Source: https://mswjs.io/docs/best-practices/network-behavior-overrides/
import { server } from '@/__tests__/helpers/msw/server'
import { http, HttpResponse } from 'msw'

test('displays error when scan API fails', async () => {
  // Override handler for this test only
  server.use(
    http.get('/api/scan/:scanId', () => {
      return HttpResponse.json({ error: 'Server error' }, { status: 500 })
    })
  )

  // Test error handling
  renderWithProviders(<ScanStatus scanId="test123" />)
  expect(await screen.findByText(/error/i)).toBeInTheDocument()
})
```

### Anti-Patterns to Avoid

- **Using `queryBy*` for existence checks**: Use `getBy*` instead; better error messages when element not found
- **waitFor with side effects**: Only put assertions in waitFor callback; side effects run multiple times
- **Not using find* queries**: `await screen.findByRole()` is simpler than `await waitFor(() => screen.getByRole())`
- **Testing implementation details**: Test what users see (via roles, labels), not class names or internal state
- **Forgetting resetHandlers**: MSW handlers accumulate without `afterEach(() => server.resetHandlers())`
- **Using restoreHandlers for cleanup**: Use `resetHandlers()` (wipes runtime handlers) not `restoreHandlers()` (marks one-time handlers unused)

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| API mocking in tests | Function-level mocks with `vi.fn()` for every fetch call | MSW request handlers | Network-level mocking works across tests, Storybook, dev; handles request matching, response types, CORS |
| Custom render boilerplate | Copy-paste provider setup in every test file | Custom `renderWithProviders` utility | Single source of truth for provider setup; easier to add global context later |
| Test data generation | Inline objects in every test | Shared fixtures in `__tests__/helpers/fixtures/` | Consistent API shapes; update once when backend changes; realistic data |
| Environment variable loading | Manual `process.env` assignments in tests | `@next/env` loadEnvConfig | Matches Next.js production behavior (reads `.env`, `.env.local`, `.env.test` in correct order) |
| Path alias resolution | Relative imports (`../../components`) in tests | `vite-tsconfig-paths` plugin | Honors `tsconfig.json` paths; tests import same way as source (`@/components`) |

**Key insight:** Modern test infrastructure relies on tool composition (Vitest + MSW + RTL) rather than custom solutions. Each tool solves one problem well; integration is straightforward via plugins and setup files.

## Common Pitfalls

### Pitfall 1: Async Server Components Not Supported

**What goes wrong:** Vitest tests fail when importing async Server Components (App Router feature)

**Why it happens:** React's async component support is too new; Vitest's rendering layer doesn't handle promises in component trees

**How to avoid:**
- Only test synchronous Client Components and Server Components in Vitest
- Use E2E tests (Playwright, Phase 27) for async RSC scenarios
- Mark async components as out-of-scope for unit tests in documentation

**Warning signs:** Error like "Objects are not valid as a React child (found: [object Promise])"

### Pitfall 2: Path Aliases Not Resolving

**What goes wrong:** `import X from '@/components/X'` throws "Cannot find module" in tests

**Why it happens:** Vite doesn't read `tsconfig.json` baseUrl/paths by default; requires explicit plugin

**How to avoid:** Install `vite-tsconfig-paths` and add to `vitest.config.ts` plugins array

**Warning signs:** Tests fail with module resolution errors despite TypeScript compilation succeeding

### Pitfall 3: MSW Handlers Accumulate Across Tests

**What goes wrong:** Test A passes; Test B (using different handler override) fails because Test A's override still active

**Why it happens:** Calling `server.use()` adds runtime handlers that persist until explicitly reset

**How to avoid:**
- Always call `afterEach(() => server.resetHandlers())` in setup file
- Use `resetHandlers()` not `restoreHandlers()` (wrong tool; for one-time handlers)

**Warning signs:** Tests pass individually but fail when run in suite; order-dependent failures

### Pitfall 4: Environment Variables Not Loaded

**What goes wrong:** `process.env.NEXT_PUBLIC_API_URL` is undefined in tests

**Why it happens:** NODE_ENV=test makes Next.js skip `.env.local`; need `.env.test` file

**How to avoid:**
- Create `.env.test` with test-specific values
- Call `loadEnvConfig(process.cwd())` in `vitest.config.ts`
- Never rely on `.env.local` in tests

**Warning signs:** Tests fail with undefined environment variables; works in dev/prod

### Pitfall 5: Using queryBy for Existence Checks

**What goes wrong:** Tests pass but provide unhelpful errors like "expected null not to be in document"

**Why it happens:** `queryBy*` returns null when not found; doesn't throw like `getBy*`

**How to avoid:**
- Use `getBy*` for existence: `expect(screen.getByRole('button')).toBeInTheDocument()`
- Use `queryBy*` only for non-existence: `expect(screen.queryByRole('alert')).not.toBeInTheDocument()`

**Warning signs:** Test failures show "expected null to..." instead of showing available elements

### Pitfall 6: Act Warnings with Async Updates

**What goes wrong:** Console warning "Warning: An update to Component inside a test was not wrapped in act(...)"

**Why it happens:** Component updates (state, props) happen outside React Testing Library's awareness

**How to avoid:**
- Use `await screen.findBy*()` for async updates (auto-wrapped in act)
- Use `waitFor` from `@testing-library/react` not `vi.waitFor` from vitest
- Use `userEvent` instead of `fireEvent` (handles async properly)

**Warning signs:** Tests pass but console shows act warnings; flaky tests in CI

### Pitfall 7: Network Errors in Tests Not Mocked

**What goes wrong:** Real network requests attempted in tests; tests fail with ECONNREFUSED or hang

**Why it happens:** MSW handler doesn't match request URL/method; falls through to real network

**How to avoid:**
- Configure MSW with `onUnhandledRequest: 'warn'` to catch unmatched requests
- Add handlers for all API endpoints used in components
- Check MSW warnings in test output

**Warning signs:** Tests timeout; console shows "MSW: unhandled request" warnings

## Code Examples

Verified patterns from official sources:

### Basic Component Test with RTL

```typescript
// Source: https://nextjs.org/docs/app/guides/testing/vitest
import { expect, test } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import Header from '@/components/Header'

test('Header renders navigation and CTA', () => {
  renderWithProviders(<Header />)

  // Query by role (accessible to screen readers)
  expect(screen.getByRole('banner')).toBeInTheDocument()
  expect(screen.getByRole('link', { name: /scan now/i })).toBeInTheDocument()
})
```

### MSW Integration Test (API Mocking)

```typescript
// Source: https://mswjs.io/docs/integrations/node/
import { expect, test } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import ScanStatus from '@/components/ScanStatus'
import { scanFixture } from '@/__tests__/helpers/fixtures/scan'

test('fetches and displays scan progress', async () => {
  renderWithProviders(<ScanStatus scanId="test123" />)

  // MSW handler returns scanFixture.inProgress by default
  expect(await screen.findByText(/scanning/i)).toBeInTheDocument()
  expect(screen.getByText(/50%/)).toBeInTheDocument()
})
```

### Error State Test with Handler Override

```typescript
// Source: https://mswjs.io/docs/best-practices/network-behavior-overrides/
import { server } from '@/__tests__/helpers/msw/server'
import { http, HttpResponse } from 'msw'

test('displays error message when API fails', async () => {
  // Override handler for this test
  server.use(
    http.get('/api/scan/:scanId', () => {
      return HttpResponse.json(
        { error: 'Server error' },
        { status: 500 }
      )
    })
  )

  renderWithProviders(<ScanStatus scanId="test123" />)
  expect(await screen.findByText(/error/i)).toBeInTheDocument()
})
```

### User Interaction with userEvent

```typescript
// Source: https://testing-library.com/docs/user-event/intro
import { expect, test } from 'vitest'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import ScanForm from '@/components/ScanForm'

test('submits form with URL', async () => {
  const user = userEvent.setup()
  renderWithProviders(<ScanForm />)

  // Type in input
  const urlInput = screen.getByRole('textbox', { name: /url/i })
  await user.type(urlInput, 'https://example.com')

  // Click submit
  const submitButton = screen.getByRole('button', { name: /scan/i })
  await user.click(submitButton)

  // Assert loading state appears
  expect(await screen.findByText(/scanning/i)).toBeInTheDocument()
})
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Jest | Vitest | 2024 (Next.js docs updated 2026-02) | Better ESM support; 2-5x faster; native TypeScript |
| jsdom (default) | happy-dom | 2024 | 2-3x faster DOM simulation; sufficient API coverage for most components |
| MSW v1 (rest.get) | MSW v2 (http.get) | 2023 (v2.0 release) | Fetch API alignment; HttpResponse instead of res/ctx composition |
| Enzyme | React Testing Library | 2020 | User-centric testing vs implementation details; React 19 compatible |
| Manual mocks | MSW network mocking | 2022 | Reusable across test/dev/Storybook; network-level vs function-level |
| Jest coverage (istanbul) | Vitest coverage (v8) | 2024 | Faster coverage with V8 native instrumentation |

**Deprecated/outdated:**
- **Enzyme**: Unmaintained; incompatible with React 19; use RTL instead
- **Jest for Next.js**: Poor ESM support; Next.js recommends Vitest in official docs (2026-02-11)
- **MSW v1 API**: `rest.get` → `http.get`; `res(ctx.json())` → `HttpResponse.json()`
- **@testing-library/react <16.1.0**: React 19 unsupported; upgrade to ^16.3.2

## Open Questions

1. **happy-dom vs jsdom for Next.js specific features**
   - What we know: happy-dom is faster (2-3x); jsdom more complete API
   - What's unclear: Do Next.js components use DOM APIs missing from happy-dom?
   - Recommendation: Start with happy-dom (user choice, faster); switch to jsdom if encountering missing APIs (warn in verification)

2. **Coverage thresholds for Phase 25**
   - What we know: Phase 28 enforces quality gates; Phase 25 configures infrastructure
   - What's unclear: Should Phase 25 set placeholder thresholds or leave at 0?
   - Recommendation: Set conservative initial thresholds (50% lines/functions, 40% branches) in Phase 25; Phase 28 raises to final values (80%/80%/75%)

3. **React 19.2 compatibility with testing libraries**
   - What we know: @testing-library/react ^16.1.0 supports React 19; project uses React 19.2.3
   - What's unclear: Any breaking changes between React 19.0 and 19.2.x affecting tests?
   - Recommendation: Proceed with @testing-library/react ^16.3.2; verify in first test run; no reported issues for 19.2.x

## Sources

### Primary (HIGH confidence)
- [Next.js Vitest Guide](https://nextjs.org/docs/app/guides/testing/vitest) - Official Next.js 16 documentation (updated 2026-02-11)
- [MSW Node.js Integration](https://mswjs.io/docs/integrations/node/) - Official MSW setup for Vitest
- [React Testing Library API](https://testing-library.com/docs/react-testing-library/api/) - Custom render wrapper pattern
- [Vitest Coverage Config](https://vitest.dev/config/coverage) - Coverage providers, thresholds, reporters
- [MSW Error Responses](https://mswjs.io/docs/http/mocking-responses/error-responses/) - Error state patterns
- [Vitest Reporters](https://vitest.dev/guide/reporters) - Dot reporter for minimal output
- [Kent C. Dodds - Common RTL Mistakes](https://kentcdodds.com/blog/common-mistakes-with-react-testing-library) - Anti-patterns and best practices

### Secondary (MEDIUM confidence)
- [Next.js with-vitest Example](https://github.com/vercel/next.js/tree/canary/examples/with-vitest) - Official example structure
- [Vitest Common Errors](https://vitest.dev/guide/common-errors) - Path alias issues, async RSC limitations
- [MSW resetHandlers vs restoreHandlers](https://mswjs.io/docs/api/setup-server/reset-handlers/) - Lifecycle management
- [GitHub: Mocking next/navigation in Vitest](https://github.com/vercel/next.js/discussions/48937) - Community pattern for useRouter mocks
- [GitHub: @next/env in Vitest](https://github.com/vercel/next.js/discussions/62021) - loadEnvConfig pattern
- [@testing-library/react releases](https://github.com/testing-library/react-testing-library/releases) - React 19 support (v16.1.0)
- [MSW v2 Release](https://mswjs.io/blog/introducing-msw-2.0/) - API changes, Fetch API alignment

### Tertiary (LOW confidence - verify during implementation)
- [happy-dom vs jsdom discussion](https://github.com/vitest-dev/vitest/discussions/1607) - Community performance comparisons (2024 data)
- [Fishery test fixtures](https://medium.com/leaselock-engineering/stepping-up-our-test-fixture-game-with-fishery-be22b76d1f22) - Factory pattern for fixtures (alternative approach)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official Next.js, MSW, RTL documentation; version compatibility verified
- Architecture: HIGH - Patterns from official docs (Next.js, MSW, RTL); established best practices
- Pitfalls: HIGH - Documented in official sources (Vitest, MSW) and authoritative blogs (Kent C. Dodds)

**Research date:** 2026-02-16
**Valid until:** 2026-03-16 (30 days - stable ecosystem, slow-moving changes)

**Next.js version tested:** 16.1.6 (current project version)
**React version tested:** 19.2.3 (current project version)
