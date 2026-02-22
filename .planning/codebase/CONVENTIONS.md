# Coding Conventions

**Analysis Date:** 2026-02-21

## Naming Patterns

**Files:**
- Component files: kebab-case (e.g., `scan-form.tsx`, `results-dashboard.tsx`)
- Page files: use Next.js App Router conventions (e.g., `app/page.tsx`, `app/scan/[id]/page.tsx`)
- Test files: mirror source structure with `.test.tsx` suffix (e.g., `__tests__/components/ScanForm.test.tsx`)
- Server action files: lowercase with descriptive names (e.g., `app/actions/scan.ts`)
- Config files: descriptive with extensions (e.g., `vitest.config.ts`, `eslint.config.mjs`)

**Functions:**
- Component functions: PascalCase (e.g., `ScanForm`, `ResultsDashboard`, `Header`)
- Exported functions: PascalCase for components, camelCase for utilities
- Server actions: camelCase (e.g., `submitScan`)
- Event handlers in components: camelCase, descriptive names (e.g., `handleClick`, `getScannerDisplayName`, `groupBySeverity`)

**Variables:**
- React hooks: camelCase (e.g., `useState`, `useEffect`, `mockState`, `mockFormAction`)
- State variables: descriptive camelCase (e.g., `grouping`, `findings`, `status`, `framework`, `pending`)
- Constants: camelCase or UPPER_SNAKE_CASE for immutable values (e.g., `sizeMap`, `BASE_URL`, `BACKEND_URL`)
- Type instances: camelCase (e.g., `testFindings`, `mockReset`, `scanFixtures`)

**Types:**
- Interfaces: PascalCase (e.g., `LogoProps`, `ResultsDashboardProps`, `ScanFormState`, `Finding`)
- Interfaces suffixed with `Props` for component prop definitions
- Type unions: PascalCase (e.g., `GroupingMode`)
- Type aliases for discriminated unions use `as const` (e.g., `scanFixtures` uses `as const`)

## Code Style

**Formatting:**
- No explicit Prettier config detected; defaults used
- Indentation: 2 spaces
- Line length: observed lines up to 120+ characters used naturally
- Semicolons: consistently used
- Quotes: single quotes in TypeScript/JavaScript, double quotes in JSX attributes

**Linting:**
- ESLint v9 with Next.js core web vitals and TypeScript plugins
- Config: `frontend/eslint.config.mjs`
- Uses recommended Next.js rules via `eslint-config-next/core-web-vitals` and `eslint-config-next/typescript`
- Runs via `npm run lint` command

**TypeScript Configuration:**
- Target: ES2017
- Strict mode enabled
- Module resolution: bundler
- Path aliases: `@/*` maps to root directory for imports
- Config: `frontend/tsconfig.json`

## Import Organization

**Order:**
1. React imports (e.g., `import { useState } from 'react'`)
2. Next.js imports (e.g., `import Link from 'next/link'`, `import Image from 'next/image'`)
3. Third-party library imports (e.g., `import { z } from 'zod'`, `import { http, HttpResponse } from 'msw'`)
4. Local imports using `@/` alias (e.g., `import { ScanForm } from '@/components/scan-form'`)

**Path Aliases:**
- `@/*` resolves to frontend root directory
- Used consistently across components, tests, and server actions
- Examples: `@/components/scan-form`, `@/app/actions/scan`, `@/__tests__/helpers/test-utils`

## Error Handling

**Patterns:**
- Server actions: Return state objects with `errors` property containing field-level error arrays (e.g., `{ errors: { url: ['error message'] } }`)
- Error state keys: `_form` for form-level/submission errors, field names for field-specific errors
- Try/catch in async operations: Catch errors and transform to user-facing messages
- Error boundary: Root error component at `app/error.tsx` logs to console and shows fallback UI
- Network errors: Caught and transformed to generic user messages (e.g., "Unable to connect to the scanning service")
- Rate limiting: Special handling with countdown timer for 429 responses

Example from `app/actions/scan.ts`:
```typescript
if (!validatedFields.success) {
  return {
    errors: validatedFields.error.flatten().fieldErrors,
  }
}

try {
  // operation
} catch (error) {
  return {
    errors: { _form: ['Unable to connect to the scanning service. Please try again later.'] }
  }
}
```

## Logging

**Framework:** `console` (direct console methods)

**Patterns:**
- Error logging: `console.error()` used in error boundaries (see `app/error.tsx` line 14)
- Debug logging: Not observed in production code
- Analytics: `window.plausible?.()` for event tracking (e.g., in `components/scan-form.tsx` line 13)
- No structured logging or logging library detected

## Comments

**When to Comment:**
- Complex logic with multiple branches or calculations (e.g., countdown timer calculation in `app/actions/scan.ts` lines 99-105)
- Non-obvious algorithm or business logic
- Workarounds or temporary solutions
- Setup/teardown documentation in test files

**JSDoc/TSDoc:**
- Minimal usage observed
- Types inferred from TypeScript annotations
- Function documentation via inline comments where needed (e.g., `renderWithProviders` has comment block in `__tests__/helpers/test-utils.tsx`)
- Component prop interfaces document via TypeScript types, not JSDoc

## Function Design

**Size:** Functions are compact, typically 15-40 lines
- Handlers and event processors: 5-20 lines
- Complex grouping/transformation logic: 15-30 lines
- Server actions: 50-90 lines when including validation and error handling

**Parameters:**
- React components: Single props parameter typed via interface
- Utility functions: Specific parameters, no large option objects unless complex
- Event handlers: Use inferred types from React
- Functions with multiple concerns use destructuring (e.g., `function ScanForm()` receives nothing, uses hooks internally)

**Return Values:**
- Components: Return JSX.Element
- Server actions: Return state objects (interface-typed)
- Utilities: Typed returns via TypeScript
- Conditional logic uses early returns to reduce nesting depth

## Module Design

**Exports:**
- Named exports for components: `export function ComponentName() { ... }`
- Named exports for utilities and types
- Index barrel files: Not used in this codebase
- Type-only imports used consistently: `import type { ScanFormState } from '@/app/actions/scan'`

**Barrel Files:**
- Not observed in this codebase
- Each component file exports its single component
- Tests import directly from source files: `import { ScanForm } from '@/components/scan-form'`

**Organization by Feature:**
- `components/`: Pure UI components (scan-form, results-dashboard, header, etc.)
- `app/`: Page components and server actions (Next.js App Router)
- `lib/`: Shared types and utilities
- `__tests__/`: Test files mirroring `components/` structure, plus helpers

---

*Convention analysis: 2026-02-21*
