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
- 🔄 **v1.8 CI & Quality Hardening** — Phases 39-41 (in progress)

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

### v1.8 CI & Quality Hardening (Phases 39-41)

- [x] **Phase 39: Backend CI Pipeline** (1/1 plan) - Add cargo test, clippy, fmt, and coverage to GitHub Actions
- [x] **Phase 40: Docker Healthchecks & Docs** - Healthcheck directives on backend/frontend containers and README fix (completed 2026-03-02)
- [x] **Phase 41: Frontend Test Coverage** - Unit tests for three v1.6 components excluded from coverage (completed 2026-03-02)

## Phase Details

### Phase 39: Backend CI Pipeline
**Goal**: Every push and PR triggers backend quality gates — tests, linting, formatting, and coverage reporting all pass in CI
**Depends on**: Nothing (additive CI work, no code dependencies)
**Requirements**: CI-01, CI-02, CI-03, CI-04
**Success Criteria** (what must be TRUE):
  1. A push to main triggers a backend-ci job that runs cargo test and fails the build on any test failure
  2. cargo clippy runs in CI with -D warnings and fails the build on any lint warning
  3. cargo fmt --check runs in CI and fails the build if code is not formatted
  4. A coverage report (llvm-cov or tarpaulin) is generated and visible in CI output after each run
**Plans**: 1 plan
Plans:
- [x] 39-01-PLAN.md -- Backend CI quality gates (test, clippy, fmt, coverage)

### Phase 40: Docker Healthchecks & Docs
**Goal**: Production containers self-report health to Docker, and the README accurately describes the tech stack
**Depends on**: Nothing (independent of Phase 39)
**Requirements**: INFRA-01, INFRA-02, DOC-01
**Success Criteria** (what must be TRUE):
  1. `docker inspect shipsecure-backend` shows a healthcheck polling /health, with status healthy after startup
  2. `docker inspect shipsecure-frontend` shows a healthcheck polling an HTTP endpoint, with status healthy after startup
  3. An unhealthy backend container is distinguishable from a healthy one without reading logs
  4. README states Next.js 16 (not 15) as the frontend framework
**Plans**: 1 plan
Plans:
- [ ] 40-01-PLAN.md -- Docker healthchecks and README fixes

### Phase 41: Frontend Test Coverage
**Goal**: The three v1.6 components excluded from coverage now have unit tests, bringing all active components under the coverage threshold
**Depends on**: Nothing (independent test authoring)
**Requirements**: TEST-01, TEST-02, TEST-03
**Success Criteria** (what must be TRUE):
  1. `vitest run --coverage` passes with domain-badge component covered (renders, shows verified/unverified states)
  2. `vitest run --coverage` passes with meta-tag-snippet component covered (renders snippet, copy interaction)
  3. `vitest run --coverage` passes with scan-history-table component covered (renders rows, severity counts, expiry states)
  4. Coverage thresholds (80/80/75) continue to pass with all three components included in scope
**Plans**: 1 plan
Plans:
- [ ] 41-01-PLAN.md -- Unit tests for domain-badge, meta-tag-snippet, scan-history-table + coverage config update

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
| 36-38. Frontend Polish | v1.7 | 7/7 | Complete | 2026-02-25 |
| 39. Backend CI Pipeline | v1.8 | Complete    | 2026-03-02 | 2026-03-02 |
| 40. Docker Healthchecks & Docs | 1/1 | Complete    | 2026-03-02 | - |
| 41. Frontend Test Coverage | 1/1 | Complete   | 2026-03-02 | - |

**Total: 8 milestones shipped, 38 phases complete, 96 plans complete. v1.8 in progress (3 phases, 1/3 complete).**

---
*Last updated: 2026-03-02 after Phase 39 Plan 01 completion (backend CI pipeline)*
