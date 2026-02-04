# TrustEdge Audit

## What This Is

A SaaS security scanning platform that targets developers using AI code generation tools (Cursor, Bolt, Lovable, etc.) who ship fast but lack security expertise. It orchestrates containerized open-source security tools, applies vibe-code-specific detection rules, and delivers actionable remediation guidance written for non-security professionals.

## Core Value

Catch security flaws in vibe-coded apps before they become breaches, with remediation guidance anyone can follow — no security expertise required.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Free URL scan with no signup (paste URL + email, get results)
- [ ] Security headers analysis (CSP, HSTS, X-Frame-Options, etc.)
- [ ] TLS/certificate analysis with SSL Labs API
- [ ] Exposed file/directory detection (/.env, /.git, /debug, etc.)
- [ ] Client-side JavaScript secret scanning (API keys, tokens in bundles)
- [ ] Vibe-code-specific checks via Nuclei templates (Supabase RLS, Firebase rules, Vercel leaks)
- [ ] One-time paid audit ($49-99) with deeper active scanning
- [ ] PDF report generation with findings by severity and copy-paste remediation steps
- [ ] Email delivery of scan results and reports
- [ ] Stripe integration for one-time audit payments
- [ ] Landing page with URL input form and clear value proposition
- [ ] Results dashboard showing findings, severity, and remediation guidance
- [ ] Scan orchestrator managing concurrent scan jobs
- [ ] Findings aggregator that deduplicates, prioritizes, and maps to remediation playbooks

### Out of Scope

- GitHub repo scanning (Semgrep, Gitleaks, dependency analysis) — fast follow after launch, not MVP
- Pro subscription tier ($149/month continuous monitoring) — add after validating one-time audit demand
- Agency tier (white-label reports, multi-repo) — future tier after Pro is proven
- Certificate expiration monitoring — Pro tier feature
- GitHub webhook triggers for automated re-scanning — Pro tier feature
- OWASP ZAP baseline scanning — defer to paid active scanning phase
- OAuth/social login — email-based auth sufficient for MVP
- Mobile app — web-first
- Real-time scan progress (WebSocket) — polling or email notification sufficient for MVP

## Context

- 45% of AI-generated code contains security flaws; 86% of AI tools fail XSS defenses; 88% fail log injection
- CVE-2025-48757 exposed 170+ Lovable apps with RLS misconfigurations leaking PII and API keys
- Lovable's built-in scanner catches vulnerabilities only 66% of the time; Bolt's fails entirely
- Founder has 40+ years cybersecurity experience (Bose, Ford, TrustEdge Labs) — deep domain credibility
- Target audience: solo developers and small teams shipping with AI tools who don't have security expertise
- Free tier is the lead generation funnel — low friction (no signup), email capture for follow-up
- One-time audit is the first revenue product — validate willingness to pay before building subscriptions
- Remediation playbooks are a key differentiator — not just "you have a vulnerability" but "here's exactly how to fix it"

## Constraints

- **Tech Stack**: Rust backend (Axum or Actix-web), Next.js frontend, PostgreSQL — chosen by founder
- **Hosting**: Render — existing account and deployment experience
- **Scanning Tools**: Containerized (Nuclei, testssl.sh, custom probes) — isolation and portability
- **Payments**: Stripe — standard, reliable
- **Free Tier**: No signup required — URL + email only, maximum conversion
- **Launch Model**: Free + One-Time Audit first, subscriptions later — validate demand before recurring billing complexity

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust over Python for backend | Performance for concurrent scanning, type safety | — Pending |
| Next.js over HTMX for frontend | Richer interactivity for results dashboard, broader ecosystem | — Pending |
| URL scanning before repo scanning | Faster to ship, lower friction for users, no GitHub auth complexity | — Pending |
| One-time audit before subscriptions | Validate willingness to pay before building recurring billing | — Pending |
| Containerized scanners | Isolation, reproducibility, easier deployment on Render | — Pending |
| No signup for free tier | Maximize conversion, capture email for follow-up | — Pending |

---
*Last updated: 2026-02-04 after initialization*
