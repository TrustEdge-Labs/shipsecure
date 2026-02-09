---
phase: 10-legal-compliance
plan: 01
subsystem: legal
tags: [legal, gdpr, ccpa, cfaa, privacy-policy, terms-of-service]
dependency_graph:
  requires: []
  provides:
    - legal/privacy-policy
    - legal/terms-of-service
    - legal/cfaa-authorization
  affects:
    - frontend/legal-pages
tech_stack:
  added: []
  patterns:
    - next-js-metadata-export
    - static-legal-pages
key_files:
  created:
    - frontend/app/privacy/page.tsx
    - frontend/app/terms/page.tsx
  modified: []
decisions:
  - id: legal-01
    title: Static legal pages vs database-driven
    chosen: Static Next.js pages with exported metadata
    rationale: Legal content rarely changes, static pages are simpler and more reliable for critical compliance documents
    alternatives:
      - Database-driven with CMS: Over-engineered for two static pages
      - Markdown with MDX: Adds unnecessary build complexity
  - id: legal-02
    title: Last updated date format
    chosen: Human-readable "February 2026" format
    rationale: More accessible and clear to users than ISO dates
    alternatives:
      - ISO date format: Less user-friendly
      - No date: Fails GDPR transparency requirements
metrics:
  duration_minutes: 3
  tasks_completed: 2
  commits: 2
  files_created: 2
  completed_at: "2026-02-09T04:45:46Z"
---

# Phase 10 Plan 01: Privacy Policy and Terms of Service Summary

**One-liner:** GDPR/CCPA-compliant Privacy Policy and CFAA-protected Terms of Service pages at /privacy and /terms

## What Was Built

Created two comprehensive legal pages as static Next.js App Router pages with full SEO metadata:

1. **Privacy Policy** (`/privacy`): 12-section GDPR/CCPA-compliant privacy documentation covering data collection, legal basis for processing, user rights, third-party services (Stripe, Plausible, Resend), international data transfers, and contact information for privacy requests.

2. **Terms of Service** (`/terms`): 10-section legal agreement covering service description, CFAA authorization requirements (18 U.S.C. section 1030), acceptable use policy, liability limitations ($49 cap for paid users), one-time payment refund policy (24-hour window), intellectual property, termination rights, and governing law.

Both pages include:
- Next.js metadata exports for SEO (title, description, robots)
- Dark mode support matching existing site design
- Last updated dates (February 2026)
- Responsive layout with prose classes for readability
- Links to third-party privacy policies
- Navigation back to homepage

## Implementation Details

### Privacy Policy Highlights

**GDPR Requirements Met:**
- Clear identification of data collected (email, URL, payment info, analytics)
- Legal basis for processing (legitimate interest, consent, contractual necessity)
- User rights section (access, deletion, portability, objection)
- Contact email for privacy requests (privacy@shipsecure.ai)
- Response timeframes (30 days GDPR, 45 days CCPA)
- Identity verification requirement for requests

**Third-Party Disclosures:**
- Stripe: Payment processing with links to privacy policy and DPA
- Plausible: EU-hosted (Germany), cookie-less, GDPR-compliant analytics
- Resend: Email delivery service

**Data Retention:**
- Scan results: 12 months unless deletion requested
- Payment records: Per Stripe policies and legal requirements (7 years)

### Terms of Service Highlights

**CFAA Protection:**
- Section 2.2 explicitly cites 18 U.S.C. section 1030
- Clear authorization requirement for all scans
- Warning about unauthorized access being illegal
- Account termination policy for violations
- Anchor `id="acceptable-use"` for consent checkbox linking

**Liability Protection:**
- "As is" service disclaimer
- No guarantee of detecting all vulnerabilities
- Liability cap at $49 for paid users, $0 for free users
- Exclusion of indirect, consequential, and punitive damages

**One-Time Payment Model:**
- Full refund within 24 hours if scan hasn't started
- No refund after scan completes or PDF delivered
- Contact support@shipsecure.ai for refund requests

## Verification Results

**Build Verification:** ✓ PASSED
```
✓ Compiled successfully in 2.9s
Route (app)
├ ○ /privacy (Static)
└ ○ /terms (Static)
```

**Content Verification:** ✓ PASSED
- Privacy Policy contains all 12 required sections
- Terms of Service contains all 10 required sections
- CFAA citation "18 U.S.C. section 1030" present
- `#acceptable-use` anchor exists
- Stripe privacy link: https://stripe.com/privacy
- Plausible data policy link: https://plausible.io/data-policy
- Last updated dates displayed
- SEO metadata exported for both pages

## Deviations from Plan

None - plan executed exactly as written.

## Commits

| Commit | Type | Description | Files |
|--------|------|-------------|-------|
| d659cbc | feat | Create Privacy Policy page at /privacy | frontend/app/privacy/page.tsx |
| e14e22d | feat | Create Terms of Service page at /terms | frontend/app/terms/page.tsx |

## Self-Check: PASSED

**Files exist:**
- ✓ FOUND: frontend/app/privacy/page.tsx
- ✓ FOUND: frontend/app/terms/page.tsx

**Commits exist:**
- ✓ FOUND: d659cbc
- ✓ FOUND: e14e22d

**Build validation:**
- ✓ Next.js build completes without errors
- ✓ Both routes appear in build output as static pages

## Next Phase Readiness

**Blockers:** None

**Notes:**
- Legal pages are now accessible but not yet linked from the main site (this will be handled in plan 10-02)
- Content should be reviewed by legal counsel before production launch (recommended but not blocking for development)
- Privacy Policy documents Plausible Analytics implementation from Phase 8
- Terms of Service #acceptable-use anchor is ready for consent checkbox integration in plan 10-02

## Integration Points

**Dependencies satisfied:**
- None (first plan in phase)

**Provided for downstream:**
- `/privacy` route with GDPR/CCPA-compliant content
- `/terms` route with CFAA authorization language
- `#acceptable-use` anchor for consent checkbox linking
- Legal email contacts: privacy@shipsecure.ai, support@shipsecure.ai

**Affected systems:**
- Frontend: New static routes at /privacy and /terms
- SEO: New indexable pages with metadata
- Legal compliance: Foundation for GDPR, CCPA, and CFAA compliance

## Testing Notes

**Manual verification recommended:**
1. Visit `/privacy` in browser to verify formatting and readability
2. Visit `/terms` in browser to verify CFAA section is prominent
3. Test dark mode toggle on both pages
4. Verify anchor link to `#acceptable-use` works
5. Test external links to Stripe and Plausible policies open in new tabs

**No automated tests needed:** Static content pages with no interactive functionality.
