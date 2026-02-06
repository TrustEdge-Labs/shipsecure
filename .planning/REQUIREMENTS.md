# Requirements: TrustEdge Audit

**Defined:** 2026-02-04
**Core Value:** Catch security flaws in vibe-coded apps before they become breaches, with remediation guidance anyone can follow.

## v1 Requirements

### Scanning

- [x] **SCAN-01**: User can submit a URL and email to start a free security scan (no signup required)
- [x] **SCAN-02**: Scanner checks security headers (CSP, HSTS, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy)
- [x] **SCAN-03**: Scanner analyzes TLS/certificate configuration via SSL Labs API (grade, protocol versions, cert expiry, cipher suites)
- [x] **SCAN-04**: Scanner probes for exposed files and directories (/.env, /.git/config, /debug, /admin, source maps, robots.txt, sitemap.xml)
- [x] **SCAN-05**: Scanner fetches and scans JavaScript bundles for hardcoded secrets (API keys, Stripe keys, Supabase anon keys, Firebase configs)

### Vibe-Code Detection

- [ ] **VIBE-01**: Custom Nuclei templates detect vibe-code-specific vulnerabilities (Supabase RLS misconfig, Firebase permissive rules, Vercel env leaks, Netlify function exposure)
- [ ] **VIBE-02**: Scanner auto-detects framework and deployment platform from HTML/JS patterns (Next.js, Vite, React, Vercel, Netlify, Railway)
- [ ] **VIBE-03**: Remediation guidance includes copy-paste code fixes specific to the detected framework

### Results & Delivery

- [x] **DLVR-01**: User sees a web results page with findings grouped by severity (Critical/High/Medium/Low/Info)
- [x] **DLVR-02**: Each finding shows plain-language explanation, risk context, and actionable remediation steps
- [x] **DLVR-03**: Results page displays an overall security score (A-F letter grade)
- [x] **DLVR-04**: User receives email with scan summary and link to full results when scan completes

### Payments & Upsell

- [ ] **PAY-01**: User can purchase a one-time deep audit ($49-99) via Stripe Checkout
- [ ] **PAY-02**: Paid audit runs additional Nuclei vibe-code templates and deeper active scanning beyond free tier
- [ ] **PAY-03**: Free results page includes clear CTAs to upgrade to paid audit

### Frontend

- [x] **UI-01**: Landing page with URL input form, email field, and clear value proposition
- [x] **UI-02**: Scan status page with progress indicator while scan runs (polling-based)
- [x] **UI-03**: Results dashboard with severity badges, expandable findings, and remediation sections

### Infrastructure

- [x] **INFRA-01**: Scan orchestrator manages concurrent scan jobs with parallel scanner execution and per-scanner timeouts
- [x] **INFRA-02**: Findings aggregator normalizes output from multiple scanners, deduplicates, and maps severity scores
- [x] **INFRA-03**: Rate limiting restricts free tier scans per email and per domain per day
- [x] **INFRA-04**: SSRF protection blocks scanning of localhost, internal IPs, and cloud metadata endpoints
- [x] **INFRA-05**: Containerized scanner execution (Nuclei, testssl.sh) with non-root user, resource limits, and read-only filesystems

## v2 Requirements

### PDF Reports

- **PDF-01**: Professional branded PDF report with executive summary, findings by severity, and remediation roadmap
- **PDF-02**: PDF attached to paid audit email delivery

### Continuous Monitoring

- **MON-01**: GitHub webhook receiver triggers automated re-scan on push
- **MON-02**: Certificate expiration monitoring with email alerts
- **MON-03**: Email alerts for new findings detected on re-scan

### Repo Scanning

- **REPO-01**: GitHub App integration for repo access
- **REPO-02**: Semgrep static analysis with custom vibe-code rulesets
- **REPO-03**: Gitleaks/TruffleHog secret detection in committed code
- **REPO-04**: Dependency scanning (npm audit, pip-audit, OSV-Scanner)

### Subscriptions

- **SUB-01**: Pro tier ($149/month) with continuous monitoring and unlimited scans
- **SUB-02**: Agency tier ($299-499/month) with white-label reports and multi-repo support

## Out of Scope

| Feature | Reason |
|---------|--------|
| OAuth/social login | Email-based flow sufficient for MVP |
| Mobile app | Web-first, mobile later |
| Real-time WebSocket scan progress | Polling sufficient for MVP |
| OWASP ZAP baseline scanning | Defer to post-MVP active scanning |
| API-first architecture | Build for web UI first, API later |
| Team/organization features | Validate individual dev PMF first |
| Scan scheduling/CRON | On-demand scanning only for MVP |
| Browser extension | URL-based scanning is core model |
| Automated fix deployment | Users need control over changes |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| SCAN-01 | Phase 2 | Complete |
| SCAN-02 | Phase 1 | Complete |
| SCAN-03 | Phase 2 | Complete |
| SCAN-04 | Phase 2 | Complete |
| SCAN-05 | Phase 2 | Complete |
| VIBE-01 | Phase 3 | Pending |
| VIBE-02 | Phase 3 | Pending |
| VIBE-03 | Phase 3 | Pending |
| DLVR-01 | Phase 2 | Complete |
| DLVR-02 | Phase 2 | Complete |
| DLVR-03 | Phase 2 | Complete |
| DLVR-04 | Phase 2 | Complete |
| PAY-01 | Phase 4 | Pending |
| PAY-02 | Phase 4 | Pending |
| PAY-03 | Phase 4 | Pending |
| UI-01 | Phase 2 | Complete |
| UI-02 | Phase 2 | Complete |
| UI-03 | Phase 2 | Complete |
| INFRA-01 | Phase 1 | Complete |
| INFRA-02 | Phase 1 | Complete |
| INFRA-03 | Phase 1 | Complete |
| INFRA-04 | Phase 1 | Complete |
| INFRA-05 | Phase 2 | Complete |

**Coverage:**
- v1 requirements: 23 total
- Mapped to phases: 23
- Unmapped: 0

**Phase distribution:**
- Phase 1 (Foundation): 6 requirements
- Phase 2 (Free Tier MVP): 11 requirements
- Phase 3 (Vibe-Code Intelligence): 3 requirements
- Phase 4 (Monetization): 3 requirements

---
*Requirements defined: 2026-02-04*
*Last updated: 2026-02-05 after Phase 2 completion*
