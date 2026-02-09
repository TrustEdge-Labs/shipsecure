# Roadmap: ShipSecure

## Milestones

- ✅ **v1.0 MVP** — Phases 01-04 (shipped 2026-02-06)
- ✅ **v1.1 DigitalOcean Deployment** — Phases 05-07 (shipped 2026-02-08)
- 🚧 **v1.2 Launch Readiness** — Phases 08-12 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 01-04) — SHIPPED 2026-02-06</summary>

- [x] Phase 01: Foundation (5/5 plans) — Rust/Axum backend, Next.js frontend, PostgreSQL schema
- [x] Phase 02: Free Tier MVP (8/8 plans) — 5 scanners, email delivery, results dashboard
- [x] Phase 03: Vibe-Code Intelligence (5/5 plans) — Framework detection, Nuclei templates, remediation guidance
- [x] Phase 04: Monetization (5/5 plans) — Stripe checkout, PDF reports, paid audit flow

See: `.planning/milestones/v1.0-ROADMAP.md`

</details>

<details>
<summary>✅ v1.1 DigitalOcean Deployment (Phases 05-07) — SHIPPED 2026-02-08</summary>

- [x] Phase 05: Codebase Preparation (4/4 plans) — Native subprocesses, config externalization, Docker builds
- [x] Phase 06: Deployment Infrastructure (4/4 plans) — Ansible provisioning, Nginx + SSL, systemd, UFW
- [x] Phase 07: Production Validation (2/2 plans) — Scanner validation, email delivery, Stripe flow, resilience

See: `.planning/milestones/v1.1-ROADMAP.md`

</details>

### 🚧 v1.2 Launch Readiness (In Progress)

**Milestone Goal:** Make shipsecure.ai credible and polished enough to launch on Hacker News — trust signals, UX polish, analytics, and discoverability basics.

#### Phase 8: Analytics & Tracking ✅
**Goal**: Privacy-friendly analytics tracking page views and conversion events
**Depends on**: Phase 7
**Requirements**: ANLYT-01, ANLYT-02
**Success Criteria** (what must be TRUE):
  1. Plausible analytics dashboard shows real-time pageviews for all site pages
  2. Custom events track scan submission and paid audit purchase conversion rates
  3. Analytics load without blocking page rendering or affecting Lighthouse scores
**Plans:** 1 plan

Plans:
- [x] 08-01-PLAN.md — Plausible integration with pageview tracking and conversion events

#### Phase 9: SEO & Discoverability ✅
**Goal**: Meta tags and Open Graph configuration for search engines and social sharing
**Depends on**: Phase 8
**Requirements**: SEO-01, SEO-02, SEO-03
**Success Criteria** (what must be TRUE):
  1. Landing page, results page, and payment success page each have unique title and description tags
  2. Sharing shipsecure.ai on Twitter/Slack/Reddit shows correct preview image, title, and description
  3. Scan results pages return noindex/nofollow headers preventing private content from appearing in search results
  4. Google Rich Results Test validates Organization and SoftwareApplication JSON-LD schemas
**Plans:** 2 plans

Plans:
- [x] 09-01-PLAN.md — Landing page metadata, OG tags, JSON-LD schemas, OG image, sitemap, robots.txt
- [x] 09-02-PLAN.md — Results page noindex/nofollow hardening and payment success page metadata

#### Phase 10: Legal Compliance ✅
**Goal**: Privacy Policy, Terms of Service, and consent mechanism for GDPR/CCPA and CFAA protection
**Depends on**: Phase 9 (Privacy Policy must document analytics implementation)
**Requirements**: LEGAL-01, LEGAL-02, LEGAL-03
**Success Criteria** (what must be TRUE):
  1. Privacy Policy page exists at /privacy covering email collection, Stripe data handling, analytics, GDPR/CCPA rights, and data deletion process
  2. Terms of Service page exists at /terms covering acceptable use, CFAA scanning authorization requirements, liability limits, and refund policy
  3. Scan submission form requires explicit consent checkbox before submitting ("I confirm I own this website or have authorization to scan it")
  4. All site pages display footer with links to Privacy Policy and Terms of Service
**Plans:** 2 plans

Plans:
- [x] 10-01-PLAN.md — Privacy Policy and Terms of Service static pages
- [x] 10-02-PLAN.md — Global footer with legal links and scan form consent checkbox

#### Phase 11: Mobile & UX Polish ✅
**Goal**: Mobile-responsive design, loading states, error handling, and visual consistency across all pages
**Depends on**: Phase 10
**Requirements**: UX-01, UX-02, UX-03, UX-04, UX-05, UX-06
**Success Criteria** (what must be TRUE):
  1. Landing page, results page, and payment pages render without horizontal scroll or overlapping elements on mobile (375px) and tablet (768px) viewports
  2. Scan submission shows stage-specific progress messages ("Checking security headers...", "Running Nuclei templates...") instead of generic spinner
  3. API failures display constructive inline error messages with suggested actions (never silent failures)
  4. Spacing, colors, button styles, and typography are consistent across all pages
  5. Landing page and results page achieve Lighthouse performance score >90 on mobile
**Plans:** 3 plans

Plans:
- [x] 11-01-PLAN.md — Mobile responsiveness, duplicate footer removal, and visual consistency
- [x] 11-02-PLAN.md — Loading skeletons, error boundaries, and stage-specific progress messages
- [x] 11-03-PLAN.md — Lighthouse performance optimization and visual verification checkpoint

#### Phase 12: Landing Page Optimization
**Goal**: Developer-focused copy, methodology transparency, and open-source attribution
**Depends on**: Phase 11 (requires responsive, polished UI for testing final copy)
**Requirements**: LAND-01, LAND-02, LAND-03
**Success Criteria** (what must be TRUE):
  1. Landing page headline clearly states product purpose and target audience (developers using AI code generation)
  2. "How it works" section explains scan methodology without marketing jargon
  3. Site footer credits Nuclei, testssl.sh, and other open-source tools used
  4. Landing page copy uses technically honest language (avoids superlatives and vague promises)
**Plans**: TBD

Plans:
- [ ] 12-01: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 8 → 9 → 10 → 11 → 12

| Phase | Milestone | Plans | Status | Completed |
|-------|-----------|-------|--------|-----------|
| 01 - Foundation | v1.0 | 5/5 | Complete | 2026-02-04 |
| 02 - Free Tier MVP | v1.0 | 8/8 | Complete | 2026-02-05 |
| 03 - Vibe-Code Intelligence | v1.0 | 5/5 | Complete | 2026-02-05 |
| 04 - Monetization | v1.0 | 5/5 | Complete | 2026-02-06 |
| 05 - Codebase Preparation | v1.1 | 4/4 | Complete | 2026-02-07 |
| 06 - Deployment Infrastructure | v1.1 | 4/4 | Complete | 2026-02-08 |
| 07 - Production Validation | v1.1 | 2/2 | Complete | 2026-02-08 |
| 08 - Analytics & Tracking | v1.2 | 1/1 | Complete | 2026-02-08 |
| 09 - SEO & Discoverability | v1.2 | 2/2 | Complete | 2026-02-08 |
| 10 - Legal Compliance | v1.2 | 2/2 | Complete | 2026-02-08 |
| 11 - Mobile & UX Polish | v1.2 | 3/3 | Complete | 2026-02-09 |
| 12 - Landing Page Optimization | v1.2 | 0/? | Not started | - |

---
*Last updated: 2026-02-09*
