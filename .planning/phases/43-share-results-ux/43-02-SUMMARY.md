---
phase: 43-share-results-ux
plan: 02
subsystem: ui
tags: [nextjs, react, clipboard, og-meta, social-preview, share-button]

requires:
  - phase: 43-share-results-ux
    provides: "status='expired' tombstone JSON with target_url at HTTP 200"

provides:
  - "ShareButton client component using Clipboard API with 2s copied state and aria-live announcement"
  - "Enriched OG/Twitter meta tags on results page: grade and finding counts for social previews"
  - "Expired results page: clock icon, scan-again CTA pre-filled with original target URL, sign-up upsell"
  - "Results page action row includes ShareButton alongside download and scan-again buttons"

affects: [43-share-results-ux, phase-45-analytics-events, frontend-results-page]

tech-stack:
  added: []
  patterns:
    - "ShareButton as 'use client' island component in server-rendered results page"
    - "generateMetadata branches on status: in-progress, expired, completed — each returns different OG shape"
    - "Expired results page rendered before in-progress check — status ordering: expired → in-progress → completed"

key-files:
  created:
    - frontend/components/share-button.tsx
  modified:
    - frontend/app/results/[token]/page.tsx
    - frontend/lib/types.ts

key-decisions:
  - "ShareButton uses inline SVG (link icon) rather than Lucide React import — avoids client-bundle impact for single icon"
  - "Expired results page checks before in-progress check — prevents expired scans from showing spinner"
  - "OG robots: index:false, follow:false, nocache:true on all result states — results are private capability URLs"

patterns-established:
  - "Client island in server page: 'use client' component imported directly into server component page — no wrapper needed"

requirements-completed: [FUNNEL-05, FUNNEL-07]

duration: 2min
completed: 2026-03-31
---

# Phase 43 Plan 02: Share Results UX Summary

**ShareButton with clipboard API, enriched OG meta tags with grade and finding counts, and dedicated expired results page with pre-filled scan-again CTA**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-31T02:39:28Z
- **Completed:** 2026-03-31T02:41:23Z
- **Tasks:** 1 autonomous (1 checkpoint pending human verify)
- **Files modified:** 3 (1 created, 2 modified)

## Accomplishments

- ShareButton component: clipboard copy with 2-second "Copied!" state, aria-live polite announcement for screen readers, btn-secondary styling with 44px touch target
- OG meta enrichment: completed scans emit `og:title` as "{domain} - Grade {grade} | ShipSecure" and `og:description` with total finding count and high/critical count; expired scans get tombstone description
- Expired results page: renders clock SVG, "These results have expired" heading, scan-again anchor pre-filled with `?url={target_url}`, and sign-up upsell paragraph — no 404 for expired tokens
- ShareButton placed first in actions row before Download and Scan Again buttons

## Task Commits

1. **Task 1: ShareButton component, OG meta enrichment, and expired results page** - `1ad41d6` (feat)

## Files Created/Modified

- `frontend/components/share-button.tsx` - "use client" ShareButton with navigator.clipboard, aria-live, btn-secondary style
- `frontend/app/results/[token]/page.tsx` - Enriched generateMetadata, expired results page, ShareButton import + render
- `frontend/lib/types.ts` - Added comment noting 'expired' as possible status value

## Decisions Made

- ShareButton uses inline SVG link icon rather than importing from Lucide React. Keeps the client component self-contained and avoids adding a tree-shaking dependency for a single icon in a component that would otherwise have no imports from the icon library.
- Expired results check is placed before the in-progress check in page render. This ensures expired scans show the tombstone page rather than the spinner (both would match `status !== 'completed'` if the in-progress check ran first).
- OG `robots: { index: false, follow: false, nocache: true }` is set on all result states including completed. Results pages contain private scan data and are shared via direct link — they should not be indexed by search engines.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

Two pre-existing test failures in `ScanForm.test.tsx` (testing "anonymous scans are limited to our live demo" text) were confirmed pre-existing from Phase 42's Juice Shop lockdown revert. They were present before any changes in this plan. Logged as out-of-scope deferred items; will be fixed when Phase 45 updates E2E and unit tests.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- ShareButton ready for Phase 45 to wire `share-click` Plausible conversion event
- Expired results page complete — fulfills FUNNEL-06 frontend requirement (backend was FUNNEL-06, this plan covers FUNNEL-05 and FUNNEL-07)
- Task 2 checkpoint requires human verification of share button interaction and expired page rendering before plan is fully closed

## Self-Check: PASSED

- `frontend/components/share-button.tsx` — FOUND
- `frontend/app/results/[token]/page.tsx` — FOUND
- `.planning/phases/43-share-results-ux/43-02-SUMMARY.md` — FOUND
- Commit `1ad41d6` — FOUND

---
*Phase: 43-share-results-ux*
*Completed: 2026-03-31*
