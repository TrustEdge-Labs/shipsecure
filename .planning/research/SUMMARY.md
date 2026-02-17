# Project Research Summary

**Project:** ShipSecure
**Domain:** Auth (Clerk), domain ownership verification, tiered scan access, results gating, scan history, rate limiting, data retention, Stripe removal
**Researched:** 2026-02-17
**Confidence:** HIGH

## Executive Summary

ShipSecure v1.6 adds a tiered access model to an existing, working security scanner. The product moves from a simple anonymous/paid-$49 binary to a three-tier model: anonymous (unauthenticated, IP-limited, low/medium findings only), developer (free Clerk signup + domain verification, full findings, monthly quota), and pro (future Stripe subscription). The recommended approach is Clerk for identity with `@clerk/nextjs` v6 on the frontend and `axum-jwt-auth` + `jsonwebtoken` for local JWT verification on the Axum backend. This milestone removes Stripe entirely for now — the $49 one-time audit is gone, and Stripe re-enters only when the pro tier is built. Net dependency count stays flat: four Rust crates removed (async-stripe, hmac, sha2, genpdf), four added (jsonwebtoken, axum-jwt-auth, svix, tokio-cron-scheduler). One frontend package added (@clerk/nextjs).

The core conversion mechanic is results gating: anonymous users see teaser cards for high/critical findings but never the details. This must be enforced server-side — the existing `GET /api/v1/results/:token` handler currently returns all findings regardless of tier, which is a critical security hole if frontend-only gating is implemented. The recommended architecture keeps the backend authoritative: for anonymous scan tokens, strip `description` and `remediation` from high/critical findings before serializing, and return a `gated: true` flag per finding. Frontend `AuthGate` renders accordingly. Domain ownership verification uses meta tag or well-known file methods (not DNS TXT — too opaque for vibe-coder target users); the existing `reqwest` and `scraper` crates handle the HTTP fetch and HTML parse with no new dependencies.

The primary risks are security-structural: frontend-only access gating, missing CORS `Authorization` header in Axum, Next.js CVE-2025-29927 middleware bypass on self-hosted deployments, and the Stripe removal cascade-deleting payment history via `ON DELETE CASCADE` on `paid_audits`. Each has a clear prevention strategy. The build order is strictly dependency-driven — DB migrations and Clerk JWT infrastructure must land before domain verification or scan history work begins. Estimated total effort is 24-30 hours across 3-4 development days.

---

## Key Findings

### Recommended Stack

The existing Axum 0.8.8 + sqlx 0.8.6 + PostgreSQL + Next.js 16.1.6 + React 19 stack needs only targeted additions. Clerk (`@clerk/nextjs` v6.37.x) is the right auth choice: it provides first-class Next.js App Router support, pre-built `<SignIn>`, `<SignUp>`, `<UserButton>` components, Google/GitHub OAuth at zero backend cost, and session management with automatic JWT refresh. On the Axum side, `clerk-rs` is explicitly rejected — it is community-maintained with low activity (v0.4.1, 8+ months stale) and is not a safe dependency for a security-critical verification path. The standard `jsonwebtoken` crate + `axum-jwt-auth` wrapper is the correct choice for local JWKS-based JWT verification with caching.

One critical Next.js 16 detail: the middleware filename changed from `middleware.ts` to `proxy.ts` in Next.js 16. Clerk docs (updated Feb 2026) explicitly note this. The code inside is identical — only the filename differs.

**Core technologies:**
- `@clerk/nextjs` v6: Auth SDK, pre-built UI components, `clerkMiddleware()` — first-class App Router support with React 19
- `jsonwebtoken` + `axum-jwt-auth`: Axum JWT verification with JWKS caching — industry-standard, avoids per-request Clerk API calls
- `svix` v1 (1.83.0, Dec 2025): Clerk webhook signature verification — Clerk uses Svix infrastructure, actively maintained
- `tokio-cron-scheduler` v0.15: In-process hourly retention cleanup — no Redis, no external scheduler, fits single-container deployment
- Tier metadata in Clerk `publicMetadata`: Store `tier`, `domainVerified`, `verifiedDomain` — readable in JWT without Clerk API roundtrip
- DB-backed rate limiting (existing pattern extended): Per-user monthly quotas via `user_id` — avoids tower-governor which is IP-only and tier-unaware

**What NOT to add:**
- `clerk-rs`: Community-maintained, low activity (v0.4.1, 8+ months stale) — security risk for JWT verification path
- `tower-governor`: IP-only rate limiting, does not understand user tiers or monthly windows
- `redis`: Not needed — single-container deployment, DB-backed rate limits are sufficient
- `next-auth`: Clerk is the decision; do not add a second auth system
- DNS TXT verification: Too opaque for vibe-coder target users who often use managed DNS (Vercel/Netlify)

### Expected Features

**Must have (table stakes):**
- Clerk SignIn/SignUp embedded + Google/GitHub OAuth — solo devs expect social login; signup abandonment increases without SSO
- Protected dashboard routes via `clerkMiddleware()` — `/dashboard/*` and `/verify-domain/*` redirect unauthenticated users
- Domain ownership verification (meta tag + well-known file methods) — cannot allow authenticated scans against unowned domains
- Scan quota display ("3 of 5 scans used this month") — no visibility creates confusion and frustration at the limit
- Results gating for anonymous scans (server-enforced) — the core conversion mechanic; never send description/remediation for high/critical on anonymous tokens
- Scan history list on dashboard — without persistence, the Developer tier offers no advantage over anonymous
- Data retention enforcement (anonymous: 24h, developer: 30 days) with expiry visible in UI
- Rate limit feedback on 429 ("Resets March 1") — silent 429s are a conversion killer
- Remove $49 Stripe audit — removes async-stripe, paid_audits write path, checkout flow

**Should have (differentiators):**
- FOMO teaser cards for high/critical findings — locked overlay showing finding category but not details; urgency is real because findings are real
- Inline upgrade prompts at 80% quota (4/5 scans used) — 31.4% higher conversion vs at-limit warning only
- Post-scan signup modal when anonymous scan finds high/critical — preserve results page context via Clerk SignUp in modal mode
- Domain verification status badge (verified/pending/failed) — communicates trust and security posture
- One-click re-scan from history — deducts quota, pre-fills URL
- Onboarding checklist for new signups (3 steps: verify, scan, fix) — converts empty dashboard into guided journey

**Defer (v2+):**
- Scan comparison / delta view — requires stable findings schema across scans; significant schema complexity
- Pro tier (Stripe subscription, unlimited scans, PDF/CSV export, API access) — build after Developer conversion proves out
- Scheduled/automated re-scans — Pro tier feature
- Scan share links for verified sites — existing capability URL already shareable; low priority uplift
- Email drip campaign after anonymous scan — requires Resend sequence integration; separate feature

### Architecture Approach

The milestone adds four orthogonal concerns to the existing system — identity, domain ownership, scan tiering, and results gating — none of which require rewriting existing components. Each integrates at a well-defined seam. The existing `tier` column on `scans`, `results_token` capability URL pattern, `reqwest` and `scraper` crates, and `CancellationToken` shutdown infrastructure in `main.rs` are all directly reusable. The central integration point is auth token flow: the Next.js frontend gets a short-lived Clerk JWT via `getToken()`, passes it as `Authorization: Bearer` to Axum, and Axum verifies against cached JWKS public keys — zero Clerk API calls in the hot path.

**Major components (all new):**
1. `src/api/auth.rs` — `ClerkUser` Axum extractor (`FromRequestParts`); JWKS cache via `axum-jwt-auth`; `Option<ClerkUser>` for endpoints serving both anonymous and authenticated users
2. `src/api/domain_verification.rs` — POST verify-start, POST verify-confirm, GET domains, DELETE domain; uses existing `reqwest` + `scraper`; requires non-optional `ClerkUser`
3. `src/cleanup.rs` — Tokio interval loop integrated into existing `task_tracker` in `main.rs`; hourly DELETE WHERE expires_at < NOW() AND status IN ('completed', 'failed')
4. `frontend/components/auth-gate.tsx` — Client component; reads `useAuth()` + `ownerVerified` field from backend; renders visible findings and gated-findings banner
5. `frontend/app/dashboard/` — Protected route group; scan history list; domain verification wizard at `/verify-domain/`

**New DB tables:** `users` (clerk_user_id FK, no duplication of Clerk data) and `verified_domains` (user_id, domain, verification_token, method, verified_at nullable).

**Modified:** `scans` gains `clerk_user_id VARCHAR(64)` nullable column and composite index; `tier` constraint extended to include `'authenticated'`; `paid_audits` FK changed from `ON DELETE CASCADE` to `ON DELETE SET NULL` before any cleanup runs.

**Build order (dependency-driven):** DB migrations (step 1) and Next.js Clerk install (step 5) can proceed in parallel. Axum JWT extractor (step 2) → users upsert (step 3) → rate limiting (step 4) → domain verification (step 8) → tier selection in create_scan (step 9) → results gating (steps 10-12) → scan history (steps 13-14) → domain verification UI (step 15) → retention cleanup (step 16).

### Critical Pitfalls

1. **Results gating is frontend-only** — The API currently returns all findings for all tokens. Any user can bypass React-layer gating with `curl`. Fix: server-side filtering in `results.rs` — strip `description` and `remediation` from high/critical findings for anonymous scan tokens before serializing. Return `gated: true` per finding. Never rely on frontend conditional rendering as the sole access control.

2. **Axum missing CORS Authorization header** — Current `main.rs` CORS config only allows `CONTENT_TYPE`. Adding `Authorization: Bearer` to requests triggers preflight failures before JWT verification can run. Fix: add `axum::http::header::AUTHORIZATION` to `allow_headers` as the very first auth change.

3. **Next.js middleware bypass (CVE-2025-29927, CVSS 9.1)** — Attackers can bypass `clerkMiddleware()` via `x-middleware-subrequest` header. ShipSecure is self-hosted; not protected by Vercel/Netlify edge. Fix: pin Next.js >= 15.2.3 (already fixed), add `proxy_set_header x-middleware-subrequest "";` to Nginx. Defense in depth: Axum verifies JWTs independently.

4. **Stripe removal cascade-deletes payment history** — `paid_audits` has `ON DELETE CASCADE` to `scans`. When the retention cleanup job runs, it will cascade-delete historical payment records. Fix: change FK to `ON DELETE SET NULL` before enabling any cleanup. Never DROP `paid_audits` or `stripe_events` tables.

5. **Domain verification TOCTOU** — A user verified at T=0 can scan a domain at T+6 months after losing ownership. Fix: store `verified_at` timestamp; at scan submission check `WHERE verified_at > NOW() - INTERVAL '30 days'`. Re-verify on demand.

---

## Implications for Roadmap

The dependency graph from ARCHITECTURE.md drives phase ordering. DB migrations and the Clerk JWT extractor are critical-path blockers for everything else. Frontend Clerk setup and backend DB/JWT work are independent and can proceed in parallel.

### Phase 1: Auth Foundation

**Rationale:** Everything else depends on this. JWT extraction, CORS fix, Clerk middleware configuration, and the `users` table must exist before any authenticated endpoint can be built. The CVE-2025-29927 Nginx fix must land before dashboard routes exist, not after.
**Delivers:** Clerk installed in Next.js (`proxy.ts`, `ClerkProvider`, `/sign-in`, `/sign-up`, `<UserButton>`); Axum `ClerkUser` extractor with JWKS caching; CORS `Authorization` header fix; `users` table migration; Clerk webhook handler (`user.created` → DB upsert with svix signature verification); Nginx `x-middleware-subrequest` header stripped.
**Addresses:** Google/GitHub OAuth, persistent sessions, protected dashboard redirect skeleton, UserButton in header.
**Avoids:** Pitfall 2 (CORS), Pitfall 4 (anonymous scan flow breakage — run existing E2E tests immediately after), Pitfall 9 (CVE-2025-29927), Pitfall 12 (ClerkProvider not at root layout).

### Phase 2: Stripe Removal and Schema Cleanup

**Rationale:** Stripe removal is a prerequisite for clean schema work. The `paid_audits` FK constraint must be changed to `SET NULL` before retention cleanup runs, and `async-stripe` must be removed from Cargo.toml before adding new dependencies creates confusion. Tables stay; only the write path and constraint change.
**Delivers:** `async-stripe`, `hmac`, `sha2`, `genpdf` removed from Cargo.toml; Stripe checkout routes removed; `paid_audits` FK changed to `ON DELETE SET NULL`; `stripe_events` archived; Stripe-era frontend components removed; `tier` constraint on `scans` extended to include `'authenticated'`; `scans.clerk_user_id` nullable column + composite index added.
**Addresses:** Remove $49 Stripe audit; clean tier enum for new tiers.
**Avoids:** Pitfall 5 (cascade delete of payment history), Pitfall 13 (email mismatch in scan flow after auth).

### Phase 3: Results Gating (Server-Side)

**Rationale:** The product's core conversion mechanic must be built server-side first. Building the frontend teaser card before the server strips finding details creates false security. The API contract change must land before the frontend AuthGate component is built.
**Delivers:** `GET /api/v1/results/:token` accepts `Option<ClerkUser>`; anonymous scan tokens return high/critical findings with `gated: true` and null `description`/`remediation`; `owner_verified` field computed and returned; `AuthGate` client component renders visible findings + gated-findings banner with Clerk SignUp modal CTA; post-scan signup modal.
**Addresses:** FOMO teaser cards, post-scan signup modal, results gating for anonymous scans.
**Avoids:** Pitfall 1 (frontend-only gating bypass).
**Uses:** `jsonwebtoken` + `axum-jwt-auth` from Phase 1; `useAuth()` + `owner_verified` in frontend.

### Phase 4: Domain Verification

**Rationale:** Depends on Phase 1 (ClerkUser extractor) and Phase 2 (scans.clerk_user_id). SSRF and shared-hosting validation rules must be designed before the UI is built.
**Delivers:** `verified_domains` table migration; POST `/api/v1/domains/verify-start` and `/verify-confirm` endpoints; meta tag and well-known file verification using existing `reqwest` + `scraper`; domain normalization (strip scheme/www, lowercase); SSRF validation on all verification fetch URLs; blocklist for shared hosting TLDs (github.io, vercel.app, netlify.app, pages.dev); `/verify-domain/` frontend wizard; verified domain badge in dashboard.
**Addresses:** Domain ownership verification, domain verification status badge.
**Avoids:** Pitfall 3 (TOCTOU — store verified_at, check 30-day TTL at scan time), Pitfall 6 (SSRF via verification URL), Pitfall 7 (shared hosting TLD spoofing), Pitfall 14 (unverified domain scan race — domain check synchronous in create_scan handler).

### Phase 5: Tiered Scan Access and Rate Limiting

**Rationale:** With JWT extraction, domain verification, and schema changes in place, the orchestrator tier extension and per-user rate limiting can be wired together cleanly.
**Delivers:** `spawn_authenticated_scan` in orchestrator; 3-arm tier match (free/authenticated/paid — 30 JS files, 300s timeout, 30-day expiry); `check_rate_limits` extended with `Option<clerk_user_id>`; per-user monthly quota tracked in DB; 429 response with `resets_at` timestamp; quota display in header ("3 of 5 scans used"); 80% quota warning banner.
**Addresses:** Per-user rate limiting, scan quota display, rate limit feedback, one-click re-scan.
**Avoids:** Pitfall 10 (tier transition race — store effective tier on scan record at creation time).

### Phase 6: Scan History Dashboard

**Rationale:** Depends on Phase 5 (scans linked to clerk_user_id with quota enforced) and Phase 1 (dashboard route protection).
**Delivers:** `GET /api/v1/users/me/scans` paginated endpoint (non-optional ClerkUser); `frontend/app/dashboard/` protected route; scan history list (domain, date, severity counts, expiry countdown, re-scan button); empty state with verification CTA; onboarding checklist for new signups (verify → scan → fix).
**Addresses:** Scan history list, one-click re-scan, onboarding checklist.

### Phase 7: Data Retention

**Rationale:** Final phase — must run after Phase 2 (FK constraint changed to SET NULL), Phase 5 (tier-based expires_at set correctly), and Phase 6 (scan history displays expiry dates). Cleanup query must exclude in-progress scans.
**Delivers:** `src/cleanup.rs` Tokio interval task integrated into `main.rs` task_tracker; hourly DELETE WHERE expires_at < NOW() AND status IN ('completed', 'failed'); tracing logs with deleted row count; retention policy: anonymous 24h, authenticated 30 days, paid NULL.
**Addresses:** Data retention enforcement, "expires in X days" in scan history.
**Avoids:** Pitfall 11 (in-progress scan deletion — status filter), Pitfall 5 (paid_audits cascade — FK already fixed in Phase 2).

### Phase Ordering Rationale

- **Phases 1 and 2 are parallel-capable across frontend/backend teams:** Next.js Clerk setup (Phase 1 frontend) and Stripe removal (Phase 2 backend) have no mutual dependency.
- **Phase 3 before Phase 4:** Results gating is the monetization foundation. Shipping domain verification before the gating API contract is locked risks interim frontend-only gating.
- **Phase 5 before Phase 6:** Scan history is useless without scans linked to users and quota enforced.
- **Phase 7 last:** Retention cleanup is a destructive operation; all FK constraints and tier-based expiry logic must be correct first.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 4 (Domain Verification):** The shared hosting TLD blocklist is a policy decision needing input — how broadly to block. The 30-day re-verification TTL needs validation against user experience tradeoffs (users who remove and re-add meta tags).
- **Phase 5 (Rate Limiting):** FEATURES.md specifies "5 scans/month" for Developer tier; ARCHITECTURE.md shows "10/day" for authenticated users. These must be reconciled before implementation. Recommendation: implement monthly window (5 scans/month) consistent with product-facing quota display.

Phases with standard patterns (skip research-phase):
- **Phase 1 (Auth Foundation):** Clerk Next.js integration is extensively documented; CORS fix is a one-line change; JWKS caching via axum-jwt-auth is a well-defined pattern.
- **Phase 2 (Stripe Removal):** Dependency removal and FK constraint changes are mechanical.
- **Phase 3 (Results Gating):** Server-side serialization filtering is straightforward Rust serde work; pattern is well-established.
- **Phase 6 (Scan History):** Standard paginated list endpoint + protected Next.js route.
- **Phase 7 (Data Retention):** Simple Tokio interval task against existing schema.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Clerk docs verified against Feb 2026 releases; axum-jwt-auth confirmed on crates.io; svix v1.83.0 released Dec 2025; Next.js 16 proxy.ts rename confirmed in official Next.js docs; clerk-rs explicitly researched and rejected |
| Features | HIGH | Clerk docs are official; domain verification pattern is Google Search Console-established; conversion stats for gating overlays (MEDIUM) are secondary but consistent across multiple sources |
| Architecture | HIGH | Existing codebase read directly (highest possible confidence); integration seams clear from code inspection; Clerk patterns MEDIUM via official docs but well-specified |
| Pitfalls | HIGH | CVE-2025-29927 is a documented CVE (CVSS 9.1); OWASP Broken Access Control A01 is authoritative; ON DELETE CASCADE risk confirmed from direct schema inspection; JWKS rotation pitfall documented by production postmortem |

**Overall confidence:** HIGH

### Gaps to Address

- **Rate limit window inconsistency:** FEATURES.md specifies "5 scans/month" for Developer tier; ARCHITECTURE.md's rate limiter section shows "10/day" for authenticated users. These must be reconciled before Phase 5. Recommendation: monthly window (5/month) for user-facing clarity.

- **`axum-jwt-auth` version:** STACK.md lists `axum-jwt-auth = "0.4"` but notes to verify current version on crates.io. Confirm at Phase 1 implementation time whether it supports async JWKS refresh in the current release.

- **Pro tier in `tier` constraint:** The constraint change in Phase 2 (`'free' | 'authenticated' | 'paid'`) could add `'pro'` now to avoid a future migration. Confirm with roadmapper whether to include it.

- **`genpdf` removal:** STACK.md flags `genpdf` as "evaluate removal — may keep if needed elsewhere." A code search is needed before Phase 2 removes it.

- **Existing E2E test suite:** PITFALLS.md repeatedly references `free-scan.spec.ts` as the regression guard. Confirm this test exists and is green before Phase 1 begins. If broken, fix it first.

---

## Sources

### Primary (HIGH confidence)
- ShipSecure codebase (direct read): `src/orchestrator/worker_pool.rs`, `src/api/scans.rs`, `src/api/results.rs`, `src/main.rs`, `src/rate_limit/middleware.rs`, `migrations/20260206000001_add_paid_audits.sql`, `frontend/app/actions/scan.ts`, `frontend/app/results/[token]/page.tsx`, `frontend/lib/api.ts`, `Cargo.toml`, `package.json`
- Clerk Next.js v6 docs (updated Feb 11, 2026): https://clerk.com/docs/nextjs/getting-started/quickstart
- Clerk v6 Next.js 16 support (6.37.1, Jan 30 2026): https://clerk.com/changelog/2024-10-22-clerk-nextjs-v6
- Next.js 16 middleware → proxy.ts rename: https://nextjs.org/docs/messages/middleware-to-proxy
- clerkMiddleware() reference: https://clerk.com/docs/reference/nextjs/clerk-middleware
- Clerk manual JWT verification: https://clerk.com/docs/guides/sessions/manual-jwt-verification
- Clerk publicMetadata RBAC pattern: https://clerk.com/docs/guides/secure/basic-rbac
- CVE-2025-29927 Next.js middleware bypass (CVSS 9.1): https://projectdiscovery.io/blog/nextjs-middleware-authorization-bypass
- Clerk response to CVE-2025-29927: https://clerk.com/blog/cve-2025-29927
- svix crate (v1.83.0, Dec 2025): https://docs.rs/crate/svix/latest
- tokio-cron-scheduler: https://crates.io/crates/tokio-cron-scheduler
- axum-jwt-auth crate: https://crates.io/crates/axum-jwt-auth
- jsonwebtoken crate: https://crates.io/crates/jsonwebtoken
- OWASP Broken Access Control A01:2025: https://owasp.org/Top10/A01_2021-Broken_Access_Control/

### Secondary (MEDIUM confidence)
- Clerk webhooks sync guide: https://clerk.com/docs/guides/development/webhooks/syncing
- Domain verification pattern (Google Search Console): https://support.google.com/webmasters/answer/9008080
- WorkOS developer guide to domain verification: https://workos.com/blog/the-developers-guide-to-domain-verification
- Freemium conversion rate benchmarks 2026: https://firstpagesage.com/seo-blog/saas-freemium-conversion-rates/
- Feature gating strategies (SaaS): https://www.withorb.com/blog/feature-gating
- Gated content conversion statistics 2025: https://www.amraandelma.com/gated-content-conversion-statistics/
- JWKS caching postmortem (Logto, Jan 2026): https://blog.logto.io/postmortem-jwks-cache
- Axum middleware layer execution order: https://docs.rs/axum/latest/axum/middleware/index.html
- tower-http CORS allow_headers Authorization: https://github.com/tower-rs/tower-http/issues/194
- Rate limiting in PostgreSQL (Neon): https://neon.com/guides/rate-limiting
- Receive webhooks with Rust/Axum (Svix guide): https://www.svix.com/guides/receiving/receive-webhooks-with-rust-axum/

### Tertiary (LOW confidence)
- clerk-rs community crate (explicitly rejected, documented as anti-pattern): https://crates.io/crates/clerk-rs
- Integrating Clerk with Next.js + Express (community pattern reference): https://mtarkar.medium.com/integrating-next-js-clerk-auth-with-express-9c7f0407c6f0

---
*Research completed: 2026-02-17*
*Ready for roadmap: yes*
