# Phase 49: Test Suite - Context

**Gathered:** 2026-04-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Comprehensive test coverage for the supply chain scanner feature (Phases 46-48). No new features — tests only across Rust backend, Vitest frontend components, and Playwright E2E.

</domain>

<decisions>
## Implementation Decisions

### Test Fixtures
- **D-01:** Use synthetic minimal JSON fixtures for unit tests — small, fast, deterministic. No real-world lockfiles (large, brittle, drift-prone).

### OSV Mocking
- **D-02:** Hardcoded mock responses inline in Rust tests. No external mock server needed — follow existing test patterns with inline fixtures.

### E2E Approach
- **D-03:** Playwright tests against local Docker with real backend. No external OSV dependency — backend handles unreachable OSV gracefully.

### Frontend Test Style
- **D-04:** Behavioral tests with user events + assertions (consistent with existing `__tests__/` patterns). No snapshot tests.

### Coverage
- **D-05:** Phase 49 tests should cover the 3 supply chain components currently excluded from coverage (supply-chain-form, supply-chain-summary, supply-chain-findings). Remove exclusions from vitest.config.ts after tests are added.

### Claude's Discretion
- Test file organization and naming
- Specific assertion granularity
- Whether to use MSW or direct fetch mocks for frontend API calls

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Test Patterns
- `frontend/__tests__/` — Vitest test directory with established patterns
- `frontend/vitest.setup.ts` — Global test mocks (next/navigation, next/image, @clerk/nextjs)
- `frontend/__tests__/helpers/test-utils.tsx` — `renderWithProviders()` wrapper
- `frontend/vitest.config.ts` — Coverage config with component exclusions to remove

### Source Files Under Test
- `src/scanners/lockfile_parser.rs` — Lockfile parser (v1/v2/v3)
- `src/scanners/osv_client.rs` — OSV API client and categorizer
- `src/api/supply_chain.rs` — Supply chain scan handler
- `src/api/results.rs` — Results endpoint (kind + supply_chain_results patch)
- `frontend/components/supply-chain-form.tsx` — 3-tab form component
- `frontend/components/supply-chain-summary.tsx` — 5-tier summary cards
- `frontend/components/supply-chain-findings.tsx` — Tiered findings list

### Phase Plans (what was built)
- `.planning/phases/48-frontend/48-01-PLAN.md` — Types, server action, backend patch
- `.planning/phases/48-frontend/48-02-PLAN.md` — Results page, nav, analytics

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `renderWithProviders()` in `test-utils.tsx` — wraps components with ClerkProvider mock
- `vitest.setup.ts` — pre-configured mocks for next/navigation, next/image, @clerk/nextjs
- Existing MSW setup patterns in `__tests__/helpers/`

### Established Patterns
- Vitest + happy-dom environment for component tests
- `fireEvent.click` over `userEvent.click` for clipboard tests (happy-dom limitation)
- `vi.useFakeTimers` for date-dependent tests
- Playwright E2E runs on port 3001 against production builds

### Integration Points
- Supply chain components import from `@/lib/supply-chain-types` and `@/app/actions/supply-chain-scan`
- Backend tests use `#[cfg(test)]` modules within source files

</code_context>

<specifics>
## Specific Ideas

No specific requirements — follow established project test patterns.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 49-test-suite*
*Context gathered: 2026-04-07*
