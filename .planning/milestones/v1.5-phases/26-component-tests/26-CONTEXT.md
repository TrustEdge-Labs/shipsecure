# Phase 26: Component Tests - Context

**Gathered:** 2026-02-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Write unit/component tests for all 9 client components (ScanForm, ResultsDashboard, GradeSummary, FindingAccordion, ProgressChecklist, UpgradeCTA, Header, Footer, Logo) plus dark mode rendering, loading skeletons, and error boundary fallback UI. Tests verify rendering, interactions, and edge cases from a user's perspective using the Vitest + RTL infrastructure established in Phase 25.

</domain>

<decisions>
## Implementation Decisions

### Test granularity
- One test file per component: `__tests__/components/ScanForm.test.tsx`, `Footer.test.tsx`, etc.
- Comprehensive depth: 5-15 tests per component covering all states, edge cases, and accessibility queries
- Use `@testing-library/user-event` for all interactions (clicks, typing, expanding) — more realistic than fireEvent
- Prefer accessible queries first: `getByRole`, `getByLabelText`, `getByText` over `data-testid` — validates accessibility implicitly

### Dark mode testing
- Single dedicated test file: `__tests__/components/dark-mode.test.tsx`
- Renders all components under both light AND dark color schemes (both directions as baseline)
- Verification level: renders without errors only — visual correctness deferred to v2 visual regression testing
- Uses matchMedia mock to simulate `prefers-color-scheme: dark`

### ScanForm strategy
- Mock `useActionState` globally via `vi.mock('react')` to replace with controllable mock — test form states directly
- Test client-side validation messages: submit with bad URL/email, assert error message appears in DOM
- Full CFAA consent flow: checkbox exists, form won't submit without it checked, visual checkbox state
- Full loading state verification: button text changes (e.g., "Scanning..."), inputs disabled, spinner/indicator visible

### Coverage expectations
- Highest priority (revenue path): ScanForm → ResultsDashboard → UpgradeCTA — most comprehensive tests
- Header: existing 4 tests from Phase 25 are sufficient — do NOT create additional Header tests in Phase 26
- ProgressChecklist: simulate stage transitions via re-rendering with different props (pending → active → complete)
- FindingAccordion: test expand/collapse via user-event click + content visibility check + aria-expanded state
- Loading skeletons (COMP-11): verify renders correct structure — 2-3 tests
- Error boundary (COMP-12): verify fallback UI renders — 2-3 tests

### Claude's Discretion
- Exact number of tests per component (within the 5-15 comprehensive range)
- Test descriptions and naming conventions
- Helper utilities needed beyond renderWithProviders
- Which specific edge cases to cover for secondary components (Footer, Logo)

</decisions>

<specifics>
## Specific Ideas

- Revenue path testing priority: scan → results → upgrade (ScanForm, ResultsDashboard, UpgradeCTA) must be the most thoroughly tested
- ProgressChecklist should re-render with changing props to simulate real scan progression
- FindingAccordion tests should verify content visibility toggles on click, not just presence

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 26-component-tests*
*Context gathered: 2026-02-17*
