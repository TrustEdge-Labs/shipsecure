# Pitfalls Research: Auth + Domain Verification + Tiered Access

**Domain:** Adding Clerk auth, domain verification, and tiered access to existing Rust/Axum + Next.js SaaS
**Researched:** 2026-02-17
**Confidence:** HIGH (codebase-specific) / HIGH (Clerk/Axum integration) / MEDIUM (domain verification patterns)

---

## Critical Pitfalls

Mistakes that cause rewrites, security holes, or complete feature failure.

---

### Pitfall 1: Results Gating Is Frontend-Only — Backend Returns All Findings

**What goes wrong:** The most dangerous hole in the tiered access design. The current `GET /api/v1/results/{token}` handler returns ALL findings regardless of tier. If results gating (hiding high/critical from anonymous users) is implemented only in the React component, any user can bypass it by hitting the API directly with `curl` or browser DevTools.

**Why it happens:** Frontend gating feels natural — "just don't render the high/critical findings." But the API contract doesn't change. The JSON response still includes every finding with full `description`, `title`, `remediation`. A single `fetch('/api/v1/results/TOKEN').then(r=>r.json())` reveals everything.

**Consequences:** The entire "teaser" conversion strategy is a security theater. Users who understand HTTP can see all critical findings without signing up. The product's core value proposition (gating as conversion driver) is undermined and the company's reputation suffers if discovered.

**Current code exposure:**
```rust
// src/api/results.rs — ALL findings returned, no tier check
let findings_json: Vec<serde_json::Value> = findings
    .iter()
    .map(|f| json!({
        "id": f.id,
        "title": f.title,
        "description": f.description,  // <-- sent for ALL severities
        "severity": ...,
        "remediation": f.remediation,  // <-- sent for ALL severities
    }))
    .collect();
```

**Prevention:**
1. Server-side tier check in `get_results_by_token`: inspect the JWT from `Authorization: Bearer <token>` header (optional — anonymous requests have no header)
2. For anonymous scans: strip `description` and `remediation` from high/critical findings before serializing
3. Return a `gated: true` flag per finding so the frontend knows what to render
4. Never rely on frontend conditional rendering as the only access control
5. Rule: if the data must not be seen, it must not be in the response

**Phase assignment:** Phase 1 (auth foundation) — must gate at API layer before any frontend gating is built

**Detection:** `curl https://shipsecure.ai/api/v1/results/TOKEN | jq '.findings[] | select(.severity == "critical")'` — if this returns full finding details for an anonymous scan token, the gate is missing

---

### Pitfall 2: Clerk JWT Verification Missing on Axum — Auth Header Not Forwarded

**What goes wrong:** Clerk handles auth on the Next.js side. The Axum backend receives requests from the Next.js frontend (server actions, route handlers). If Axum doesn't verify the Clerk JWT on protected endpoints, any unauthenticated caller who knows the API URL can call backend endpoints directly. The frontend auth check becomes the only gate — and it's bypassable.

**Why it happens:** Developers add Clerk to Next.js (easy, first-class support), see the UI gating work, and assume the backend is protected. But Axum doesn't know about Clerk at all. The `Authorization: Bearer <clerk_jwt>` header must be explicitly passed in every server-to-backend request AND verified by Axum middleware.

**Specific Axum issues:**

1. **CORS missing `AUTHORIZATION` header:** Current `main.rs` CORS config only allows `CONTENT_TYPE`. When the frontend adds `Authorization: Bearer <token>`, the preflight `OPTIONS` request will be rejected with a CORS error before it reaches Axum.
   ```rust
   // CURRENT (broken for auth):
   .allow_headers([axum::http::header::CONTENT_TYPE])

   // MUST BECOME:
   .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION])
   ```

2. **Middleware ordering traps:** In Axum, `layer()` wraps routes bottom-to-top. If JWT verification middleware is added after CORS, CORS preflight failures will block auth from ever running. CORS must be outermost (applied last via `.layer()`), JWT verification runs inner.

3. **JWKS caching not implemented:** Clerk uses RS256. The backend must fetch Clerk's JWKS endpoint to get the public key. Fetching on every request is too slow (100-300ms per call) and will get rate-limited. Use `axum-jwks` or `axum-jwt-auth` crate with automatic JWKS caching and refresh. Cache TTL should be ~5 minutes with stale-while-revalidate.

4. **Wrong claims validated:** Must check `exp` (expiration), `nbf` (not before), `iss` (must be `https://clerk.{your-domain}` or `https://{clerk-instance}.clerk.accounts.dev`), and `azp` (authorized parties — should be your frontend origin). Accepting tokens with wrong `iss` allows token reuse from other Clerk instances.

**Consequences:** Backend endpoints callable without authentication. Domain verification bypass. Tier enforcement bypass. Scan history readable by anyone.

**Prevention:**
1. Add `AUTHORIZATION` to CORS `allow_headers` before any auth work
2. Create Axum `FromRequestParts` extractor for `ClerkClaims` that parses and validates Bearer token
3. Use `axum-jwks` crate with `CLERK_JWKS_URL` env var (networkless alternative: `CLERK_JWT_KEY` PEM key)
4. Make JWT verification optional per-route: anonymous endpoints accept no token, protected endpoints require valid token, gated endpoints adjust response based on token presence
5. Test directly: `curl -H "Authorization: Bearer INVALID" https://api.shipsecure.ai/protected-endpoint` — must return 401

**Phase assignment:** Phase 1 (auth foundation) — JWKS caching and CORS fix must land before any protected endpoint

---

### Pitfall 3: Domain Verification TOCTOU — Ownership Check and Scan Happen at Different Times

**What goes wrong:** A user verifies ownership of `victim.com` at T=0. At T=30 days, the user loses control of `victim.com` (domain expires, transferred, or they leave the company). At T=31 days, they can still trigger authenticated scans against `victim.com` because verification status is stored but not rechecked.

**Why it happens:** Domain verification is treated as a one-time event: "verified = true, never recheck." But domain ownership is dynamic. The time between check (verification) and use (authenticated scanning) can span months.

**Consequences:** Users scan domains they no longer own. A malicious user who verified `victim.com` as an employee, then left, can scan their former employer's site with full authenticated access.

**Second attack:** Race condition during verification. If ShipSecure fetches the meta tag or file from `target.com` to verify, and then immediately scans `target.com` — an attacker can briefly place the verification token on a CDN-fronted site, pass verification, remove the token, then redirect the domain to a different target before the scan runs.

**Prevention:**
1. Re-verify domain ownership on each authenticated scan (lightweight HEAD request to check token still present) — not just at initial verification time
2. Set verification TTL (e.g., 30 days) — verified status must be renewed, not permanent
3. Separate verification check from scan dispatch: verify, then queue scan, then confirm domain still verified before scan executes
4. Store `domain_verified_at` timestamp alongside `domain_verified` boolean — query: `WHERE domain_verified = true AND domain_verified_at > NOW() - INTERVAL '30 days'`
5. For file upload method: check the well-known URL at scan time, not just at verification time

**Phase assignment:** Phase 2 (domain verification) — core security design of the verification system

---

### Pitfall 4: Breaking the Existing Anonymous Scan Flow

**What goes wrong:** When Clerk middleware is added to Next.js, the route protection configuration must explicitly enumerate public routes. Developers often protect "everything" and then add exceptions. The existing scan flow (`/`, `/results/[token]`, `/scan/[id]`, `/payment/success`) must stay fully public with no authentication requirement. A misconfigured `createRouteMatcher` causes the homepage (and thus all anonymous scanning) to redirect to Clerk's sign-in page.

**Why it happens:** Clerk's `clerkMiddleware()` does NOT protect routes by default (changed from old `authMiddleware()`). However, when developers add `auth().protect()` inside the middleware for protected routes, a common mistake is `isPublicRoute` logic inversion — protecting everything when you meant to protect only dashboard routes.

**Second break:** The `POST /api/v1/scans` Axum endpoint currently takes `email` as a required field. After adding auth, the temptation is to remove the email requirement (Clerk provides the email). But existing anonymous scans (which continue to work) still need email. If the `email` field becomes optional and the anonymous flow doesn't send it, the rate limiter breaks (email-based rate limit returns 0, everyone gets unlimited scans).

**Third break:** The Axum backend's existing rate limiter uses `email` + IP. With authenticated users, the rate limit should use `user_id` (from JWT sub claim) instead of email. Without this change, authenticated users share rate limit buckets with anonymous users who happen to use the same email address.

**Current code vulnerability:**
```rust
// src/rate_limit/middleware.rs — email used for rate limiting
let email_count = scans::count_scans_by_email_today(pool, email).await?;
if email_count >= 3 { return Err(ApiError::RateLimited(...)); }
```
After auth: authenticated users should be rate-limited by `user_id`, not email. Email-based limiting is bypassable by signing up with a different email.

**Prevention:**
1. Keep the existing scan flow working throughout the entire v1.6 build — run E2E test `free-scan.spec.ts` after every phase
2. In Clerk middleware: use `createRouteMatcher` with explicit allowlist of public routes, never a denylist
3. Public routes minimum set: `["/", "/results/(.*)", "/scan/(.*)", "/payment/(.*)", "/privacy", "/terms", "/api/v1/scans", "/api/v1/results/(.*)", "/api/v1/stats/(.*)", "/api/v1/checkout", "/api/v1/webhooks/(.*)"]`
4. Add `user_id` column to `scans` table as nullable — populate from JWT `sub` claim when token present, null for anonymous
5. Rate limit function signature becomes `check_rate_limits(pool, user_id: Option<&str>, email: &str, ip: &str)` — use user_id if Some, else email

**Phase assignment:** Phase 1 (auth foundation) — middleware config is the very first thing that can break everything

---

### Pitfall 5: Stripe Removal Destroys Historical Paid Audit Records

**What goes wrong:** Removing the `async-stripe` dependency and dropping the Stripe-related code also removes the only handling path for the `paid_audits` table. If the database migration for Stripe removal uses `DROP TABLE paid_audits` or `DROP TABLE stripe_events`, all historical paid audit records are destroyed. This includes completed transactions that users paid $49 for.

**Why it happens:** "We're removing Stripe, so we remove Stripe tables" — logical but wrong. The `paid_audits` table has business records with `stripe_checkout_session_id`, `stripe_payment_intent_id`, `amount_cents`, and `customer_email`. These are evidence of completed transactions.

**Second risk:** The `paid_audits` table has `ON DELETE CASCADE` from `scans`. If a data retention job deletes old scans (24hr for anonymous), it will cascade-delete associated `paid_audits` records for those scans. This means completed payments are silently erased.

**Current schema risk:**
```sql
-- migrations/20260206000001_add_paid_audits.sql
CREATE TABLE paid_audits (
    ...
    scan_id UUID NOT NULL REFERENCES scans(id) ON DELETE CASCADE,  -- <-- DANGER
    ...
);
```

**Third risk:** Removing the `tier` column logic while Stripe code is deleted but paid audits exist — scans that were tier='paid' may have their tier reset or become inconsistent.

**Prevention:**
1. Never DROP the `paid_audits` or `stripe_events` tables — archive them
2. Change `ON DELETE CASCADE` to `ON DELETE SET NULL` before running any cleanup jobs
3. Migration plan for Stripe removal: (a) stop writing new records, (b) change foreign key constraint to `SET NULL`, (c) remove Stripe dependency from Cargo.toml, (d) keep tables intact permanently
4. Add a `removed_at` timestamp column to `paid_audits` rather than deleting rows
5. Keep historical `tier = 'paid'` values on scan records — they remain accurate historical markers
6. If Stripe tables must eventually be cleaned: export CSV of all records with status='completed' before any deletion

**Phase assignment:** Phase 3 (Stripe removal) — must audit foreign keys and constraint types before touching Stripe code

---

## Moderate Pitfalls

---

### Pitfall 6: File Upload Verification Path Traversal

**What goes wrong:** Domain verification via file upload asks users to place a file at `/.well-known/shipsecure-verify-{token}.txt`. The backend fetches `https://{domain}/.well-known/{filename}` to verify. If the `filename` or `domain` is not validated before constructing the URL, an attacker can craft a domain like `victim.com/../../../internal-ip` or use a file token containing path traversal to fetch internal resources via SSRF.

**Why it happens:** The domain verification fetch reuses the same HTTP client as the scanner. If SSRF validation (`ssrf::validate_scan_target`) is only called for scan URLs and not for verification URLs, the verification fetcher is an open SSRF vector.

**Consequences:** Internal network access, metadata service exposure, or content of internal files returned to attacker via the verification response.

**Prevention:**
1. Run `ssrf::validate_scan_target()` on the domain before constructing any verification fetch URL
2. The file path must be hardcoded — never constructed from user input: `format!("https://{}/.well-known/shipsecure-verify-{}.txt", domain, token)` where `token` is UUID generated by the server
3. Validate the domain is a public FQDN (not IP, not localhost, not internal RFC-1918) before any fetch
4. Limit the response read to first 1KB — do not load full file content
5. Only accept exact match of the expected token string — no partial matches, no regex

**Phase assignment:** Phase 2 (domain verification) — treat as a second SSRF attack surface

---

### Pitfall 7: Meta Tag Verification Spoofed via Shared Hosting / CDN Injection

**What goes wrong:** Domain ownership via meta tag (`<meta name="shipsecure-verify" content="{token}">`) is insufficient proof for shared hosting platforms (GitHub Pages, Netlify, Vercel) where multiple users can serve content from subdomains or paths. An attacker on `shared-host.com` can place the meta tag in their own hosted site, then claim verification of `shared-host.com` — which they "own" in a strict sense but don't control entirely.

**More critical attack:** A user who has a free GitHub Pages site at `their-project.github.io` verifies ownership. But `github.io` is a shared domain — another attacker who also has a `github.io` page should not be able to scan github.io targets broadly.

**Prevention:**
1. Require verification at the root domain, not a subdomain — `shipsecure.ai` not `app.shipsecure.ai`
2. Enforce that scan targets must match the exact verified domain or explicit subdomains of it
3. Block verification of known shared hosting TLDs: `github.io`, `vercel.app`, `netlify.app`, `pages.dev` — these are not "owned" domains
4. Consider adding a domain denylist for platforms that serve user content at subdomains
5. The verification requirement (scan target must match verified domain) is the real security boundary — enforce this at scan dispatch time

**Phase assignment:** Phase 2 (domain verification) — domain validation rules must be designed before verification UI

---

### Pitfall 8: JWKS Key Rotation Causes Auth Outage

**What goes wrong:** Clerk rotates its signing keys periodically. If the Axum backend caches the JWKS response indefinitely (or with very long TTL), it will continue to validate tokens against the old public key after rotation. All users get 401 errors until the cache expires.

**Inverse problem:** If TTL is too short (e.g., 10 seconds) and Clerk has a brief JWKS endpoint outage, every request triggers a live fetch, causing cascading failures: fetch timeouts on every request, scan API becomes unavailable.

**Prevention:**
1. Use `axum-jwks` or `axum-jwt-auth` — both implement automatic JWKS caching with stale-while-revalidate semantics
2. Set cache TTL to 5 minutes with stale fallback: serve cached keys for up to 10 minutes even if refresh fails
3. On JWT decode failure (unknown kid): immediately try to refresh JWKS once before returning 401 — this handles mid-request key rotation gracefully
4. Alert on JWKS fetch failures (log `tracing::warn!` on every failed refresh) — surface in Prometheus metrics

**Phase assignment:** Phase 1 (auth foundation) — choose JWKS caching approach before writing any verification code

---

### Pitfall 9: Next.js CVE-2025-29927 — Middleware Auth Bypass

**What goes wrong:** A critical vulnerability (CVSS 9.1) in Next.js allows attackers to bypass `clerkMiddleware()` by sending `x-middleware-subrequest` header with a crafted value. Any route "protected" by Next.js middleware alone becomes accessible without authentication. This is particularly dangerous for the ShipSecure scan history dashboard and user profile routes.

**Why it matters specifically here:** ShipSecure is self-hosted (DigitalOcean/Nginx). Vercel and Netlify deployments are automatically protected by their edge network, but self-hosted instances are not. The Nginx reverse proxy must strip this header.

**Consequences:** Authenticated dashboard routes exposed. Scan history readable by unauthenticated users. But — this only affects routes where middleware is the sole protection. If Axum also verifies the JWT (Pitfall 2 fix), the backend remains protected even if the Next.js middleware is bypassed.

**Prevention:**
1. Pin Next.js version to >=15.2.3 (already fixed)
2. Add Nginx header stripping: `proxy_set_header x-middleware-subrequest "";` in the Next.js proxy block
3. Never rely solely on Next.js middleware for data access control — Axum must enforce JWT verification on all authenticated endpoints (defense in depth)
4. Run `curl -H "x-middleware-subrequest: pages-edge-server" https://shipsecure.ai/dashboard` — must redirect to Clerk login, not serve dashboard

**Phase assignment:** Phase 1 (auth foundation) — Nginx config update needed before any dashboard routes exist

---

### Pitfall 10: Rate Limit Race Condition at Tier Transition

**What goes wrong:** When an anonymous user signs up mid-session, their in-flight scan request may have already passed the anonymous rate limit check but the rate limit tier transitions immediately. Edge cases:

1. Anonymous user has 1/1 scans used. Signs up. The "upgrade" should reset their limit to the Developer tier. But if the backend checks the DB mid-transition (between user creation and tier assignment propagating), they may get a 429 on their first authenticated request.

2. Conversely: an authenticated user's session token expires mid-scan. The scan was dispatched against tier=developer quotas. The rate limit check at scan creation used authenticated tier. After expiry, any follow-up requests (poll for scan status) fail auth and fall into anonymous tier — but the scan is running with developer-tier scan depth.

**Prevention:**
1. Rate limit check uses the tier at time of scan creation, stored on the scan record — not the current user state
2. Add `tier` to scan creation request handling: record the effective tier when the scan is queued, don't re-evaluate mid-scan
3. Tier transitions (anonymous → developer) reset quotas from the moment of account creation, not mid-day — implement as a timestamp: `quota_reset_at = account_created_at`
4. Scan polling endpoints (`GET /api/v1/scans/{id}`) should be unauthenticated — the scan ID is the capability. Do not gate status polling behind auth.

**Phase assignment:** Phase 4 (tiered access enforcement) — implement alongside the scan creation changes

---

### Pitfall 11: Scan Data Retention Delete Races With Active Scans

**What goes wrong:** A background job deletes anonymous scans older than 24 hours. If a scan was submitted at 23:59 and the cleanup job runs at 00:01, the scan may be deleted while it's still `status = 'in_progress'`. The orchestrator holds the scan ID in memory (in `ScanOrchestrator`), the DB record is gone, and subsequent DB writes from the scan (updating stage flags, setting results_token) silently fail.

**Why it happens:** The cleanup job queries `WHERE created_at < NOW() - INTERVAL '24 hours'` without checking `status`. In-progress scans can be up to 5-10 minutes old for complex scans.

**Current orchestrator risk:**
```rust
// src/orchestrator — scan ID held in memory after DB record could be deleted
state.orchestrator.spawn_scan(scan.id, scan.target_url.clone(), ...);
// If retention job deletes scan record during execution:
// db::scans::update_scan_stage(...) -> row not found, silently ignored
```

**Prevention:**
1. Cleanup query must exclude in-progress scans: `WHERE created_at < NOW() - INTERVAL '24 hours' AND status IN ('completed', 'failed')`
2. Add a `locked_until` or `protection_expires_at` column: set to `NOW() + INTERVAL '30 minutes'` when scan is dispatched
3. Soft delete pattern: set `deleted_at` timestamp, run cleanup as `WHERE deleted_at IS NOT NULL AND deleted_at < NOW() - INTERVAL '1 hour'` — gives time to detect and recover
4. Cleanup job should log the count of rows it deletes per run — alert if count suddenly spikes

**Phase assignment:** Phase 5 (data retention) — cleanup logic must be carefully staged

---

## Minor Pitfalls

---

### Pitfall 12: Clerk `useAuth()` / `auth()` Not Available in Non-Clerk Pages

**What goes wrong:** After adding Clerk, developers use `useAuth()` or `auth()` in components that are rendered for routes that don't go through `ClerkProvider`. In Next.js App Router, `ClerkProvider` must wrap the root `layout.tsx`. If it only wraps certain route groups, server components outside those groups calling `auth()` will throw.

**Prevention:** Wrap `ClerkProvider` in the root `app/layout.tsx`, not individual route group layouts. Use `auth()` only in server components inside protected routes.

**Phase assignment:** Phase 1 (auth foundation)

---

### Pitfall 13: Email Notification Mismatch After Auth

**What goes wrong:** The current system sends scan results to the submitted email. After adding Clerk auth, authenticated users have an email in Clerk's user record. If the submitted `email` in the scan request doesn't match the Clerk account email, results are emailed to the wrong address.

**Prevention:** For authenticated requests, ignore the form's `email` field and pull the user's primary email from the JWT claims or Clerk backend API. Keep the email field as user-override only for anonymous scans.

**Phase assignment:** Phase 3 (Stripe removal / scan flow refactor)

---

### Pitfall 14: Scanner Dispatch Runs Against Unverified Domain in Authenticated Flow

**What goes wrong:** The domain verification flow verifies that the user owns a domain. But if domain verification and scan dispatch are in separate API calls, a race allows: (1) verify domain A, (2) immediately request scan against domain B (before the verification status is checked for B). The authenticated scan quota is consumed against a domain the user hasn't verified.

**Prevention:** Domain verification check must be synchronous with scan dispatch — not a UI-level workflow. The Axum `create_scan` handler for authenticated users must query `domain_verifications WHERE user_id = $1 AND domain = $2 AND verified_at IS NOT NULL` before accepting the scan.

**Phase assignment:** Phase 2 (domain verification) — verified domain → scan dispatch linkage

---

### Pitfall 15: Clerk Webhook Signature Verification Missing

**What goes wrong:** When Clerk fires webhooks (user.created, user.deleted, session.ended), the Axum backend may handle these to sync user state. If the `svix-signature` header is not verified, any external actor can send fake user.deleted events and trigger data deletion.

**Prevention:** Verify Clerk webhook signatures using the `CLERK_WEBHOOK_SECRET` (Svix-based HMAC). The existing Stripe webhook verification pattern in `src/api/webhooks.rs` should be the template.

**Phase assignment:** Phase 1 or 2, whenever Clerk webhooks are implemented

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Auth foundation (Clerk + Axum JWT) | CORS missing Authorization header breaks all authenticated requests | Fix `.allow_headers()` before any JWT middleware |
| Auth foundation | Next.js middleware bypass (CVE-2025-29927) | Pin Next.js >=15.2.3, strip header in Nginx |
| Auth foundation | JWKS cache not implemented, rate-limited on Clerk's endpoint | Use axum-jwks or axum-jwt-auth with 5min TTL |
| Auth foundation | clerkMiddleware protecting existing public scan routes | Use explicit opt-in route protection, run E2E tests |
| Domain verification | TOCTOU — verified once, used forever | Re-verify at scan time, set 30-day TTL |
| Domain verification | Meta tag verification insufficient for shared platforms | Block shared hosting TLDs, require root domain |
| Domain verification | File upload path not SSRF-validated | Run ssrf::validate_scan_target() on verification URL |
| Tiered access / results gating | API returns all findings regardless of tier | Server-side filtering in results.rs before serializing |
| Tiered access / rate limiting | Rate limit uses email not user_id for authenticated users | Add user_id column, rate limit by user_id when present |
| Tiered access | Tier transition race at signup mid-session | Store effective tier on scan record at creation time |
| Stripe removal | paid_audits ON DELETE CASCADE destroys payment history | Change to SET NULL, never DROP table |
| Stripe removal | Historical tier='paid' scans become inconsistent | Keep column, document it as legacy |
| Data retention | Cleanup deletes in-progress scans | Filter by status IN ('completed', 'failed') only |
| Data retention | Cleanup cascades into paid_audits | Fix FK constraint before enabling cleanup job |

---

## Security Checklist for Each Phase

### Auth Phase
- [ ] `curl -H "Authorization: Bearer INVALID" /api/v1/scans` returns 401
- [ ] `curl /api/v1/scans` (no auth header) still creates anonymous scan successfully
- [ ] `curl https://shipsecure.ai/dashboard` (no Clerk session) redirects to login
- [ ] `curl -H "x-middleware-subrequest: pages-edge-server" https://shipsecure.ai/dashboard` redirects to login
- [ ] Existing E2E test `free-scan.spec.ts` passes after all auth changes

### Results Gating Phase
- [ ] `curl /api/v1/results/ANONYMOUS_TOKEN | jq '.findings[] | select(.severity == "critical") | .description'` returns null or empty string
- [ ] Same token + valid auth JWT in Authorization header returns full description
- [ ] Frontend shows teaser card for gated findings without leaking content from DOM

### Domain Verification Phase
- [ ] Verification fetch URL passes SSRF validation
- [ ] Attempting to verify `github.io`, `vercel.app` etc. returns validation error
- [ ] Second scan request after domain verification checks ownership again, not just DB flag
- [ ] File upload token is server-generated UUID, not user-supplied

### Data Retention Phase
- [ ] Cleanup query includes `AND status IN ('completed', 'failed')`
- [ ] paid_audits FK is `ON DELETE SET NULL` before cleanup job runs
- [ ] Cleanup job log line shows count of deleted rows per run

---

## Sources

- ShipSecure codebase inspection: `src/api/scans.rs`, `src/api/results.rs`, `src/main.rs`, `src/rate_limit/middleware.rs`, `migrations/20260206000001_add_paid_audits.sql`
- CVE-2025-29927 Next.js middleware bypass (CVSS 9.1): https://projectdiscovery.io/blog/nextjs-middleware-authorization-bypass
- Clerk response to CVE-2025-29927: https://clerk.com/blog/cve-2025-29927
- Clerk manual JWT verification: https://clerk.com/docs/guides/sessions/manual-jwt-verification
- Clerk clerkMiddleware docs (opt-in protection model): https://clerk.com/docs/reference/nextjs/clerk-middleware
- Clerk server-side token forwarding: https://clerk.com/docs/guides/development/making-requests
- axum-jwks crate (JWKS caching for Axum): https://crates.io/crates/axum-jwks
- axum-jwt-auth crate (JWKS with auto-refresh): https://crates.io/crates/axum-jwt-auth
- JWKS caching postmortem (Logto, Jan 2026): https://blog.logto.io/postmortem-jwks-cache
- OWASP Broken Access Control A01:2025 (frontend-only gating is bypassable): https://owasp.org/Top10/A01_2021-Broken_Access_Control/
- Gated content client-side bypass patterns: https://www.aleksandrhovhannisyan.com/blog/gated-content/
- tower-http CORS allow_headers Authorization: https://github.com/tower-rs/tower-http/issues/194
- TOCTOU race condition class (CWE-367): https://cwe.mitre.org/data/definitions/367.html
- File upload SSRF via path traversal: https://blog.doyensec.com/2025/01/09/cspt-file-upload.html
- ON DELETE CASCADE data loss risks: https://www.dbvis.com/thetable/postgres-on-delete-cascade-a-guide/
- Axum middleware layer execution order (bottom-to-top): https://docs.rs/axum/latest/axum/middleware/index.html
