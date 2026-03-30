# Roadmap: ShipSecure

## Milestones

- ✅ **v1.0 MVP** — Phases 01-04 (shipped 2026-02-06)
- ✅ **v1.1 DigitalOcean Deployment** — Phases 05-07 (shipped 2026-02-08)
- ✅ **v1.2 Launch Readiness** — Phases 08-12 (shipped 2026-02-10)
- ✅ **v1.3 Brand Identity** — Phases 13-18 (shipped 2026-02-11)
- ✅ **v1.4 Observability** — Phases 19-24 (shipped 2026-02-16)
- ✅ **v1.5 Frontend Testing** — Phases 25-28 (shipped 2026-02-17)
- ✅ **v1.6 Auth & Tiered Access** — Phases 29-35 (shipped 2026-02-19)
- ✅ **v1.7 Frontend Polish** — Phases 36-38 (shipped 2026-02-25)
- ✅ **v1.8 CI & Quality Hardening** — Phases 39-41 (shipped 2026-03-02)
- 🚧 **v1.9 Customer Acquisition** — Phases 42-45 (in progress)

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

<details>
<summary>✅ v1.7 Frontend Polish (Phases 36-38) — SHIPPED 2026-02-25</summary>

- [x] Phase 36: Accessibility & Touch Targets (2/2 plans) — 44px touch targets, logo hit area, checkbox a11y, table row link fix
- [x] Phase 37: UX & Hydration Fixes (2/2 plans) — suppressHydrationWarning, email helper text, dashboard polling
- [x] Phase 38: Design Consistency & Analytics (3/3 plans) — --card-radius token, PageContainer component, Plausible data-domain

See: `.planning/milestones/v1.7-ROADMAP.md`

</details>

<details>
<summary>✅ v1.8 CI & Quality Hardening (Phases 39-41) — SHIPPED 2026-03-02</summary>

- [x] Phase 39: Backend CI Pipeline (1/1 plan) — cargo fmt, clippy (-D warnings), test, and llvm-cov coverage in GitHub Actions
- [x] Phase 40: Docker Healthchecks & Docs (1/1 plan) — Backend/frontend healthcheck directives, service_healthy depends_on, README Next.js 16 fix
- [x] Phase 41: Frontend Test Coverage (1/1 plan) — 30 unit tests for DomainBadge, MetaTagSnippet, ScanHistoryTable; coverage exclusions removed

See: `.planning/milestones/v1.8-ROADMAP.md`

</details>

### 🚧 v1.9 Customer Acquisition (In Progress)

**Milestone Goal:** Get the first 10 authenticated users scanning their own sites through funnel polish, CVE-driven content marketing, and community launch on HN/Reddit.

- [ ] **Phase 42: Funnel Unlock** — Reopen anonymous scans, raise rate limits, remove domain verification gate for authenticated users
- [ ] **Phase 43: Share & Results UX** — Share button, OG meta tags, expired results page with scan-again CTA
- [ ] **Phase 44: Content Routes** — /blog MDX infrastructure and /check/{platform} landing pages for Lovable, Bolt, v0
- [ ] **Phase 45: Analytics Events** — Plausible conversion events wired to scan, signup, and share actions

## Phase Details

### Phase 42: Funnel Unlock
**Goal**: Any visitor can scan any URL with no demo lockdown, and authenticated users can scan without domain verification
**Depends on**: Phase 41
**Requirements**: FUNNEL-01, FUNNEL-02, FUNNEL-03, FUNNEL-04
**Success Criteria** (what must be TRUE):
  1. Anonymous user can paste any public URL and receive scan results (not locked to Juice Shop demo)
  2. Anonymous scans are blocked after 3 from the same IP in a 24-hour window with a clear error message
  3. A domain that has been scanned 5 times in the past hour returns cached results instead of re-scanning
  4. Authenticated user can scan any URL without completing domain verification first
**Plans:** 2 plans
Plans:
- [ ] 42-01-PLAN.md — Backend rate limit overhaul + domain verification removal
- [ ] 42-02-PLAN.md — Frontend form unlock + E2E test updates
**UI hint**: yes

### Phase 43: Share & Results UX
**Goal**: Scan results are shareable with rich social previews and expired results guide users back into the funnel
**Depends on**: Phase 42
**Requirements**: FUNNEL-05, FUNNEL-06, FUNNEL-07
**Success Criteria** (what must be TRUE):
  1. Results page shows a share button that copies the capability URL to clipboard
  2. Pasting a results URL into Slack or Twitter renders an OG preview card showing the scan grade and finding count
  3. Visiting an expired results URL shows a dedicated page (not a 404) with the original target URL pre-filled in the scan form
**Plans**: TBD
**UI hint**: yes

### Phase 44: Content Routes
**Goal**: /blog and /check/{platform} routes exist and serve as inbound marketing surfaces for CVE-driven traffic
**Depends on**: Phase 41
**Requirements**: CONTENT-01, CONTENT-02, CONTENT-03, CONTENT-04
**Success Criteria** (what must be TRUE):
  1. /blog renders an MDX post with correct typography when a post file exists
  2. /blog shows a "coming soon" page with a scan CTA when no posts exist
  3. /check/lovable, /check/bolt, and /check/v0 each load with platform-specific accent colors and CVE context copy
  4. Each /check/{platform} page pre-fills the scan form URL field with a platform-appropriate placeholder (e.g., a .lovable.app URL)
**Plans**: TBD
**UI hint**: yes

### Phase 45: Analytics Events
**Goal**: Plausible captures the three conversion events that matter for measuring funnel health
**Depends on**: Phase 42, Phase 43
**Requirements**: ANALYTICS-01, ANALYTICS-02, ANALYTICS-03
**Success Criteria** (what must be TRUE):
  1. Submitting an anonymous scan fires a "Scan Started" custom event visible in the Plausible dashboard
  2. Completing Clerk signup fires a "Signup Completed" custom event visible in the Plausible dashboard
  3. Clicking the share button fires a "Share Clicked" custom event visible in the Plausible dashboard
**Plans**: TBD

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1-4. MVP | v1.0 | 23/23 | Complete | 2026-02-06 |
| 5-7. Deployment | v1.1 | 10/10 | Complete | 2026-02-08 |
| 8-12. Launch | v1.2 | 10/10 | Complete | 2026-02-10 |
| 13-18. Brand | v1.3 | 10/10 | Complete | 2026-02-11 |
| 19-24. Observability | v1.4 | 11/11 | Complete | 2026-02-16 |
| 25-28. Testing | v1.5 | 11/11 | Complete | 2026-02-17 |
| 29-35. Auth & Tiered Access | v1.6 | 13/13 | Complete | 2026-02-19 |
| 36-38. Frontend Polish | v1.7 | 7/7 | Complete | 2026-02-25 |
| 39-41. CI & Quality Hardening | v1.8 | 3/3 | Complete | 2026-03-02 |
| 42. Funnel Unlock | v1.9 | 0/2 | Planning | - |
| 43. Share & Results UX | v1.9 | 0/TBD | Not started | - |
| 44. Content Routes | v1.9 | 0/TBD | Not started | - |
| 45. Analytics Events | v1.9 | 0/TBD | Not started | - |

**Total: 9 milestones shipped, 41 phases complete, 98 plans complete. v1.9 in progress (4 phases).**

---
*Last updated: 2026-03-30 after Phase 42 planning*
