---
phase: 15-layout-refactor
plan: 01
subsystem: design-tokens
tags: [layout, css-variables, design-tokens, phase-preparation]
dependency_graph:
  requires:
    - "13-01: OKLch primitives and @theme inline semantic tokens"
    - "13-02: Design token migration (dark mode semantic overrides)"
  provides:
    - "Layout dimension token --header-height: 64px available globally"
    - "Header insertion point documented in root layout for Phase 16"
  affects:
    - "Phase 16: Header component will consume --header-height token"
    - "Future layout changes: Single source of truth for header height"
tech_stack:
  added:
    - "CSS Custom Property: --header-height (layout dimension token)"
  patterns:
    - "@theme inline directive for Tailwind utility generation"
    - "Layout dimensions separate from color themes (no dark mode override)"
key_files:
  created: []
  modified:
    - path: "frontend/app/globals.css"
      summary: "Added --header-height: 64px in @theme inline block with documentation"
      lines_changed: 8
    - path: "frontend/app/layout.tsx"
      summary: "Added Phase 16 header insertion point comment"
      lines_changed: 1
decisions:
  - decision: "Define layout dimension token in light mode @theme inline only"
    rationale: "Layout dimensions don't change between color modes - only define once to avoid duplication and maintain consistency"
    alternatives: "Could define in both light and dark mode blocks, but unnecessary and error-prone"
  - decision: "No spacer div or padding-top added in this phase"
    rationale: "Adding 64px offset before header exists would create unexplained whitespace, violating 'no layout shift' success criterion"
    alternatives: "Add pt-[--header-height] now, but would break current visual appearance"
  - decision: "Document header slot with comment only, defer structural changes"
    rationale: "Phase 16 will handle actual layout restructuring - this phase only establishes token foundation"
    alternatives: "Could add empty <header> element now, but unnecessary DOM node"
metrics:
  duration_minutes: 1
  tasks_completed: 2
  files_modified: 2
  build_time_seconds: 3
  completed_date: "2026-02-11"
---

# Phase 15 Plan 01: Layout Token Definition Summary

**One-liner:** Define --header-height: 64px CSS custom property in design token system with documented header insertion point for zero-layout-shift Phase 16 integration

## Objective

Define the `--header-height: 64px` CSS custom property in the design token system and prepare the root layout for Phase 16 header integration, without changing any visual output on existing routes.

## Tasks Completed

### Task 1: Define header-height layout token and prepare layout structure
**Status:** ✅ Complete
**Commit:** bb33706

Added `--header-height: 64px` CSS variable inside the existing `@theme inline` block in `globals.css`, positioned after all color tokens (line 217). Included comprehensive documentation comment explaining:
- Purpose: Reserved for Phase 16 sticky header
- Usage: Header height and main content padding-top offset
- Caution: Do not change without verifying all routes

Added Phase 16 header insertion point comment in `layout.tsx` at line 46, marking exactly where the `<Header />` component will be inserted. No structural changes to the existing flexbox layout - comment is purely documentation.

**Files modified:**
- `frontend/app/globals.css`: +8 lines (token definition + documentation)
- `frontend/app/layout.tsx`: +1 line (insertion point comment)

**Verification:**
- ✅ Build passes: `npx next build` succeeds in 3.2s
- ✅ Token defined: `grep -- '--header-height: 64px' frontend/app/globals.css` returns match at line 217
- ✅ Comment exists: `grep 'Phase 16.*Header' frontend/app/layout.tsx` returns match at line 46

### Task 2: Verify all routes maintain current spacing with no layout shift
**Status:** ✅ Complete
**Type:** Verification only (no files modified)

Ran comprehensive verification to ensure the token addition has zero impact on existing routes:

1. **Build verification**: Full production build succeeds, all 10 routes pre-render successfully (TypeScript compilation, CSS processing, static generation all pass)

2. **Conflicting usage check**: `grep -r "header-height" frontend/` returns ONLY `globals.css` - no other files reference the token (layout.tsx comment says "Header", not "header-height")

3. **h-screen conflict check**: `grep -r "\bh-screen\b" frontend/` returns ZERO matches - excellent finding, means no fixed-height conflicts

4. **min-h-screen pattern verification**: 8 files use `min-h-screen` (layout.tsx, 4 page routes, 2 loading/error states, global error boundary) - this is the CORRECT pattern for sticky header compatibility, as `min-h-screen` is flexible and won't conflict with header offset

5. **global-error.tsx isolation**: Uses inline `minHeight: '100vh'` style, renders outside root layout, unaffected by token definition

6. **Tailwind utility availability**: `@theme inline` directive automatically generates utility class support, so `h-[--header-height]` will work in Phase 16 without additional configuration

**Key Finding:** The codebase exclusively uses `min-h-screen` (flexible) rather than `h-screen` (fixed), which is exactly the pattern needed for Phase 16 sticky header. No layout conflicts exist. No pages will experience layout shift when header is added.

## Deviations from Plan

None - plan executed exactly as written.

## Success Criteria

- ✅ CSS variable `--header-height: 64px` is defined in globals.css `@theme inline` block
- ✅ layout.tsx has documented header insertion point for Phase 16
- ✅ Next.js build passes with zero errors (3.2s compilation, all routes pre-render)
- ✅ No existing route spacing is affected (token defined but unused)
- ✅ No `h-screen` conflicts exist in the codebase (all pages use safe `min-h-screen`)

## Technical Details

### Token Definition Location

Added inside the light mode `@theme inline` block (lines 114-209) ONLY, not in the dark mode `@media (prefers-color-scheme: dark)` override block. This is correct because:
- Layout dimensions are color-mode-independent (header height doesn't change in dark mode)
- Defining once avoids duplication and maintains single source of truth
- Dark mode semantic token overrides are for colors only

### Layout Structure

Current layout structure (unchanged):
```tsx
<div className="flex flex-col min-h-screen">
  {/* Phase 16: Insert <Header /> sticky component here */}
  <div className="flex-1">
    {children}
  </div>
  <Footer />
</div>
```

No spacer div, no padding-top, no visual changes. Comment documents the insertion point. The flexbox layout (`flex-col min-h-screen` + `flex-1` on content) ensures content stretches to fill available space.

### Why No Padding/Spacer Now

Adding `pt-[--header-height]` or a 64px spacer div before the header exists would add unexplained whitespace to every page, violating the "no layout shift" success criterion. Phase 16 will add BOTH the header AND the offset simultaneously, maintaining zero layout shift.

### Phase 16 Integration Pattern

When Phase 16 adds the sticky header, the pattern will be:
```tsx
<div className="flex flex-col min-h-screen">
  <Header className="sticky top-0 h-[--header-height]" />
  <div className="flex-1">
    {children}
  </div>
  <Footer />
</div>
```

The existing `min-h-screen` on the container and `flex-1` on content ensure proper spacing automatically. No additional padding needed.

## Impact Analysis

### Immediate Impact
- Zero visual changes to any route
- Token available but not consumed by any component
- Build time unchanged (3.2s)
- No performance impact

### Phase 16 Preparation
- Header height value centralized (single source of truth)
- Insertion point clearly documented
- Layout structure verified compatible (min-h-screen pattern throughout)
- No refactoring needed when header is added

### Risk Mitigation
- Comprehensive verification ensures no unintended side effects
- No `h-screen` conflicts in codebase (would cause overlap issues)
- All pages use flexible `min-h-screen` (safe for sticky header)
- Token documentation warns against changing without verification

## Files Changed

| File | Changes | Purpose |
|------|---------|---------|
| frontend/app/globals.css | +8 lines | Define --header-height: 64px token with documentation |
| frontend/app/layout.tsx | +1 line | Document Phase 16 header insertion point |

**Total:** 2 files modified, 9 lines added, 0 lines removed

## Next Steps

**For Phase 16 (Header Component):**
1. Create `<Header />` component with `sticky top-0 h-[--header-height]` classes
2. Insert header at documented location in layout.tsx (line 46)
3. Verify all routes maintain spacing with header present
4. Test sticky behavior on scroll across all routes

**No further layout preparation needed** - this phase establishes the foundation, Phase 16 consumes it.

## Self-Check: PASSED

**Files created:** None (0/0)

**Files modified:**
- ✅ FOUND: frontend/app/globals.css (contains --header-height: 64px at line 217)
- ✅ FOUND: frontend/app/layout.tsx (contains Phase 16 comment at line 46)

**Commits:**
- ✅ FOUND: bb33706 (feat(15-01): define header-height layout token and prepare layout structure)

All claims verified. Summary accurate.
