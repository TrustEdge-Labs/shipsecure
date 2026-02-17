# Phase 25: Test Infrastructure - Context

**Gathered:** 2026-02-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Set up Vitest, MSW, and React Testing Library so developers can run `npm test` and see a working test suite with mocking infrastructure ready for component and integration tests. All API mock handlers created here. First passing tests prove the full stack works.

</domain>

<decisions>
## Implementation Decisions

### Test file organization
- Tests live in a separate `frontend/__tests__/` directory, NOT colocated with components
- Internal structure mirrors `src/` — e.g., `__tests__/components/Header.test.tsx` for `src/components/Header.tsx`
- Naming convention: `.test.tsx` (Vitest default)
- Test utilities (custom render, MSW handlers, fixtures) live in `__tests__/helpers/`

### Mock data strategy
- Realistic fixtures with full API response shapes (real-looking URLs, grades, findings)
- Shared fixture files in `__tests__/helpers/fixtures/` — single source of truth, reusable across tests
- All API endpoints covered upfront: scan, results, checkout, webhook handlers all created in Phase 25
- Error scenarios included from the start: success + error variants (500s, timeouts, 404s) for each endpoint

### First passing test
- Header component gets the first test — has navigation links, CTA, logo rendering
- Test depth: basic assertions — renders + verify logo, nav links, CTA are present
- Include one MSW integration test that exercises scan status fetch — proves data fetching pipeline works end-to-end
- Two proof points: component rendering works (Header) AND API mocking works (scan status fetch)

### Test runner UX
- `npm test` runs in watch mode with coverage by default
- Minimal (dots) output — compact, only show failures in detail
- Scripts: `test` (watch + coverage), `test:e2e` (Playwright, Phase 27), `test:ci` (single-run for CI)

### Claude's Discretion
- Vitest configuration details and plugin setup
- Path alias resolution approach
- MSW server setup/teardown mechanics
- Custom RTL render wrapper implementation
- Coverage threshold values (Phase 28 enforces, but Phase 25 configures)

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches for Vitest + Next.js setup.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 25-test-infrastructure*
*Context gathered: 2026-02-16*
