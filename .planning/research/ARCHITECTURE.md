# Architecture Patterns — Auth, Domain Verification, Tiered Access

**Project:** ShipSecure
**Milestone:** Auth + Domain Verification + Tiered Access
**Researched:** 2026-02-17
**Overall confidence:** HIGH (existing code read directly; Clerk patterns MEDIUM via official docs)

---

## Recommended Architecture

The milestone adds four orthogonal concerns to the existing Rust/Axum + Next.js + PostgreSQL system:

1. **Identity** — Clerk provides JWTs; Next.js middleware reads them; Axum verifies them locally via JWKS
2. **Domain ownership** — Stored per-user in PostgreSQL; checked at scan submission time
3. **Scan tiering** — Orchestrator gets a third `authenticated` tier between existing `free` and `paid`
4. **Results gating** — Frontend reads the full findings payload already returned by backend; visibility controlled by auth state client-side

None of these require rewriting existing components. Each integrates at a well-defined seam. The existing `tier` column in `scans`, `results_token` capability URL pattern, and `scraper` crate already in `Cargo.toml` are all reusable.

---

## Component Boundaries

### Existing Components (Unchanged)

| Component | File | Responsibility |
|-----------|------|---------------|
| Scan orchestrator | `src/orchestrator/worker_pool.rs` | Spawns scan tasks; already has `spawn_scan` (free) and `spawn_paid_scan` (paid); tier config is a 2-arm match |
| Scan API handler | `src/api/scans.rs` + `AppState` | POST /api/v1/scans, GET /api/v1/scans/:id |
| Results API handler | `src/api/results.rs` | GET /api/v1/results/:token, GET /api/v1/results/:token/download |
| Rate limiter | `src/rate_limit/middleware.rs` | 3/day email + 10/day IP; uses `count_scans_by_email_today` and `count_scans_by_ip_today` |
| Scan DB queries | `src/db/scans.rs` | All scan CRUD; `claim_pending_scan` (SELECT FOR UPDATE SKIP LOCKED), `count_scans_by_*_today` |
| Scan server action | `frontend/app/actions/scan.ts` | Server action submitting scan to backend; reads `BACKEND_URL` env |
| Results page | `frontend/app/results/[token]/page.tsx` | Server component; fetches + renders; already reads `data.tier` to show upgrade CTA |
| Frontend API client | `frontend/lib/api.ts` | `createScan`, `getScan`, `getScanByToken`, `getScanCount` |

### New Components

| Component | Location | Responsibility |
|-----------|----------|---------------|
| Clerk middleware | `frontend/middleware.ts` (new file) | `clerkMiddleware()` runs on every request; makes auth state available to server helpers |
| ClerkProvider | `frontend/app/layout.tsx` (modify) | Wraps app so `auth()`, `useAuth()`, `useUser()` work |
| Clerk JWT extractor | `src/api/auth.rs` (new file) | Axum `FromRequestParts` extractor; reads `Authorization: Bearer`; verifies against Clerk JWKS; extracts `clerk_user_id` |
| JWKS key cache | inside `src/api/auth.rs` | In-process cache (e.g. `tokio::sync::RwLock<JwksCache>`); refreshed on startup + 24h TTL; avoids per-request network call |
| Domain verification endpoints | `src/api/domain_verification.rs` (new file) | POST /api/v1/domains/verify-start, POST /api/v1/domains/verify-confirm, GET /api/v1/domains, DELETE /api/v1/domains/:domain |
| `domains` migration | `migrations/` (new file) | `users` table + `verified_domains` table; `clerk_user_id` column on `scans`; tier constraint extension |
| User DB queries | `src/db/users.rs` (new file) | Upsert user on auth'd scan submission; look up user by clerk_user_id |
| Domain DB queries | `src/db/domains.rs` (new file) | Insert pending verification; confirm ownership; `is_domain_verified(user_id, domain) -> bool` |
| Scan history endpoint | `src/api/scans.rs` (extend) | GET /api/v1/users/me/scans — requires `ClerkUser` extractor; paginated |
| Authenticated tier | `src/orchestrator/worker_pool.rs` (extend) | `spawn_authenticated_scan`; extend 2-arm match to 3-arm |
| Retention cleanup task | `src/cleanup.rs` (new file) + `src/main.rs` (extend) | Tokio interval loop; deletes expired scans hourly |
| Dashboard route | `frontend/app/dashboard/` (new route group) | Protected; scan history list; requires `auth().protect()` |
| Domain verification UI | `frontend/app/verify-domain/` (new route) | Domain ownership wizard; meta tag display; confirm button |
| Auth gate component | `frontend/components/auth-gate.tsx` (new) | Client component; wraps findings; blurs/hides by severity tier + auth state |

---

## Data Flow: Auth Token from Frontend to Backend

This is the central integration point. Every authenticated request follows this path.

```
Browser (Clerk session cookie auto-managed by @clerk/nextjs SDK)
  |
  v
Next.js middleware.ts
  clerkMiddleware() intercepts every request
  Sets auth state in request context (no redirect unless explicitly protected)
  |
  v
Next.js Server Action or Server Component
  const { getToken } = await auth()   // auth() from @clerk/nextjs/server
  const token = await getToken()      // short-lived JWT (~60s), auto-refreshed
  |
  v
fetch(BACKEND_URL + "/api/v1/scans", {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    "Authorization": `Bearer ${token}`,  // only added when token is non-null
  },
  body: JSON.stringify(payload),
})
  |
  v
Axum handler
  ClerkUser extractor reads Authorization header
  Fetches JWKS from Clerk endpoint (cached in-process)
  Verifies JWT signature + expiry + issuer claim
  Extracts claims.sub as clerk_user_id: String (e.g. "user_2abc123...")
  |
  v
Handler logic with verified Option<ClerkUser>
  None  -> anonymous path (free tier, existing behavior)
  Some  -> authenticated path (upsert user row, domain check, higher rate limit)
```

**Design rationale — local JWT verification:** Axum verifies JWTs using the JWKS public keys fetched from Clerk's well-known endpoint. It does NOT call Clerk's backend API per-request. This keeps auth verification at sub-millisecond overhead with no external dependency on Clerk's API availability during the request path.

**Token lifetime:** Clerk JWTs have a 60-second default expiry. The `getToken()` call in the frontend SDK transparently refreshes them before expiry. No manual refresh logic is needed.

---

## JWT Verification in Axum

### Library Choice

Use `axum-jwt-auth` (crates.io, built on `jsonwebtoken`) with remote JWKS support. It fetches and caches JWKS keys, verifies signatures, and exposes claims as a typed struct via an Axum extractor.

**Alternative: `clerk-rs`** provides a Clerk-specific `ClerkLayer`, but it is community-maintained (not Clerk-official) and its Axum integration adds a mandatory layer to the router rather than a per-route extractor. The extractor pattern is preferable because it allows `Option<ClerkUser>` for endpoints that serve both anonymous and authenticated users — which is exactly what the scan submission endpoint needs.

**Alternative: `axum-jwks`** is simpler but requires more manual claim parsing. `axum-jwt-auth` handles more of the JWKS lifecycle.

### Clerk JWKS Endpoint

```
https://<your-clerk-frontend-api-host>/.well-known/jwks.json
```

No Clerk secret key is required to read this endpoint. It is publicly accessible by design. The Backend API also exposes `https://api.clerk.com/v1/jwks` (requires `CLERK_SECRET_KEY`), but the frontend API JWKS URL is preferred to avoid exposing the secret key in the backend service.

### Custom Claims Struct

```rust
// src/api/auth.rs
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ClerkClaims {
    pub sub: String,            // Clerk user ID, stable identifier: "user_2abc..."
    pub iss: String,            // Clerk issuer URL; validate matches CLERK_ISSUER env
    pub exp: usize,             // Expiry unix timestamp; validated by jsonwebtoken
    pub iat: usize,             // Issued-at
    pub email: Option<String>,  // Present only if included in Clerk JWT template
    pub azp: Option<String>,    // Authorized party; can validate against frontend URL
}
```

### Axum Extractor Pattern

```rust
// src/api/auth.rs
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::{headers::{Authorization, Bearer}, TypedHeader};

pub struct ClerkUser(pub ClerkClaims);

#[async_trait]
impl<S> FromRequestParts<S> for ClerkUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| ApiError::Unauthorized)?;

        let claims = verify_clerk_jwt(bearer.token()).await
            .map_err(|_| ApiError::Unauthorized)?;

        Ok(ClerkUser(claims))
    }
}

// Option<ClerkUser> works automatically via Axum's blanket impl
// — returns None instead of 401 when Authorization header is absent
```

### Handler Usage (Optional Auth Pattern)

```rust
// src/api/scans.rs — create_scan handler
pub async fn create_scan(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(request_id): Extension<RequestId>,
    clerk_user: Option<ClerkUser>,      // None = anonymous, Some = authenticated
    Json(req): Json<CreateScanRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    // Existing: SSRF validation
    // Existing: rate limiting (extend to accept optional clerk_user_id)
    // New: domain verification lookup if clerk_user is Some
    // New: tier selection
    // Existing: create_scan DB call (add clerk_user_id parameter)
    // Existing: spawn orchestrator task (select which spawn_* method)
}
```

---

## Database Schema Changes

### New Tables

**`users` — maps Clerk user IDs to internal application users**

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    clerk_user_id VARCHAR(64) NOT NULL UNIQUE,   -- "user_2abc...", stable Clerk identifier
    email VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_clerk_user_id ON users (clerk_user_id);
```

**`verified_domains` — domain ownership state per user**

```sql
CREATE TABLE verified_domains (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    domain VARCHAR(255) NOT NULL,                -- "example.com", lowercase, no scheme
    verification_token VARCHAR(64) NOT NULL UNIQUE,
    verification_method VARCHAR(20) NOT NULL CHECK (
        verification_method IN ('meta_tag', 'file')
    ),
    verified_at TIMESTAMPTZ,                     -- NULL = pending, non-NULL = verified
    last_checked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, domain)
);

CREATE INDEX idx_verified_domains_user_id ON verified_domains (user_id);
CREATE INDEX idx_verified_domains_domain ON verified_domains (domain);
-- Partial index for "is this domain verified?" lookups at scan submission time
CREATE INDEX idx_verified_domains_verified
    ON verified_domains (domain)
    WHERE verified_at IS NOT NULL;
```

### Modified Tables

**`scans` — add `clerk_user_id` column**

```sql
ALTER TABLE scans ADD COLUMN clerk_user_id VARCHAR(64);

-- Enables scan history queries
CREATE INDEX idx_scans_clerk_user_id
    ON scans (clerk_user_id, created_at DESC)
    WHERE clerk_user_id IS NOT NULL;
```

**Extend `tier` constraint to include `authenticated`**

The existing migration `20260206000001_add_paid_audits.sql` adds `CHECK (tier IN ('free', 'paid'))`. This constraint must be updated:

```sql
ALTER TABLE scans DROP CONSTRAINT IF EXISTS scans_tier_check;
ALTER TABLE scans ADD CONSTRAINT scans_tier_check
    CHECK (tier IN ('free', 'authenticated', 'paid'));
```

### Cascade Delete Verification

On expired scan cleanup, cascade deletes propagate to `findings` (already confirmed: `findings` references `scans(id) ON DELETE CASCADE` from migration 2) and `paid_audits` (already confirmed: `paid_audits` references `scans(id) ON DELETE CASCADE`). No orphaned data will remain after scan deletion.

---

## Domain Verification Architecture

### Flow: Meta Tag Method

```
1. User (authenticated) requests verification for "example.com"
   POST /api/v1/domains/verify-start
   Body: { "domain": "example.com", "method": "meta_tag" }
   Auth: Required (ClerkUser extractor, non-optional)

   Axum:
   - Normalize domain (lowercase, strip scheme/path, strip www)
   - Upsert user row
   - Generate verification_token = "shipsecure-verify-" + 32 random alphanumeric chars
   - INSERT verified_domains (user_id, domain, verification_token, method, verified_at = NULL)
   - Return { "token": "shipsecure-verify-abc123...", "meta_tag": "<meta name=\"shipsecure-site-verification\" content=\"abc123...\">" }

2. User adds the meta tag to their site's <head>

3. User clicks "Verify Now"
   POST /api/v1/domains/verify-confirm
   Body: { "domain": "example.com" }
   Auth: Required

   Axum:
   - Look up verification_token for (user_id, domain) in verified_domains
   - Fetch https://example.com (using reqwest, already in Cargo.toml)
   - Parse HTML with scraper crate (already in Cargo.toml)
   - Search for: <meta name="shipsecure-site-verification" content="{token}">
   - If found: UPDATE verified_domains SET verified_at = NOW(), last_checked_at = NOW()
   - Return { "verified": true } or { "verified": false, "error": "Meta tag not found" }
```

### Flow: File Method (Alternative)

```
Verification file path: https://example.com/.well-known/shipsecure-verify-{token}.txt
File content: token value

Axum fetches the URL, checks content matches token.
Simpler for server-side verification, requires filesystem access on target.
```

The meta tag method is preferred because it works on all hosting platforms including static sites and CDNs.

### Verification at Scan Submission

```rust
// In create_scan handler, BEFORE spawning orchestrator:
let domain = extract_domain_from_url(&validated_url);  // "example.com"

let tier = match clerk_user.as_ref() {
    None => "free",  // anonymous — existing behavior unchanged
    Some(user) => {
        // Upsert user row
        let internal_user = db::users::upsert_user(
            &state.pool,
            &user.0.sub,
            user.0.email.as_deref().unwrap_or("")
        ).await?;

        // Check domain verification
        let is_verified = db::domains::is_domain_verified(
            &state.pool,
            internal_user.id,
            &domain
        ).await?;

        // For orchestrator purposes: authenticated is the tier for all logged-in users
        // Domain verification status is stored separately; frontend uses it for gating
        "authenticated"
    }
};
```

**Domain verification affects results visibility in the frontend, not scanner depth.** The `authenticated` tier controls how many JS files are scanned and timeout durations. Whether findings are visible depends on frontend logic checking both auth state and domain verification status.

### Domain Normalization

```rust
fn extract_domain_from_url(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
        .map(|h| h.strip_prefix("www.").unwrap_or(&h).to_string())
        .unwrap_or_default()
}
```

The `url` crate is already in `Cargo.toml`.

---

## Scan Tiering in the Orchestrator

### Current 2-Arm Configuration

From `src/orchestrator/worker_pool.rs` (direct code read):

```rust
let (max_js_files, extended_files, vibecode_timeout, other_timeout) = match tier {
    "paid" => (50, true, Duration::from_secs(600), Duration::from_secs(60)),
    _      => (20, false, Duration::from_secs(180), Duration::from_secs(60)),
};
```

### New 3-Arm Configuration

```rust
let (max_js_files, extended_files, vibecode_timeout, other_timeout) = match tier {
    "paid"          => (50, true,  Duration::from_secs(600), Duration::from_secs(60)),
    "authenticated" => (30, true,  Duration::from_secs(300), Duration::from_secs(60)),
    _               => (20, false, Duration::from_secs(180), Duration::from_secs(60)),
};
```

The `_` arm handles both `"free"` and any unrecognized string, preserving backward compatibility.

### New `spawn_authenticated_scan` Method

```rust
/// Spawn an AUTHENTICATED tier scan.
/// Config: 30 JS files (vs 20 free / 50 paid), extended exposed files, 300s vibecode timeout.
/// Called when a Clerk-authenticated user submits a scan.
pub fn spawn_authenticated_scan(
    &self,
    scan_id: Uuid,
    target_url: String,
    clerk_user_id: String,
    request_id: Option<Uuid>,
) {
    // Same structure as spawn_scan / spawn_paid_scan
    // Passes "authenticated" tier string to execute_scan_internal
}
```

### Tier Selection in `create_scan`

Tier selection logic lives in the API handler, not the orchestrator. The orchestrator receives the tier as an opaque string and applies config. This separation keeps the orchestrator free of auth concerns.

```rust
match tier {
    "free"          => state.orchestrator.spawn_scan(scan_id, target_url, request_id),
    "authenticated" => state.orchestrator.spawn_authenticated_scan(scan_id, target_url, clerk_user_id, request_id),
    "paid"          => state.orchestrator.spawn_paid_scan(scan_id, target_url, request_id),
    _               => state.orchestrator.spawn_scan(scan_id, target_url, request_id),
}
```

### Expiry by Tier

Set `expires_at` in the orchestrator when completing a scan:

```rust
// In execute_scan_internal, on completion:
let expires_at = match tier {
    "paid"          => None,                        // no expiry
    "authenticated" => Some(now + 30 days),
    _               => Some(now + 3 days),          // existing behavior
};
```

The existing `expires_at` column and the existing results expiry UI in `frontend/app/results/[token]/page.tsx` already handle display of this value.

---

## Results Gating in the Frontend

### Design Decision: Backend Returns Full Payload, Frontend Gates

The backend's `GET /api/v1/results/:token` already returns all findings regardless of tier. This is intentional: the capability URL (unguessable token) is the authorization mechanism for results access. Gating is a UI-layer concern based on auth state.

**Rationale for keeping gating frontend-only:**
- The capability URL is shareable — a user may share their link with a colleague. Backend filtering would require the results endpoint to also verify JWT, complicating the stateless token-based design.
- All findings are already stored in the DB; not returning them wastes no computation.
- Frontend gating is easily adjustable without backend deploys.

### New Field in Results API Response

The backend should return whether the authenticated viewer has verified domain ownership:

```json
{
  "owner_verified": true,
  "findings": [...],
  "tier": "authenticated",
  ...
}
```

Axum computes this by: if the request includes a valid JWT (`Option<ClerkUser>` is `Some`), check `db::domains::is_domain_verified(user_id, domain)` where domain is extracted from `target_url`. If no JWT, `owner_verified: false`.

### Gating Rules

| Scan Tier | Auth Status | Domain Verified | Findings Visible |
|-----------|-------------|----------------|-----------------|
| free | Not signed in | N/A | Low + Medium only |
| free | Signed in | No | Low + Medium only |
| free | Signed in | Yes | All |
| authenticated | Not signed in | N/A | Low only |
| authenticated | Signed in | No | All |
| authenticated | Signed in | Yes | All |
| paid | Any | Any | All (existing behavior) |

### `AuthGate` Component

```tsx
// frontend/components/auth-gate.tsx
'use client'

import { useAuth } from '@clerk/nextjs'
import { Finding } from '@/lib/types'

interface AuthGateProps {
  findings: Finding[]
  scanTier: string
  ownerVerified: boolean
}

function getGatedSeverities(tier: string, isSignedIn: boolean, ownerVerified: boolean): Set<string> {
  if (tier === 'paid') return new Set()             // show all
  if (ownerVerified) return new Set()               // verified owner sees all
  if (tier === 'authenticated') {
    return isSignedIn ? new Set() : new Set(['medium', 'high', 'critical'])
  }
  // free tier
  return isSignedIn ? new Set(['high', 'critical']) : new Set(['high', 'critical'])
}

export function AuthGate({ findings, scanTier, ownerVerified }: AuthGateProps) {
  const { isSignedIn } = useAuth()
  const gated = getGatedSeverities(scanTier, !!isSignedIn, ownerVerified)

  const visible = findings.filter(f => !gated.has(f.severity))
  const hidden  = findings.filter(f =>  gated.has(f.severity))

  return (
    <>
      {visible.map(f => <FindingCard key={f.id} finding={f} />)}
      {hidden.length > 0 && <GatedFindingsBanner count={hidden.length} tier={scanTier} isSignedIn={!!isSignedIn} />}
    </>
  )
}
```

---

## Per-User Rate Limiting

### Current State

`src/rate_limit/middleware.rs` checks:
- Email: 3 scans/day (via `count_scans_by_email_today`)
- IP: 10 scans/day (via `count_scans_by_ip_today`)

### Modified Logic

```rust
// src/rate_limit/middleware.rs
pub async fn check_rate_limits(
    pool: &PgPool,
    email: &str,
    ip: &str,
    clerk_user_id: Option<&str>,     // NEW — from ClerkUser extractor
) -> Result<(), ApiError> {
    if let Some(user_id) = clerk_user_id {
        // Authenticated users: higher daily limit, bound to stable user ID
        let count = scans::count_scans_by_user_today(pool, user_id).await?;
        let limit = 10;  // 10/day authenticated vs 3/day anonymous
        if count >= limit {
            return Err(ApiError::RateLimited(format!(
                "You've reached your daily scan limit of {} scans. Try again tomorrow.", limit
            )));
        }
        // Skip IP + email checks for authenticated users
        return Ok(());
    }

    // Anonymous: existing email (3/day) + IP (10/day) limits unchanged
    check_email_limit(pool, email).await?;
    check_ip_limit(pool, ip).await?;
    Ok(())
}
```

### New DB Query

```sql
-- src/db/scans.rs: count_scans_by_user_today
SELECT COUNT(*)::bigint FROM scans
WHERE clerk_user_id = $1
  AND created_at > NOW() - INTERVAL '1 day';
```

Uses the `idx_scans_clerk_user_id` index (partial, `WHERE clerk_user_id IS NOT NULL`).

---

## Scan History

### New Endpoint

```
GET /api/v1/users/me/scans
Authorization: Bearer <token>    (required; 401 if absent or invalid)
Query: ?page=1&limit=20
```

```rust
// src/api/scans.rs
pub async fn get_user_scan_history(
    State(state): State<AppState>,
    clerk_user: ClerkUser,      // Non-optional: 401 if no valid JWT
    Query(params): Query<PaginationParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let scans = db::scans::get_scans_by_user(
        &state.pool,
        &clerk_user.0.sub,
        params.limit.unwrap_or(20).min(100),
        params.page.unwrap_or(1).saturating_sub(1),
    ).await?;

    Ok(Json(json!({ "scans": scans, "page": params.page })))
}
```

### DB Query

```sql
-- src/db/scans.rs: get_scans_by_user
SELECT id, target_url, status, score, tier, results_token,
       created_at, completed_at, expires_at
FROM scans
WHERE clerk_user_id = $1
ORDER BY created_at DESC
LIMIT $2 OFFSET $3;
```

O(log n) with `idx_scans_clerk_user_id` composite index.

### Frontend: Dashboard Route

```
frontend/app/dashboard/page.tsx      — Server Component; auth protected
frontend/app/dashboard/layout.tsx    — calls auth().protect() (redirects to sign-in)
```

```tsx
// frontend/app/dashboard/layout.tsx
import { auth } from '@clerk/nextjs/server'
import { redirect } from 'next/navigation'

export default async function DashboardLayout({ children }: { children: React.ReactNode }) {
  const { userId } = await auth()
  if (!userId) redirect('/sign-in')
  return <>{children}</>
}
```

---

## Data Retention Cleanup

### Pattern

Tokio background task using the existing `TaskTracker` + `CancellationToken` shutdown coordination infrastructure in `src/main.rs`.

```rust
// src/cleanup.rs (new file)
use sqlx::PgPool;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub async fn run_retention_cleanup(pool: PgPool, shutdown: CancellationToken) {
    let mut interval = tokio::time::interval(Duration::from_secs(3600)); // hourly

    loop {
        tokio::select! {
            _ = interval.tick() => {
                match delete_expired_scans(&pool).await {
                    Ok(count) if count > 0 =>
                        tracing::info!(deleted = count, "retention_cleanup_complete"),
                    Ok(_) => {}  // nothing to clean
                    Err(e) =>
                        tracing::error!(error = %e, "retention_cleanup_failed"),
                }
            }
            _ = shutdown.cancelled() => {
                tracing::info!("Retention cleanup task shutting down");
                break;
            }
        }
    }
}

async fn delete_expired_scans(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM scans WHERE expires_at IS NOT NULL AND expires_at < NOW()"
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
```

```rust
// src/main.rs — after orchestrator creation, before axum::serve:
let cleanup_pool = pool.clone();
let cleanup_token = shutdown_token.clone();
task_tracker.spawn(async move {
    cleanup::run_retention_cleanup(cleanup_pool, cleanup_token).await;
});
```

### Retention Policy by Tier

| Tier | `expires_at` | Duration |
|------|-------------|---------|
| `free` | `NOW() + INTERVAL '3 days'` | Current behavior, retained |
| `authenticated` | `NOW() + INTERVAL '30 days'` | New |
| `paid` | `NULL` | No expiry (current behavior retained) |

### Cascade Verification

- `findings` has `ON DELETE CASCADE` to `scans` (confirmed in migration 2)
- `paid_audits` has `ON DELETE CASCADE` to `scans` (confirmed in migration 6)
- `verified_domains` cascades to `users`, not to `scans` — no issue
- New scan data is tied to `clerk_user_id` as a string column, not a FK — no cascade needed

---

## New API Routes Summary

| Method | Path | Auth | Purpose |
|--------|------|------|---------|
| POST | `/api/v1/domains/verify-start` | Required | Begin domain verification, returns token |
| POST | `/api/v1/domains/verify-confirm` | Required | Check meta tag / file, set verified_at |
| GET | `/api/v1/domains` | Required | List user's domains with verification status |
| DELETE | `/api/v1/domains/:domain` | Required | Remove domain ownership record |
| GET | `/api/v1/users/me/scans` | Required | Paginated scan history for authenticated user |

Existing routes that gain optional auth support:

| Method | Path | Auth Change | Effect |
|--------|------|------------|--------|
| POST | `/api/v1/scans` | Optional (was none) | Selects tier, stores clerk_user_id, applies user rate limit |
| GET | `/api/v1/results/:token` | Optional (was none) | Returns `owner_verified` field if authenticated viewer owns domain |

---

## Build Order (Dependency-Driven)

Dependencies flow top-to-bottom. Work in this order within a milestone to avoid blocked work.

| Step | Component | Depends On | Work |
|------|-----------|-----------|------|
| 1 | DB migrations | — | Add `users`, `verified_domains` tables; add `clerk_user_id` to `scans`; extend `tier` constraint |
| 2 | `ClerkJwtExtractor` + JWKS cache | Step 1 (for `clerk_user_id` storage) | `src/api/auth.rs`; `Option<ClerkUser>` extractor |
| 3 | `users` upsert on scan creation | Step 2 | `src/db/users.rs`; called from `create_scan` handler |
| 4 | Per-user rate limiting | Steps 2, 3 | Extend `check_rate_limits` signature; add `count_scans_by_user_today` query |
| 5 | Next.js Clerk install | — (parallel with steps 2-4) | `@clerk/nextjs` package; `ClerkProvider` in layout; `middleware.ts` |
| 6 | `getToken()` in scan server action | Step 5 | Modify `frontend/app/actions/scan.ts` to add `Authorization` header when signed in |
| 7 | Orchestrator: authenticated tier | Step 1 (tier constraint) | `spawn_authenticated_scan`; 3-arm match in `execute_scan_internal`; tier-based expiry |
| 8 | Domain verification endpoints | Steps 1, 2, 3 | `src/api/domain_verification.rs`; `src/db/domains.rs` |
| 9 | Tier selection in `create_scan` | Steps 2, 4, 7, 8 | Wire together: auth extract → domain check → rate limit → spawn |
| 10 | Results endpoint `owner_verified` | Steps 2, 8 | Modify `get_results_by_token` to accept `Option<ClerkUser>` and compute field |
| 11 | `AuthGate` component | Step 5 | `frontend/components/auth-gate.tsx`; uses `useAuth()` |
| 12 | Results page auth gating | Steps 10, 11 | Modify `frontend/app/results/[token]/page.tsx` to use `AuthGate` |
| 13 | Scan history endpoint | Steps 1, 2 | `GET /api/v1/users/me/scans` |
| 14 | Dashboard route | Steps 5, 13 | `frontend/app/dashboard/` protected route |
| 15 | Domain verification UI | Steps 5, 8 | `frontend/app/verify-domain/` wizard |
| 16 | Retention cleanup task | Step 7 (for tier-based expiry logic) | `src/cleanup.rs`; hook into `main.rs` task_tracker |

Steps 1-4 (Axum/DB) and Step 5 (Next.js Clerk install) can proceed in parallel across backend and frontend developers.

---

## Environment Variables Required

| Variable | Used By | Purpose |
|----------|---------|---------|
| `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY` | Next.js (client + server) | Clerk SDK initialization |
| `CLERK_SECRET_KEY` | Next.js server only (optional) | Only needed if using `currentUser()` backend SDK calls |
| `NEXT_PUBLIC_CLERK_SIGN_IN_URL` | Next.js | e.g. `/sign-in` |
| `NEXT_PUBLIC_CLERK_SIGN_UP_URL` | Next.js | e.g. `/sign-up` |
| `NEXT_PUBLIC_CLERK_AFTER_SIGN_IN_URL` | Next.js | e.g. `/dashboard` |
| `CLERK_JWKS_URL` | Axum (`src/api/auth.rs`) | e.g. `https://<clerk-domain>/.well-known/jwks.json` |
| `CLERK_ISSUER` | Axum (`src/api/auth.rs`) | Issuer claim to validate, e.g. `https://<clerk-domain>` |

Axum does NOT need `CLERK_SECRET_KEY`. JWKS verification uses only the public key endpoint.

## Cargo.toml Additions

```toml
# JWT verification with JWKS support
axum-jwt-auth = "0.4"
jsonwebtoken = "9"

# Typed header extraction (Authorization: Bearer)
axum-extra = { version = "0.10", features = ["typed-header"] }
headers = "0.4"
```

## frontend/package.json Addition

```json
"@clerk/nextjs": "^6"
```

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: JWT Verification via Clerk Backend API Per-Request

**What goes wrong:** Calling `https://api.clerk.com/v1/sessions/{id}/verify` on every Axum request.

**Consequences:** 50-200ms latency per request added; rate limit exposure against Clerk's API; hard availability dependency on Clerk during request path.

**Prevention:** Verify JWTs locally using JWKS public keys. Cache JWKS (rotate on cache miss or 24h TTL). One network round-trip on startup, zero per request.

### Anti-Pattern 2: Filtering Findings in the Backend Response

**What goes wrong:** Backend `results.rs` returns only findings appropriate for the caller's auth state.

**Consequences:** Breaks the capability URL sharing model (anonymous friend sees fewer findings than owner). Creates cache invalidation complexity (same token, different responses based on headers). Mixes auth concerns into stateless token-based access.

**Prevention:** Backend always returns all findings. Frontend `AuthGate` component filters rendering based on `useAuth()` + `ownerVerified` field. This is the same pattern used by the existing `data.tier === 'free'` check in `ResultsPage`.

### Anti-Pattern 3: Storing JWTs in the Database

**What goes wrong:** Persisting the short-lived session token to reference later.

**Consequences:** 60-second expiry makes stored tokens immediately stale. Security risk if database is compromised. No benefit over just re-validating on each request.

**Prevention:** Store only `clerk_user_id` (the `sub` claim). It is stable across sessions. Re-verify identity on each API call via JWT signature check.

### Anti-Pattern 4: Re-Verifying Domain on Every Scan

**What goes wrong:** Fetching `https://example.com` to check the meta tag on every scan submission.

**Consequences:** 200-500ms of synchronous HTTP latency added to scan submission path. Meta tag presence is not required to persist continuously — it only needs to be present at the moment of verification.

**Prevention:** Verify once (user-initiated via `/api/v1/domains/verify-confirm`); set `verified_at` in DB. Check `is_domain_verified` (indexed DB lookup, sub-millisecond) at scan submission. Let users re-verify manually if they want to confirm continued ownership.

### Anti-Pattern 5: Protecting All Routes in `middleware.ts`

**What goes wrong:** Using `auth().protect()` or a route matcher that protects `/` and the scan submission path.

**Consequences:** Anonymous scanning (core free-tier use case) is broken. Zero-friction onboarding eliminated.

**Prevention:** `clerkMiddleware()` in `middleware.ts` runs on all routes but does NOT redirect. Only `/dashboard/*` and `/verify-domain/*` call `auth().protect()` (or check `userId` and redirect). The scan submission server action calls `auth()` optionally and proceeds anonymously if `userId` is null.

### Anti-Pattern 6: Blocking Scan Submission on Domain Verification

**What goes wrong:** Requiring verified domain ownership before allowing any scan submission.

**Consequences:** Eliminates the product's key value proposition (fast, no-friction security scans). Users without ownership can still run scans on their own sites; they just don't get the verified-owner results visibility.

**Prevention:** Domain verification is checked to determine results gating only. Scans always proceed. The `authenticated` tier (logged-in users regardless of verification status) gets better scanner config than `free` as an incentive to sign up.

---

## Scalability Considerations

All changes below are additive. Existing throughput is unaffected.

| Concern | Impact | Mitigation |
|---------|--------|-----------|
| JWKS fetch latency | Startup-time only (cache after first fetch) | In-process `RwLock`; no Redis needed at current scale |
| Domain verification check at scan time | 1 indexed SELECT per authenticated scan submission | Sub-millisecond with partial index on `(domain) WHERE verified_at IS NOT NULL` |
| Rate limit query for authenticated users | 1 COUNT query per authenticated scan (replaces 2 for anonymous) | Net neutral; uses `idx_scans_clerk_user_id` |
| Scan history queries | O(log n) with composite index | Pagination keeps result sets bounded |
| Retention cleanup | 1 DELETE batch per hour; runs off hot path | Does not block any request handler |
| Clerk session cookie size | ~2-4KB per request from browser | Standard; existing infrastructure handles |

---

## Sources

- Existing codebase (`src/orchestrator/worker_pool.rs`, `src/api/scans.rs`, `src/api/results.rs`, `src/rate_limit/middleware.rs`, `src/db/scans.rs`, `src/main.rs`, `src/models/scan.rs`, migrations, `frontend/app/actions/scan.ts`, `frontend/app/results/[token]/page.tsx`, `frontend/lib/api.ts`, `Cargo.toml`, `package.json`) — HIGH confidence (direct read)
- [Clerk: clerkMiddleware() Next.js docs](https://clerk.com/docs/reference/nextjs/clerk-middleware) — MEDIUM confidence
- [Clerk: auth() App Router reference](https://clerk.com/docs/reference/nextjs/app-router/auth) — MEDIUM confidence
- [Clerk: Session Tokens (60s lifetime)](https://clerk.com/docs/guides/sessions/session-tokens) — MEDIUM confidence
- [Clerk: Manual JWT Verification](https://clerk.com/docs/guides/sessions/manual-jwt-verification) — MEDIUM confidence
- [Clerk: JWKS Backend API reference](https://clerk.com/docs/reference/backend-api/tag/JWKS) — MEDIUM confidence
- [Clerk: Server Actions reference](https://clerk.com/docs/reference/nextjs/app-router/server-actions) — MEDIUM confidence
- [axum-jwt-auth crate](https://crates.io/crates/axum-jwt-auth) — MEDIUM confidence (crates.io confirmed)
- [axum-jwks crate](https://lib.rs/crates/axum-jwks) — MEDIUM confidence (lib.rs confirmed)
- [clerk-rs community SDK](https://github.com/DarrenBaldwin07/clerk-rs) — LOW confidence (community, not official)
- [Tokio task spawning patterns](https://tokio.rs/tokio/tutorial/spawning) — HIGH confidence (official Tokio docs)
- [Integrating Clerk with Next.js + Express (pattern reference)](https://mtarkar.medium.com/integrating-next-js-clerk-auth-with-express-9c7f0407c6f0) — LOW confidence (community article)
