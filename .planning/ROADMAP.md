# Roadmap: ShipSecure

## Milestones

- ✅ **v1.0 MVP** — Phases 01-04 (shipped 2026-02-06)
- ✅ **v1.1 DigitalOcean Deployment** — Phases 05-07 (shipped 2026-02-08)
- ✅ **v1.2 Launch Readiness** — Phases 08-12 (shipped 2026-02-10)
- ✅ **v1.3 Brand Identity** — Phases 13-18 (shipped 2026-02-11)
- ✅ **v1.4 Observability** — Phases 19-24 (shipped 2026-02-16)
- ✅ **v1.5 Frontend Testing** — Phases 25-28 (shipped 2026-02-17)
- ✅ **v1.6 Auth & Tiered Access** — Phases 29-35 (shipped 2026-02-19)
- 🚧 **v1.7 Frontend Polish** — Phases 36-38 (in progress)

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

<details>
<summary>✅ v1.3 Brand Identity (Phases 13-18) — SHIPPED 2026-02-11</summary>

- [x] Phase 13: Design Token System (3/3 plans) — OKLch primitives, semantic tokens, dark mode, WCAG AA
- [x] Phase 14: Logo Component (2/2 plans) — Shield logo, responsive variants, professional PNG
- [x] Phase 15: Layout Refactor (1/1 plan) — Header-height token, layout preparation
- [x] Phase 16: Header & Navigation (1/1 plan) — Sticky header, responsive logo, CTA, keyboard nav
- [x] Phase 17: Icon System & Migration (1/1 plan) — Lucide React SVG icons replacing emoji
- [x] Phase 18: Favicon & OG Image (2/2 plans) — Branded favicon (ICO+SVG), Apple touch icon, OG image

See: `.planning/milestones/v1.3-ROADMAP.md`

</details>

<details>
<summary>✅ v1.4 Observability (Phases 19-24) — SHIPPED 2026-02-16</summary>

- [x] Phase 19: Structured JSON Logging (2/2 plans) — Environment-driven JSON/text logging, scan lifecycle instrumentation
- [x] Phase 20: Request Tracing (2/2 plans) — UUID correlation IDs, TraceLayer middleware, end-to-end propagation
- [x] Phase 21: Health Checks (1/1 plan) — Liveness + readiness endpoints, DB validation, scan capacity
- [x] Phase 22: Prometheus Metrics (2/2 plans) — HTTP counters, scan histograms, queue depth, rate limit tracking
- [x] Phase 23: Graceful Shutdown (2/2 plans) — TaskTracker, SIGTERM handling, scan draining, 503 rejection
- [x] Phase 24: Infrastructure Integration (2/2 plans) — Nginx /metrics security, Docker grace periods, systemd timeout, DO metrics agent

See: `.planning/milestones/v1.4-ROADMAP.md`

</details>

<details>
<summary>✅ v1.5 Frontend Testing (Phases 25-28) — SHIPPED 2026-02-17</summary>

- [x] Phase 25: Test Infrastructure (2/2 plans) — Vitest, MSW, RTL setup with working foundation and first passing test
- [x] Phase 26: Component Tests (4/4 plans) — Unit tests for all 9 client components plus dark mode, loading, and error states
- [x] Phase 27: E2E Tests (3/3 plans) — Playwright setup with free scan, paid audit, and error flow coverage
- [x] Phase 28: CI/CD and Quality Gates (2/2 plans) — GitHub Actions pipeline with coverage enforcement and branch protection

See: `.planning/milestones/v1.5-ROADMAP.md`

</details>

<details>
<summary>✅ v1.6 Auth & Tiered Access (Phases 29-35) — SHIPPED 2026-02-19</summary>

- [x] Phase 29: Auth Foundation (3/3 plans) — Clerk Next.js integration, Axum JWT verification with JWKS caching, CORS fix, CVE-2025-29927 Nginx mitigation
- [x] Phase 30: Stripe Removal and Schema Cleanup (1/1 plan) — Remove async-stripe/hmac/sha2/genpdf, paid_audits FK SET NULL, tier constraint, clerk_user_id column
- [x] Phase 31: Results Gating (2/2 plans) — Server-side high/critical finding suppression, gated flag, frontend teaser cards with lock overlay
- [x] Phase 32: Domain Verification (2/2 plans) — Meta tag verification, shared-hosting TLD blocklist, 30-day TTL, domain wizard UI
- [x] Phase 33: Tiered Scan Access and Rate Limiting (2/2 plans) — Anonymous-light vs authenticated-full configs, per-IP and per-user rate limits, quota display
- [x] Phase 34: Scan History Dashboard (2/2 plans) — Paginated scan history, severity counts, expiry countdown, quota status sidebar
- [x] Phase 35: Data Retention (1/1 plan) — Hourly Tokio cleanup task, 24h anonymous / 30d Developer expiry, graceful shutdown integration

See: `.planning/milestones/v1.6-ROADMAP.md`

</details>

### 🚧 v1.7 Frontend Polish (In Progress)

**Milestone Goal:** Elevate the existing UI to production quality — correct touch targets, screen reader accessibility, hydration stability, UX copy clarity, design token consistency, and analytics integrity. No new features; all changes are in the Next.js frontend.

## Phase Details

### Phase 36: Accessibility and Touch Targets
**Goal**: Every interactive element meets WCAG touch target and screen reader standards
**Depends on**: Phase 35
**Requirements**: TOUCH-01, TOUCH-02, A11Y-01, A11Y-02
**Success Criteria** (what must be TRUE):
  1. Header nav links and buttons are tappable on mobile without mis-tapping adjacent elements (min 44px height)
  2. Tapping the logo navigates home with a comfortable hit area — no need to aim precisely at the icon
  3. The CFAA consent checkbox is visually prominent and activates with a single finger tap on mobile
  4. Screen readers announce each scan history row as a single link, not a link nested inside a link
**Plans**: 2 plans

Plans:
- [ ] 36-01-PLAN.md — Header touch targets (logo p-2 -m-2, nav links min-h-[44px])
- [ ] 36-02-PLAN.md — Checkbox a11y (w-5 h-5 + wrapper) and scan table duplicate link fix

### Phase 37: UX and Hydration Fixes
**Goal**: The app renders without console errors and form copy sets correct user expectations
**Depends on**: Phase 36
**Requirements**: HYDR-01, UX-01, UX-02
**Success Criteria** (what must be TRUE):
  1. No React hydration mismatch warnings appear in the browser console on any page
  2. The scan form email field label or helper text makes clear that results will be emailed to that address
  3. An active scan in the dashboard history updates its status automatically without requiring a manual page refresh
**Plans**: 2 plans

Plans:
- [ ] 37-01-PLAN.md — Hydration fix (suppressHydrationWarning on body) and email field helper text
- [ ] 37-02-PLAN.md — Dashboard active-scan polling via ActiveScansPoller client component

### Phase 38: Design Consistency and Analytics
**Goal**: Visual layout is uniform across all pages and analytics tracking is correctly attributed
**Depends on**: Phase 37
**Requirements**: DESIGN-01, DESIGN-02, ANLYT-01
**Success Criteria** (what must be TRUE):
  1. Card and panel elements use the same border radius on every page (no inconsistent rounding)
  2. All pages share the same max-width and horizontal padding via a single shared layout component
  3. Plausible analytics dashboard shows traffic attributed to shipsecure.ai (data-domain attribute present on script tag)
**Plans**: TBD

## Progress

| Phase | Milestone | Plans | Status | Completed |
|-------|-----------|-------|--------|-----------|
| 1-4. MVP | v1.0 | 23/23 | Complete | 2026-02-06 |
| 5-7. Deployment | v1.1 | 10/10 | Complete | 2026-02-08 |
| 8-12. Launch | v1.2 | 10/10 | Complete | 2026-02-10 |
| 13-18. Brand | v1.3 | 10/10 | Complete | 2026-02-11 |
| 19-24. Observability | v1.4 | 11/11 | Complete | 2026-02-16 |
| 25-28. Testing | v1.5 | 11/11 | Complete | 2026-02-17 |
| 29-35. Auth & Tiered Access | v1.6 | 13/13 | Complete | 2026-02-19 |
| 36. Accessibility & Touch | 2/2 | Complete    | 2026-02-25 | - |
| 37. UX & Hydration | v1.7 | 0/TBD | Not started | - |
| 38. Design & Analytics | v1.7 | 0/TBD | Not started | - |

**Total: 7 milestones shipped, 35 phases, 88 plans completed. v1.7 in progress.**

---
*Last updated: 2026-02-24 after v1.7 roadmap creation*
