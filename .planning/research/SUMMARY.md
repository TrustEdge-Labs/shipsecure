# Project Research Summary

**Project:** TrustEdge Audit
**Domain:** Security scanning SaaS for AI-generated web applications
**Researched:** 2026-02-04
**Confidence:** MEDIUM

## Executive Summary

TrustEdge Audit is a security scanning SaaS specifically targeting AI-generated web applications (Cursor, Bolt, Lovable output). Based on research into security scanning platforms, the recommended approach is a Rust backend with containerized scanner tools, PostgreSQL for data persistence, and Next.js frontend, deployed on Render. This architecture balances performance, safety, and operational simplicity while avoiding the complexity of enterprise-grade monitoring tools.

The product differentiates itself by targeting non-security developers who need to ship safely but lack security expertise. Unlike competitors (Snyk, Detectify) that focus on continuous monitoring for security teams, TrustEdge offers one-time audits with vibe-code-specific detection and plain-language remediation. The free tier provides immediate value (basic scans via email), while the paid tier ($99-299) delivers comprehensive audits with manual review and PDF reports.

Critical risks center on credibility: false positives will kill the brand immediately, and legal liability (CFAA compliance) requires careful TOS/consent design. Performance is essential — scans taking over 2 minutes will cause user abandonment. The recommended mitigation is strict confidence filtering for findings, legal review before launch, parallel scanner execution, and aggressive caching of external API calls (especially SSL Labs).

## Key Findings

### Recommended Stack

**Backend:** Rust with Axum provides idiomatic async/await patterns, excellent for I/O-bound scanning workloads. Axum's tower middleware ecosystem handles auth, rate limiting, and tracing better than Actix-web's actor model complexity. SQLx enables compile-time query verification without ORM bloat.

**Core technologies:**
- **Axum + Tokio**: HTTP server and async runtime — simpler than Actix-web, better ecosystem fit
- **PostgreSQL + SQLx**: Data persistence with JSONB for flexible scanner output — compile-time query checking, native async
- **bollard**: Docker API client for containerized scanner isolation — Rust-native, handles Nuclei/testssl.sh containers
- **Next.js 14 (App Router)**: Frontend with SSR for results pages — chosen by founder, excellent DX
- **Resend**: Email delivery for scan results — better DX than AWS SES, generous free tier
- **printpdf**: PDF report generation — pure Rust, no Chrome dependency
- **Database-as-queue**: PostgreSQL polling for job orchestration — avoids Redis complexity for MVP

**Critical version requirements:**
- Axum 0.7.x with tower-http for middleware
- SQLx 0.8.x with postgres, json, uuid features
- Next.js 14.x App Router (stable)

**Confidence:** MEDIUM for version numbers (based on Jan 2025 training data), HIGH for architectural patterns.

### Expected Features

**Must have (table stakes):**
- SSL/TLS configuration scan (certificate validity, protocol versions, cipher suites)
- Security headers check (CSP, HSTS, X-Frame-Options, X-Content-Type-Options)
- Basic OWASP Top 10 checks (XSS, injection points)
- Severity scoring (Critical/High/Medium/Low)
- Email delivery of results (no signup friction for free tier)
- Clear pass/fail visual status
- Response time under 5 minutes
- Remediation links to fixing issues

**Should have (competitive differentiators):**
- Framework detection (Next.js, Vite, CRA patterns)
- No-jargon explanations for non-security developers
- Copy-paste code fixes specific to detected framework
- Basic vibe-code pattern detection (common AI scaffolding issues)
- Manual expert review for paid tier
- PDF report (professional, shareable with stakeholders)
- Video walkthrough of findings (5-10 min recording)
- Compliance framework mapping (SOC 2, GDPR basics)

**Defer (v2+):**
- Scan history dashboard (requires user accounts)
- Progress tracking across rescans
- Shareable security badge
- One-click fix PRs (requires GitHub integration)
- Real-time monitoring/alerts (different market)

**Confidence:** MEDIUM (based on competitor analysis through early 2025).

### Architecture Approach

Security scanning SaaS follows a producer-consumer pattern with clear separation between web API, job orchestration, scanner execution, and result processing. The recommended architecture uses an in-process worker pool (tokio tasks) polling PostgreSQL for pending jobs, executing scanners (mix of in-process Rust and containerized tools), normalizing findings to a common schema, and delivering results via email/PDF.

**Major components:**
1. **Web API (Axum)** — HTTP request handling, scan creation, status polling, results retrieval
2. **Database (PostgreSQL)** — Persistent storage with tables for scans, scan_jobs, findings, users, payments
3. **Scanner Orchestrator (Rust worker pool)** — Job queue processing with semaphore-based concurrency control (max 5 concurrent)
4. **Scanner Execution Layer** — Mix of in-process (headers, secrets) and containerized (TLS, Nuclei) scanners
5. **Findings Processing** — Parse scanner output, normalize to common schema, deduplicate, score severity
6. **Frontend (Next.js)** — Landing page (SSG), results dashboard (SSR), polling for scan status
7. **Email Service (Resend)** — Transactional emails with templates for free/paid tiers
8. **PDF Generator (printpdf)** — Professional reports for paid tier

**Key patterns:**
- Async job queue (create scan, return immediately, poll for status)
- Graceful scanner failure (partial results better than none)
- Idempotent job processing (reprocessing same job produces same result)
- Timeout enforcement (5 min hard limit per scanner)
- Container security (non-root user, resource limits, network isolation)

**Confidence:** HIGH (producer-consumer pattern is standard for scanning SaaS).

### Critical Pitfalls

1. **False Positive Epidemic** — Scanner produces too many false positives, users lose trust immediately and never return. Prevention: strict confidence filtering, validation layer, manual review of first 100 scans before launch, explainability for every finding.

2. **Legal Liability Landmine** — Scanning websites without proper authorization creates CFAA exposure and potential lawsuits. Prevention: TOS acceptance even for "no signup" free tier, explicit consent ("you confirm you own or have authorization"), result privacy (auto-expire after 30 days), legal review BEFORE launch.

3. **SSL Labs API Abuse → IP Ban** — Aggressive use of SSL Labs API gets entire platform IP-banned, breaking core TLS scanning. Prevention: read API terms thoroughly, aggressive caching (24-48 hours), single-threaded requests with delays, fallback to testssl.sh container.

4. **Scanner Itself is Insecure** — Security tool has vulnerabilities (SSRF, container escape, code injection). Prevention: SSRF protection (blocklist localhost, internal IPs, cloud metadata endpoints), container hardening (non-root, read-only FS, resource limits), input sanitization (never pass user input to shell), secret redaction in logs.

5. **Performance Death Spiral** — Scans take 5+ minutes, users abandon, product feels broken. Prevention: parallel scanner execution (tokio), streaming results as they arrive, progress indicators, timeouts (30-60s per scanner), container pre-warming on Render.

**Additional moderate pitfalls:**
- Race to bottom pricing (security SaaS commands premium, research Snyk/GitGuardian pricing)
- Async complexity explosion (use scanner trait abstraction from day 1)
- Report quality mismatch (test remediation steps, use plain language)
- Database bloat (retention policy: delete free scans after 7-30 days)
- Free tier abuse (require email, rate limit to 2-3 scans/domain/week)

**Confidence:** HIGH for false positives and performance (universal patterns), MEDIUM for legal (needs attorney), MEDIUM for SSL Labs (needs API docs verification).

## Implications for Roadmap

Based on research, suggested phase structure follows dependency order and risk mitigation:

### Phase 1: Core Infrastructure + Simple Scanner
**Rationale:** Establish foundation before adding complexity. Start with simplest scanner (headers) to validate end-to-end flow before containerized scanners.

**Delivers:**
- PostgreSQL schema (scans, scan_jobs, findings tables)
- Rust backend skeleton (Axum, SQLx, health check)
- Headers scanner (in-process, reqwest-based)
- POST /api/scans and GET /api/scans/:id endpoints
- Worker loop polling for pending jobs
- Deployed to Render

**Addresses (from FEATURES.md):**
- Security headers check (table stakes)
- Severity scoring (table stakes)

**Avoids (from PITFALLS.md):**
- Async complexity explosion — establish scanner trait abstraction upfront
- Database bloat — include retention policy in initial schema

**Research flag:** Standard patterns, no additional research needed.

---

### Phase 2: Frontend MVP + Email Delivery
**Rationale:** Enable end-to-end user experience before adding more scanners. Users need to see results and receive emails to validate value proposition.

**Delivers:**
- Next.js landing page with URL + email form
- Polling mechanism (GET /api/scans/:id every 2s)
- Results page displaying findings
- Resend integration for email delivery
- Free tier email template (summary + link)
- Deployed to Render

**Addresses (from FEATURES.md):**
- Email delivery of results (table stakes)
- Clear pass/fail visual status (table stakes)
- Mobile-friendly results (table stakes)
- Response time under 5 minutes (table stakes)

**Avoids (from PITFALLS.md):**
- Performance death spiral — implement polling, progress indicators
- Legal liability — add TOS acceptance checkbox before scan
- Free tier abuse — require email, implement basic rate limiting

**Research flag:** Standard patterns, no additional research needed.

---

### Phase 3: Additional Scanners (TLS, Secrets, Files)
**Rationale:** Expand scanning capabilities once infrastructure and UX proven. Introduces containerized scanners (testssl.sh, Nuclei).

**Delivers:**
- TLS scanner (testssl.sh in container via bollard)
- Secrets scanner (regex-based, in-process)
- File scanner (Nuclei in container)
- Findings normalization for each scanner
- Deduplication logic
- Parallel scanner execution (tokio)

**Addresses (from FEATURES.md):**
- SSL/TLS configuration scan (table stakes)
- Known vulnerability detection (table stakes)
- Basic XSS/injection checks (table stakes)

**Avoids (from PITFALLS.md):**
- SSL Labs API abuse — implement testssl.sh as fallback, cache results
- Scanner security — SSRF protection, container hardening, non-root users
- False positive epidemic — strict confidence filtering, validation layer

**Research flag:** NEEDS PHASE RESEARCH
- SSL Labs API: rate limits, terms, caching strategy
- Container security: Render Docker support, resource limits
- Scanner tool selection: testssl.sh vs alternatives, Nuclei template customization

---

### Phase 4: Framework Detection + Copy-Paste Fixes
**Rationale:** Differentiation features that make TrustEdge valuable for non-security developers. Requires existing scan results to test against.

**Delivers:**
- Framework detection (Next.js, Vite, CRA patterns)
- Remediation playbook mapping (YAML-based)
- Copy-paste code fixes per framework
- No-jargon explanations
- Enhanced results page with remediation tabs

**Addresses (from FEATURES.md):**
- Framework detection (differentiator)
- Copy-paste fixes (differentiator)
- No-jargon explanations (differentiator)
- Remediation links (table stakes)

**Avoids (from PITFALLS.md):**
- Report quality mismatch — test remediation steps, use plain language
- False positives — context-aware detection reduces noise

**Research flag:** NEEDS PHASE RESEARCH
- Framework fingerprinting: detection patterns for Next.js, Vite, CRA
- Remediation templates: framework-specific code examples
- Vibe-code patterns: common AI scaffolding issues

---

### Phase 5: Payments + Paid Tier Scanning
**Rationale:** Monetization after free tier provides value. Paid tier requires all free features plus deeper scanning.

**Delivers:**
- Stripe integration (checkout session, webhook)
- Tier-based scan logic (free vs paid)
- Deeper scanning for paid tier (more comprehensive checks)
- Paid tier email template
- Payment tracking in database

**Addresses (from FEATURES.md):**
- Paid audit: manual review prep (differentiator)
- Priority email support (differentiator)

**Avoids (from PITFALLS.md):**
- Race to bottom pricing — research security SaaS pricing before setting
- Free tier abuse — implement tier enforcement

**Research flag:** Standard Stripe patterns, no additional research needed.

---

### Phase 6: PDF Reports + Manual Review
**Rationale:** Paid tier deliverable after payment flow established. Manual review requires all scanners operational.

**Delivers:**
- printpdf integration
- PDF report template (executive summary, findings, remediation roadmap)
- GET /api/scans/:id/pdf endpoint
- PDF caching in database
- Manual review workflow (for founder)
- Video walkthrough recording workflow

**Addresses (from FEATURES.md):**
- PDF report (differentiator)
- Manual expert review (differentiator)
- Video walkthrough (differentiator)
- Compliance-speak translation (differentiator)

**Avoids (from PITFALLS.md):**
- Report quality mismatch — professional design, tested remediation steps
- Performance — cache PDFs, don't regenerate on every request

**Research flag:** Standard patterns, no additional research needed.

---

### Phase 7: Dashboard + Scan History
**Rationale:** User accounts and history after paid tier proven. Enables retention and upsell.

**Delivers:**
- User authentication (JWT or session)
- Dashboard page (list user's scans)
- Scan history with comparison
- Rescan functionality
- Progress tracking dashboard

**Addresses (from FEATURES.md):**
- Scan history (deferred to v2)
- Progress tracking dashboard (differentiator)

**Avoids (from PITFALLS.md):**
- Database bloat — pagination, compression of old scans

**Research flag:** Standard auth patterns, no additional research needed.

---

### Phase Ordering Rationale

**Dependency-driven:**
- Phase 1 before 2: Infrastructure required for frontend
- Phase 2 before 3: UX validation before expanding scanners
- Phase 3 before 4: Scan results needed to build framework detection
- Phase 5 before 6: Payment flow before paid deliverables
- Phase 6 before 7: Core paid features before dashboard polish

**Risk mitigation:**
- Legal review happens before Phase 2 launch (TOS, consent)
- False positive prevention starts in Phase 3 (confidence filtering)
- Performance optimization throughout (parallel execution, timeouts, caching)

**Grouping logic:**
- Phases 1-2: MVP foundation (infrastructure + UX)
- Phases 3-4: Free tier feature completeness (scanners + differentiation)
- Phases 5-6: Paid tier implementation (monetization + deliverables)
- Phase 7: Retention features (dashboard, history)

### Research Flags

**Phases needing deeper research during planning:**
- **Phase 3 (Additional Scanners):** SSL Labs API documentation, container security on Render, scanner tool selection and configuration
- **Phase 4 (Framework Detection):** Framework fingerprinting patterns, vibe-code detection heuristics, remediation template creation

**Phases with standard patterns (skip /gsd:research-phase):**
- **Phase 1:** Standard Rust backend setup, PostgreSQL schema design
- **Phase 2:** Standard Next.js + React patterns, email integration
- **Phase 5:** Standard Stripe integration patterns
- **Phase 6:** Standard PDF generation, template design
- **Phase 7:** Standard auth + dashboard patterns

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | MEDIUM | Version numbers based on Jan 2025 training data; architectural patterns HIGH confidence |
| Features | MEDIUM | Based on competitor analysis through early 2025; table stakes patterns HIGH confidence |
| Architecture | HIGH | Producer-consumer pattern is standard for scanning SaaS; Rust async patterns well-understood |
| Pitfalls | MEDIUM-HIGH | False positives and performance are universal patterns (HIGH); legal and SSL Labs need verification (MEDIUM) |

**Overall confidence:** MEDIUM

### Gaps to Address

**Requires verification before implementation:**

1. **SSL Labs API:** Current rate limits, terms of service, what triggers bans
   - Action: Read https://github.com/ssllabs/ssllabs-scan/wiki/Documentation
   - Impact: Phase 3 scanner implementation

2. **CFAA compliance:** Specific legal requirements for security scanning services
   - Action: Consult attorney specializing in CFAA/cybersecurity law
   - Impact: PRE-MVP requirement (TOS, consent flow)

3. **Render platform:** Container resource limits, Docker-in-Docker support, cold start characteristics
   - Action: Read https://render.com/docs/docker and test deployment
   - Impact: Phase 1 deployment, Phase 3 containerized scanners

4. **Security SaaS pricing:** Market rates for comparable products
   - Action: Research Snyk, GitGuardian, Aikido Security pricing
   - Impact: Phase 5 pricing decisions

5. **Rust crate versions:** Current stable versions of Axum, SQLx, bollard, etc.
   - Action: Check crates.io for latest releases
   - Impact: Phase 1 dependency selection

**Assumptions needing validation:**

- PostgreSQL polling queue is sufficient for <1000 scans/day (may need Redis at scale)
- printpdf produces professional-quality reports (may need WeasyPrint alternative)
- Render free tier supports Docker workloads (may require paid plan)
- $99-299 one-time audit pricing is defensible (needs market validation)
- Framework detection via HTML/JS pattern matching is feasible (needs prototyping)

**Known unknowns for future phases:**

- Container image caching on Render (affects scanner cold start time)
- PostgreSQL connection pool size optimization for Render
- PDF report size limits (affects database BYTEA storage strategy)
- Scanner timeout tuning (95th percentile runtime per scanner type)
- Optimal concurrency limit (when to switch from in-process to separate worker service)

## Sources

### Primary (HIGH confidence)
- STACK.md: Rust/Axum/PostgreSQL patterns, container orchestration
- ARCHITECTURE.md: Producer-consumer patterns for scanning SaaS, job queue design
- PITFALLS.md: False positives, legal liability, performance patterns

### Secondary (MEDIUM confidence)
- FEATURES.md: Competitor analysis (Snyk, Detectify, Mozilla Observatory) through early 2025
- Training data: Security scanning platform architectures as of January 2025

### Tertiary (LOW confidence - needs validation)
- Render platform capabilities (Docker support, resource limits)
- SSL Labs API rate limits (training data suggests 1 scan/host/hour)
- Current version numbers for Rust crates (Axum 0.7.x, SQLx 0.8.x)
- Security SaaS pricing benchmarks ($99-299 suggested range)

### Recommended authoritative sources for validation:
- https://docs.rs (Rust crate documentation)
- https://nextjs.org/docs (Next.js official docs)
- https://stripe.com/docs (Stripe API docs)
- https://render.com/docs (Render deployment guides)
- https://github.com/tokio-rs/axum (Axum examples)
- https://github.com/ssllabs/ssllabs-scan/wiki/Documentation (SSL Labs API)
- OWASP Scanner Guidance (scanner deployment best practices)

---

**Research completed:** 2026-02-04
**Ready for roadmap:** Yes

**Next steps:**
1. Legal review of TOS/consent flow (CRITICAL, PRE-MVP)
2. Verify Render Docker support and resource limits
3. Research SSL Labs API documentation
4. Validate security SaaS pricing with competitor analysis
5. Check current stable versions of Rust dependencies
6. Create roadmap based on suggested 7-phase structure
