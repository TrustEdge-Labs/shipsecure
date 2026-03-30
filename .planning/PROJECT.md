# ShipSecure

## What This Is

A SaaS security scanning platform that targets developers using AI code generation tools (Cursor, Bolt, Lovable, etc.) who ship fast but lack security expertise. It orchestrates open-source security tools as native subprocesses, applies vibe-code-specific detection rules, auto-detects frameworks, and delivers actionable remediation guidance with copy-paste code fixes — no security expertise required. Three-tier access model: anonymous instant audit (lead gen), free Developer tier with domain verification and scan history, and future Pro tier for businesses. Live at https://shipsecure.ai.

## Core Value

Catch security flaws in vibe-coded apps before they become breaches, with remediation guidance anyone can follow — no security expertise required.

## Requirements

### Validated

- Free URL scan with no signup (paste URL + email, get results) — v1.0
- Security headers analysis (CSP, HSTS, X-Frame-Options, etc.) — v1.0
- TLS/certificate analysis with SSL Labs API — v1.0
- Exposed file/directory detection (/.env, /.git, /debug, etc.) — v1.0
- Client-side JavaScript secret scanning (API keys, tokens in bundles) — v1.0
- Vibe-code-specific checks via Nuclei templates (Supabase RLS, Firebase rules, Vercel leaks) — v1.0
- Framework/platform auto-detection (Next.js, Vite, React, Vercel, Netlify, Railway) — v1.0
- Copy-paste remediation fixes specific to detected framework — v1.0
- One-time paid audit ($49) with deeper scanning via Stripe — v1.0
- PDF report generation with executive summary, findings by severity, and remediation roadmap — v1.0
- Email delivery of scan results and PDF reports via Resend — v1.0
- Landing page with URL input form and clear value proposition — v1.0
- Results dashboard showing findings, severity, and remediation guidance — v1.0
- Scan orchestrator managing concurrent scan jobs — v1.0
- SSRF protection, rate limiting, Docker-hardened container execution — v1.0
- ✓ Single-droplet DigitalOcean deployment with Docker, PostgreSQL, Nginx, and SSL — v1.1
- ✓ Remove all Render references from codebase and config — v1.1
- ✓ Production-ready Nginx reverse proxy with Let's Encrypt SSL — v1.1
- ✓ Systemd service management for auto-start and process supervision — v1.1
- ✓ Firewall hardening (UFW) for production security — v1.1
- ✓ Production environment configuration and secrets management — v1.1
- ✓ Nuclei running natively as subprocess (no Docker-in-Docker) — v1.1
- ✓ Plausible analytics with pageview tracking and conversion events — v1.2
- ✓ SEO metadata, Open Graph tags, JSON-LD schemas, OG image, sitemap, robots.txt — v1.2
- ✓ Privacy Policy and Terms of Service pages with CFAA authorization consent — v1.2
- ✓ Global footer with legal links on all pages — v1.2
- ✓ Mobile-responsive design across all pages (375px-1024px) — v1.2
- ✓ Stage-specific scan progress feedback and loading skeletons — v1.2
- ✓ Error boundaries with constructive inline error messages — v1.2
- ✓ Visual consistency and Lighthouse performance >90 — v1.2
- ✓ Developer-focused landing page with methodology transparency — v1.2
- ✓ Open-source tool attribution (Nuclei, testssl.sh) in footer — v1.2
- ✓ Automatic CI/CD deploy pipeline (push → build → deploy) — v1.2
- ✓ Design token system with OKLch primitives and semantic naming — v1.3
- ✓ Dark mode via prefers-color-scheme with WCAG AA contrast compliance — v1.3
- ✓ Professional logo component with responsive icon/compact/full variants — v1.3
- ✓ Sticky header with logo, navigation, and "Scan Now" CTA — v1.3
- ✓ SVG icon system (Lucide React) replacing emoji across landing page — v1.3
- ✓ Branded favicon (ICO + SVG with dark mode) and Apple touch icon — v1.3
- ✓ Open Graph image with logo composite on branded background — v1.3

- ✓ Structured JSON logging with tracing_subscriber JSON formatter (env-toggled) — v1.4
- ✓ Structured fields and scan lifecycle context propagation via tracing spans — v1.4
- ✓ Request correlation IDs via tower-http trace middleware — v1.4
- ✓ Prometheus /metrics endpoint with request latency, scan counts, error rates, queue depth — v1.4
- ✓ DigitalOcean metrics agent installed via Ansible for infrastructure monitoring — v1.4
- ✓ Rich GET /health endpoint with DB connectivity, scanner availability, queue depth — v1.4
- ✓ Graceful shutdown handling (SIGTERM/SIGINT) with in-flight scan draining — v1.4
- ✓ Ansible playbook updates for all infrastructure changes (metrics agent, Nginx, systemd) — v1.4
- ✓ Vitest + React Testing Library test infrastructure with MSW mock handlers and custom render wrapper — v1.5
- ✓ 106 unit/component tests covering all client components, dark mode, loading, and error states — v1.5
- ✓ Playwright E2E tests for free scan, paid audit, and error flows against production builds — v1.5
- ✓ GitHub Actions CI pipeline with unit-tests and e2e-tests jobs on every PR and push to main — v1.5
- ✓ Coverage enforcement at 80/80/75 thresholds via Vitest config (actual: 96.77/94.11/89.32) — v1.5
- ✓ Branch protection on main requiring all CI checks to pass with no admin bypass — v1.5

- ✓ Clerk authentication with email/password, Google, and GitHub OAuth — v1.6
- ✓ Axum JWT verification via cached JWKS public keys (no per-request Clerk API calls) — v1.6
- ✓ CVE-2025-29927 Nginx mitigation (x-middleware-subrequest header strip) — v1.6
- ✓ Stripe removal — async-stripe/hmac/sha2/genpdf removed, paid_audits FK SET NULL — v1.6
- ✓ Server-side results gating — high/critical findings stripped for anonymous, frontend lock overlay with signup CTA — v1.6
- ✓ Domain ownership verification via meta tag with shared-hosting TLD blocklist and 30-day TTL — v1.6
- ✓ Tiered scan configs — anonymous-light (20 JS/180s) vs authenticated-full (30 JS/300s) — v1.6
- ✓ Rate limiting — 1/IP/24h anonymous, 5/user/month Developer with 429 + resets_at — v1.6
- ✓ Scan history dashboard with severity counts, expiry countdown, quota status, verified domains sidebar — v1.6
- ✓ Data retention — hourly cleanup task, 24h anonymous / 30d Developer expiry with 24h grace period — v1.6

- ✓ WCAG 2.5.5 touch targets (44px min) on header nav links and logo — v1.7
- ✓ CFAA checkbox enlarged (w-5 h-5) with tap-area padding wrapper — v1.7
- ✓ Dashboard table row a11y — single link per row, no duplicate announcements — v1.7
- ✓ React hydration fix — suppressHydrationWarning on html + body — v1.7
- ✓ Scan form email helper text ("We'll email your scan results") — v1.7
- ✓ Dashboard active-scan polling via router.refresh() every 7s — v1.7
- ✓ --card-radius design token (0.75rem) applied to all card/panel elements — v1.7
- ✓ PageContainer shared layout component with configurable max-width — v1.7
- ✓ Plausible data-domain="shipsecure.ai" on analytics script tag — v1.7

- ✓ Backend CI pipeline with cargo fmt, clippy (-D warnings), and test gates on every push/PR — v1.8
- ✓ Backend test coverage reporting via cargo-llvm-cov in CI — v1.8
- ✓ Docker healthcheck directives on both production containers with service_healthy startup ordering — v1.8
- ✓ Frontend /api/health lightweight endpoint for container health probing — v1.8
- ✓ Unit tests for DomainBadge, MetaTagSnippet, and ScanHistoryTable components — v1.8
- ✓ Coverage thresholds enforced across all active components (no exclusions remaining) — v1.8
- ✓ README accuracy (Next.js 16, proxy.ts) — v1.8

### Active

## Current Milestone: v1.9 Customer Acquisition

**Goal:** Get the first 10 authenticated users scanning their own sites through funnel polish, CVE-driven content marketing, and community launch on HN/Reddit.

**Target features:**
- Reopen anonymous scans for any URL (revert Juice Shop lockdown)
- Per-target rate limiting (5/domain/hour, cached results)
- Increase anonymous rate limit to 3/IP/day
- Drop domain verification requirement for authenticated scans
- Share results button (copy URL + text OG meta tags)
- Expired results page with "scan again" CTA
- Plausible conversion events (demo-scan-started, signup-completed, first-real-scan, share-click)
- /blog route with MDX for CVE content hosting
- /check/{platform} landing pages (Lovable, Bolt, v0) with platform-specific visual treatment
- Empty blog "coming soon" page with scan CTA
- DESIGN.md formalized (Geist, industrial/utilitarian, no purple)

### Out of Scope

- GitHub repo scanning (Semgrep, Gitleaks, dependency analysis) — fast follow after auth foundation
- Pro subscription tier (unlimited sites, deep scans, automation, API, PDF/CSV exports, permanent history) — build after Developer tier proves conversion
- Agency tier (white-label reports, multi-repo) — future tier after Pro is proven
- Certificate expiration monitoring — Pro tier feature
- GitHub webhook triggers for automated re-scanning — Pro tier feature
- Scheduled scans / CI/CD integration — Pro tier feature
- Mobile app — web-first
- Real-time scan progress (WebSocket) — polling sufficient for now
- Cookie consent banner — not needed with cookieless Plausible analytics

## Context

- 45% of AI-generated code contains security flaws; 86% of AI tools fail XSS defenses; 88% fail log injection
- CVE-2025-48757 exposed 170+ Lovable apps with RLS misconfigurations leaking PII and API keys
- Lovable's built-in scanner catches vulnerabilities only 66% of the time; Bolt's fails entirely
- Founder has 40+ years cybersecurity experience (Bose, Ford, TrustEdge Labs) — deep domain credibility
- Target audience: solo developers and small teams shipping with AI tools who don't have security expertise
- Three-tier access: anonymous instant audit (lead gen) → free Developer tier (verified domain, history) → paid Pro tier (future)
- "Teaser" conversion strategy: anonymous scans show full low/medium findings but gate high/critical behind signup — proves scanner power while creating FOMO
- Anonymous instant audit is the primary lead gen funnel (paste any URL, see results, sign up for more)
- Remediation playbooks are a key differentiator — not just "you have a vulnerability" but "here's exactly how to fix it"
- **v1.0 shipped 2026-02-06:** ~7,000 LOC Rust, ~21,000 LOC TypeScript, 165 files, 4 phases, 23 plans
- **v1.1 shipped 2026-02-08:** Production live at https://shipsecure.ai, 77 files changed, 3 phases, 10 plans
- **v1.2 shipped 2026-02-10:** Launch-ready polish, 67 files changed, 5 phases, 10 plans, 2 days
- **v1.3 shipped 2026-02-11:** Brand identity — design tokens, logo, header, icons, favicon, 62 files changed, 6 phases, 10 plans
- **v1.4 shipped 2026-02-16:** Observability — structured logging, request tracing, health checks, Prometheus metrics, graceful shutdown, 47 files changed, 6 phases, 11 plans
- **v1.5 shipped 2026-02-17:** Frontend testing — 106 unit tests, Playwright E2E, CI pipeline with branch protection, 72 files changed, 4 phases, 11 plans
- **v1.6 shipped 2026-02-19:** Auth & Tiered Access — Clerk auth, domain verification, results gating, tiered scans, rate limiting, scan history dashboard, data retention, 95 files changed, 7 phases, 13 plans, 2 days
- **Post-v1.6 deployment hardening (2026-02-21):** 19 commits fixing CI/CD pipeline, Docker Compose standalone mode, systemd integration, env var management. Production deploy pipeline now reliable.
- **v1.7 shipped 2026-02-25:** Frontend polish — touch targets, a11y, hydration fix, email copy, dashboard polling, design tokens, PageContainer, Plausible fix, 33 files changed, 3 phases, 7 plans
- **v1.8 shipped 2026-03-02:** CI & quality hardening — backend CI pipeline, Docker healthchecks, frontend test coverage for v1.6 components, 54 files changed, 3 phases, 3 plans, 1 day
- **Current:** 9 milestones shipped, 41 phases, 98 plans completed. Zero customers, zero revenue. Distribution problem, not product problem.
- **v1.9 context (2026-03-30):** CEO review redirected from Skill Scan v1 to customer acquisition. 8+ competing scanners exist. CVE-driven content marketing chosen as distribution channel. DESIGN.md formalized (Geist, industrial/utilitarian).

## Constraints

- **Tech Stack**: Rust backend (Axum), Next.js frontend, PostgreSQL
- **Hosting**: DigitalOcean — single droplet with Docker, Nginx reverse proxy, Let's Encrypt SSL
- **Scanning Tools**: Native subprocesses (Nuclei, testssl.sh, custom probes) — installed on host
- **Auth**: Clerk — managed auth with Next.js middleware, pre-built components, session management
- **Email**: Resend — transactional email for scan results and PDF reports
- **Access Model**: Anonymous instant audit (1 scan, 24hr) → Developer tier (signup, 3-5/month, verified domain) → Pro tier (future)
- **Domain Verification**: Meta tag or file upload — user proves site ownership before authenticated scanning
- **CI/CD**: GitHub Actions → GHCR images → auto SSH deploy to production

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust over Python for backend | Performance for concurrent scanning, type safety | ✓ Good — 5 scanners run in parallel with semaphore control |
| Next.js over HTMX for frontend | Richer interactivity for results dashboard, broader ecosystem | ✓ Good — polling, conditional rendering, client components work well |
| URL scanning before repo scanning | Faster to ship, lower friction for users, no GitHub auth complexity | ✓ Good — shipped in 3 days |
| One-time audit before subscriptions | Validate willingness to pay before building recurring billing | Pending — production live, needs real users |
| DigitalOcean over Render | Full Docker access on droplet, no Docker-in-Docker limitation | ✓ Good — full control, Nuclei runs as native subprocess |
| No signup for free tier | Maximize conversion, capture email for follow-up | ✓ Good — zero friction to first scan |
| Capability URL for results | Unguessable token, no auth needed, shareable | ✓ Good — simple, enables sharing |
| Database-as-queue for scans | Simple, no Redis/RabbitMQ dependency for MVP | ✓ Good — sufficient for MVP scale |
| In-memory PDF generation | No filesystem I/O, efficient for email attachment | ✓ Good — genpdf produces Vec<u8> directly |
| Native subprocesses over Docker-in-Docker | Simpler, faster, no nested container complexity | ✓ Good — Nuclei/testssl.sh run directly on host |
| Ansible for infrastructure automation | Reproducible provisioning, idempotent, standard tooling | ✓ Good — 3-play structure handles SSH port transition cleanly |
| Reserved IP for DNS stability | IP survives droplet destroy/recreate, no DNS changes needed | ✓ Good — clean separation of compute and networking |
| DigitalOcean Managed PostgreSQL | No backup management, automatic failover, connection pooling | ✓ Good — requires doadmin user for schema operations |
| Systemd oneshot for Docker Compose | Tracks compose lifecycle cleanly, RemainAfterExit=yes | ✓ Good — restart/stop/start all work correctly |
| Plausible over Google Analytics | Privacy-friendly, no cookies, simpler integration | ✓ Good — direct script with proxy config bypasses ad blockers |
| Next.js App Router conventions for UX | loading.tsx, error.tsx over custom components | ✓ Good — built-in Suspense, better performance |
| Developer-focused copy over marketing | Technical honesty for HN audience | ✓ Good — passed copy quality checks, no marketing anti-patterns |
| Footer OSS attribution over credits page | Always visible, follows OSS best practices | ✓ Good — Nuclei MIT + testssl.sh GPLv2 properly credited |
| CFAA consent checkbox on scan form | Frontend-only gate, Zod validation | ✓ Good — explicit authorization before scanning |
| Auto CI/CD deploy via SSH | appleboy/ssh-action after image builds | ✓ Good — push to main triggers full build and deploy |
| Two-layer design tokens (OKLch + semantic) | Future-proof color system, automatic dark mode | ✓ Good — zero `dark:` classes remain, all via prefers-color-scheme |
| Professional PNG logo over generated SVG | User provided designed logo with shield, padlock, signal waves | ✓ Good — multi-color brand identity, scales from favicon to full |
| Lucide React over Heroicons | Larger icon set, better tree-shaking | ✓ Good — consistent SVG icons inheriting theme colors |
| Geometric shield SVG for favicon | SVG favicons must be vector, fine details illegible at 16x16 | ✓ Good — clean shield reads well at all sizes |
| LOG_FORMAT env var for JSON/text toggle | 12-factor app standard, no recompilation, sensible defaults by build profile | ✓ Good — zero config in dev, production-ready JSON by default |
| tracing + tracing-subscriber over log crate | Structured spans, async-aware, ecosystem standard for Axum/Tower | ✓ Good — spans propagate context through async tasks |
| Nullable request_id column with partial index | Not all scans originate from HTTP (webhooks, future CLI) | ✓ Good — flexible without schema waste |
| HealthCache with std::sync::Mutex | Cache ops are synchronous, no await inside lock | ✓ Good — simpler than tokio::Mutex, no deadlock risk |
| Histogram buckets as constants not env vars | Changing buckets invalidates historical Prometheus data | ✓ Good — stable data for monitoring |
| Status grouping (2xx/4xx/5xx) over individual codes | Reduces Prometheus label cardinality | ✓ Good — cleaner dashboards |
| tokio-util TaskTracker over raw tokio::spawn | Tracks all background tasks for coordinated shutdown | ✓ Good — clean drain, no orphaned tasks |
| Shutdown middleware as outermost layer | Rejects new scans with 503 while draining in-flight | ✓ Good — clean separation of concerns |
| systemd TimeoutStopSec=95s (Docker 90s + 5s buffer) | Prevents systemd from killing Docker before graceful shutdown completes | ✓ Good — clean shutdown chain verified in production |
| Remove app-level /metrics IP check | Docker networking breaks is_loopback(); Nginx + Docker port binding sufficient | ✓ Good — defense-in-depth at infrastructure layer |
| Vitest over Jest for unit tests | Better ESM support, faster, Next.js recommended | ✓ Good — happy-dom + tsconfigPaths work seamlessly |
| MSW for API mocking in tests | Realistic request interception, reusable handlers | ✓ Good — consistent mocking across unit and integration tests |
| Playwright over Cypress for E2E | Faster, lighter, Next.js testmode integration | ✓ Good — testProxy enables clean server component mocking |
| Coverage scoped to components/** only | Server-side app/lib files have 0% unit coverage (tested by E2E) | ✓ Good — 96.77% lines achievable without gaming thresholds |
| E2E tests on port 3001 | Avoids conflict with dev server or other services on 3000 | ✓ Good — clean isolation for CI and local runs |
| Sequential CI jobs (unit → E2E) | Avoid wasting E2E resources if unit tests fail | ✓ Good — fast feedback on unit failures, E2E only runs on passing code |
| Branch protection with enforce_admins | No bypass even for repo owner, strict quality gate | ✓ Good — first PR through CI caught a real bug (browser mismatch) |

| Tiered access over open scanning | Domain verification prevents unauthorized scanning of others' sites; teaser strategy converts anonymous → registered | ✓ Good — anonymous/authenticated tiers enforced server-side, rate limits prevent abuse |
| Standalone docker-compose.prod.yml (no dev merge) | Docker Compose merge behavior is unreliable — duplicate ports, build directives leaking, depends_on to disabled services | ✓ Good — ended 6-hour debugging loop, clean separation of dev and prod |
| Systemd manages Docker on server (not CI) | Direct docker compose in CI fights with systemd; scp files + systemctl restart is reliable and idempotent | ✓ Good — deploy workflow simplified, crash recovery works automatically |
| HOSTNAME=0.0.0.0 required for frontend container | Docker sets HOSTNAME to container ID which Next.js tries to resolve via DNS, causing getaddrinfo EAI_AGAIN crash | ✓ Good — one env var, permanent fix |
| All env vars explicit in docker-compose.prod.yml | env_file inheritance from dev compose causes missing variables, silent failures in production | ✓ Good — every required var visible in one file |
| Clerk over Auth.js/Supabase Auth | Managed service, fastest path to production auth, pre-built Next.js components | ✓ Good — 3 OAuth providers working, JWKS JWT verification clean, webhook sync reliable |
| Remove $49 one-time audit | Replace with tier model — deep scans become Developer/Pro feature, simplifies product | ✓ Good — 5 crates removed, schema cleaned, simpler product |
| Teaser results gating | Show full low/med findings, gate high/critical behind signup — proves value while driving conversion | ✓ Good — server-side gating, curl-proof, lock overlay CTA drives signup |
| Meta tag verification only (no file upload) | Lower friction than DNS TXT, file upload deferred | ✓ Good — simple meta tag flow, shared-hosting blocklist prevents abuse |
| jsonwebtoken + axum-jwt-auth for Rust JWT | Local JWKS-based verification, no per-request Clerk API calls | ✓ Good — cached JWKS keys, RS256 verification, zero latency overhead |
| 24h grace period before scan deletion | Users see "Expired" badge in dashboard before data disappears | ✓ Good — interval_at deferred task, TaskTracker integration for graceful shutdown |
| Independent backend-ci job (no needs:) | Backend CI runs in parallel with frontend jobs — no cross-dependency | ✓ Good — faster CI feedback, no coupling |
| cargo fmt first in CI pipeline | Fastest gate runs first — fail early on formatting before clippy/test | ✓ Good — saves CI minutes on trivial failures |
| Coverage report-only (no --fail-under) | Establish baseline before setting thresholds | ✓ Good — per REQUIREMENTS.md "Out of Scope" |
| Healthchecks in compose, not Dockerfiles | Environment-specific tuning without rebuilding images | ✓ Good — different timings for dev vs prod |
| Backend curl, frontend wget for healthchecks | Use what's already in the base image (debian:bookworm-slim has curl, node:20-alpine has wget) | ✓ Good — no extra packages needed |
| service_healthy depends_on condition | Frontend waits for backend DB connectivity, not just container start | ✓ Good — prevents frontend startup race condition |
| fireEvent.click over userEvent.click for clipboard tests | happy-dom's Permissions API security context rejects full pointer event chain | ✓ Good — vi.spyOn + fireEvent is reliable in happy-dom |
| vi.useFakeTimers for date-dependent component tests | Deterministic assertions regardless of when tests run | ✓ Good — no flaky date-boundary failures |

---
*Last updated: 2026-03-02 after v1.8 milestone*
