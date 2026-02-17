# Technology Stack — Frontend Testing Infrastructure

**Project:** ShipSecure
**Domain:** Frontend testing (unit, component, E2E)
**Researched:** 2026-02-16
**Confidence:** HIGH

## Context

Stack additions for frontend testing on existing Next.js 16 + React 19 application. The application already has:
- Next.js 16.1.6 (App Router), React 19.2.3, TypeScript 5
- Tailwind CSS 4, Zod 4, Lucide React icons
- ESLint 9, @types/node@^20
- 9 components, 5 App Router pages, server actions
- Rust/Axum backend, GitHub Actions CI/CD

## Recommended Stack Additions

### Core Testing Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **vitest** | ^4.0.18 | Unit/component test runner | Built on Vite for fast execution, native ESM support, first-class TypeScript integration. Vitest 4 (released 2026) includes stable Browser Mode and React 19 support. Next.js officially recommends Vitest for unit testing. |
| **@playwright/test** | ^1.58.2 | E2E test framework | Industry standard for E2E testing, supports all browsers (Chromium, Firefox, WebKit), officially recommended by Next.js. Version 1.58 (Jan 2026) includes Playwright Trace support and visual regression testing. |
| **@testing-library/react** | ^16.3.2 | Component testing utilities | Official React 19 recommendation (replaces deprecated react-test-renderer). Encourages testing user behavior over implementation details. Version 16+ fully supports React 19's async rendering. |
| **happy-dom** | ^20.6.1 | DOM environment for Vitest | 2-4x faster than jsdom for component tests. Sufficient API coverage for React Testing Library. Use jsdom only if you hit API limitations. |

### Supporting Testing Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **@testing-library/jest-dom** | ^6.9.1 | Custom DOM matchers | Always. Provides `toBeInTheDocument()`, `toHaveClass()`, etc. Makes assertions more readable. |
| **@testing-library/user-event** | ^14.6.1 | User interaction simulation | Always for component tests. Simulates real browser events (click, type, hover) more accurately than fireEvent. |
| **@vitest/ui** | ^4.0.18 | Web-based test UI | Optional. Provides browser-based test runner UI for development. Enable with `vitest --ui`. |
| **@vitest/coverage-v8** | ^4.0.18 | Code coverage reports | Recommended. Uses V8's native coverage (faster than istanbul). Generates HTML/JSON reports. |

### Build & Configuration Tools

| Tool | Version | Purpose | Notes |
|------|---------|---------|-------|
| **@vitejs/plugin-react** | ^5.1.4 | Vite React plugin | Required for Vitest to transform JSX/TSX. Use this (not @vitejs/plugin-react-swc) for compatibility with testing environment. |
| **vite-tsconfig-paths** | ^6.1.1 | TypeScript path mapping | Resolves Next.js `@/*` import aliases in tests. Version 6 adds on-demand discovery and automatic reloads. |

## What NOT to Add

| Library | Why Avoid | Use Instead |
|---------|-----------|-------------|
| **jest** | Poor ESM support, slow with Next.js 16 App Router, complex configuration for React 19 | vitest (official Next.js recommendation) |
| **react-test-renderer** | Deprecated in React 19 | @testing-library/react |
| **enzyme** | Unmaintained, no React 19 support, encourages testing implementation details | @testing-library/react |
| **jsdom** (as default) | 2-4x slower than happy-dom, unnecessary for most React components | happy-dom (fallback to jsdom only if needed) |
| **@vitejs/plugin-react-swc** | Breaks testing environment, incompatible with Vitest DOM environments | @vitejs/plugin-react |
| **cypress** | More complex setup than Playwright, slower parallelization, harder CI integration | @playwright/test |
| **@vitest/coverage-istanbul** | Slower than V8 coverage, more complex configuration | @vitest/coverage-v8 |

## New Dependencies Summary

```bash
# Core testing dependencies
npm install -D vitest@^4.0.18 \
  @playwright/test@^1.58.2 \
  @testing-library/react@^16.3.2 \
  @testing-library/jest-dom@^6.9.1 \
  @testing-library/user-event@^14.6.1 \
  happy-dom@^20.6.1

# Vitest configuration & plugins
npm install -D @vitejs/plugin-react@^5.1.4 \
  vite-tsconfig-paths@^6.1.1

# Optional but recommended
npm install -D @vitest/ui@^4.0.18 \
  @vitest/coverage-v8@^4.0.18

# Playwright browsers (run after npm install)
npx playwright install
```

**Total new dev dependencies:** 10 core + 2 optional = 12 packages
**Build size impact:** Zero (all dev dependencies, not bundled)

## Integration Points

### Vitest with Next.js 16 App Router

**Key consideration:** Vitest does NOT support async Server Components. This is a React ecosystem limitation, not a Vitest bug.

**Pattern:**
- **Client Components:** Full unit testing with Vitest + RTL ✅
- **Synchronous Server Components:** Unit testing supported ✅
- **Async Server Components:** Use Playwright E2E tests only ⚠️

**Rationale:** Since App Router pushes data fetching to the server, E2E tests become the primary surface area for testing async flows. This is the recommended Next.js approach per official docs (Feb 11, 2026 update).

### Configuration Files Required

| File | Purpose | Critical Settings |
|------|---------|-------------------|
| `vitest.config.ts` | Vitest configuration | `environment: 'happy-dom'`, `globals: true`, `setupFiles: './vitest.setup.ts'`, plugins: [react(), tsconfigPaths()] |
| `vitest.setup.ts` | Test environment setup | Import `@testing-library/jest-dom` for custom matchers |
| `playwright.config.ts` | Playwright configuration | `webServer` (auto-start dev server), `baseURL: 'http://localhost:3000'`, projects for each browser |
| `tsconfig.json` (update) | TypeScript for tests | Add `"vitest/globals"` to `types` array for global test functions |

### Example vitest.config.ts

```typescript
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import tsconfigPaths from 'vite-tsconfig-paths'

export default defineConfig({
  plugins: [react(), tsconfigPaths()],
  test: {
    environment: 'happy-dom',
    globals: true,
    setupFiles: './vitest.setup.ts',
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'json'],
      exclude: ['node_modules/', '.next/', 'dist/'],
    },
  },
})
```

### Example playwright.config.ts

```typescript
import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  use: {
    baseURL: 'http://localhost:3000',
    trace: 'on-first-retry',
  },
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:3000',
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
  projects: [
    { name: 'chromium', use: { browserName: 'chromium' } },
    { name: 'firefox', use: { browserName: 'firefox' } },
    { name: 'webkit', use: { browserName: 'webkit' } },
  ],
})
```

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| vitest@4.0.18 | React 19.2.3 | ✅ Fully compatible. Vitest 4 released with React 19 support. |
| @testing-library/react@16.3.2 | React 19.2.3, react-dom 19.2.3 | ✅ Version 16+ required for React 19. Earlier versions expect React 18. |
| @playwright/test@1.58.2 | Next.js 16.1.6 | ✅ No framework-specific dependencies. Works with any Next.js version. |
| happy-dom@20.6.1 | @testing-library/react@16.3.2 | ✅ Sufficient API coverage for React Testing Library. |
| vite-tsconfig-paths@6.1.1 | Next.js TypeScript config | ✅ Resolves `@/*` aliases automatically. No special configuration needed. |
| @types/node@^20 | Node.js 20+ | ⚠️ Current project uses @types/node@^20. This is fine but consider updating to @types/node@^22 or @types/node@^25 to match modern Node versions (Feb 2026). |

## React 19 Testing Changes

**Important change:** `act()` moved from `react-dom/test-utils` to `react` package.

**No action required for React Testing Library users** — RTL 16.3+ handles this automatically. All async operations are wrapped in `act()` internally.

**If using manual act():** Import from `react` instead of `react-dom/test-utils`:
```typescript
// ❌ Old (React 18)
import { act } from 'react-dom/test-utils'

// ✅ New (React 19)
import { act } from 'react'
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| happy-dom | jsdom | If you need specific browser APIs that happy-dom doesn't implement (rare). Check Vitest error messages first. |
| Vitest | Jest | Never for new Next.js projects. Jest has poor ESM support, slower execution, complex config. Vitest is Next.js recommendation. |
| @playwright/test | Cypress | If you need component testing in real browsers (Playwright now supports this). Otherwise Playwright has better parallelization and CI performance. |
| @vitejs/plugin-react | @vitejs/plugin-react-swc | Never for testing. SWC plugin lacks testing environment compatibility. Use standard plugin for tests. |
| @vitest/coverage-v8 | @vitest/coverage-istanbul | Only if you have existing istanbul tooling. V8 coverage is faster and simpler. |

## Performance Considerations

| Addition | Impact | Notes |
|----------|--------|-------|
| Vitest unit tests | ~50-200ms per test file | Fast due to Vite's transform caching and happy-dom's speed |
| Playwright E2E tests | ~2-5s per test | Includes browser startup, navigation, assertions |
| Test coverage reporting | ~10-20% longer test runs | V8 coverage has minimal overhead |
| happy-dom vs jsdom | 2-4x faster | Measured across React Testing Library test suites |

## CI/CD Integration

### GitHub Actions Example

```yaml
- name: Install dependencies
  run: cd frontend && npm ci

- name: Install Playwright browsers
  run: cd frontend && npx playwright install --with-deps

- name: Run unit tests
  run: cd frontend && npm run test:unit

- name: Run E2E tests
  run: cd frontend && npm run test:e2e

- name: Upload coverage
  uses: codecov/codecov-action@v3
  with:
    files: ./frontend/coverage/coverage-final.json
```

### Recommended package.json scripts

```json
{
  "scripts": {
    "test": "vitest",
    "test:unit": "vitest run",
    "test:ui": "vitest --ui",
    "test:coverage": "vitest run --coverage",
    "test:e2e": "playwright test",
    "test:e2e:ui": "playwright test --ui",
    "test:e2e:debug": "playwright test --debug"
  }
}
```

## Sources

**Official Documentation (HIGH confidence):**
- [Next.js Testing: Vitest](https://nextjs.org/docs/app/guides/testing/vitest) — Feb 11, 2026 update confirms Vitest recommendation
- [Next.js Testing: Playwright](https://nextjs.org/docs/pages/guides/testing/playwright) — Feb 11, 2026 update confirms Playwright setup
- [React 19 Upgrade Guide](https://react.dev/blog/2024/04/25/react-19-upgrade-guide) — Official React team recommendation for testing-library migration
- [Vitest 4.0 Release](https://vitest.dev/blog/vitest-4) — Browser Mode stable, React 19 support confirmed
- [Playwright Release Notes](https://playwright.dev/docs/release-notes) — Version 1.58 features

**Package Registries (HIGH confidence):**
- [@vitejs/plugin-react@5.1.4](https://www.npmjs.com/package/@vitejs/plugin-react) — Published Feb 10, 2026
- [vitest@4.0.18](https://www.npmjs.com/package/vitest) — Published Jan 23, 2026
- [@playwright/test@1.58.2](https://www.npmjs.com/package/@playwright/test) — Published Feb 6, 2026
- [@testing-library/react@16.3.2](https://www.npmjs.com/package/@testing-library/react) — Published Jan 16, 2026
- [happy-dom@20.6.1](https://www.npmjs.com/package/happy-dom) — Published Feb 12, 2026
- [vite-tsconfig-paths@6.1.1](https://www.npmjs.com/package/vite-tsconfig-paths) — Published Feb 11, 2026
- [@testing-library/jest-dom@6.9.1](https://www.npmjs.com/package/@testing-library/jest-dom) — Published Oct 2025
- [@testing-library/user-event@14.6.1](https://www.npmjs.com/package/@testing-library/user-event) — Published Feb 2025

**Community Resources (MEDIUM confidence):**
- [Vitest vs jsdom vs happy-dom Discussion](https://github.com/vitest-dev/vitest/discussions/1607) — Performance comparisons
- [jsdom vs happy-dom: Navigating the Nuances](https://blog.seancoughlin.me/jsdom-vs-happy-dom-navigating-the-nuances-of-javascript-testing) — API coverage comparison
- [Testing Async RSCs with Next.js](https://abelcastro.dev/blog/testing-async-react-server-components-with-next-js) — Explains async Server Component limitations
- [How to Unit Test React Components](https://oneuptime.com/blog/post/2026-01-15-unit-test-react-vitest-testing-library/view) — Vitest + RTL setup patterns

---
*Stack research for: ShipSecure Frontend Testing Infrastructure*
*Researched: 2026-02-16*
