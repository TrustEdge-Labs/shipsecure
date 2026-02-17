# Technology Stack — Auth & Tiered Access Milestone

**Project:** ShipSecure
**Domain:** Auth (Clerk), domain verification, tiered scan access, results gating, scan history, rate limiting, data retention
**Researched:** 2026-02-17
**Confidence:** HIGH (verified against current docs and ecosystem)

## Context

This milestone adds user authentication and tiered access to an existing working system:

- **Existing:** Axum 0.8.8 + sqlx 0.8.6 + PostgreSQL + Next.js 16.1.6 + React 19 + Resend
- **Removing:** async-stripe, Stripe webhooks (paid_audits table, stripe_events table get dropped)
- **Adding:** Clerk auth, domain verification, per-tier rate limits, severity gating, scan history, retention enforcement

The tier model being implemented:
- **anonymous:** 1 scan per IP/24h, 24h retention, low/medium findings only
- **developer:** Clerk signup + verified domain, 5 scans/month, full findings, 30-day retention
- **pro:** (future) — schema should accommodate, no implementation this milestone

---

## Frontend Stack Additions

### Core Auth Library

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **@clerk/nextjs** | 6.37.x (latest) | Auth SDK, UI components, middleware | First-class Next.js App Router support with React 19, pre-built `<SignIn>`, `<SignUp>`, `<UserButton>` components. Ships `clerkMiddleware()` for route protection and `auth()` for server components. Verified active as of Feb 11, 2026. |

**Installation:**
```bash
npm install @clerk/nextjs
```

**Critical Next.js 16 detail:** Next.js 16 renamed `middleware.ts` → `proxy.ts`. Clerk docs (updated Feb 2026) explicitly note: use `proxy.ts` for Next.js 16+, `middleware.ts` for ≤15. The Clerk code inside the file is identical — only the filename differs.

**Key APIs used in this milestone:**

```typescript
// proxy.ts (Next.js 16 — not middleware.ts)
import { clerkMiddleware, createRouteMatcher } from '@clerk/nextjs/server'

const isProtectedRoute = createRouteMatcher(['/dashboard(.*)', '/scan/history(.*)'])
export const proxy = clerkMiddleware(async (auth, req) => {
  if (isProtectedRoute(req)) await auth.protect()
})

// Server Component / Server Action
import { auth, clerkClient, currentUser } from '@clerk/nextjs/server'
const { userId } = await auth()              // get auth state
const user = await currentUser()            // full user object
const client = await clerkClient()
await client.users.updateUserMetadata(userId, {
  publicMetadata: { tier: 'developer', domainVerified: true, verifiedDomain: 'example.com' }
})

// Client Component
import { useAuth, useUser } from '@clerk/nextjs'
const { getToken } = useAuth()
const token = await getToken()  // JWT for backend API calls
```

**Tier storage decision:** Store `tier`, `domainVerified`, and `verifiedDomain` in Clerk `publicMetadata`. This is readable on both frontend (via session claims) and backend (via JWT). Avoid private metadata for tier — it requires a network call to Clerk API on every request, whereas public metadata is embedded in the JWT. **Size constraint:** Clerk cookies cap at ~1.2KB of custom claims; keep metadata minimal.

### No Additional Frontend Libraries Needed

| Capability | How | Why No New Library |
|------------|-----|-------------------|
| Domain verification meta tag display | Next.js `metadata` API or direct JSX | Built into Next.js |
| File upload for verification token | Native `FormData` + Route Handler | Web standard, no formidable needed in App Router |
| Tier-gated UI | Conditional rendering from `publicMetadata` | Plain React |
| Scan history display | Existing fetch pattern to backend | Already have `api.ts` utility |

---

## Backend Stack Additions (Rust/Axum)

### JWT Verification from Clerk

**Recommendation: `jsonwebtoken` crate with manual JWKS fetch, NOT clerk-rs.**

Rationale:
- `clerk-rs` (v0.4.1, last updated 8 months ago as of Feb 2026) is community-maintained, low download count (~1,456/month), and shows limited maintenance activity. HIGH risk for a security-critical dependency.
- `jsonwebtoken` (Keats) is the de facto standard Rust JWT library with active maintenance, used across thousands of production services.
- Clerk's JWKS endpoint is `https://api.clerk.com/v1/jwks` — standard JWKS format, compatible with any JWKS-aware JWT verifier.
- `axum-jwt-auth` wraps `jsonwebtoken` and adds JWKS caching + background refresh.

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **jsonwebtoken** | 9.x | JWT decode + verify | Industry standard, RSA/ECDSA/HMAC support, built-in JWK type support |
| **axum-jwt-auth** | latest | Axum extractor + JWKS caching | Provides `RemoteJwksDecoder` that fetches from Clerk's `/v1/jwks`, caches in memory, refreshes in background. Eliminates manual JWKS boilerplate. |

**Integration pattern:**
```rust
// In AppState or as lazy initialized extractor
use axum_jwt_auth::{JwtDecoderState, RemoteJwksDecoder};

// Extractor in handlers — optional auth
async fn create_scan(
    State(state): State<AppState>,
    claims: Option<Claims>,  // None = anonymous
    Json(req): Json<CreateScanRequest>,
) -> Result<...> {
    let tier = claims.as_ref()
        .and_then(|c| c.public_metadata.tier.as_deref())
        .unwrap_or("anonymous");
    ...
}
```

**Alternative considered — direct `reqwest` + `jsonwebtoken`:** Manually fetch JWKS, cache with `tokio::sync::RwLock`, verify manually. Works but ~150 lines of boilerplate vs using `axum-jwt-auth`. Not worth DIY.

### Clerk Webhook Reception

Clerk uses **Svix** to deliver webhooks. The Rust `svix` crate provides verified signature checking.

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **svix** | 1.83.0 (2025-12-15) | Webhook signature verification | Clerk's own webhook infrastructure. Required for secure `user.created` / `user.updated` event processing. Actively maintained — released Dec 2025. |

**Webhook events needed for this milestone:**
- `user.created` → insert row into `users` table with `clerk_id`, `email`, `tier = 'anonymous'`
- `user.updated` (metadata change) → sync `tier` + `domain_verified` to local `users` table

**Why sync to local DB at all?** Scan queries need `user_id` foreign keys and tier checks. Pulling tier from Clerk API on every scan request adds ~50-100ms latency + external dependency. Local sync is the right tradeoff.

```rust
// Axum webhook handler
use svix::webhooks::Webhook;

async fn handle_clerk_webhook(
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    let wh = Webhook::new(std::env::var("CLERK_WEBHOOK_SECRET")?)?;
    wh.verify(&body, &headers)
        .map_err(|_| ApiError::Unauthorized)?;
    // parse body, route on event type
}
```

### Domain Verification

No new Rust library needed. Domain verification uses:
1. **Meta tag method:** Backend fetches target URL via existing `reqwest` (already in deps), parses HTML with existing `scraper` (already in deps), checks for `<meta name="shipsecure-verification" content="{token}">`.
2. **File method:** Backend fetches `https://{domain}/.well-known/shipsecure-verify.txt` via existing `reqwest`, checks content matches stored token.

Token generation uses existing `rand` + `hex` crates (already in deps).

**No new dependencies for domain verification.**

### Per-Tier Rate Limiting

**Recommendation: Extend the existing DB-backed rate limiter.** Do NOT add `tower-governor` or similar in-memory crate.

Why DB-backed over in-memory:
- ShipSecure runs as a single Docker container on one DigitalOcean droplet → in-memory would work, but DB approach handles future horizontal scaling without rework.
- The existing `check_rate_limits()` function already queries PostgreSQL for scan counts. Adding tier-aware limits is a function signature change, not a new library.
- `tower-governor` is IP-based and doesn't understand user tiers or monthly windows.

**Rate limit tiers:**

| Tier | Window | Limit | Logic |
|------|--------|-------|-------|
| anonymous | 24 hours | 1 scan per IP | Existing IP check, reduce from 10 → 1 |
| developer | calendar month | 5 scans per user_id | New: count by `user_id` + month |
| pro | calendar month | configurable | Future |

**No new dependencies.** Update `check_rate_limits` signature to accept `Option<UserId>` and `tier`.

### Data Retention Enforcement

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **tokio-cron-scheduler** | 0.15.x | Scheduled DELETE for expired scans | Pure tokio cron scheduler. Runs in-process. No Redis, no external scheduler. |

**Retention policy:**
- anonymous scans: 24h (`expires_at` already on `Scan` model)
- developer scans: 30 days
- Cron runs hourly: `DELETE FROM scans WHERE expires_at < NOW()`

**Why tokio-cron-scheduler over alternatives:**
- PostgreSQL `pg_cron` extension: Not available on DigitalOcean managed Postgres basic tier without requesting it. Avoid operational complexity.
- Separate cron service: Overkill for single-container deployment.
- `tokio::time::interval` loop in main.rs: Would work but tokio-cron-scheduler adds cron expression syntax and error isolation.

```toml
tokio-cron-scheduler = "0.15"
```

---

## Database Schema Changes

No new ORM or migration tool — `sqlx migrate` already in use.

**New migrations needed:**

```sql
-- Migration 1: Add users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    clerk_id VARCHAR(255) NOT NULL UNIQUE,
    email TEXT NOT NULL,
    tier VARCHAR(20) NOT NULL DEFAULT 'anonymous'
        CHECK (tier IN ('anonymous', 'developer', 'pro')),
    domain_verified BOOLEAN NOT NULL DEFAULT FALSE,
    verified_domain TEXT,
    domain_verification_token TEXT,
    scan_count_this_month INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_users_clerk_id ON users(clerk_id);

-- Migration 2: Link scans to users
ALTER TABLE scans ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE scans ADD COLUMN tier VARCHAR(20) NOT NULL DEFAULT 'anonymous';
-- Note: tier column already exists as VARCHAR(10) — needs ALTER to expand + add 'developer'
-- Drop constraint, update type, re-add constraint

-- Migration 3: Drop Stripe tables (no longer needed)
DROP TABLE IF EXISTS paid_audits;
DROP TABLE IF EXISTS stripe_events;
ALTER TABLE scans DROP COLUMN IF EXISTS tier; -- will be re-added with correct values

-- Migration 4: Expand results_token usage for anonymous gating
-- results_token already exists on scans, repurpose as anonymous session token
```

**Note on existing `tier` column:** `scans.tier` currently allows `'free'|'paid'`. This constraint must be dropped and replaced with `'anonymous'|'developer'|'pro'`.

---

## Environment Variables Added

```bash
# Clerk (frontend)
NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY=pk_...
CLERK_SECRET_KEY=sk_...
NEXT_PUBLIC_CLERK_SIGN_IN_URL=/sign-in
NEXT_PUBLIC_CLERK_SIGN_UP_URL=/sign-up
NEXT_PUBLIC_CLERK_AFTER_SIGN_IN_URL=/dashboard
NEXT_PUBLIC_CLERK_AFTER_SIGN_UP_URL=/onboarding

# Clerk (backend — for JWT verification)
CLERK_JWKS_URL=https://api.clerk.com/v1/jwks
# OR use PEM public key for networkless verification:
CLERK_JWT_KEY=-----BEGIN PUBLIC KEY-----...

# Clerk webhooks (backend)
CLERK_WEBHOOK_SECRET=whsec_...
```

**JWT verification approach decision:** Use `CLERK_JWKS_URL` (remote JWKS) in development for simplicity. For production, Clerk Dashboard → API Keys → JWT public key → `CLERK_JWT_KEY` for networkless verification. Networkless is preferred: no outbound call on every request, no latency, no external SPOF.

---

## What NOT to Add

| Library | Why Skip |
|---------|---------|
| `next-auth` / `auth.js` | Clerk is the decision — don't add a second auth system |
| `clerk-rs` | Community-maintained, low activity (v0.4.1 from 8+ months ago). Use `jsonwebtoken` + `axum-jwt-auth` instead |
| `tower-governor` | IP-only rate limiting, doesn't understand user tiers or monthly windows |
| `redis` | Not needed — single-container deployment, DB-backed rate limits sufficient |
| `stripe` (keep removing) | Being removed this milestone; async-stripe dependency comes out of Cargo.toml |
| `genpdf` | Stripe PDF generation is gone; evaluate removal (may keep if needed elsewhere) |
| `Clerk Organizations` | B2B feature; overkill for individual user tiers |
| Separate job queue | Data retention is a simple scheduled DELETE; tokio-cron-scheduler is sufficient |
| `multer` / `formidable` | Next.js App Router's native `req.formData()` handles file upload without libraries |

---

## Complete Dependency Changes

### Cargo.toml Changes

```toml
# REMOVE:
# async-stripe = { version = "0.41", ... }
# hmac = "0.12"  -- only used for Stripe webhook verification
# sha2 = "0.10"  -- only used for Stripe webhook verification
# genpdf = "0.2" -- only used for Stripe PDF generation

# ADD:
jsonwebtoken = "9"
axum-jwt-auth = "0.4"  # verify current version on crates.io
svix = "1"             # Clerk webhook verification (1.83.0 current)
tokio-cron-scheduler = "0.15"

# KEEP (already present, used by new features):
# reqwest -- domain verification HTTP fetches
# scraper -- meta tag parsing for domain verification
# rand    -- verification token generation
# hex     -- token encoding
# base64  -- may be needed for JWT claims
# axum-extra -- for TypedHeader<Authorization<Bearer>> extractor
```

**Net change:** Remove 4 crates (async-stripe, hmac, sha2, genpdf), add 4 crates. Zero net growth in dependency count.

### package.json Changes

```bash
# ADD:
npm install @clerk/nextjs

# REMOVE (if present — check):
# Any stripe-related frontend packages
```

---

## Integration Architecture

```
Browser
  │
  ├── Clerk hosted UI (SignIn/SignUp components)
  │     └── Clerk CDN (no self-hosted auth pages needed)
  │
  ├── Next.js App Router
  │     ├── proxy.ts: clerkMiddleware() — route protection
  │     ├── Server Components: auth() → userId, publicMetadata.tier
  │     ├── Server Actions: clerkClient().users.updateUserMetadata()
  │     └── Route Handlers: /api/verify-domain (fetch → Axum)
  │
  └── Axum API (Bearer JWT in Authorization header)
        ├── JWT verification: axum-jwt-auth (JWKS from Clerk)
        ├── Tier-aware rate limiter: reads tier from JWT claims
        ├── Results gating: filters findings by severity per tier
        ├── Clerk webhook handler: /api/v1/webhooks/clerk (svix verify)
        └── Data retention: tokio-cron-scheduler hourly cleanup
```

**Token flow for API calls:**
1. Client component calls `getToken()` from `useAuth()`
2. Passes as `Authorization: Bearer {token}` to Axum
3. Axum `axum-jwt-auth` extractor decodes + verifies against Clerk JWKS
4. `Claims` struct available in handler with `userId`, `publicMetadata.tier`
5. Handler applies tier-appropriate scan limit and finding filter

---

## Sources

- Clerk Next.js v6 docs (updated Feb 11, 2026): https://clerk.com/docs/nextjs/getting-started/quickstart
- Clerk v6 Next.js 16 support (6.37.1, Jan 30 2026): https://clerk.com/changelog/2024-10-22-clerk-nextjs-v6
- Next.js 16 middleware → proxy.ts rename: https://nextjs.org/docs/messages/middleware-to-proxy
- clerkMiddleware() reference: https://clerk.com/docs/reference/nextjs/clerk-middleware
- Clerk publicMetadata RBAC pattern: https://clerk.com/docs/guides/secure/basic-rbac
- Clerk manual JWT verification: https://clerk.com/docs/guides/sessions/manual-jwt-verification
- Clerk JWKS endpoint (api.clerk.com/v1/jwks): https://clerk.com/docs/guides/sessions/manual-jwt-verification
- jsonwebtoken crate: https://crates.io/crates/jsonwebtoken
- axum-jwt-auth crate (remote JWKS support): https://crates.io/crates/axum-jwt-auth
- clerk-rs crate (community, v0.4.1): https://crates.io/crates/clerk-rs
- svix crate (v1.83.0, Dec 2025): https://docs.rs/crate/svix/latest
- Receive webhooks with Rust/Axum (Svix guide): https://www.svix.com/guides/receiving/receive-webhooks-with-rust-axum/
- tokio-cron-scheduler: https://crates.io/crates/tokio-cron-scheduler
- axum-extra TypedHeader: https://docs.rs/axum-extra/latest/axum_extra/struct.TypedHeader.html
