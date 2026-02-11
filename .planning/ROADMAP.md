# Roadmap: ShipSecure

## Milestones

- ✅ **v1.0 MVP** — Phases 01-04 (shipped 2026-02-06)
- ✅ **v1.1 DigitalOcean Deployment** — Phases 05-07 (shipped 2026-02-08)
- ✅ **v1.2 Launch Readiness** — Phases 08-12 (shipped 2026-02-10)
- 🚧 **v1.3 Brand Identity** — Phases 13-18 (in progress)

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

<details>
<summary>✅ v1.2 Launch Readiness (Phases 08-12) — SHIPPED 2026-02-10</summary>

- [x] Phase 08: Analytics & Tracking (1/1 plan) — Plausible analytics, conversion events
- [x] Phase 09: SEO & Discoverability (2/2 plans) — Meta tags, OG image, JSON-LD, sitemap, robots.txt
- [x] Phase 10: Legal Compliance (2/2 plans) — Privacy Policy, TOS, CFAA consent checkbox
- [x] Phase 11: Mobile & UX Polish (3/3 plans) — Mobile responsive, loading states, error boundaries, Lighthouse
- [x] Phase 12: Landing Page Optimization (2/2 plans) — Developer-focused copy, methodology transparency, OSS attribution

See: `.planning/milestones/v1.2-ROADMAP.md`

</details>

### 🚧 v1.3 Brand Identity (In Progress)

**Milestone Goal:** Give ShipSecure a proper visual identity — logo, refined color system, branded favicon, header navbar, and clean SVG icons — so the product looks as credible as it is.

#### Phase 13: Design Token System
**Goal**: Establish design token foundation with semantic color naming and dark mode support
**Depends on**: Phase 12
**Requirements**: COLOR-01, COLOR-02, COLOR-03, COLOR-04
**Success Criteria** (what must be TRUE):
  1. Design tokens defined via Tailwind v4 `@theme` with semantic naming (brand-primary, surface-primary, text-secondary, border-subtle)
  2. Dark mode overrides work correctly via `prefers-color-scheme` for all tokens
  3. All color combinations meet WCAG AA contrast ratio (4.5:1 minimum)
**Plans**: 3 plans

Plans:
- [x] 13-01-PLAN.md -- Define design token system + migrate 3 core components
- [x] 13-02-PLAN.md -- Migrate remaining 14 components/pages to semantic tokens
- [x] 13-03-PLAN.md -- WCAG contrast validation + visual dark mode verification

#### Phase 14: Logo Component
**Goal**: Create theme-aware SVG logo component that scales from favicon to full size
**Depends on**: Phase 13
**Requirements**: LOGO-01, LOGO-02, LOGO-03, LOGO-04
**Success Criteria** (what must be TRUE):
  1. Logo mark renders correctly in light and dark mode using `currentColor`
  2. Wordmark renders correctly in light and dark mode using `currentColor`
  3. Logo scales cleanly from 16x16px (favicon size) to full desktop size without pixelation
  4. Logo component has proper aria-label and role="img" for accessibility
**Plans**: 2 plans

Plans:
- [x] 14-01-PLAN.md -- Shield design tokens + Logo component with 3 SVG size variants
- [x] 14-02-PLAN.md -- Logo preview page + visual verification (user provided designed PNG logo)

#### Phase 15: Layout Refactor
**Goal**: Prepare layout structure for header integration without causing layout shift
**Depends on**: Phase 14
**Requirements**: HDR-03
**Success Criteria** (what must be TRUE):
  1. CSS variable `--header-height: 64px` defined and used consistently across layout
  2. All existing routes maintain current spacing after header-height variable integration
  3. No layout shift occurs on any route when header is eventually added
**Plans**: 1 plan

Plans:
- [x] 15-01-PLAN.md -- Define header-height layout token + verify all routes maintain spacing

#### Phase 16: Header & Navigation
**Goal**: Add branded header with logo, nav, and CTA across all pages
**Depends on**: Phase 15
**Requirements**: HDR-01, HDR-02
**Success Criteria** (what must be TRUE):
  1. Sticky header displays on all pages (/, /results, /scan, /privacy, /terms) with logo and "Scan Now" CTA
  2. Header shows full wordmark on desktop (>=640px) and icon mark only on mobile (<640px)
  3. Header remains accessible via keyboard navigation (Tab key cycles through logo, nav links, CTA)
**Plans**: 1 plan

Plans:
- [ ] 16-01-PLAN.md -- Sticky header component with responsive logo, CTA, and keyboard accessibility

#### Phase 17: Icon System & Migration
**Goal**: Replace emoji with consistent SVG icon system using Lucide React
**Depends on**: Phase 13
**Requirements**: ICON-01, ICON-02, ICON-03
**Success Criteria** (what must be TRUE):
  1. Landing page feature grid displays SVG icons instead of emoji (🔒, 🚀, 🎯, etc.)
  2. All SVG icons use consistent sizing (w-5 h-5 or w-6 h-6) and inherit color via `currentColor`
  3. Decorative icons have `aria-hidden="true"` and standalone icons have `aria-label`
**Plans**: TBD

Plans:
- [ ] 17-01: TBD

#### Phase 18: Favicon & OG Image
**Goal**: Deploy branded favicon and update OG image with logo
**Depends on**: Phase 14
**Requirements**: FAV-01, FAV-02, FAV-03, OG-01
**Success Criteria** (what must be TRUE):
  1. Favicon displays in browser tabs (ICO + SVG formats) and adapts to dark mode via SVG `prefers-color-scheme`
  2. Apple touch icon (180x180 PNG) renders on iOS home screen
  3. Open Graph image includes branded logo and color system when shared on social media
**Plans**: TBD

Plans:
- [ ] 18-01: TBD

## Progress

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
| 12 - Landing Page Optimization | v1.2 | 2/2 | Complete | 2026-02-09 |
| 13 - Design Token System | v1.3 | 3/3 | Complete | 2026-02-10 |
| 14 - Logo Component | v1.3 | 2/2 | Complete | 2026-02-11 |
| 15 - Layout Refactor | v1.3 | 1/1 | Complete | 2026-02-11 |
| 16 - Header & Navigation | v1.3 | 0/1 | Planned | - |
| 17 - Icon System & Migration | v1.3 | 0/0 | Not started | - |
| 18 - Favicon & OG Image | v1.3 | 0/0 | Not started | - |

---
*Last updated: 2026-02-11*
