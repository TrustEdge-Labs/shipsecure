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

### Active

**Current Milestone: v1.6 Auth & Tiered Access**

**Goal:** Add user authentication (Clerk), domain ownership verification, and a tiered access model that converts anonymous scanners into registered users through a "teaser" strategy — revealing critical findings only to authenticated users with verified domains.

**Target features:**
- Clerk auth integration (signup, signin, sessions, Next.js middleware)
- Domain ownership verification (meta tag or file upload)
- Tiered scan configuration (anonymous-light vs authenticated-full)
- Severity-based results gating (full details for low/med, teaser for high/critical on anonymous scans)
- Scan history dashboard for authenticated users
- Per-tier scan rate limits (1 anonymous, 3-5/month Developer)
- Data retention enforcement (24hr anonymous, 7-14 days Developer)
- Remove $49 one-time Stripe audit (replaced by tier model)
- User dashboard with verified domain, scan history, quota status

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
- Anonymous instant audit remains zero-friction lead gen (no signup), but now capped at 1 scan with 24hr retention
- Remediation playbooks are a key differentiator — not just "you have a vulnerability" but "here's exactly how to fix it"
- **v1.0 shipped 2026-02-06:** ~7,000 LOC Rust, ~21,000 LOC TypeScript, 165 files, 4 phases, 23 plans
- **v1.1 shipped 2026-02-08:** Production live at https://shipsecure.ai, 77 files changed, 3 phases, 10 plans
- **v1.2 shipped 2026-02-10:** Launch-ready polish, 67 files changed, 5 phases, 10 plans, 2 days
- **v1.3 shipped 2026-02-11:** Brand identity — design tokens, logo, header, icons, favicon, 62 files changed, 6 phases, 10 plans
- **v1.4 shipped 2026-02-16:** Observability — structured logging, request tracing, health checks, Prometheus metrics, graceful shutdown, 47 files changed, 6 phases, 11 plans
- **v1.5 shipped 2026-02-17:** Frontend testing — 106 unit tests, Playwright E2E, CI pipeline with branch protection, 72 files changed, 4 phases, 11 plans
- **Current:** ~7,877 LOC Rust, 6 milestones shipped, 28 phases, 75 plans completed

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

| Tiered access over open scanning | Domain verification prevents unauthorized scanning of others' sites; teaser strategy converts anonymous → registered | — Pending |
| Clerk over Auth.js/Supabase Auth | Managed service, fastest path to production auth, pre-built Next.js components | — Pending |
| Remove $49 one-time audit | Replace with tier model — deep scans become Developer/Pro feature, simplifies product | — Pending |
| Teaser results gating | Show full low/med findings, gate high/critical behind signup — proves value while driving conversion | — Pending |
| Meta tag + file upload for domain verification | Lower friction than DNS TXT for target audience (vibe-coders may not control DNS) | — Pending |

---
*Last updated: 2026-02-17 after v1.6 milestone started*
