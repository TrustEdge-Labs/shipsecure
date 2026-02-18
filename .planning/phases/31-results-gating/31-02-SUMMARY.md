---
phase: 31-results-gating
plan: "02"
subsystem: ui
tags: [nextjs, react, clerk, typescript, auth-gate, results, gating]

# Dependency graph
requires:
  - phase: 31-results-gating/31-01
    provides: gated field per-finding and owner_verified in backend API response
  - phase: 29-auth-foundation
    provides: Clerk Next.js SDK and auth() Server Component helper

provides:
  - AuthGate client component with lock overlay and Clerk SignUp modal CTA
  - FindingAccordion wraps body content in AuthGate for server-driven gating
  - Results page Server Component forwards Clerk session JWT to backend via Authorization header
  - Download button hidden for non-owners via owner_verified conditional render

affects: [32-domain-verification, 33-rate-limiting]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "AuthGate pattern: client component receiving gated bool from server-rendered data; shows lock overlay or passes children through"
    - "Server Component JWT forwarding: auth().getToken() in Next.js Server Component, conditional Authorization header on backend fetch"
    - "Strict equality for optional bool: finding.gated === true handles undefined (missing field) safely without coercion"

key-files:
  created:
    - frontend/components/auth-gate.tsx
  modified:
    - frontend/lib/types.ts
    - frontend/components/finding-accordion.tsx
    - frontend/app/results/[token]/page.tsx

key-decisions:
  - "AuthGate receives pre-computed gated bool from server-rendered finding data — no client-side auth check needed"
  - "generateMetadata also forwards session token — consistent auth posture even for metadata fetches"
  - "Spacer div in AuthGate lock overlay maintains accordion height for visual continuity"

patterns-established:
  - "Lock overlay pattern: absolute inset-0 overlay over invisible spacer div; spacer sets height, overlay shows CTA"
  - "Server Component auth forwarding: auth() from @clerk/nextjs/server, getToken(), conditional header"

requirements-completed: [GATE-03, GATE-04]

# Metrics
duration: 2min
completed: 2026-02-18
---

# Phase 31 Plan 02: Results Gating Summary

**AuthGate client component with Clerk SignUp CTA and Next.js Server Component JWT forwarding; unauthenticated users see lock overlay on high/critical findings, owners see full details**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-18T11:51:54Z
- **Completed:** 2026-02-18T11:54:11Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Created `AuthGate` client component: renders lock overlay with severity label, scanner category, and "Sign up free to view" button that calls Clerk `openSignUp({})` modal; passes children through when not gated
- Updated `Finding` TypeScript interface: `description` and `remediation` are now `string | null` (null when gated), `gated?: boolean` added; `ScanResponse` gets `owner_verified: boolean`
- `FindingAccordion` wraps description/remediation body in `<AuthGate gated={finding.gated === true} ...>` — strict equality safely handles the undefined case
- Results page Server Component calls `auth().getToken()` from `@clerk/nextjs/server` and attaches `Authorization: Bearer <token>` to backend fetch; both main page and `generateMetadata` use this pattern
- Download Markdown Report button is conditionally rendered: `{data.owner_verified && (...)}` hides it for non-owners

## Task Commits

Each task was committed atomically:

1. **Task 1: Update TypeScript types and create AuthGate component** - `3c0ac6e` (feat)
2. **Task 2: Integrate AuthGate into FindingAccordion and forward auth token in results page** - `4eed227` (feat)

**Plan metadata:** _(docs commit pending)_

## Files Created/Modified

- `frontend/components/auth-gate.tsx` - Lock overlay client component using Clerk `openSignUp()`; `use client`, `useClerk` hook, severity/scanner display, invisible spacer for height
- `frontend/lib/types.ts` - `Finding.description` and `.remediation` changed to `string | null`; `Finding.gated?: boolean` added; `ScanResponse.owner_verified: boolean` added
- `frontend/components/finding-accordion.tsx` - Imports `AuthGate` and wraps expandable body; passes `finding.gated === true`, `finding.severity`, and display scanner name
- `frontend/app/results/[token]/page.tsx` - Imports `auth` from `@clerk/nextjs/server`; extracts and forwards session JWT; conditionally renders download button on `owner_verified`

## Decisions Made

- AuthGate receives a pre-computed `gated` boolean from server-rendered finding data — no client-side JWT check needed; the server already computed gating via Plan 01 backend logic
- `generateMetadata` also forwards session token — consistent auth posture; minor benefit but matches the main handler pattern
- Spacer `<div className="invisible">` in the lock overlay maintains accordion content height so the expanded state looks intentional rather than collapsed

## Deviations from Plan

None — plan executed exactly as written. All implementation details (strict equality, both fetch handlers, spacer div) followed the plan specification.

## Issues Encountered

None — plan executed smoothly. Types, build, and all integration grep checks passed on first attempt.

## User Setup Required

None - no external service configuration required. Clerk SDK was already installed in Phase 29.

## Next Phase Readiness

- Frontend gating complete — Phase 31 is fully done (backend gating in 31-01, frontend AuthGate in 31-02)
- End-to-end flow: anonymous user hits results page → Server Component fetches without JWT → backend returns `gated: true` + null details for high/critical → AuthGate shows lock overlay with sign-up CTA
- Authenticated owner flow: Server Component fetches with JWT → backend returns `owner_verified: true` + full details → AuthGate passes children through, download button visible
- Phase 32 (domain verification) can proceed; no dependencies on this phase

---
*Phase: 31-results-gating*
*Completed: 2026-02-18*

## Self-Check: PASSED

- FOUND: frontend/components/auth-gate.tsx
- FOUND: frontend/lib/types.ts
- FOUND: frontend/components/finding-accordion.tsx
- FOUND: frontend/app/results/[token]/page.tsx
- FOUND: .planning/phases/31-results-gating/31-02-SUMMARY.md
- FOUND commit 3c0ac6e (Task 1)
- FOUND commit 4eed227 (Task 2)
