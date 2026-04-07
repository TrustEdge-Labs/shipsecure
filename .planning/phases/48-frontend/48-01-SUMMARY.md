---
phase: 48-frontend
plan: 01
subsystem: ui
tags: [react, typescript, nextjs, supply-chain, rust, axum]

# Dependency graph
requires:
  - phase: 47-backend
    provides: POST /api/v1/scans/supply-chain endpoint with 3 input modes + RFC 7807 error responses
provides:
  - TypeScript types for supply chain API response (supply-chain-types.ts)
  - 3-tab form component (GitHub URL / Upload / Paste) with loading + error states
  - /supply-chain page route with OG metadata
  - Client-side fetch action with per-mode Content-Type handling
  - Backend results endpoint patched to return kind + supply_chain_results for all scans
affects:
  - 48-02 (results page reads kind + supply_chain_results from patched results endpoint)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Client-side async fetch function (not Next.js server action) for multipart upload support
    - Drag-and-drop file zone with hidden input + ref pattern
    - sessionStorage fallback when share link is unavailable

key-files:
  created:
    - frontend/lib/supply-chain-types.ts
    - frontend/app/actions/supply-chain-scan.ts
    - frontend/components/supply-chain-form.tsx
    - frontend/app/supply-chain/page.tsx
  modified:
    - src/api/results.rs

key-decisions:
  - "Used client-side fetch (not Next.js server action) for submitSupplyChainScan — server actions cannot handle multipart FormData for file uploads in this pattern"
  - "sessionStorage key 'supply-chain-inline-results' as fallback when share_unavailable=true, redirects to /supply-chain/results/inline"
  - "Pre-existing next build failure is Clerk publishableKey placeholder in local .env.local — confirmed pre-existing before our changes"

patterns-established:
  - "Supply chain form uses useState tabs with conditional rendering, not URL routing"
  - "Error mapping from RFC 7807 error type URL to user-friendly string in mapErrorResponse()"

requirements-completed: [FE-01, FE-02, FE-04]

# Metrics
duration: 25min
completed: 2026-04-07
---

# Phase 48 Plan 01: Supply Chain Input Page Summary

**3-tab supply chain form (GitHub URL / Upload / Paste) with loading spinner, RFC 7807 error mapping, and backend results endpoint patched to expose kind + supply_chain_results**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-04-07T13:40:00Z
- **Completed:** 2026-04-07T14:05:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- TypeScript types matching backend response shape exactly (SupplyChainFinding, UnscannedDep, SupplyChainResults, SupplyChainScanResponse, SupplyChainResultsPageData)
- Client-side fetch action handles 3 Content-Types: JSON for github/paste modes, multipart/form-data for upload
- Form component: segmented tab control, drag-and-drop drop zone, textarea with font-mono, loading spinner during fetch
- Error messages mapped from RFC 7807 error type URLs to user-friendly strings per D-09
- Results endpoint (GET /api/v1/results/:token) now returns `kind` and `supply_chain_results` for both active and expired scans

## Task Commits

1. **Task 1: Types + server action + backend results patch** - `0ebd972` (feat)
2. **Task 2: Supply chain form component + page route** - `db54329` (feat)

**Plan metadata:** (final docs commit follows)

## Files Created/Modified
- `frontend/lib/supply-chain-types.ts` — TypeScript interfaces for all supply chain API shapes
- `frontend/app/actions/supply-chain-scan.ts` — submitSupplyChainScan() with 3-mode fetch dispatch + error mapping
- `frontend/components/supply-chain-form.tsx` — 3-tab client component with drag-drop, loading, error states
- `frontend/app/supply-chain/page.tsx` — /supply-chain server component page with OG metadata
- `src/api/results.rs` — Patched GET /api/v1/results/:token to include kind + supply_chain_results

## Decisions Made
- Used client-side async function (not Next.js server action with `'use server'`) for `submitSupplyChainScan` — multipart FormData upload requires direct fetch in browser context
- On `share_unavailable=true`: store full response in sessionStorage under `supply-chain-inline-results`, redirect to `/supply-chain/results/inline` for the results page to read
- Plausible event `supply_chain_scan_started` fires with `input_method` prop on form submit

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered
- `npx next build` fails with `@clerk/clerk-react: The publishableKey passed to Clerk is invalid` — confirmed pre-existing issue (same failure occurs without our changes). TypeScript compilation passes cleanly (`Compiled successfully in 3.3s`). The CI environment has a real Clerk key so this does not affect production builds.

## Known Stubs

None — all form inputs are wired to `submitSupplyChainScan`, all success/error paths navigate correctly.

## Threat Flags

No new threat surface introduced beyond the plan's threat model. The `/supply-chain` page is a static server component (no new API routes). The form uses relative URLs only (`/api/v1/scans/supply-chain`) — T-48-02 mitigation applied as specified.

## Self-Check

Files exist:
- `frontend/lib/supply-chain-types.ts` — FOUND
- `frontend/app/actions/supply-chain-scan.ts` — FOUND
- `frontend/components/supply-chain-form.tsx` — FOUND
- `frontend/app/supply-chain/page.tsx` — FOUND

Commits exist:
- `0ebd972` — FOUND (feat(48-01): add supply chain types, server action, and results endpoint patch)
- `db54329` — FOUND (feat(48-01): add supply chain form component and page route)

## Self-Check: PASSED

## Next Phase Readiness
- Plan 02 (results page) can now read `kind` and `supply_chain_results` from the patched results endpoint
- `SupplyChainResultsPageData` type in supply-chain-types.ts is ready for the results page to consume
- sessionStorage key `supply-chain-inline-results` is documented for the inline results fallback route

---
*Phase: 48-frontend*
*Completed: 2026-04-07*
