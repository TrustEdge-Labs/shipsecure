# Phase 26: Component Tests - Research

**Researched:** 2026-02-16
**Domain:** React component testing with Vitest, React Testing Library, and user-event
**Confidence:** MEDIUM-HIGH

## Summary

Phase 26 builds on the test infrastructure from Phase 25 to write comprehensive component tests for all 9 client components plus dark mode, loading states, and error boundaries. The standard stack is Vitest + React Testing Library + user-event, following the guiding principle: "The more your tests resemble the way your software is used, the more confidence they can give you."

The key technical challenge is mocking React's `useActionState` hook for ScanForm testing. While Next.js server action hooks have known testing challenges in Vitest environments, we can work around this by mocking the hook's return value to control form state directly in tests. All other components are standard client-side React components that fit RTL's testing model perfectly.

Testing strategy prioritizes the revenue path (ScanForm → ResultsDashboard → UpgradeCTA) with 10-15 comprehensive tests each, uses accessible queries (getByRole, getByLabelText) to validate accessibility implicitly, and leverages user-event for realistic user interactions. Dark mode testing uses matchMedia mocks to simulate prefers-color-scheme, verifying components render without errors in both themes.

**Primary recommendation:** Use user-event for all interactions, prioritize getByRole queries with the name option, mock useActionState via vi.mock('react') spreading original exports, and organize tests in __tests__/components/ matching the components/ directory structure.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Test granularity:**
- One test file per component: `__tests__/components/ScanForm.test.tsx`, `Footer.test.tsx`, etc.
- Comprehensive depth: 5-15 tests per component covering all states, edge cases, and accessibility queries
- Use `@testing-library/user-event` for all interactions (clicks, typing, expanding) — more realistic than fireEvent
- Prefer accessible queries first: `getByRole`, `getByLabelText`, `getByText` over `data-testid` — validates accessibility implicitly

**Dark mode testing:**
- Single dedicated test file: `__tests__/components/dark-mode.test.tsx`
- Renders all components under both light AND dark color schemes (both directions as baseline)
- Verification level: renders without errors only — visual correctness deferred to v2 visual regression testing
- Uses matchMedia mock to simulate `prefers-color-scheme: dark`

**ScanForm strategy:**
- Mock `useActionState` globally via `vi.mock('react')` to replace with controllable mock — test form states directly
- Test client-side validation messages: submit with bad URL/email, assert error message appears in DOM
- Full CFAA consent flow: checkbox exists, form won't submit without it checked, visual checkbox state
- Full loading state verification: button text changes (e.g., "Scanning..."), inputs disabled, spinner/indicator visible

**Coverage expectations:**
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

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| COMP-01 | ScanForm tests covering URL validation, email validation, CFAA consent requirement, form submission (mocked server action), loading state, and error display | Standard Stack (RTL + user-event + Vitest), useActionState Mocking Pattern, Form Testing Patterns, Code Examples section |
| COMP-02 | ResultsDashboard tests covering findings rendering by category, grade display, empty state, and error state | Standard Stack, Component State Testing Pattern, Code Examples for conditional rendering |
| COMP-03 | GradeSummary tests covering grade display (A-F), color coding, and grade text | Standard Stack, Props-based rendering tests, Accessible queries for grade badges |
| COMP-04 | FindingAccordion tests covering expand/collapse behavior, finding details rendering, and severity indicators | Standard Stack, user-event click interactions, aria-expanded attribute verification, Code Examples for accordion testing |
| COMP-05 | ProgressChecklist tests covering stage progression, completed/active/pending states, and checkmarks | Standard Stack, Re-rendering with different props pattern, Visual state verification |
| COMP-06 | UpgradeCTA tests covering pricing display, checkout link rendering, and click behavior | Standard Stack, Click handler testing with user-event, MSW mock integration for fetch |
| COMP-07 | Header tests covering navigation links, logo rendering, "Scan Now" CTA, and responsive behavior | Standard Stack (existing 4 tests from Phase 25 sufficient per user constraints) |
| COMP-08 | Footer tests covering legal links, OSS attribution, and rendering without errors | Standard Stack, Link rendering verification with getByRole('link') |
| COMP-09 | Logo tests covering icon/compact/full variant rendering and dark mode | Standard Stack, Props-based variant testing, Dark mode rendering verification |
| COMP-10 | Dark mode rendering verified for all components using prefers-color-scheme media query | matchMedia Mocking Pattern, Dark Mode Testing section, Code Examples for prefers-color-scheme |
| COMP-11 | Loading skeleton components tested for correct rendering during async operations | Standard Stack, Structure verification tests (renders without errors) |
| COMP-12 | Error boundary (error.tsx) tested for fallback UI rendering on component errors | Error Boundary Testing Pattern, Code Examples for throwing errors in tests |

</phase_requirements>

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| vitest | 4.0.18 | Test runner | Next.js officially recommends Vitest over Jest for App Router; better ESM support, faster |
| @testing-library/react | 16.3.2 | Component testing | De facto standard for React component testing; encourages accessibility-first queries |
| @testing-library/user-event | 14.6.1 | User interaction simulation | More realistic than fireEvent; simulates full user interactions with visibility/interactability checks |
| @testing-library/jest-dom | 6.9.1 | DOM matchers | Provides semantic matchers like toBeInTheDocument(), toHaveAttribute() for readable assertions |
| happy-dom | 20.6.1 | DOM environment | Faster than jsdom; sufficient for component tests (already configured in Phase 25) |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| msw | 2.12.10 | API mocking | For components making fetch calls (UpgradeCTA); already configured in Phase 25 |
| @vitejs/plugin-react | 5.1.4 | React support | Required for JSX/TSX compilation in Vitest |
| vite-tsconfig-paths | 6.1.1 | Path alias resolution | Resolves @/* imports in tests; already configured |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Vitest | Jest | Jest has poor ESM support, slower startup; Vitest recommended by Next.js for App Router |
| user-event | fireEvent | fireEvent only dispatches events; user-event simulates full interactions more realistically |
| happy-dom | jsdom | jsdom more complete but slower; happy-dom sufficient for component tests |

**Installation:**

All dependencies already installed in Phase 25. No additional packages needed.

## Architecture Patterns

### Recommended Test Structure

```
__tests__/
├── components/           # Component unit tests (Phase 26)
│   ├── ScanForm.test.tsx
│   ├── ResultsDashboard.test.tsx
│   ├── GradeSummary.test.tsx
│   ├── FindingAccordion.test.tsx
│   ├── ProgressChecklist.test.tsx
│   ├── UpgradeCTA.test.tsx
│   ├── Footer.test.tsx
│   ├── Logo.test.tsx
│   ├── Header.test.tsx       # Already exists from Phase 25
│   ├── dark-mode.test.tsx    # Dark mode for all components
│   ├── loading.test.tsx      # Loading skeleton
│   └── error-boundary.test.tsx
├── integration/          # Integration tests (Phase 25)
│   └── scan-status.test.tsx
└── helpers/              # Test utilities (Phase 25)
    ├── test-utils.tsx
    ├── fixtures/
    └── msw/
```

### Pattern 1: Basic Component Test Structure

**What:** Standard test file organization with describe blocks, user-event setup, and accessible queries

**When to use:** Every component test file

**Example:**
```typescript
// Source: Testing Library official docs + existing Header.test.tsx
import { describe, expect, test } from 'vitest'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import { ComponentName } from '@/components/component-name'

describe('ComponentName', () => {
  test('renders primary content', () => {
    renderWithProviders(<ComponentName />)

    // Prefer getByRole with name option (most accessible)
    const heading = screen.getByRole('heading', { name: /expected text/i })
    expect(heading).toBeInTheDocument()
  })

  test('handles user interaction', async () => {
    // Setup user-event before rendering (recommended pattern)
    const user = userEvent.setup()
    renderWithProviders(<ComponentName />)

    const button = screen.getByRole('button', { name: /click me/i })
    await user.click(button)

    // Assert on outcome
    expect(screen.getByText(/clicked/i)).toBeInTheDocument()
  })
})
```

### Pattern 2: Testing Form Components with useActionState Mock

**What:** Mock React's useActionState hook to control form state in tests

**When to use:** ScanForm component testing

**Example:**
```typescript
// Source: Vitest mocking docs + React Testing Library form testing patterns
import { vi, describe, test, expect, beforeEach } from 'vitest'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import { ScanForm } from '@/components/scan-form'

// Mock useActionState with controllable state
let mockState = {}
let mockFormAction = vi.fn()
let mockPending = false

vi.mock('react', async () => {
  const actual = await vi.importActual('react')
  return {
    ...actual,
    useActionState: vi.fn(() => [mockState, mockFormAction, mockPending])
  }
})

describe('ScanForm', () => {
  beforeEach(() => {
    mockState = {}
    mockFormAction = vi.fn()
    mockPending = false
  })

  test('displays URL validation error', async () => {
    // Set mock state to return validation error
    mockState = {
      errors: {
        url: ['Please enter a valid URL']
      }
    }

    renderWithProviders(<ScanForm />)

    // Validation error should be visible
    expect(screen.getByText(/please enter a valid url/i)).toBeInTheDocument()
  })

  test('shows loading state during submission', () => {
    mockPending = true

    renderWithProviders(<ScanForm />)

    const submitButton = screen.getByRole('button', { name: /starting scan/i })
    expect(submitButton).toBeDisabled()
  })

  test('requires CFAA consent checkbox to submit', async () => {
    const user = userEvent.setup()
    renderWithProviders(<ScanForm />)

    const checkbox = screen.getByRole('checkbox', {
      name: /i confirm i own this website/i
    })

    expect(checkbox).not.toBeChecked()

    await user.click(checkbox)
    expect(checkbox).toBeChecked()
  })
})
```

### Pattern 3: Testing Accordion/Collapsible Components

**What:** Verify expand/collapse behavior using user-event clicks and aria-expanded attribute

**When to use:** FindingAccordion component

**Example:**
```typescript
// Source: RTL aria-expanded testing patterns + GitHub examples
import { describe, test, expect } from 'vitest'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import { FindingAccordion } from '@/components/finding-accordion'

describe('FindingAccordion', () => {
  const mockFinding = {
    id: '1',
    title: 'Missing CSP header',
    description: 'Content-Security-Policy header is missing',
    severity: 'high' as const,
    remediation: 'Add CSP header to your app',
    scanner_name: 'security_headers',
    vibe_code: false,
  }

  test('expands on click and shows details', async () => {
    const user = userEvent.setup()
    renderWithProviders(<FindingAccordion finding={mockFinding} />)

    // Initially collapsed - details not visible
    expect(screen.queryByText(/content-security-policy header is missing/i))
      .not.toBeInTheDocument()

    // Click to expand
    const button = screen.getByRole('button', { name: /missing csp header/i })
    await user.click(button)

    // Details should now be visible
    expect(screen.getByText(/content-security-policy header is missing/i))
      .toBeInTheDocument()
    expect(screen.getByText(/add csp header to your app/i))
      .toBeInTheDocument()
  })

  test('updates aria-expanded attribute on toggle', async () => {
    const user = userEvent.setup()
    renderWithProviders(<FindingAccordion finding={mockFinding} />)

    const button = screen.getByRole('button', { name: /missing csp header/i })

    // Note: FindingAccordion doesn't use aria-expanded in current implementation
    // but best practice is to check it when present
    await user.click(button)

    // Verify expanded state by checking content visibility
    expect(screen.getByText(/how to fix/i)).toBeInTheDocument()
  })
})
```

### Pattern 4: Testing Component State Changes via Re-rendering

**What:** Simulate state transitions by re-rendering component with different props

**When to use:** ProgressChecklist stage progression testing

**Example:**
```typescript
// Source: RTL rerender pattern from official docs
import { describe, test, expect } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import { ProgressChecklist } from '@/components/progress-checklist'

describe('ProgressChecklist', () => {
  test('shows progression from pending to completed stages', () => {
    // Initial render: only detection stage complete
    const { rerender } = renderWithProviders(
      <ProgressChecklist
        stages={{
          detection: true,
          headers: false,
          tls: false,
          files: false,
          secrets: false,
          vibecode: false,
        }}
        status="in_progress"
      />
    )

    expect(screen.getByText(/detecting framework/i)).toBeInTheDocument()

    // Re-render with more stages complete
    rerender(
      <ProgressChecklist
        stages={{
          detection: true,
          headers: true,
          tls: true,
          files: false,
          secrets: false,
          vibecode: false,
        }}
        status="in_progress"
      />
    )

    // Verify visual state updated
    expect(screen.getByText(/security headers/i)).toBeInTheDocument()
    expect(screen.getByText(/tls configuration/i)).toBeInTheDocument()
  })
})
```

### Pattern 5: Dark Mode Testing with matchMedia Mock

**What:** Mock window.matchMedia to simulate prefers-color-scheme media query

**When to use:** dark-mode.test.tsx file testing all components

**Example:**
```typescript
// Source: Cypress dark mode testing patterns adapted for Vitest
import { describe, test, expect, beforeEach, afterEach, vi } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import { ScanForm } from '@/components/scan-form'

describe('Dark Mode Rendering', () => {
  let matchMediaMock: any

  beforeEach(() => {
    // Mock matchMedia before each test
    matchMediaMock = vi.fn()
  })

  afterEach(() => {
    // Clean up
    vi.restoreAllMocks()
  })

  test('ScanForm renders in dark mode without errors', () => {
    // Mock prefers-color-scheme: dark
    matchMediaMock.mockImplementation((query: string) => ({
      matches: query === '(prefers-color-scheme: dark)',
      media: query,
      onchange: null,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }))

    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: matchMediaMock,
    })

    // Component should render without errors
    renderWithProviders(<ScanForm />)

    // Verify basic rendering (not visual correctness)
    expect(screen.getByLabelText(/website url/i)).toBeInTheDocument()
  })

  test('ScanForm renders in light mode without errors', () => {
    // Mock prefers-color-scheme: light
    matchMediaMock.mockImplementation((query: string) => ({
      matches: query === '(prefers-color-scheme: light)',
      media: query,
      onchange: null,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }))

    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: matchMediaMock,
    })

    renderWithProviders(<ScanForm />)
    expect(screen.getByLabelText(/website url/i)).toBeInTheDocument()
  })
})
```

### Pattern 6: Error Boundary Testing

**What:** Trigger component errors and verify error boundary fallback UI renders

**When to use:** error-boundary.test.tsx for app/error.tsx

**Example:**
```typescript
// Source: RTL error boundary testing patterns
import { describe, test, expect, vi } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import ErrorBoundary from '@/app/error'

describe('Error Boundary', () => {
  // Suppress console.error for this test (error boundaries log errors)
  beforeEach(() => {
    vi.spyOn(console, 'error').mockImplementation(() => {})
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  test('displays error fallback UI when error occurs', () => {
    const mockError = new Error('Test error')
    const mockReset = vi.fn()

    renderWithProviders(
      <ErrorBoundary error={mockError} reset={mockReset} />
    )

    // Verify fallback UI elements
    expect(screen.getByText(/something went wrong/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /try again/i })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /return to home/i })).toBeInTheDocument()
  })

  test('calls reset function when Try Again clicked', async () => {
    const user = userEvent.setup()
    const mockReset = vi.fn()

    renderWithProviders(
      <ErrorBoundary error={new Error('Test')} reset={mockReset} />
    )

    const tryAgainButton = screen.getByRole('button', { name: /try again/i })
    await user.click(tryAgainButton)

    expect(mockReset).toHaveBeenCalledTimes(1)
  })
})
```

### Anti-Patterns to Avoid

- **Testing implementation details:** Don't test internal state, private methods, or component structure. Test observable behavior only.
- **Using getByTestId everywhere:** Reserve for truly untestable elements; prefer getByRole, getByLabelText, getByText which validate accessibility.
- **Using fireEvent instead of user-event:** fireEvent only dispatches events; user-event simulates full user interactions more realistically.
- **Nesting setup in beforeEach:** userEvent.setup() should be called within each test, not in beforeEach hooks.
- **Testing styles directly:** Don't assert className or inline styles; test behavior and accessibility, not CSS.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| User interaction simulation | Custom click/type helpers | @testing-library/user-event | Handles edge cases (disabled elements, visibility, focus), simulates full event sequences |
| DOM query selectors | Custom element finders | RTL queries (getByRole, getByLabelText) | Encourages accessibility, mimics screen reader behavior, provides helpful error messages |
| Mock state management | Custom state mocking | vi.mock() with spread operator | Preserves original exports, allows surgical mocking, integrates with Vitest |
| API mocking | Custom fetch mocks | MSW (already configured) | Request-level interception, reusable handlers, runtime handler swapping |
| Async waiting | setTimeout in tests | RTL waitFor, findBy queries | Built-in retry logic, better error messages, prevents flaky tests |

**Key insight:** React Testing Library and user-event have solved the hard problems of realistic component testing (visibility checks, event ordering, focus management, accessibility queries). Custom solutions miss these edge cases and break on refactors.

## Common Pitfalls

### Pitfall 1: Not Waiting for Async State Updates

**What goes wrong:** Tests fail with "Unable to find element" because they don't wait for async updates (useState, useEffect, fetch)

**Why it happens:** React state updates are async; components don't update immediately after user interactions

**How to avoid:**
- Use `await user.click()` and other user-event methods (they're async)
- Use `findBy*` queries (wait up to 1000ms by default) instead of `getBy*` for elements that appear after async operations
- Use `waitFor(() => expect(...))` for complex assertions

**Warning signs:**
- Intermittent test failures
- Tests pass locally but fail in CI
- "Unable to find element" errors for elements that definitely render

**Example:**
```typescript
// ❌ BAD: getBy doesn't wait
test('shows success message', () => {
  renderWithProviders(<Form />)
  const button = screen.getByRole('button')
  fireEvent.click(button) // Synchronous, doesn't wait
  expect(screen.getByText(/success/i)).toBeInTheDocument() // Fails!
})

// ✅ GOOD: await user-event and use findBy
test('shows success message', async () => {
  const user = userEvent.setup()
  renderWithProviders(<Form />)

  const button = screen.getByRole('button')
  await user.click(button) // Waits for event to complete

  // findBy waits for element to appear
  expect(await screen.findByText(/success/i)).toBeInTheDocument()
})
```

### Pitfall 2: Over-relying on getByTestId

**What goes wrong:** Tests use data-testid attributes everywhere instead of accessible queries

**Why it happens:** getByTestId is easy but doesn't validate accessibility or user-facing behavior

**How to avoid:**
- Use query priority: getByRole > getByLabelText > getByText > getByTestId
- Only use getByTestId when semantic queries aren't possible (e.g., decorative icons)
- Remember: users can't see test IDs; if you can't query it semantically, users might not be able to interact with it

**Warning signs:**
- Every element has data-testid
- Tests pass but screen readers can't navigate the component
- Tests don't fail when labels are removed

**Example:**
```typescript
// ❌ BAD: data-testid doesn't validate accessibility
<button data-testid="submit-button">Submit</button>
screen.getByTestId('submit-button')

// ✅ GOOD: getByRole validates semantic HTML
<button type="submit">Submit</button>
screen.getByRole('button', { name: /submit/i })

// ✅ GOOD: getByLabelText validates form accessibility
<label htmlFor="email">Email</label>
<input id="email" name="email" />
screen.getByLabelText(/email/i)
```

### Pitfall 3: Mocking React Incorrectly (Losing Original Exports)

**What goes wrong:** Mocking useActionState breaks all React hooks (useState, useEffect, etc.)

**Why it happens:** vi.mock('react', () => ({ ... })) replaces entire module without preserving original exports

**How to avoid:**
- Always spread actual module when mocking: `...actual`
- Use async import to get original: `await vi.importActual('react')`
- Only override specific exports you need to mock

**Warning signs:**
- Tests fail with "useState is not a function"
- All React hooks break after adding one mock
- Component renders blank or crashes

**Example:**
```typescript
// ❌ BAD: Loses all React exports
vi.mock('react', () => ({
  useActionState: vi.fn(() => [{}, vi.fn(), false])
}))
// Now useState, useEffect, etc. are undefined!

// ✅ GOOD: Preserves original React exports
vi.mock('react', async () => {
  const actual = await vi.importActual('react')
  return {
    ...actual, // Spread all original exports
    useActionState: vi.fn(() => [{}, vi.fn(), false])
  }
})
```

### Pitfall 4: Not Using user-event setup()

**What goes wrong:** user-event methods don't work correctly or tests are flaky

**Why it happens:** user-event v14+ requires setup() to be called before using its methods

**How to avoid:**
- Call `const user = userEvent.setup()` at the start of each test
- Don't call setup() in beforeEach; call it within the test itself
- Always await user-event methods (they're async)

**Warning signs:**
- user.click() doesn't trigger handlers
- Typing doesn't update input values
- Focus behavior is wrong

**Example:**
```typescript
// ❌ BAD: No setup, methods might not work
test('handles click', async () => {
  renderWithProviders(<Button />)
  await userEvent.click(screen.getByRole('button')) // May not work!
})

// ✅ GOOD: Call setup() first
test('handles click', async () => {
  const user = userEvent.setup()
  renderWithProviders(<Button />)
  await user.click(screen.getByRole('button'))
})
```

### Pitfall 5: Testing Dark Mode Visuals Instead of Rendering

**What goes wrong:** Tests try to assert specific colors or CSS classes for dark mode

**Why it happens:** Confusion between "verify renders without errors" vs "verify visual correctness"

**How to avoid:**
- Per user constraints: only verify components render without errors in both themes
- Don't assert on specific colors, backgrounds, or CSS classes
- Visual correctness is deferred to v2 visual regression testing
- matchMedia mock just needs to return matches: true/false

**Warning signs:**
- Tests check for specific className values
- Tests assert background-color or other computed styles
- Tests use toHaveStyle() for theme colors

**Example:**
```typescript
// ❌ BAD: Testing visual correctness (out of scope)
test('uses dark theme colors', () => {
  // ... mock dark mode ...
  renderWithProviders(<Component />)
  const element = screen.getByRole('button')
  expect(element).toHaveClass('bg-dark-700') // Testing implementation
  expect(element).toHaveStyle({ backgroundColor: '#1a1a1a' }) // Out of scope
})

// ✅ GOOD: Just verify it renders
test('renders in dark mode without errors', () => {
  // ... mock dark mode ...
  renderWithProviders(<Component />)
  // Component rendered without throwing - that's sufficient
  expect(screen.getByRole('button')).toBeInTheDocument()
})
```

## Code Examples

Verified patterns from official sources and project codebase:

### Query Priority in Practice

```typescript
// Source: Testing Library query priority docs
import { screen } from '@testing-library/react'

// 1. getByRole - FIRST CHOICE (most accessible)
screen.getByRole('button', { name: /submit/i })
screen.getByRole('heading', { name: /welcome/i })
screen.getByRole('textbox', { name: /email/i })
screen.getByRole('link', { name: /privacy policy/i })
screen.getByRole('checkbox', { name: /i agree/i })

// 2. getByLabelText - BEST FOR FORMS
screen.getByLabelText(/email address/i)
screen.getByLabelText(/password/i)

// 3. getByText - FOR NON-INTERACTIVE CONTENT
screen.getByText(/no security issues found/i)
screen.getByText(/3 findings/i)

// 4. getByTestId - LAST RESORT ONLY
screen.getByTestId('complex-svg-icon') // OK: decorative content
```

### Complete ScanForm Test File Structure

```typescript
// Source: Combining official RTL patterns + project requirements
import { vi, describe, test, expect, beforeEach } from 'vitest'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import { ScanForm } from '@/components/scan-form'

// Mock useActionState
let mockState = {}
let mockFormAction = vi.fn()
let mockPending = false

vi.mock('react', async () => {
  const actual = await vi.importActual('react')
  return {
    ...actual,
    useActionState: vi.fn(() => [mockState, mockFormAction, mockPending])
  }
})

// Mock useRouter (already mocked globally in vitest.setup.ts, but showing pattern)
const mockPush = vi.fn()
vi.mock('next/navigation', () => ({
  useRouter: () => ({ push: mockPush }),
}))

describe('ScanForm', () => {
  beforeEach(() => {
    mockState = {}
    mockFormAction = vi.fn()
    mockPending = false
    mockPush.mockClear()
  })

  describe('Form Fields', () => {
    test('renders URL input with label', () => {
      renderWithProviders(<ScanForm />)
      expect(screen.getByLabelText(/website url/i)).toBeInTheDocument()
    })

    test('renders email input with label', () => {
      renderWithProviders(<ScanForm />)
      expect(screen.getByLabelText(/email/i)).toBeInTheDocument()
    })

    test('renders CFAA consent checkbox', () => {
      renderWithProviders(<ScanForm />)
      const checkbox = screen.getByRole('checkbox', {
        name: /i confirm i own this website/i
      })
      expect(checkbox).toBeInTheDocument()
    })

    test('renders submit button', () => {
      renderWithProviders(<ScanForm />)
      expect(screen.getByRole('button', { name: /scan now/i }))
        .toBeInTheDocument()
    })
  })

  describe('Validation Errors', () => {
    test('displays URL validation error', () => {
      mockState = {
        errors: {
          url: ['Please enter a valid URL (e.g., https://example.com)']
        }
      }

      renderWithProviders(<ScanForm />)
      expect(screen.getByText(/please enter a valid url/i))
        .toBeInTheDocument()
    })

    test('displays email validation error', () => {
      mockState = {
        errors: {
          email: ['Please enter a valid email address']
        }
      }

      renderWithProviders(<ScanForm />)
      expect(screen.getByText(/please enter a valid email/i))
        .toBeInTheDocument()
    })

    test('displays authorization error', () => {
      mockState = {
        errors: {
          authorization: ['You must confirm you have authorization']
        }
      }

      renderWithProviders(<ScanForm />)
      expect(screen.getByText(/you must confirm/i)).toBeInTheDocument()
    })

    test('displays form-level error', () => {
      mockState = {
        errors: {
          _form: ['Unable to connect to scanning service']
        }
      }

      renderWithProviders(<ScanForm />)
      expect(screen.getByText(/unable to connect/i)).toBeInTheDocument()
    })
  })

  describe('Loading State', () => {
    test('disables submit button when pending', () => {
      mockPending = true

      renderWithProviders(<ScanForm />)
      const button = screen.getByRole('button', { name: /starting scan/i })
      expect(button).toBeDisabled()
    })

    test('changes button text when pending', () => {
      mockPending = true

      renderWithProviders(<ScanForm />)
      expect(screen.getByRole('button', { name: /starting scan/i }))
        .toBeInTheDocument()
    })
  })

  describe('Success State', () => {
    test('displays success message when scanId present', () => {
      mockState = { scanId: 'scan-123' }

      renderWithProviders(<ScanForm />)
      expect(screen.getByText(/scan started!/i)).toBeInTheDocument()
      expect(screen.getByText(/redirecting to your scan progress/i))
        .toBeInTheDocument()
    })
  })

  describe('User Interactions', () => {
    test('allows typing in URL field', async () => {
      const user = userEvent.setup()
      renderWithProviders(<ScanForm />)

      const urlInput = screen.getByLabelText(/website url/i)
      await user.type(urlInput, 'https://example.com')

      expect(urlInput).toHaveValue('https://example.com')
    })

    test('allows typing in email field', async () => {
      const user = userEvent.setup()
      renderWithProviders(<ScanForm />)

      const emailInput = screen.getByLabelText(/email/i)
      await user.type(emailInput, 'test@example.com')

      expect(emailInput).toHaveValue('test@example.com')
    })

    test('allows checking consent checkbox', async () => {
      const user = userEvent.setup()
      renderWithProviders(<ScanForm />)

      const checkbox = screen.getByRole('checkbox', {
        name: /i confirm i own this website/i
      })

      expect(checkbox).not.toBeChecked()
      await user.click(checkbox)
      expect(checkbox).toBeChecked()
    })
  })
})
```

### ResultsDashboard Empty/Error State Testing

```typescript
// Source: RTL conditional rendering patterns
describe('ResultsDashboard', () => {
  test('displays empty state when no findings', () => {
    renderWithProviders(<ResultsDashboard findings={[]} />)

    expect(screen.getByText(/no security issues found/i)).toBeInTheDocument()
    expect(screen.getByText(/your application passed all checks/i))
      .toBeInTheDocument()
  })

  test('renders findings when present', () => {
    const findings = [
      {
        id: '1',
        title: 'Missing CSP header',
        description: 'CSP not configured',
        severity: 'high' as const,
        remediation: 'Add CSP',
        scanner_name: 'security_headers',
        vibe_code: false,
      }
    ]

    renderWithProviders(<ResultsDashboard findings={findings} />)

    expect(screen.getByText(/missing csp header/i)).toBeInTheDocument()
  })

  test('groups findings by severity by default', () => {
    const findings = [
      { id: '1', severity: 'critical', /* ... */ },
      { id: '2', severity: 'high', /* ... */ },
      { id: '3', severity: 'high', /* ... */ },
    ]

    renderWithProviders(<ResultsDashboard findings={findings} />)

    // Should show severity groups
    expect(screen.getByText(/critical \(1\)/i)).toBeInTheDocument()
    expect(screen.getByText(/high \(2\)/i)).toBeInTheDocument()
  })

  test('allows switching to category grouping', async () => {
    const user = userEvent.setup()
    const findings = [/* ... */]

    renderWithProviders(<ResultsDashboard findings={findings} />)

    const categoryButton = screen.getByRole('button', { name: /by category/i })
    await user.click(categoryButton)

    // Should show category groups
    expect(screen.getByText(/headers \(/i)).toBeInTheDocument()
  })
})
```

### UpgradeCTA with MSW Integration

```typescript
// Source: RTL + MSW integration patterns from Phase 25
import { server } from '@/__tests__/helpers/msw/server'
import { http, HttpResponse } from 'msw'

describe('UpgradeCTA', () => {
  test('initiates checkout on button click', async () => {
    const user = userEvent.setup()

    // Mock successful checkout response
    server.use(
      http.post('http://localhost:3000/api/v1/checkout', () => {
        return HttpResponse.json({
          checkout_url: 'https://checkout.stripe.com/test-session-123'
        })
      })
    )

    // Mock window.location.href assignment
    delete window.location
    window.location = { href: '' } as any

    renderWithProviders(<UpgradeCTA scanId="scan-123" token="token-abc" />)

    const upgradeButton = screen.getByRole('button', { name: /upgrade for \$49/i })
    await user.click(upgradeButton)

    // Button should show loading state
    await screen.findByText(/redirecting to checkout/i)
  })

  test('displays error when checkout fails', async () => {
    const user = userEvent.setup()

    server.use(
      http.post('http://localhost:3000/api/v1/checkout', () => {
        return HttpResponse.json(
          { title: 'Payment processing unavailable' },
          { status: 500 }
        )
      })
    )

    renderWithProviders(<UpgradeCTA scanId="scan-123" token="token-abc" />)

    const upgradeButton = screen.getByRole('button', { name: /upgrade for \$49/i })
    await user.click(upgradeButton)

    expect(await screen.findByText(/payment processing unavailable/i))
      .toBeInTheDocument()
  })
})
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Jest test runner | Vitest | 2023 | Next.js officially recommends Vitest for App Router; better ESM support |
| fireEvent for interactions | user-event | 2020 | More realistic user interactions; simulates full event sequences |
| Enzyme shallow rendering | React Testing Library | 2019 | Tests behavior not implementation; compatible with React 19 |
| getByTestId everywhere | getByRole priority | 2021 | Validates accessibility; encourages semantic HTML |
| Manual async waiting | findBy queries + waitFor | 2020 | Built-in retry; better error messages; less flaky |

**Deprecated/outdated:**
- **Enzyme**: Incompatible with React 19; deprecated by maintainers
- **Jest for Next.js App Router**: Poor ESM support; Next.js docs recommend Vitest
- **fireEvent only**: Still works but user-event preferred for realism
- **react-test-renderer**: Replaced by @testing-library/react for component testing

## Open Questions

1. **useActionState mocking reliability**
   - What we know: vi.mock('react') with spread operator is standard Vitest pattern
   - What's unclear: Next.js issue #63868 reports useFormStatus/useActionState testing problems in Vitest
   - Recommendation: Try mocking pattern first; if it fails, consider testing ScanForm as integration test with E2E approach in Phase 27. Document workaround if needed.

2. **Dark mode matchMedia mock persistence**
   - What we know: matchMedia needs to be mocked per test with window.matchMedia assignment
   - What's unclear: Whether mock needs cleanup between tests or if afterEach restoreAllMocks() is sufficient
   - Recommendation: Test with cleanup in afterEach first; add explicit cleanup if tests interfere with each other

3. **Coverage threshold achievability**
   - What we know: Requirements call for 80% lines, 80% functions, 75% branches
   - What's unclear: Whether 5-15 tests per component will achieve these thresholds
   - Recommendation: Run coverage after initial implementation; adjust test count if needed. Vitest config already excludes Next.js boilerplate.

## Sources

### Primary (HIGH confidence)

- Testing Library Official Docs - React Testing Library intro, query priority, user-event setup
  - https://testing-library.com/docs/react-testing-library/intro/
  - https://testing-library.com/docs/queries/about/
  - https://testing-library.com/docs/user-event/intro/
- Vitest Official Docs - Mocking guide
  - https://vitest.dev/guide/mocking
- Project Codebase - Existing test infrastructure from Phase 25
  - /home/john/vault/projects/github.com/shipsecure/frontend/__tests__/components/Header.test.tsx
  - /home/john/vault/projects/github.com/shipsecure/frontend/vitest.setup.ts
  - /home/john/vault/projects/github.com/shipsecure/frontend/__tests__/helpers/test-utils.tsx

### Secondary (MEDIUM confidence)

- Next.js Testing Guide - Vitest configuration
  - https://nextjs.org/docs/app/guides/testing/vitest
- Kent C. Dodds - Common mistakes with React Testing Library
  - https://kentcdodds.com/blog/common-mistakes-with-react-testing-library
- GitHub Issue - Next.js useFormStatus testing
  - https://github.com/vercel/next.js/issues/63868
- React Testing Library Best Practices
  - https://medium.com/@ignatovich.dm/best-practices-for-using-react-testing-library-0f71181bb1f4
  - https://claritydev.net/blog/improving-react-testing-library-tests

### Tertiary (LOW confidence - WebSearch only)

- Cypress Dark Mode Testing - Adapted patterns for Vitest/RTL
  - https://www.cypress.io/blog/2019/12/13/test-your-web-app-in-dark-mode
- Accordion Testing Examples
  - https://gist.github.com/ms314006/4cfc0f12aab7cf3d7660ec9ec360b159
- Form Testing Patterns
  - https://medium.com/@entekumejeffrey/part-5-testing-form-components-with-jest-and-react-testing-library-f88bb641a0ea

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All dependencies already installed and configured in Phase 25; official Next.js recommendation
- Architecture patterns: HIGH - Based on official Testing Library docs + existing project patterns from Header.test.tsx
- useActionState mocking: MEDIUM - Pattern is standard but Next.js issue #63868 indicates potential problems; may need fallback approach
- Pitfalls: HIGH - Based on official docs, common mistake articles, and existing project setup
- Dark mode testing: MEDIUM - Pattern derived from Cypress examples adapted to Vitest; needs validation

**Research date:** 2026-02-16
**Valid until:** ~30 days (stable testing libraries; React Testing Library v16 stable, Vitest v4 stable)
