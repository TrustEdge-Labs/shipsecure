# Pitfalls Research: Frontend Testing Infrastructure

**Domain:** Adding Vitest + React Testing Library + Playwright to existing Next.js 16 + React 19 App Router project
**Researched:** 2026-02-16
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: Attempting to Unit Test Async Server Components

**What goes wrong:**
Tests hang, timeout, or fail with cryptic errors when trying to render async Server Components with Vitest/React Testing Library. The test runner simply doesn't support them.

**Why it happens:**
Async Server Components are new to the React ecosystem and Vitest currently does not support them. Developers assume that if they can test Client Components, they can test Server Components the same way. The jsdom/happy-dom environments used for unit tests cannot replicate the server-side async rendering pipeline.

**How to avoid:**
- Use **E2E tests (Playwright)** for async Server Components
- Only write unit tests for **synchronous** Server and Client Components
- Document which components are async vs sync in your codebase
- Default to E2E for Server Components unless they're explicitly synchronous

**Warning signs:**
- Tests that hang indefinitely with no clear error
- "Cannot read property of undefined" errors when rendering components
- Tests that work for Client Components but fail for similar Server Components
- Missing test environment initialization errors

**Phase to address:**
Phase 1 (Vitest Setup) — Documentation and component classification
Phase 3 (Playwright E2E) — Testing strategy for async components

---

### Pitfall 2: Breaking Server Actions with Incorrect Mocking

**What goes wrong:**
Server actions fail silently, return undefined, or throw "not a function" errors when components using them are tested. Forms that work in production completely break in tests.

**Why it happens:**
Server actions use 'use server' directive and cannot run in jsdom. Developers try to mock them like regular functions but miss the Next.js internal machinery (action IDs, form state management, progressive enhancement). The `useActionState` hook requires special handling that standard mocks don't provide.

**How to avoid:**
- Mock server actions at the **module level** with `vi.mock()`
- Return proper `ScanFormState` structure matching production types
- Mock `useActionState` separately from the action itself
- For comprehensive testing, use Playwright to test server action flows end-to-end
- Never test server actions in isolation with Vitest — they need the Next.js runtime

**Warning signs:**
- "useActionState is not a function" errors
- Form submissions that don't trigger state updates
- Tests passing but forms broken in production
- Missing validation errors that work in production
- `pending` state never changing in tests

**Phase to address:**
Phase 1 (Vitest Setup) — Mock patterns and documentation
Phase 2 (Component Tests) — Form testing strategies
Phase 3 (Playwright E2E) — End-to-end validation of server actions

---

### Pitfall 3: next/navigation Mock Configuration Hell

**What goes wrong:**
Components using `useRouter`, `usePathname`, or `useSearchParams` throw "invariant expected app router to be mounted" errors or "NextRouter was not mounted" errors. Tests become a graveyard of failed mock attempts.

**Why it happens:**
Next.js App Router (next/navigation) requires different mocking than Pages Router (next/router). Developers copy-paste old mock patterns, use the wrong import, or forget to mock all hooks used by the component. The module must be mocked BEFORE any component imports it.

**How to avoid:**
- Use `vi.mock('next/navigation')` in test setup
- Mock ALL navigation hooks your component uses: `useRouter`, `usePathname`, `useSearchParams`
- Use `vi.hoisted()` to ensure mocks are available before imports
- Consider using `next-router-mock` package for consistent mocking
- Double-check imports: `next/navigation` (App Router) not `next/router` (Pages Router)

**Mock pattern for Vitest:**
```typescript
import { vi } from 'vitest'

// Hoist mocks to top level
const mockPush = vi.fn()
const mockReplace = vi.fn()

vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: mockPush,
    replace: mockReplace,
    back: vi.fn(),
    forward: vi.fn(),
    refresh: vi.fn(),
  }),
  usePathname: () => '/test-path',
  useSearchParams: () => new URLSearchParams(),
}))
```

**Warning signs:**
- "invariant expected app router to be mounted" errors
- Tests passing when run individually but failing when run together
- Mocks working in one test file but not others
- Components that navigate successfully in production failing in tests

**Phase to address:**
Phase 1 (Vitest Setup) — Mock utilities and patterns
Phase 2 (Component Tests) — Navigation testing strategies

---

### Pitfall 4: TypeScript Path Alias Resolution Failures

**What goes wrong:**
Tests fail with "Failed to resolve import '@/...'" errors despite TypeScript compiling successfully. Components can't find their imports when run through Vitest.

**Why it happens:**
Vite (underlying Vitest) doesn't automatically read `tsconfig.json` paths. TypeScript uses `paths` in tsconfig for type checking, but Vite needs separate `resolve.alias` configuration. Next.js automatically resolves these at build time, making the issue invisible until tests run.

**How to avoid:**
- Install `vite-tsconfig-paths` plugin: `npm install -D vite-tsconfig-paths`
- Add plugin to `vitest.config.ts`:
  ```typescript
  import { defineConfig } from 'vitest/config'
  import react from '@vitejs/plugin-react'
  import tsconfigPaths from 'vite-tsconfig-paths'

  export default defineConfig({
    plugins: [tsconfigPaths(), react()],
    test: {
      environment: 'jsdom',
    },
  })
  ```
- Verify `tsconfig.json` has correct path mappings: `"@/*": ["./app/*"]` with asterisk
- Never use both `vite-tsconfig-paths` AND manual `resolve.alias` — pick one

**Warning signs:**
- "Failed to resolve import" errors for @/ imports
- Tests failing but `tsc` passing
- Some imports resolving, others not (inconsistent path depths)
- Works in Next.js dev/build but fails in tests

**Phase to address:**
Phase 1 (Vitest Setup) — Initial configuration

---

### Pitfall 5: Environment Variables Not Loading in Tests

**What goes wrong:**
`process.env.BACKEND_URL` and other environment variables return undefined in tests, causing API calls to fail or point to wrong URLs. Tests that should hit mock backends hit production or 404.

**Why it happens:**
Next.js automatically loads `.env*` files at build/dev time, but Vitest doesn't. The test environment doesn't have Next.js's env loading mechanism. Developers assume `.env.test` or `.env.local` will be automatically loaded.

**How to avoid:**
- Install `@next/env` package (usually already included with Next.js)
- Load environment variables in `vitest.config.ts`:
  ```typescript
  import { loadEnvConfig } from '@next/env'
  import { defineConfig } from 'vitest/config'

  loadEnvConfig(process.cwd())

  export default defineConfig({
    // ... rest of config
  })
  ```
- Create `.env.test` for test-specific overrides
- **IMPORTANT:** `.env.local` is NOT loaded in test environment (by design)
- Mock environment variables explicitly in tests that need different values:
  ```typescript
  vi.stubEnv('BACKEND_URL', 'http://mock-backend:3000')
  ```

**Warning signs:**
- `process.env.BACKEND_URL` is undefined in server action tests
- API calls going to `undefined/api/v1/scans`
- Tests failing with network errors but code working in dev
- Different behavior between `npm run dev` and `npm test`

**Phase to address:**
Phase 1 (Vitest Setup) — Environment configuration

---

### Pitfall 6: React 19 Suspense Rendering Changes Break Tests

**What goes wrong:**
Tests using Suspense get stuck rendering the fallback on React 19. Following RTL's suggestion to "wrap in act" makes it impossible to assert against the fallback state. Suspense behavior is completely different between React 18 and React 19.

**Why it happens:**
React 19 enabled concurrent rendering by default and changed how Suspense commits fallbacks. In React 19, the fallback is committed immediately and suspended siblings render afterward, while in React 18, behavior was different. React Testing Library hasn't fully adapted to these changes yet.

**How to avoid:**
- Expect different Suspense behavior in React 19 vs 18
- Use `findBy*` queries (async) instead of `getBy*` for content that appears after Suspense
- Don't rely on testing Suspense fallback states in unit tests — they're flaky
- Consider E2E tests for complex Suspense hierarchies
- Wait for React Testing Library updates to fully support React 19 patterns
- Avoid shallow rendering (deprecated and blocks React upgrades)

**Warning signs:**
- Tests stuck on loading states
- Warnings about unwrapped `act` calls
- Following RTL warnings makes tests worse
- Tests passing in React 18 but failing in React 19
- Suspense fallbacks displaying faster than expected

**Phase to address:**
Phase 2 (Component Tests) — Suspense testing patterns
Phase 3 (Playwright E2E) — Complex async flows

---

### Pitfall 7: Testing Client Components Without "use client" Awareness

**What goes wrong:**
Components using hooks (useState, useEffect, useRouter) fail with "hooks can only be used in client components" despite having 'use client' directive. Tests treat Client Components like Server Components or vice versa.

**Why it happens:**
The test environment doesn't process 'use client' directives the way Next.js does. Vitest renders everything as if it's a Client Component (in jsdom), masking Server Component violations. Developers write tests that pass but would fail in actual Next.js runtime.

**How to avoid:**
- Document which components are Client vs Server in your test structure
- Don't test 'use client' components with server-only features
- Be aware that Vitest renders everything in a client-like environment
- Use E2E tests to verify actual Server/Client boundaries
- Never import Server Component code directly into Client Component tests

**Warning signs:**
- Tests passing but production builds failing with "use client" errors
- Hooks working in tests but failing in production
- Import errors about server-only modules in client code

**Phase to address:**
Phase 1 (Vitest Setup) — Documentation of component types
Phase 2 (Component Tests) — Separation of Server/Client test strategies

---

### Pitfall 8: Playwright Running Against Dev Server Instead of Production Build

**What goes wrong:**
E2E tests run against `next dev` and pass, but production has different behavior (errors, missing optimizations, broken features). Tests become slow and unreliable due to hot reloading and debug features.

**Why it happens:**
Developers configure `webServer.command: 'npm run dev'` in `playwright.config.ts` because it's faster to start. The dev server includes hot reloading, verbose logging, and unoptimized bundles that don't exist in production. Tests pass against behavior that users will never see.

**How to avoid:**
- **ALWAYS** test against production build: `npm run build && npm run start`
- Configure Playwright webServer:
  ```typescript
  webServer: {
    command: 'npm run build && npm run start',
    port: 3000,
    reuseExistingServer: !process.env.CI,
  }
  ```
- Use `reuseExistingServer: !process.env.CI` to speed up local testing
- In CI, always build fresh and test production build
- Document this requirement clearly for team members

**Warning signs:**
- Tests running slowly (hot reload overhead)
- Flaky tests that sometimes pass/fail
- Different behavior between local tests and CI
- Production bugs that tests didn't catch
- Console flooded with Next.js dev logs during tests

**Phase to address:**
Phase 3 (Playwright E2E) — Initial setup

---

### Pitfall 9: Not Mocking External API Calls in Playwright

**What goes wrong:**
E2E tests hit real backend services, fail due to network issues/rate limits, pollute production databases with test data, or create billing charges. Tests become flaky and slow.

**Why it happens:**
Developers assume E2E means "test everything including external services." They don't realize Playwright can intercept network requests. Tests initially pass but become unreliable as backend services change or rate limits kick in.

**How to avoid:**
- Use Playwright's route interception to mock external APIs:
  ```typescript
  await page.route('**/api/v1/scans', async route => {
    await route.fulfill({
      status: 200,
      body: JSON.stringify({ id: 'test-scan-123' }),
    })
  })
  ```
- Consider Next.js experimental `testMode` with MSW fixtures
- Mock at the network boundary, not server actions
- Create fixtures for common API responses
- Use real backend only for critical integration tests
- Document which tests hit real services vs mocks

**Warning signs:**
- Tests failing on weekends (backend maintenance)
- Flaky tests that timeout randomly
- Unexpected charges from external services
- Test data appearing in production databases
- Rate limit errors in test output

**Phase to address:**
Phase 3 (Playwright E2E) — Network mocking patterns
Phase 4 (CI Integration) — Consistent test environments

---

### Pitfall 10: Forgetting to Save Playwright Authentication State

**What goes wrong:**
Every test logs in from scratch, making test suites unbearably slow. Authentication flows run hundreds of times unnecessarily. CI builds timeout due to repeated login calls.

**Why it happens:**
Developers put login code in `beforeEach` hooks thinking it's required for test isolation. They don't know about Playwright's state persistence feature. Each test takes 2-5 seconds just for authentication setup.

**How to avoid:**
- Use Playwright's setup projects to authenticate once:
  ```typescript
  // auth.setup.ts
  test('authenticate', async ({ page }) => {
    await page.goto('/login')
    await page.fill('input[name=email]', 'test@example.com')
    await page.fill('input[name=password]', 'password')
    await page.click('button[type=submit]')
    await page.context().storageState({ path: 'auth.json' })
  })

  // playwright.config.ts
  projects: [
    { name: 'setup', testMatch: /auth.setup\.ts/ },
    {
      name: 'chromium',
      use: { storageState: 'auth.json' },
      dependencies: ['setup'],
    },
  ]
  ```
- Reuse authenticated state across tests
- Only re-authenticate in tests that specifically test login/logout

**Warning signs:**
- Tests taking 2-5 seconds each before actual assertions
- Hundreds of login attempts in backend logs
- CI builds timing out on authentication
- Rate limiting from login endpoint

**Phase to address:**
Phase 3 (Playwright E2E) — Authentication patterns
Phase 4 (CI Integration) — Performance optimization

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Skip E2E tests for async Server Components | Faster initial setup | Can't verify critical user flows work end-to-end | Never — async components need E2E coverage |
| Use `any` types in test mocks | Tests write faster | Type safety lost, runtime errors not caught | Only for complex third-party types with poor definitions |
| Mock entire modules instead of specific functions | Simpler mock setup | Tests become brittle when module structure changes | Early prototyping, refactor once patterns stabilize |
| Test against dev server | Faster test startup | Production bugs slip through | Local development only, never in CI |
| Skip coverage thresholds initially | Can start testing immediately | Coverage silently degrades over time | First 2 weeks of setup, then enforce minimums |
| Inline mocks in every test file | No shared test infrastructure needed | Duplicate mock code, inconsistent patterns | Small projects (<10 test files) |
| Use jsdom for all tests | Single environment, simpler setup | Slower than happy-dom, higher memory usage | Projects with complex DOM manipulation needs |
| Disable ESLint test rules | Faster to write tests | Bad patterns spread, test quality degrades | Never — enforce from day one |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Zod validation in server actions | Mock server action without Zod validation logic | Import and test Zod schema separately, or test via E2E |
| Stripe checkout redirect | Test actual redirect to Stripe | Mock `redirect()` from next/navigation, verify called with correct URL |
| Dynamic routes ([token], [id]) | Hard-code route params in tests | Use Playwright's parameterized routing, test multiple param values |
| API polling (scan status) | Let tests wait for real polling timeout | Mock time with `vi.useFakeTimers()`, fast-forward clock |
| CSS custom properties (dark mode) | Assume CSS variables work in jsdom | jsdom has limited CSS support — test visual states via Playwright |
| GitHub Actions CI/CD | Assume same environment as local | Explicitly configure Node version, install deps, cache properly |
| Form validation errors | Only test happy path | Test each validation rule, ensure error messages match UI |
| Next.js Image component | Test with real images | Mock next/image or use Playwright for visual tests |
| Lucide icons | Render actual SVG in tests | Mock as simple div or use data-testid for assertions |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Running all Playwright tests serially | Tests take 10+ minutes | Use workers: `workers: process.env.CI ? 2 : 4` | >50 E2E tests |
| Not caching node_modules in CI | Every build downloads 200MB+ | Cache node_modules by package-lock hash | First CI run |
| Using jsdom when happy-dom would suffice | Tests slower than necessary, higher memory | Try happy-dom first, fall back to jsdom if needed | Large test suites (100+ tests) |
| No coverage thresholds | Coverage slowly drops to 20% | Enforce minimums: `branches: 60, functions: 70, lines: 75` | After 3-6 months without enforcement |
| Testing all permutations of form inputs | Thousands of redundant tests | Test boundary cases + happy path, not every combination | >20 test cases per form |
| Playwright screenshots for every test | CI artifacts in gigabytes, slow uploads | Screenshot only on failure: `screenshot: 'only-on-failure'` | >100 E2E tests |
| Re-rendering entire page for each assertion | Component tests take seconds each | Use Testing Library's query methods, avoid redundant renders | >50 component tests |
| Loading real backend in Playwright | Tests timeout, flaky failures | Mock network at route level, test backend separately | First E2E test run |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Committing `.env.test` with real credentials | Leaked secrets in git history | Use dummy values, document real config in 1Password |
| Testing against production backend | Test data pollutes production, potential data corruption | Always mock external services, or use dedicated test environment |
| Storing Playwright auth.json in git | Session tokens exposed publicly | Add `auth.json` to `.gitignore`, generate in CI |
| Hard-coding API keys in test fixtures | Keys leaked in test snapshots | Use environment variables, rotate keys regularly |
| Skipping authorization checks in mocks | Tests pass but production vulnerable | Mock authorization but verify it's called with correct params |
| Testing with user data from production | GDPR/privacy violations, data leak risk | Generate synthetic test data, never use real user info |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Not testing error messages | Users see cryptic technical errors | Assert on exact error message text, verify user-friendly |
| Skipping loading states | No feedback during slow operations | Test that loading states appear, verify accessible text |
| Missing accessibility in tests | Broken screen reader experience | Use RTL's `getByRole`, ensure proper ARIA attributes |
| Not testing mobile viewports | Broken layout on phones | Playwright viewport tests: 375px, 768px, 1440px |
| Assuming instant navigation | Users see flash of wrong content | Test transition states, verify no content flash |
| Ignoring keyboard navigation | Non-mouse users can't use app | Test tab order, Enter/Space activation in E2E tests |
| Testing only success paths | Users get stuck on errors with no guidance | Test every error state, verify recovery paths |

## "Looks Done But Isn't" Checklist

- [ ] **Vitest setup:** Often missing environment variable loading — verify `loadEnvConfig` in config
- [ ] **Server actions:** Often missing proper type mocks — verify mock returns match `ScanFormState`
- [ ] **next/navigation mocks:** Often missing one of useRouter/usePathname/useSearchParams — verify all used hooks mocked
- [ ] **Path aliases:** Often missing vite-tsconfig-paths plugin — verify @/ imports resolve
- [ ] **Playwright config:** Often testing dev instead of production — verify `npm run build && npm run start`
- [ ] **Coverage thresholds:** Often not enforced — verify vitest.config has coverage.thresholds
- [ ] **CI integration:** Often missing cache configuration — verify node_modules cached
- [ ] **Auth state:** Often re-authenticating every test — verify storageState reuse
- [ ] **Network mocking:** Often hitting real APIs — verify route interception for external calls
- [ ] **Error boundaries:** Often not tested — verify error.tsx and global-error.tsx tested

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Async Server Component unit tests | LOW | Delete unit tests, rewrite as Playwright E2E tests |
| Broken server action mocks | LOW | Replace with module-level vi.mock, return proper types |
| next/navigation errors | LOW | Add vi.mock('next/navigation') with all hooks, use vi.hoisted |
| Path alias failures | LOW | Install vite-tsconfig-paths, add to plugins array |
| Missing environment variables | LOW | Add loadEnvConfig to vitest.config.ts, create .env.test |
| React 19 Suspense issues | MEDIUM | Replace getBy with findBy, accept fallback testing limitations |
| Dev server testing | MEDIUM | Update playwright.config webServer command, rebuild test data |
| No API mocking | MEDIUM | Add page.route() calls, create API fixture files |
| Slow auth in every test | MEDIUM | Create auth.setup.ts, configure storageState in config |
| jsdom performance issues | LOW | Switch to happy-dom in vitest.config, verify tests still pass |
| No coverage enforcement | HIGH | Add thresholds, fix uncovered code, don't lower thresholds |
| Tests hitting production | HIGH | Audit all tests for external calls, add comprehensive mocking |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Async Server Components | Phase 1 (Vitest Setup) | Documentation clarifies unit test limitations |
| Server action mocking | Phase 1 (Vitest Setup) | Mock utilities created and documented |
| next/navigation mocking | Phase 1 (Vitest Setup) | Mock pattern tested with sample component |
| Path alias resolution | Phase 1 (Vitest Setup) | Import with @/ works in first test |
| Environment variables | Phase 1 (Vitest Setup) | process.env.BACKEND_URL defined in test |
| React 19 Suspense | Phase 2 (Component Tests) | Suspense components tested successfully |
| Client vs Server awareness | Phase 2 (Component Tests) | Test suite structure reflects component types |
| Production build testing | Phase 3 (Playwright E2E) | Config uses build + start, not dev |
| External API mocking | Phase 3 (Playwright E2E) | No network calls to real backend in tests |
| Auth state reuse | Phase 3 (Playwright E2E) | Setup project creates auth.json once |
| Coverage enforcement | Phase 4 (CI Integration) | CI fails if coverage drops below threshold |
| CI performance | Phase 4 (CI Integration) | Test suite completes in <5 minutes |

## Next.js 16 + React 19 Specific Warnings

**App Router Migration:** If migrating from Pages Router, old test patterns won't work. Don't copy-paste Page Router mocks.

**React 19 Breaking Changes:** useFormState renamed to useActionState. Update mocks accordingly. Old RTL examples may use deprecated patterns.

**Server Actions:** The 'use server' directive is production-ready in Next.js 16 but still has sharp edges in testing. Default to E2E for complex flows.

**Concurrent Rendering:** React 19 enables this by default. Tests may expose race conditions that were hidden in React 18. This is good — fix the races, don't disable concurrency.

**Turbopack (optional):** If using Turbopack for dev, don't test against it — Playwright should test Webpack production build.

**Edge Runtime:** If using edge runtime for any routes, note that Vitest's 'edge-runtime' environment is experimental. Default to E2E for edge routes.

## Sources

**Official Documentation (HIGH confidence):**
- [Testing: Vitest | Next.js](https://nextjs.org/docs/app/guides/testing/vitest) — Updated 2026-02-11
- [Testing: Playwright | Next.js](https://nextjs.org/docs/pages/guides/testing/playwright)
- [Guides: Testing | Next.js](https://nextjs.org/docs/app/guides/testing)
- [React v19 – React](https://react.dev/blog/2024/12/05/react-19)
- [React 19 Upgrade Guide – React](https://react.dev/blog/2024/04/25/react-19-upgrade-guide)

**Community Resources (MEDIUM confidence):**
- [App Router pitfalls: common Next.js mistakes](https://imidef.com/en/2026-02-11-app-router-pitfalls)
- [Test Strategy in the Next.js App Router Era](https://shinagawa-web.com/en/blogs/nextjs-app-router-testing-setup)
- [How to Test Next.js Apps with Playwright: Complete Guide](https://www.getautonoma.com/blog/nextjs-playwright-testing-guide)
- [jsdom vs happy-dom: Navigating the Nuances](https://blog.seancoughlin.me/jsdom-vs-happy-dom-navigating-the-nuances-of-javascript-testing)

**GitHub Discussions (MEDIUM confidence):**
- [How to test next/navigation? · vercel/next.js #42527](https://github.com/vercel/next.js/discussions/42527)
- [How do you unit test server actions? · vercel/next.js #69036](https://github.com/vercel/next.js/discussions/69036)
- [Integration tests with RTL and server actions · vercel/next.js #56304](https://github.com/vercel/next.js/discussions/56304)
- [@testing-library/react behaves differently with Suspense in React 18 and React 19 · Issue #1375](https://github.com/testing-library/react-testing-library/issues/1375)
- [Setting Up Vitest to Support TypeScript Path Aliases](https://www.timsanteford.com/posts/setting-up-vitest-to-support-typescript-path-aliases/)

**Testing Library Resources (HIGH confidence):**
- [FAQ | Testing Library](https://testing-library.com/docs/react-testing-library/faq/)
- [Async waits in React Testing Library | Reflect](https://reflect.run/articles/async-waits-in-react-testing-library/)
- [Using waitFor in React Testing Library Explained | Testim](https://www.testim.io/blog/react-testing-library-waitfor/)

**Playwright Resources (HIGH confidence):**
- [TestConfig | Playwright](https://playwright.dev/docs/api/class-testconfig)
- [Configuration | Playwright](https://playwright.dev/docs/test-configuration)

**Package Documentation (HIGH confidence):**
- [vite-tsconfig-paths - npm](https://www.npmjs.com/package/vite-tsconfig-paths)
- [next-router-mock - npm](https://www.npmjs.com/package/next-router-mock)
- [Coverage | Guide | Vitest](https://vitest.dev/guide/coverage)

---

*Pitfalls research for: Frontend Testing Infrastructure (Next.js 16 + React 19)*
*Researched: 2026-02-16*
*Confidence: HIGH — Based on official Next.js/React documentation (updated Feb 2026), verified community patterns, and project-specific analysis*
