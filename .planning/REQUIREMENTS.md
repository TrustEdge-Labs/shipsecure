# Requirements: ShipSecure

**Defined:** 2026-04-06
**Core Value:** Catch security flaws in vibe-coded apps before they become breaches, with remediation guidance anyone can follow — no security expertise required.

## v2.0 Requirements

Requirements for supply chain scanning MVP. Each maps to roadmap phases.

### Lockfile Parsing

- [ ] **LOCK-01**: User can submit a package-lock.json (v1/v2/v3) and get all dependencies extracted
- [ ] **LOCK-02**: Parser handles lockfileVersion 1/2 (nested dependencies key) and v3 (flat packages key)
- [ ] **LOCK-03**: Parser deduplicates packages appearing at multiple paths in v3 format
- [ ] **LOCK-04**: Non-npm deps (git:, file:, link:, tarball) are counted as "unscanned"

### OSV Integration

- [ ] **OSV-01**: All extracted deps are checked against OSV.dev /v1/querybatch API
- [ ] **OSV-02**: Deps are chunked at 1000/batch and queried in parallel via futures::join_all
- [ ] **OSV-03**: OSV results are categorized: MAL- prefix → Infected, CVSS>=7/HIGH/CRITICAL → Vulnerable, other match → Advisory
- [ ] **OSV-04**: If any OSV chunk fails after 1 retry, entire scan fails with clear error

### Input & API

- [ ] **API-01**: User can paste a GitHub repo URL and get supply chain scan results
- [ ] **API-02**: User can upload a package-lock.json file (max 5MB) and get results
- [ ] **API-03**: User can paste raw lockfile content and get results
- [ ] **API-04**: GitHub URL is parsed strictly, lockfile fetched from raw.githubusercontent.com (main/master fallback)
- [ ] **API-05**: Endpoint is excluded from Clerk JWT middleware (anonymous access allowed)
- [ ] **API-06**: Existing rate limiter applied + 5000 dep count cap + 5MB body limit

### Results & Sharing

- [ ] **RES-01**: Results page displays tiered summary cards (Infected/Vulnerable/Advisory/No Known Issues/Unscanned)
- [ ] **RES-02**: Each finding shows package name, version, OSV advisory ID, description, and fix action
- [ ] **RES-03**: Results are shareable via URL with 30-day expiry
- [ ] **RES-04**: DB write failure returns results inline with "Share link unavailable" warning

### Database

- [ ] **DB-01**: Migration adds kind column to scans table (VARCHAR, default 'web_app')
- [ ] **DB-02**: Migration adds supply_chain_results JSONB column
- [ ] **DB-03**: All existing scans queries audited for kind awareness
- [ ] **DB-04**: Supply chain scans stored with expires_at explicitly set (30 days)

### Frontend

- [ ] **FE-01**: /supply-chain page with tabbed input (GitHub URL / Upload / Paste)
- [ ] **FE-02**: Loading state with "Scanning N dependencies..." spinner
- [ ] **FE-03**: /supply-chain/results/[token] page with tiered results display
- [ ] **FE-04**: Error states for all failure modes (GitHub 404, OSV down, invalid lockfile, zero deps)
- [ ] **FE-05**: Plausible events: supply_chain_scan_started, supply_chain_scan_completed, infected_found, vulnerable_found, share_clicked

### Testing

- [ ] **TEST-01**: 25 Rust unit tests (lockfile parser, OSV client, categorizer, URL parser, handler)
- [ ] **TEST-02**: 2 Rust integration tests (full scan flow with mocked OSV, rate limiting)
- [ ] **TEST-03**: 4 Vitest frontend tests (form submission, results rendering)
- [ ] **TEST-04**: 2 Playwright E2E tests (full flow, error state)

## Future Requirements

### Supply Chain Phase 2 (earned by demand)

- **SC2-01**: Yarn.lock and pnpm-lock.yaml parsing support
- **SC2-02**: Structural risk scoring (maintainer count, install scripts, provenance)
- **SC2-03**: GitHub App integration for continuous monitoring
- **SC2-04**: Normalized supply_chain_findings table (migrate from JSONB)
- **SC2-05**: Slack/email alerts when new CVE matches dependency graph
- **SC2-06**: Python/Rust/Go ecosystem support

### Supply Chain Phase 3

- **SC3-01**: Org-wide supply chain posture dashboard
- **SC3-02**: Policy engine (block PRs violating dep policies)
- **SC3-03**: "What if" compromise simulation

## Out of Scope

| Feature | Reason |
|---------|--------|
| SBOM generation | Not part of narrowest wedge |
| Transitive dependency graph visualization | Nice-to-have, not MVP |
| Private repo scanning | Requires GitHub OAuth |
| Billing/pricing tiers | Validate demand first |
| Default branch detection via GitHub API | Hardcode main/master for MVP |
| Behavioral analysis (runtime execution) | Socket.dev territory, can't out-engineer $65M |
| /blog route and /check/{platform} pages | Deprioritized by supply chain pivot |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| LOCK-01 | TBD | Pending |
| LOCK-02 | TBD | Pending |
| LOCK-03 | TBD | Pending |
| LOCK-04 | TBD | Pending |
| OSV-01 | TBD | Pending |
| OSV-02 | TBD | Pending |
| OSV-03 | TBD | Pending |
| OSV-04 | TBD | Pending |
| API-01 | TBD | Pending |
| API-02 | TBD | Pending |
| API-03 | TBD | Pending |
| API-04 | TBD | Pending |
| API-05 | TBD | Pending |
| API-06 | TBD | Pending |
| RES-01 | TBD | Pending |
| RES-02 | TBD | Pending |
| RES-03 | TBD | Pending |
| RES-04 | TBD | Pending |
| DB-01 | TBD | Pending |
| DB-02 | TBD | Pending |
| DB-03 | TBD | Pending |
| DB-04 | TBD | Pending |
| FE-01 | TBD | Pending |
| FE-02 | TBD | Pending |
| FE-03 | TBD | Pending |
| FE-04 | TBD | Pending |
| FE-05 | TBD | Pending |
| TEST-01 | TBD | Pending |
| TEST-02 | TBD | Pending |
| TEST-03 | TBD | Pending |
| TEST-04 | TBD | Pending |

**Coverage:**
- v2.0 requirements: 24 total (+ 7 testing)
- Mapped to phases: 0
- Unmapped: 31

---
*Requirements defined: 2026-04-06*
*Last updated: 2026-04-06 after initial definition*
