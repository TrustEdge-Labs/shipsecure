# Roadmap: TrustEdge Audit

**Project:** Security scanning SaaS for AI-generated web applications
**Created:** 2026-02-04
**Depth:** Quick (4 phases)
**Coverage:** 23/23 v1 requirements mapped

## Overview

TrustEdge Audit's roadmap delivers a complete security scanning platform in 4 phases: foundation infrastructure with core scanning capabilities, complete free tier user experience with all table-stakes scanners, vibe-code-specific intelligence and differentiated remediation, and paid tier monetization with professional reports. Each phase delivers observable user value and enables the next phase.

## Phases

### Phase 1: Foundation — COMPLETE

**Goal:** Backend infrastructure operational with core scanning capability

**Dependencies:** None (initial phase)

**Status:** Complete (verified 2026-02-05)

Plans:
- [x] 01-01-PLAN.md -- Rust project scaffold, dependencies, domain models, database migrations
- [x] 01-02-PLAN.md -- SSRF protection, security headers scanner, findings aggregator
- [x] 01-03-PLAN.md -- Scan orchestrator worker pool, database access layer
- [x] 01-04-PLAN.md -- API handlers, rate limiting, error handling, router wiring
- [x] 01-05-PLAN.md -- Docker infrastructure, end-to-end verification, human approval

**Requirements:**
- INFRA-01: Scan orchestrator manages concurrent scan jobs
- INFRA-02: Findings aggregator normalizes scanner output
- INFRA-03: Rate limiting restricts free tier scans
- INFRA-04: SSRF protection blocks dangerous targets
- SCAN-02: Security headers analysis

**Success Criteria:**
1. Developer can POST a URL to /api/scans and receive a scan ID
2. Backend executes security headers scan and stores findings in PostgreSQL
3. Developer can GET /api/scans/:id and retrieve scan status and results
4. System blocks scanning of localhost, internal IPs, and cloud metadata endpoints
5. System enforces rate limits (max 3 scans per email per day)

---

### Phase 2: Free Tier MVP — COMPLETE

**Goal:** Users can scan any URL for free and receive comprehensive security results via email

**Dependencies:** Phase 1 (infrastructure must exist)

**Status:** Complete (verified 2026-02-05)

Plans:
- [x] 02-01-PLAN.md -- Database schema for tokens/stages + Next.js frontend scaffold
- [x] 02-02-PLAN.md -- TLS scanner (SSL Labs) + exposed files scanner
- [x] 02-03-PLAN.md -- JS secrets scanner + containerized scanner execution (Docker)
- [x] 02-04-PLAN.md -- Orchestrator wiring (all 4 scanners + stage tracking) + email via Resend
- [x] 02-05-PLAN.md -- API extensions (token results, markdown download, scan counter)
- [x] 02-06-PLAN.md -- Landing page with scan form and value proposition
- [x] 02-07-PLAN.md -- Progress page with stage checklist + results dashboard
- [x] 02-08-PLAN.md -- Docker Compose full stack + E2E tests + human verification

**Requirements:**
- UI-01: Landing page with URL input form
- UI-02: Scan status page with progress indicator
- UI-03: Results dashboard with severity badges
- SCAN-01: User submits URL and email (no signup)
- SCAN-03: TLS/certificate analysis via SSL Labs API
- SCAN-04: Exposed file/directory detection
- SCAN-05: JavaScript secret scanning
- INFRA-05: Containerized scanner execution with security hardening (Nuclei, testssl.sh)
- DLVR-01: Results page with findings by severity
- DLVR-02: Plain-language explanations and remediation steps
- DLVR-03: Overall security score (A-F grade)
- DLVR-04: Email delivery when scan completes

**Success Criteria:**
1. User pastes a URL and email on landing page, clicks scan, and receives scan ID
2. User sees scan progress page that polls every 2 seconds until completion
3. User views results dashboard showing Critical/High/Medium/Low findings with severity badges
4. User receives email with scan summary and link to full results within 5 minutes
5. Each finding displays plain-language explanation and actionable remediation steps
6. Results page displays overall security score (A-F letter grade)

---

### Phase 3: Vibe-Code Intelligence

**Goal:** TrustEdge auto-detects frameworks and provides copy-paste remediation fixes

**Dependencies:** Phase 2 (requires scan results to test detection patterns)

**Requirements:**
- VIBE-01: Custom Nuclei templates detect vibe-code vulnerabilities
- VIBE-02: Auto-detect framework and platform from HTML/JS patterns
- VIBE-03: Copy-paste code fixes specific to detected framework

**Success Criteria:**
1. Scanner correctly identifies Next.js, Vite, React apps from HTML/JS patterns
2. Scanner correctly identifies Vercel, Netlify, Railway deployment platforms
3. Results page displays detected framework and platform badges
4. Remediation steps include copy-paste code fixes tailored to detected framework
5. Scanner detects Supabase RLS misconfigurations, Firebase permissive rules, and Vercel env leaks

---

### Phase 4: Monetization

**Goal:** Users can purchase paid audits and receive professional PDF reports

**Dependencies:** Phase 3 (paid tier requires all free features operational)

**Requirements:**
- PAY-01: One-time paid audit via Stripe Checkout
- PAY-02: Paid audit runs deeper scanning beyond free tier
- PAY-03: Free results page includes upgrade CTAs

**Success Criteria:**
1. User clicks "Upgrade to Deep Audit" CTA on free results page
2. User completes Stripe Checkout for $49-99 one-time payment
3. System initiates deeper scan with additional Nuclei templates and active probes
4. User receives email with PDF report attached
5. PDF report includes executive summary, findings by severity, and remediation roadmap

---

## Progress

| Phase | Status | Requirements | Success Criteria | Completion |
|-------|--------|--------------|------------------|------------|
| 1 - Foundation | Complete | 5 | 5/5 | 100% |
| 2 - Free Tier MVP | Complete | 12 | 6/6 | 100% |
| 3 - Vibe-Code Intelligence | Pending | 3 | 5 | 0% |
| 4 - Monetization | Pending | 3 | 5 | 0% |

**Total:** 23/23 requirements mapped, 21 success criteria defined
**Phase 1:** Verified 2026-02-05 — all 5 success criteria passed
**Phase 2:** Verified 2026-02-05 — all 6 success criteria passed

---

## Deferred to v2

- PDF report generation (PDF-01, PDF-02) -- moved to Phase 4 paid tier instead
- Continuous monitoring (MON-01, MON-02, MON-03) -- requires GitHub webhooks and cert monitoring
- Repo scanning (REPO-01, REPO-02, REPO-03, REPO-04) -- Semgrep, Gitleaks, dependency scanning
- Subscription tiers (SUB-01, SUB-02) -- Pro and Agency recurring billing

---

**Roadmap created:** 2026-02-04
**Last updated:** 2026-02-05
**Next step:** `/gsd:plan-phase 3`
