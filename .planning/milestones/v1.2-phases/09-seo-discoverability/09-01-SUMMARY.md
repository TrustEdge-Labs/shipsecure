---
phase: 09-seo-discoverability
plan: 01
subsystem: seo
tags: [next.js, metadata, open-graph, json-ld, sitemap, robots-txt, social-sharing]

# Dependency graph
requires:
  - phase: 08-analytics
    provides: Analytics tracking on landing page
provides:
  - Complete SEO metadata (title, description, keywords, canonical URL)
  - Open Graph tags for social sharing on Twitter, Slack, Reddit
  - Dynamic OG image generation (1200x630 PNG)
  - JSON-LD structured data (Organization and SoftwareApplication schemas)
  - sitemap.xml listing public pages
  - robots.txt controlling crawler access
  - Enhanced robots directives for private pages (results, payment)
affects: [10-legal, 12-landing-page]

# Tech tracking
tech-stack:
  added: [next/og (ImageResponse for OG images)]
  patterns: [Next.js metadata API, JSON-LD structured data, sitemap/robots generation]

key-files:
  created:
    - frontend/app/opengraph-image.tsx
    - frontend/app/sitemap.ts
    - frontend/app/robots.ts
    - frontend/app/payment/success/layout.tsx
  modified:
    - frontend/app/layout.tsx
    - frontend/app/page.tsx
    - frontend/app/results/[token]/page.tsx

key-decisions:
  - "Used Next.js metadataBase in root layout for absolute URL resolution"
  - "Generated OG image at edge runtime with system fonts (no custom fonts to keep bundle under 500KB)"
  - "Sitemap currently lists only landing page; will expand when privacy/terms pages added in Phase 10"
  - "Used both robots.txt disallow AND noindex meta tags for results/payment pages (defense-in-depth)"

patterns-established:
  - "Pattern 1: Page-level metadata exports override root layout defaults"
  - "Pattern 2: JSON-LD schemas embedded in page component for search engine discovery"
  - "Pattern 3: Dynamic route generation for sitemap and robots using Next.js MetadataRoute"

# Metrics
duration: 2m 22s
completed: 2026-02-09
---

# Phase 09 Plan 01: SEO Foundation Summary

**Landing page optimized for search engines and social sharing with OG image generation, JSON-LD schemas, sitemap, and robots control**

## Performance

- **Duration:** 2 minutes 22 seconds
- **Started:** 2026-02-09T04:19:51Z
- **Completed:** 2026-02-09T04:22:13Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Landing page has complete SEO metadata (title under 60 chars, description under 155 chars, keywords, canonical URL)
- Social sharing on Twitter/Slack/Reddit shows branded preview with dynamic 1200x630 OG image
- Google-discoverable JSON-LD structured data for Organization and SoftwareApplication schemas
- sitemap.xml lists public pages; robots.txt controls crawler access to private paths
- Enhanced robots directives on results and payment pages for better crawler control

## Task Commits

Each task was committed atomically:

1. **Task 1: Root layout metadataBase + landing page metadata with OG tags and JSON-LD** - `f35d550` (feat)
2. **Task 2: OG image generation + sitemap + robots.txt** - `745e21d` (feat)

## Files Created/Modified
- `frontend/app/layout.tsx` - Added metadataBase for absolute URL resolution
- `frontend/app/page.tsx` - Complete metadata export with OG tags, twitter card, JSON-LD schemas
- `frontend/app/opengraph-image.tsx` - Dynamic 1200x630 branded OG image with ShipSecure branding
- `frontend/app/sitemap.ts` - Dynamic sitemap.xml generation (currently landing page only)
- `frontend/app/robots.ts` - Crawler control allowing public content, disallowing private paths
- `frontend/app/results/[token]/page.tsx` - Strengthened robots directives (noindex, nofollow, nocache)
- `frontend/app/payment/success/layout.tsx` - Created with proper metadata and robots directives

## Decisions Made
- **metadataBase in root layout:** Ensures relative OG image URLs resolve to absolute URLs (required by social platforms)
- **System fonts only in OG image:** Avoids custom font loading to keep edge function bundle under 500KB limit
- **Sitemap single entry:** Only landing page listed; privacy/terms pages will be added in Phase 10
- **Defense-in-depth robots:** Used both robots.txt disallow AND meta robots noindex for private pages (some crawlers respect only one)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Enhanced robots directives for results and payment pages**
- **Found during:** Task 1 (metadata implementation)
- **Issue:** Results pages had only `robots: { index: false }` - missing nofollow and nocache directives for better crawler control
- **Fix:** Added `follow: false` and `nocache: true` to results page metadata; created payment/success/layout.tsx with proper metadata
- **Files modified:** frontend/app/results/[token]/page.tsx, frontend/app/payment/success/layout.tsx (new)
- **Verification:** Build succeeded, routes render correctly
- **Committed in:** f35d550 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 2 - missing critical functionality)
**Impact on plan:** Auto-fix necessary for comprehensive SEO control. No scope creep - enhances existing private page handling.

## Issues Encountered
None - all tasks executed as planned.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- SEO foundation complete, ready for Phase 10 (Legal compliance)
- Privacy Policy and Terms of Service should be added to sitemap once created
- OG image successfully generates at 1200x630 with branding
- JSON-LD schemas validated via plan verification steps

## Self-Check: PASSED

All claimed files and commits verified:
- frontend/app/layout.tsx: FOUND
- frontend/app/page.tsx: FOUND
- frontend/app/opengraph-image.tsx: FOUND
- frontend/app/sitemap.ts: FOUND
- frontend/app/robots.ts: FOUND
- frontend/app/payment/success/layout.tsx: FOUND
- Commit f35d550: FOUND
- Commit 745e21d: FOUND

---
*Phase: 09-seo-discoverability*
*Completed: 2026-02-09*
