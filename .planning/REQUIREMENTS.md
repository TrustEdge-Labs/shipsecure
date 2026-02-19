# Requirements: ShipSecure

**Defined:** 2026-02-17
**Core Value:** Catch security flaws in vibe-coded apps before they become breaches, with remediation guidance anyone can follow — no security expertise required.

## v1.6 Requirements

Requirements for Auth & Tiered Access milestone. Each maps to roadmap phases.

### Authentication

- [x] **AUTH-01**: User can sign up with email/password via Clerk
- [x] **AUTH-02**: User can sign up/in with Google OAuth
- [x] **AUTH-03**: User can sign up/in with GitHub OAuth
- [x] **AUTH-04**: User session persists across browser restarts
- [x] **AUTH-05**: Signed-in user sees UserButton (avatar/menu) in sticky header
- [x] **AUTH-06**: Dashboard routes redirect unauthenticated users to sign-in

### Domain Verification

- [x] **DOMN-01**: User can add a domain and receive a unique verification token
- [x] **DOMN-02**: User can verify domain ownership via HTML meta tag
- [x] **DOMN-03**: Verified domain displays green badge in dashboard
- [x] **DOMN-04**: System blocks verification of shared hosting TLDs (github.io, vercel.app, netlify.app, pages.dev)
- [x] **DOMN-05**: Domain verification expires after 30 days requiring re-verification

### Results Gating

- [x] **GATE-01**: API strips description/remediation from high/critical findings for anonymous scan tokens
- [x] **GATE-02**: API returns `gated: true` flag and `owner_verified` field on results
- [x] **GATE-03**: Frontend renders teaser cards with lock overlay for gated findings
- [x] **GATE-04**: Teaser cards show severity and category but not details, with "Sign up free" CTA

### Scan Tiering

- [x] **TIER-01**: Anonymous scans use lighter config (20 JS files, 180s vibecode timeout)
- [x] **TIER-02**: Authenticated scans use enhanced config (30 JS files, 300s vibecode timeout, extended exposed files)
- [x] **TIER-03**: Anonymous users limited to 1 scan per IP per 24 hours
- [x] **TIER-04**: Developer tier users limited to 5 scans per calendar month
- [x] **TIER-05**: Rate limit exceeded returns 429 with friendly message and `resets_at` timestamp
- [x] **TIER-06**: Authenticated scans require verified domain ownership

### Dashboard

- [x] **DASH-01**: Authenticated user can view paginated scan history (domain, date, severity counts, expiry)
- [x] **DASH-02**: Dashboard shows quota status ("3 of 5 scans used this month, resets Mar 1")

### Data Retention

- [x] **RETN-01**: Anonymous scan results expire after 24 hours
- [x] **RETN-02**: Developer tier scan results expire after 30 days
- [x] **RETN-03**: Background cleanup task deletes expired completed/failed scans hourly

### Cleanup

- [x] **CLEN-01**: Remove Stripe checkout flow, paid audit routes, and async-stripe/hmac/sha2/genpdf dependencies
- [x] **CLEN-02**: Change paid_audits FK to ON DELETE SET NULL; preserve all historical payment records
- [x] **CLEN-03**: Add clerk_user_id column to scans; extend tier constraint to include 'authenticated'

### Infrastructure

- [x] **INFR-01**: CORS config allows Authorization header for JWT bearer tokens
- [x] **INFR-02**: Nginx strips x-middleware-subrequest header (CVE-2025-29927 mitigation)
- [x] **INFR-03**: Clerk webhook handler verifies svix signatures on user.created events
- [x] **INFR-04**: Axum verifies Clerk JWTs locally via cached JWKS public keys (no per-request Clerk API calls)

## Future Requirements

Deferred to future release. Tracked but not in current roadmap.

### Pro Tier

- **PRO-01**: User can subscribe to Pro tier via Stripe
- **PRO-02**: Pro users get unlimited verified sites
- **PRO-03**: Pro users get unlimited scans per month
- **PRO-04**: Pro users get permanent scan history (no expiry)
- **PRO-05**: Pro users get deep scan mode (50 JS files, 600s timeout, extended templates)
- **PRO-06**: Pro users can export PDF and CSV reports
- **PRO-07**: Pro users get API access for CI/CD integration
- **PRO-08**: Pro users can schedule automated re-scans

### Conversion Enhancements

- **CONV-01**: Post-scan signup modal when anonymous scan finds high/critical findings
- **CONV-02**: Inline upgrade prompt at 80% quota (4/5 scans used)
- **CONV-03**: One-click re-scan from scan history
- **CONV-04**: Post-signup onboarding checklist (verify, scan, fix)
- **CONV-05**: Scan comparison / delta view across time

### Domain Verification Extensions

- **DOMN-06**: File upload verification method (/.well-known/shipsecure-verify.txt)
- **DOMN-07**: Periodic re-verification of domain ownership

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| DNS TXT record verification | Too opaque for vibe-coder target users who often use managed DNS (Vercel/Netlify) |
| Custom auth (JWT, sessions, passwords) | Clerk solves this completely; building auth is weeks of work |
| Multi-user organizations/teams | Target user is solo vibe-coder; Pro+ feature |
| Custom role/permission system | Three tiers map to a simple tier field; RBAC is over-engineering |
| Real-time quota countdown (WebSocket) | Poll on page load is sufficient |
| clerk-rs Rust SDK | Community-maintained, v0.4.1, 8+ months stale; security risk for JWT path |
| Redis for rate limiting | Not needed at single-container scale; DB-backed sufficient |
| GitHub repo scanning | Separate feature; auth foundation must land first |
| Mobile app | Web-first |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| AUTH-01 | Phase 29 | Complete |
| AUTH-02 | Phase 29 | Complete |
| AUTH-03 | Phase 29 | Complete |
| AUTH-04 | Phase 29 | Complete |
| AUTH-05 | Phase 29 | Complete |
| AUTH-06 | Phase 29 | Complete |
| DOMN-01 | Phase 32 | Complete |
| DOMN-02 | Phase 32 | Complete |
| DOMN-03 | Phase 32 | Complete |
| DOMN-04 | Phase 32 | Complete |
| DOMN-05 | Phase 32 | Complete |
| GATE-01 | Phase 31 | Complete |
| GATE-02 | Phase 31 | Complete |
| GATE-03 | Phase 31 | Complete |
| GATE-04 | Phase 31 | Complete |
| TIER-01 | Phase 33 | Complete |
| TIER-02 | Phase 33 | Complete |
| TIER-03 | Phase 33 | Complete |
| TIER-04 | Phase 33 | Complete |
| TIER-05 | Phase 33 | Complete |
| TIER-06 | Phase 33 | Complete |
| DASH-01 | Phase 34 | Complete |
| DASH-02 | Phase 34 | Complete |
| RETN-01 | Phase 35 | Complete |
| RETN-02 | Phase 35 | Complete |
| RETN-03 | Phase 35 | Complete |
| CLEN-01 | Phase 30 | Complete |
| CLEN-02 | Phase 30 | Complete |
| CLEN-03 | Phase 30 | Complete |
| INFR-01 | Phase 29 | Complete |
| INFR-02 | Phase 29 | Complete |
| INFR-03 | Phase 29 | Complete |
| INFR-04 | Phase 29 | Complete |

**Coverage:**
- v1.6 requirements: 33 total
- Mapped to phases: 33
- Unmapped: 0

---
*Requirements defined: 2026-02-17*
*Last updated: 2026-02-18 — AUTH-01 through AUTH-06 marked complete (Phase 29 Plan 02)*
