# TrustEdge Audit

## What This Is

A SaaS security scanning platform that targets developers using AI code generation tools (Cursor, Bolt, Lovable, etc.) who ship fast but lack security expertise. It orchestrates containerized open-source security tools, applies vibe-code-specific detection rules, auto-detects frameworks, and delivers actionable remediation guidance with copy-paste code fixes — no security expertise required. Includes a free tier (no signup) and a paid deep audit ($49) with professional PDF reports.

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

### Active

(Requirements for next milestone — define with `/gsd:new-milestone`)

### Out of Scope

- GitHub repo scanning (Semgrep, Gitleaks, dependency analysis) — fast follow after launch
- Pro subscription tier ($149/month continuous monitoring) — add after validating one-time audit demand
- Agency tier (white-label reports, multi-repo) — future tier after Pro is proven
- Certificate expiration monitoring — Pro tier feature
- GitHub webhook triggers for automated re-scanning — Pro tier feature
- OAuth/social login — email-based flow sufficient for now
- Mobile app — web-first
- Real-time scan progress (WebSocket) — polling sufficient for now

## Context

- 45% of AI-generated code contains security flaws; 86% of AI tools fail XSS defenses; 88% fail log injection
- CVE-2025-48757 exposed 170+ Lovable apps with RLS misconfigurations leaking PII and API keys
- Lovable's built-in scanner catches vulnerabilities only 66% of the time; Bolt's fails entirely
- Founder has 40+ years cybersecurity experience (Bose, Ford, TrustEdge Labs) — deep domain credibility
- Target audience: solo developers and small teams shipping with AI tools who don't have security expertise
- Free tier is the lead generation funnel — low friction (no signup), email capture for follow-up
- One-time audit is the first revenue product — validate willingness to pay before building subscriptions
- Remediation playbooks are a key differentiator — not just "you have a vulnerability" but "here's exactly how to fix it"
- **v1.0 shipped 2026-02-06:** ~7,000 LOC Rust, ~21,000 LOC TypeScript, 165 files, 4 phases, 23 plans

## Constraints

- **Tech Stack**: Rust backend (Axum), Next.js frontend, PostgreSQL
- **Hosting**: Render — existing account and deployment experience
- **Scanning Tools**: Containerized (Nuclei, testssl.sh, custom probes) — isolation and portability
- **Payments**: Stripe — standard, reliable
- **Free Tier**: No signup required — URL + email only, maximum conversion
- **Launch Model**: Free + One-Time Audit first, subscriptions later

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust over Python for backend | Performance for concurrent scanning, type safety | Good — 5 scanners run in parallel with semaphore control |
| Next.js over HTMX for frontend | Richer interactivity for results dashboard, broader ecosystem | Good — polling, conditional rendering, client components work well |
| URL scanning before repo scanning | Faster to ship, lower friction for users, no GitHub auth complexity | Good — shipped in 3 days |
| One-time audit before subscriptions | Validate willingness to pay before building recurring billing | Pending — needs production deployment to validate |
| Containerized scanners | Isolation, reproducibility, easier deployment on Render | Good — Docker CIS hardened with 8 security flags |
| No signup for free tier | Maximize conversion, capture email for follow-up | Good — zero friction to first scan |
| Capability URL for results | Unguessable token, no auth needed, shareable | Good — simple, enables sharing |
| Database-as-queue for scans | Simple, no Redis/RabbitMQ dependency for MVP | Good — sufficient for MVP scale |
| In-memory PDF generation | No filesystem I/O, efficient for email attachment | Good — genpdf produces Vec<u8> directly |

---
*Last updated: 2026-02-06 after v1.0 milestone*
