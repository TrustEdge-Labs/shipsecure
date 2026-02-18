# Roadmap: ShipSecure

## Milestones

- ✅ **v1.0 MVP** — Phases 01-04 (shipped 2026-02-06)
- ✅ **v1.1 DigitalOcean Deployment** — Phases 05-07 (shipped 2026-02-08)
- ✅ **v1.2 Launch Readiness** — Phases 08-12 (shipped 2026-02-10)
- ✅ **v1.3 Brand Identity** — Phases 13-18 (shipped 2026-02-11)
- ✅ **v1.4 Observability** — Phases 19-24 (shipped 2026-02-16)
- ✅ **v1.5 Frontend Testing** — Phases 25-28 (shipped 2026-02-17)
- 🚧 **v1.6 Auth & Tiered Access** — Phases 29-35 (in progress)

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

### 🚧 v1.6 Auth & Tiered Access (In Progress)

**Milestone Goal:** Convert anonymous scanners into registered users through Clerk auth, domain ownership verification, server-enforced results gating, and a tiered access model — replacing the $49 Stripe audit with a free Developer tier.

#### Phases

- [x] **Phase 29: Auth Foundation** — Clerk Next.js integration, Axum JWT verification with JWKS caching, CORS fix, CVE-2025-29927 Nginx mitigation, users table, and webhook sync
- [x] **Phase 30: Stripe Removal and Schema Cleanup** — Remove async-stripe/hmac/sha2/genpdf, delete Stripe checkout routes, change paid_audits FK to SET NULL, extend tier constraint, add clerk_user_id to scans (completed 2026-02-18)
- [x] **Phase 31: Results Gating** — Server-side high/critical finding suppression for anonymous tokens, gated flag in API response, frontend teaser cards with lock overlay and signup CTA (completed 2026-02-18)
- [ ] **Phase 32: Domain Verification** — verified_domains table, verify-start/verify-confirm API endpoints, meta tag verification, shared-hosting TLD blocklist, 30-day TTL, domain wizard UI
- [ ] **Phase 33: Tiered Scan Access and Rate Limiting** — Tiered scan configs (anonymous-light vs authenticated-full), per-IP anonymous rate limit, per-user monthly quota, 429 with resets_at, quota display
- [ ] **Phase 34: Scan History Dashboard** — Paginated scan history endpoint, protected dashboard route, scan list with severity counts and expiry, quota status display
- [ ] **Phase 35: Data Retention** — Hourly Tokio cleanup task, 24-hour anonymous expiry, 30-day Developer expiry, in-progress scan protection

## Phase Details

### Phase 29: Auth Foundation
**Goal**: Users can authenticate with Clerk and the backend can verify their identity on every request
**Depends on**: Phase 28 (CI gates green — E2E must pass before auth routes are added)
**Requirements**: INFR-01, INFR-02, INFR-03, INFR-04, AUTH-01, AUTH-02, AUTH-03, AUTH-04, AUTH-05, AUTH-06
**Success Criteria** (what must be TRUE):
  1. User can sign up and sign in with email/password, Google, or GitHub — session persists across browser restarts
  2. Signed-in user sees UserButton (avatar/dropdown) in the sticky header on every page
  3. Navigating to any `/dashboard/*` route while unauthenticated redirects to the sign-in page
  4. Axum accepts an `Authorization: Bearer <JWT>` header without preflight errors — CORS allows the Authorization header
  5. Nginx strips `x-middleware-subrequest` from all upstream requests — CVE-2025-29927 mitigated at infrastructure layer
**Plans**: 3 plans

Plans:
- [x] 29-01-PLAN.md — Backend auth infrastructure: CORS Authorization header fix, ClerkUser Axum extractor with JWKS caching, users table migration, Clerk webhook handler with svix signature verification
- [x] 29-02-PLAN.md — Frontend auth integration: @clerk/nextjs install, ClerkProvider in root layout, proxy.ts middleware, /sign-in and /sign-up routes, UserButton in header, dashboard route protection skeleton
- [x] 29-03-PLAN.md — Nginx CVE mitigation and production wiring: x-middleware-subrequest header strip in Nginx config, environment variable wiring for CLERK_SECRET_KEY and JWKS URL

### Phase 30: Stripe Removal and Schema Cleanup
**Goal**: The codebase is free of Stripe dependencies and the schema is ready for authenticated scan ownership
**Depends on**: Phase 29 (clerk_user_id column needs users table to exist)
**Requirements**: CLEN-01, CLEN-02, CLEN-03
**Success Criteria** (what must be TRUE):
  1. The application compiles with async-stripe, hmac, sha2, and genpdf removed from Cargo.toml — no compilation errors
  2. Deleting a scan row does not cascade-delete rows in paid_audits — historical payment records are preserved
  3. The scans table accepts a clerk_user_id value and an 'authenticated' tier value without constraint violations
**Plans**: 1 plan

Plans:
- [ ] 30-01-PLAN.md — Stripe removal and schema migration: remove 5 Rust crates and all Stripe backend code, delete frontend UpgradeCTA/payment components and fix affected tests, create migration for paid_audits FK SET NULL + tier constraint + clerk_user_id column

### Phase 31: Results Gating
**Goal**: Anonymous users see teaser cards for high/critical findings that drive signup — and cannot bypass gating by calling the API directly
**Depends on**: Phase 30 (scans.clerk_user_id and tier constraint in place; ClerkUser extractor from Phase 29)
**Requirements**: GATE-01, GATE-02, GATE-03, GATE-04
**Success Criteria** (what must be TRUE):
  1. A `curl` call to `GET /api/v1/results/:token` for an anonymous scan returns high/critical findings with null description and null remediation — server strips these fields, not the frontend
  2. The results API response includes a `gated: true` field on each stripped finding and an `owner_verified` boolean at the response level
  3. Authenticated users viewing their own scan results see full description and remediation for all finding severities
  4. Unauthenticated users viewing results see lock-overlay teaser cards for high/critical findings with a "Sign up free" CTA — severity and category are visible but details are not
**Plans**: 2 plans

Plans:
- [ ] 31-01-PLAN.md — Server-side gating: add clerk_user_id to Scan struct and all SELECT queries, optional JWT extraction, owner_verified computation, strip description/remediation from high/critical for non-owners, add gated field to findings
- [ ] 31-02-PLAN.md — Frontend AuthGate component: TypeScript type updates, AuthGate client component with lock overlay and Clerk SignUp modal CTA, FindingAccordion integration, results page auth token forwarding

### Phase 32: Domain Verification
**Goal**: Authenticated users can prove they own a domain, and only verified domains can receive authenticated scans
**Depends on**: Phase 29 (ClerkUser extractor), Phase 30 (scans.clerk_user_id)
**Requirements**: DOMN-01, DOMN-02, DOMN-03, DOMN-04, DOMN-05
**Success Criteria** (what must be TRUE):
  1. User can request a verification token for a domain and receive a unique token to embed as an HTML meta tag
  2. After placing the meta tag, user can confirm verification — domain shows green "Verified" badge in the dashboard
  3. Attempting to verify a domain on a shared-hosting TLD (github.io, vercel.app, netlify.app, pages.dev) returns an error — not a token
  4. A domain verified more than 30 days ago is treated as unverified — user must re-verify before running authenticated scans
**Plans**: TBD

Plans:
- [ ] 32-01: Domain verification backend — verified_domains table migration, POST /api/v1/domains/verify-start and /verify-confirm endpoints, meta tag fetch/parse using existing reqwest+scraper, domain normalization, SSRF protection, shared-hosting TLD blocklist, 30-day TTL enforcement in scan submission
- [ ] 32-02: Domain verification frontend — /verify-domain/ wizard UI, domain input with normalization display, meta tag snippet copy, verify button with polling, verified/pending/failed badge, integration into dashboard layout

### Phase 33: Tiered Scan Access and Rate Limiting
**Goal**: Anonymous and authenticated scans run with appropriate depth limits, and each tier is enforced at the API layer
**Depends on**: Phase 29 (ClerkUser extractor), Phase 30 (scans.clerk_user_id, tier constraint), Phase 32 (domain verification — TIER-06 requires it)
**Requirements**: TIER-01, TIER-02, TIER-03, TIER-04, TIER-05, TIER-06
**Success Criteria** (what must be TRUE):
  1. An anonymous scan submission uses the light config (20 JS files, 180s vibecode timeout) — observable from scan duration and coverage
  2. An authenticated scan submission uses the enhanced config (30 JS files, 300s timeout, extended exposed-file checks) — and is rejected if the target domain is not verified
  3. A second anonymous scan submission from the same IP within 24 hours returns HTTP 429 with a `resets_at` timestamp and a human-readable message
  4. A Developer-tier user who has submitted 5 scans in the current calendar month receives HTTP 429 with a `resets_at` of the first day of next month
**Plans**: TBD

Plans:
- [ ] 33-01: Tiered scan orchestration — spawn_authenticated_scan with enhanced config, 3-arm tier match in create_scan handler (anonymous/authenticated/paid), domain verification check at scan submission, tier value stored on scan record at creation time
- [ ] 33-02: Rate limiting and quota display — extend check_rate_limits with Option<clerk_user_id>, per-IP 1/24h anonymous limit, per-user 5/month Developer limit using DB-backed monthly window, 429 response with resets_at, quota display in header ("3 of 5 scans used this month, resets Mar 1")

### Phase 34: Scan History Dashboard
**Goal**: Authenticated users can see all their past scans with severity summaries, expiry countdowns, and quota status
**Depends on**: Phase 29 (dashboard route protection), Phase 33 (scans linked to clerk_user_id with quota enforced)
**Requirements**: DASH-01, DASH-02
**Success Criteria** (what must be TRUE):
  1. Authenticated user navigating to `/dashboard` sees a paginated list of their scans — each row shows domain, scan date, severity counts (critical/high/medium/low), and days until expiry
  2. Dashboard shows current quota status in a persistent banner: "X of 5 scans used this month, resets [Date]"
**Plans**: TBD

Plans:
- [ ] 34-01: Scan history backend — GET /api/v1/users/me/scans paginated endpoint requiring non-optional ClerkUser, returns domain, created_at, severity counts, expires_at
- [ ] 34-02: Dashboard frontend — frontend/app/dashboard/ protected route, scan history list component, empty state with verify-domain CTA, quota status banner, expiry countdown display

### Phase 35: Data Retention
**Goal**: Expired scans are automatically deleted on schedule — anonymous scans after 24 hours, Developer scans after 30 days — without touching in-progress scans or payment records
**Depends on**: Phase 30 (paid_audits FK changed to SET NULL), Phase 33 (tier-based expires_at set correctly on scan creation), Phase 34 (dashboard shows expiry dates before cleanup begins)
**Requirements**: RETN-01, RETN-02, RETN-03
**Success Criteria** (what must be TRUE):
  1. An anonymous scan's results are inaccessible via its results token more than 24 hours after completion
  2. A Developer-tier scan's results remain accessible for 30 days after completion
  3. The cleanup task runs hourly and logs the count of deleted scans — in-progress scans are never deleted regardless of age
**Plans**: TBD

Plans:
- [ ] 35-01: Retention cleanup task — src/cleanup.rs Tokio interval task integrated into main.rs task_tracker, hourly DELETE WHERE expires_at < NOW() AND status IN ('completed', 'failed'), tracing log with deleted row count, verified anonymous 24h and Developer 30-day expiry values set at scan creation

## Progress

**Execution Order:** 29 → 30 → 31 → 32 → 33 → 34 → 35

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1-4. MVP | v1.0 | 23/23 | Complete | 2026-02-06 |
| 5-7. Deployment | v1.1 | 10/10 | Complete | 2026-02-08 |
| 8-12. Launch | v1.2 | 10/10 | Complete | 2026-02-10 |
| 13-18. Brand | v1.3 | 10/10 | Complete | 2026-02-11 |
| 19-24. Observability | v1.4 | 11/11 | Complete | 2026-02-16 |
| 25-28. Testing | v1.5 | 11/11 | Complete | 2026-02-17 |
| 29. Auth Foundation | v1.6 | Complete    | 2026-02-18 | 2026-02-18 |
| 30. Stripe Removal & Schema | 1/1 | Complete    | 2026-02-18 | - |
| 31. Results Gating | 2/2 | Complete   | 2026-02-18 | - |
| 32. Domain Verification | v1.6 | 0/2 | Not started | - |
| 33. Tiered Access & Rate Limiting | v1.6 | 0/2 | Not started | - |
| 34. Scan History Dashboard | v1.6 | 0/2 | Not started | - |
| 35. Data Retention | v1.6 | 0/1 | Not started | - |

---
*Last updated: 2026-02-18*
