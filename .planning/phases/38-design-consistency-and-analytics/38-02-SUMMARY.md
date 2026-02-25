---
phase: 38-design-consistency-and-analytics
plan: "02"
subsystem: ui
tags: [tailwind, css-variables, design-tokens, next.js]

# Dependency graph
requires:
  - phase: 38-design-consistency-and-analytics-01
    provides: existing design token patterns in globals.css
provides:
  - "--card-radius CSS custom property (0.75rem) in globals.css @theme inline block"
  - "Unified rounded-(card) utility applied to all card/panel containers in scan, results, and error pages"
affects:
  - future phases adding new card surfaces (should use rounded-(card))
  - any UI audit or design consistency work

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "CSS custom property design tokens in Tailwind v4 @theme inline block"
    - "rounded-(card) Tailwind v4 CSS-variable utility for card border radius"
    - "Scope rule: only outer card container divs get card-radius; buttons/inputs/circles keep their own radius"

key-files:
  created: []
  modified:
    - frontend/app/globals.css
    - frontend/app/scan/[id]/page.tsx
    - frontend/app/results/[token]/page.tsx
    - frontend/app/results/[token]/loading.tsx
    - frontend/app/results/[token]/error.tsx
    - frontend/app/error.tsx

key-decisions:
  - "--card-radius set to 0.75rem (12px = rounded-xl) matching the dominant dashboard card radius rather than the previously-used rounded-lg (8px) on scan pages — scan pages updated to match"
  - "Buttons retain rounded-lg, spinners/circles retain rounded-full — scope boundary enforced at card container level only"

patterns-established:
  - "Card container pattern: bg-surface-elevated rounded-(card) shadow-md — all card surfaces use this"
  - "Small inline alerts/badges inside cards keep their own rounded-md for visual hierarchy"

requirements-completed:
  - DESIGN-01

# Metrics
duration: 1min
completed: 2026-02-25
---

# Phase 38 Plan 02: Card Radius Design Token Summary

**`--card-radius: 0.75rem` token defined in globals.css and applied via `rounded-(card)` to 12 card container elements across 5 pages, replacing the previous mix of rounded-lg/rounded-xl**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-02-25T03:36:23Z
- **Completed:** 2026-02-25T03:41:00Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Defined `--card-radius: 0.75rem` inside `@theme inline` Layout Dimensions section of globals.css with documentation comment
- Applied `rounded-(card)` to 4 card containers in scan/[id]/page.tsx (loading, not-found, failed, and progress states)
- Applied `rounded-(card)` to 3 card containers in results/[token]/page.tsx (in-progress state + 2 content panels)
- Applied `rounded-(card)` to 3 skeleton card containers in results/[token]/loading.tsx
- Applied `rounded-(card)` to outer card in results/[token]/error.tsx
- Applied `rounded-(card)` to outer card in app/error.tsx (root error boundary)
- TypeScript compilation passes with no errors

## Task Commits

Each task was committed atomically:

1. **Task 1: Define --card-radius token in globals.css** - `eb32c05` (feat)
2. **Task 2: Apply card-radius token to scan/results/error card elements** - `b0dbd55` (feat)

**Plan metadata:** (final commit hash — see below)

## Files Created/Modified
- `frontend/app/globals.css` - Added --card-radius: 0.75rem to @theme inline Layout Dimensions section
- `frontend/app/scan/[id]/page.tsx` - 4 outer card divs now use rounded-(card)
- `frontend/app/results/[token]/page.tsx` - 3 outer card divs now use rounded-(card)
- `frontend/app/results/[token]/loading.tsx` - 3 skeleton card divs now use rounded-(card)
- `frontend/app/results/[token]/error.tsx` - outer card div now uses rounded-(card)
- `frontend/app/error.tsx` - outer card div now uses rounded-(card)

## Decisions Made
- Value set to `0.75rem` (12px = rounded-xl) — this matches the dashboard card radius. Scan progress pages previously used `rounded-lg` (8px) and were updated to match the rest of the app.
- Scope: only outer card container divs received `rounded-(card)`. Buttons retain `rounded-lg`, spinners retain `rounded-full`, small inline alert divs inside cards retain `rounded-md` for visual hierarchy within cards.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Card radius token is now the single source of truth — any future card surfaces should use `rounded-(card)`
- Ready for any further design consistency work (e.g., spacing tokens, shadow tokens)

---
*Phase: 38-design-consistency-and-analytics*
*Completed: 2026-02-25*
