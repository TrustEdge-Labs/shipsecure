# Phase 29: Auth Foundation - Research

**Researched:** 2026-02-17
**Domain:** Clerk authentication (Next.js 16 frontend + Axum backend), JWT verification, Svix webhooks, Nginx CVE mitigation
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Sign-in/Sign-up flow:**
- Dedicated full pages at /sign-in and /sign-up (not modal, not inline)
- Use Clerk's default appearance — no custom theming to match dark mode
- After sign-in: always redirect to /dashboard
- After sign-up: same destination as sign-in (/dashboard) — no separate onboarding page

**Header auth integration:**
- UserButton (avatar/dropdown) replaces the 'Scan Now' CTA position (right side of header) when signed in
- When signed out: show a 'Sign In' button in that same position (no 'Scan Now' CTA in header)
- UserButton dropdown uses Clerk defaults only (avatar, email, 'Manage account', 'Sign out') — no custom menu items
- No tier badge or quota display in header — that's Phase 33-34 scope

**Dashboard skeleton:**
- Protected /dashboard route using same full-width layout as the rest of the site (no sidebar nav)
- Empty state: welcome message greeting user by name ("Welcome, John") + prominent 'Verify your domain' CTA
- CTA links to /verify-domain even though that page won't exist until Phase 32 (will 404 in the interim — acceptable)

**Auth error handling:**
- Unauthenticated /dashboard access: silent redirect to /sign-in (no flash message)
- Invalid/expired JWT: API returns 401 with generic 'Authentication required' message — don't leak JWT details
- Webhook failure: log the error and return 500 — rely on Clerk/Svix automatic retry with exponential backoff
- Session expiry: no background polling — handle on next user action (API returns 401, frontend redirects to sign-in)

### Claude's Discretion
- Clerk component configuration details (e.g., which OAuth providers to show, form field ordering)
- Loading states during auth redirects
- Exact layout spacing and typography for dashboard welcome page
- proxy.ts middleware configuration specifics

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| AUTH-01 | User can sign up with email/password via Clerk | @clerk/nextjs SignIn/SignUp components, ClerkProvider, dedicated page routes |
| AUTH-02 | User can sign up/in with Google OAuth | Clerk handles OAuth by default; Google enabled in Clerk Dashboard |
| AUTH-03 | User can sign up/in with GitHub OAuth | Clerk handles OAuth by default; GitHub enabled in Clerk Dashboard |
| AUTH-04 | User session persists across browser restarts | Clerk stores session in cookies automatically; no extra config needed |
| AUTH-05 | Signed-in user sees UserButton (avatar/menu) in sticky header | `<UserButton />` + `<SignedIn>` / `<SignedOut>` conditional rendering in header |
| AUTH-06 | Dashboard routes redirect unauthenticated users to sign-in | `auth.protect()` in proxy.ts via `createRouteMatcher(['/dashboard(.*)'])` |
| INFR-01 | CORS config allows Authorization header for JWT bearer tokens | Add `axum::http::header::AUTHORIZATION` to `allow_headers` in tower-http CorsLayer |
| INFR-02 | Nginx strips x-middleware-subrequest header (CVE-2025-29927) | `proxy_set_header x-middleware-subrequest ""` in /api/ and / location blocks |
| INFR-03 | Clerk webhook handler verifies svix signatures on user.created events | svix crate `Webhook::new(secret).verify(&body, &headers)` |
| INFR-04 | Axum verifies Clerk JWTs locally via cached JWKS public keys | axum-jwt-auth 0.6 `RemoteJwksDecoder` with Clerk's JWKS endpoint |
</phase_requirements>

---

## Summary

This phase adds Clerk authentication to the existing ShipSecure stack: a Next.js 16 frontend and Axum 0.8 backend. The work splits cleanly into three plans: (1) backend auth infrastructure (CORS fix, JWT extractor, users table migration, Clerk webhook), (2) frontend auth integration (ClerkProvider, proxy.ts, sign-in/sign-up pages, header UserButton, dashboard route), and (3) Nginx CVE mitigation and production environment wiring.

**Critical Next.js 16 fact:** The project runs Next.js 16.1.6 where the middleware file is `proxy.ts` (not `middleware.ts`). The function export remains `export default clerkMiddleware()` — only the filename changes. Clerk's documentation explicitly supports `proxy.ts` for Next.js 16+. The existing `next.config.ts` already has `experimental: { testProxy: ... }` indicating the project is aware of this convention.

The backend JWT path uses `axum-jwt-auth` v0.6.3 (latest as of Jan 2026) with `RemoteJwksDecoder` pointing to Clerk's JWKS endpoint. This gives JWKS key caching and background refresh with no per-request Clerk API calls, satisfying INFR-04. For the Clerk webhook, the `svix` crate v1.85.0 provides a three-line signature verification call. The Nginx CVE mitigation is a one-line `proxy_set_header` addition to the existing Jinja2 template.

**Primary recommendation:** Install `@clerk/nextjs@latest` (v6.x) on the frontend and `axum-jwt-auth = "0.6"` + `svix = "1"` on the backend. Name the middleware file `proxy.ts` (not `middleware.ts`) since the project runs Next.js 16.1.6.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| @clerk/nextjs | 6.37.4 (latest) | ClerkProvider, UserButton, SignIn, SignUp, clerkMiddleware, auth() | Official Clerk SDK for Next.js App Router; supports proxy.ts in Next.js 16 |
| axum-jwt-auth | 0.6.3 | RemoteJwksDecoder for JWKS-cached JWT verification in Axum handlers | Only maintained Axum-native JWT extractor with remote JWKS support; Jan 2026 release |
| svix | 1.85.0 | Clerk webhook signature verification | Clerk uses Svix for webhook delivery; official Svix Rust crate |
| tower-http | 0.6.8 (already present) | CorsLayer — add AUTHORIZATION to allow_headers | Already in project; just needs header list update |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| sqlx | 0.8.6 (already present) | users table migration | For storing Clerk user IDs linked to the system |
| jsonwebtoken | (transitive via axum-jwt-auth) | Validation struct with Algorithm::RS256 | Provided by axum-jwt-auth; configure via Validation::new(Algorithm::RS256) |
| http | 1.0.0 | HeaderMap for svix webhook verification | Required by svix crate for header extraction |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| axum-jwt-auth | axum-jwks | axum-jwks is less actively maintained; axum-jwt-auth has Jan 2026 release, background refresh built-in |
| axum-jwt-auth | clerk-rs | Explicitly rejected — v0.4.1, 8+ months stale, security risk per prior decision |
| axum-jwt-auth | manual jsonwebtoken | Requires hand-rolling JWKS fetching + caching — see Don't Hand-Roll section |
| svix crate | manual HMAC-SHA256 | Already doing this for Stripe; for Clerk/Svix, the crate is 3 lines vs 40 lines of manual verification |

**Installation (frontend):**
```bash
npm install @clerk/nextjs@latest
```

**Installation (backend, add to Cargo.toml):**
```toml
axum-jwt-auth = "0.6"
svix = "1"
http = "1.0"
```

---

## Architecture Patterns

### Recommended Project Structure (additions)

```
frontend/
├── app/
│   ├── dashboard/
│   │   └── page.tsx          # Protected dashboard — uses auth() + currentUser()
│   ├── sign-in/
│   │   └── [[...sign-in]]/
│   │       └── page.tsx      # Clerk SignIn component (catch-all required)
│   └── sign-up/
│       └── [[...sign-up]]/
│           └── page.tsx      # Clerk SignUp component (catch-all required)
├── components/
│   └── header.tsx            # Updated: SignedIn/SignedOut + UserButton
├── proxy.ts                  # Next.js 16: clerkMiddleware + createRouteMatcher
└── ...

src/
├── api/
│   ├── auth.rs               # New: ClerkUser extractor (wraps Claims<ClerkClaims>)
│   ├── webhooks.rs           # Existing: add handle_clerk_webhook handler
│   └── mod.rs                # Add clerk_webhooks module
└── ...

migrations/
└── 20260217000001_create_users.sql  # New: users table with clerk_user_id

infrastructure/
└── templates/
    └── shipsecure.nginx.conf.j2     # Updated: add proxy_set_header x-middleware-subrequest ""
```

### Pattern 1: Next.js 16 proxy.ts with clerkMiddleware
**What:** Clerk middleware runs on every request, protecting /dashboard/* and making /sign-in, /sign-up public.
**When to use:** All authenticated routes in this project.

```typescript
// Source: https://clerk.com/docs/reference/nextjs/clerk-middleware
// File: frontend/proxy.ts (Next.js 16 — NOT middleware.ts)
import { clerkMiddleware, createRouteMatcher } from '@clerk/nextjs/server'

const isProtectedRoute = createRouteMatcher(['/dashboard(.*)'])

export default clerkMiddleware(async (auth, req) => {
  if (isProtectedRoute(req)) await auth.protect()
})

export const config = {
  matcher: [
    '/((?!_next|[^?]*\\.(?:html?|css|js(?!on)|jpe?g|webp|png|gif|svg|ttf|woff2?|ico|csv|docx?|xlsx?|zip|webmanifest)).*)',
    '/(api|trpc)(.*)',
  ],
}
```

Note: The `config` export stays named `config` (not renamed to `proxy`). Only the file name and default function export name change.

### Pattern 2: ClerkProvider in Root Layout
**What:** Wrap the existing root layout with ClerkProvider; keep header separate.
**When to use:** Required — all Clerk components and hooks need this context.

```typescript
// Source: https://clerk.com/docs/nextjs/getting-started/quickstart
// File: frontend/app/layout.tsx (updated)
import { ClerkProvider } from '@clerk/nextjs'

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <ClerkProvider>
      <html lang="en" suppressHydrationWarning>
        {/* existing head content */}
        <body className={`${inter.variable} font-sans antialiased`}>
          <div className="flex flex-col min-h-screen">
            <Header />
            <div className="flex-1">{children}</div>
            <Footer />
          </div>
        </body>
      </html>
    </ClerkProvider>
  )
}
```

### Pattern 3: Dedicated Sign-In/Sign-Up Pages (catch-all routes)
**What:** Clerk's hosted components require the catch-all `[[...sign-in]]` route segment.
**When to use:** For dedicated pages — the catch-all handles Clerk's internal redirects.

```typescript
// Source: https://clerk.com/docs/nextjs/guides/development/custom-sign-in-or-up-page
// File: frontend/app/sign-in/[[...sign-in]]/page.tsx
import { SignIn } from '@clerk/nextjs'

export default function Page() {
  return <SignIn />
}

// File: frontend/app/sign-up/[[...sign-up]]/page.tsx
import { SignUp } from '@clerk/nextjs'

export default function Page() {
  return <SignUp />
}
```

Required environment variables for custom pages + redirect:
```env
NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY=pk_...
CLERK_SECRET_KEY=sk_...
NEXT_PUBLIC_CLERK_SIGN_IN_URL=/sign-in
NEXT_PUBLIC_CLERK_SIGN_UP_URL=/sign-up
NEXT_PUBLIC_CLERK_AFTER_SIGN_IN_URL=/dashboard
NEXT_PUBLIC_CLERK_AFTER_SIGN_UP_URL=/dashboard
```

### Pattern 4: Header with SignedIn/SignedOut
**What:** Conditional rendering based on auth state — no server calls needed.

```typescript
// Source: https://clerk.com/docs/nextjs/getting-started/quickstart
// File: frontend/components/header.tsx (updated)
import { SignedIn, SignedOut, UserButton } from '@clerk/nextjs'
import Link from 'next/link'

export function Header() {
  return (
    <header className="sticky top-0 z-50 bg-surface-primary border-b border-border-subtle">
      <nav aria-label="Main navigation">
        <div className="container mx-auto px-4 h-[var(--header-height)] flex items-center justify-between">
          {/* Logo — unchanged */}
          <Link href="/">...</Link>

          {/* Auth CTA — replaces 'Scan Now' */}
          <SignedOut>
            <Link
              href="/sign-in"
              className="px-4 py-2 bg-brand-primary hover:bg-brand-primary-hover text-text-inverse font-semibold rounded-lg transition text-sm sm:text-base"
            >
              Sign In
            </Link>
          </SignedOut>
          <SignedIn>
            <UserButton />
          </SignedIn>
        </div>
      </nav>
    </header>
  )
}
```

### Pattern 5: Protected Dashboard with currentUser()
**What:** Server component that reads user data and renders greeting.

```typescript
// Source: https://clerk.com/docs/references/nextjs/current-user
// File: frontend/app/dashboard/page.tsx
import { auth, currentUser } from '@clerk/nextjs/server'
import { redirect } from 'next/navigation'

export default async function DashboardPage() {
  const { userId } = await auth()
  if (!userId) redirect('/sign-in')

  const user = await currentUser()

  return (
    <main className="container mx-auto px-4 py-16 max-w-4xl">
      <h1 className="text-3xl font-bold mb-4">
        Welcome, {user?.firstName ?? 'there'}
      </h1>
      <a href="/verify-domain" className="...">
        Verify your domain
      </a>
    </main>
  )
}
```

Note: `auth.protect()` in proxy.ts handles the unauthenticated redirect. The `if (!userId) redirect(...)` inside the page is a belt-and-suspenders defence, not the primary mechanism.

### Pattern 6: Axum JWT Extractor with RemoteJwksDecoder
**What:** axum-jwt-auth's `RemoteJwksDecoder` fetches and caches Clerk's public keys; `Claims<ClerkClaims>` is used as an extractor in protected handlers.

```rust
// Source: https://docs.rs/axum-jwt-auth/latest/axum_jwt_auth/
// File: src/api/auth.rs (new)
use axum_jwt_auth::{Claims, RemoteJwksDecoder};
use jsonwebtoken::{Algorithm, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClerkClaims {
    pub sub: String,       // Clerk user ID (e.g., "user_2abc...")
    pub exp: usize,
    pub iat: usize,
    pub nbf: Option<usize>,
    pub azp: Option<String>, // authorized party (origin)
    pub sid: Option<String>, // session ID
}

// In main.rs / startup:
let jwks_url = std::env::var("CLERK_JWKS_URL")
    .expect("CLERK_JWKS_URL must be set");

let mut validation = Validation::new(Algorithm::RS256);
// Clerk JWTs: do not validate audience by default (Clerk doesn't set aud in default tokens)
validation.validate_aud = false;

let decoder = RemoteJwksDecoder::<ClerkClaims>::builder()
    .jwks_url(jwks_url)
    .validation(validation)
    .build()
    .expect("Failed to build JWKS decoder");

let decoder = Arc::new(decoder);
let _shutdown = decoder.initialize().await
    .expect("Failed to initialize JWKS decoder");

// Use in handler:
pub async fn get_scan(
    Claims(claims): Claims<ClerkClaims>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // claims.sub = the Clerk user ID
    // Return 401 automatically if JWT is missing/invalid
    ...
}
```

Note on `CLERK_JWKS_URL`: Use `https://api.clerk.com/v1/jwks` (Backend API endpoint) for production. For instance-specific URL: `https://<YOUR_CLERK_FRONTEND_API>/.well-known/jwks.json`.

### Pattern 7: Svix Webhook Verification
**What:** Three-line verification using the `svix` crate — no manual HMAC.

```rust
// Source: https://docs.svix.com/receiving/verifying-payloads/how
// Cargo.toml: svix = "1", http = "1.0"
use svix::webhooks::Webhook;
use axum::body::Bytes;
use axum::http::HeaderMap;

pub async fn handle_clerk_webhook(
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    let secret = std::env::var("CLERK_WEBHOOK_SIGNING_SECRET")
        .map_err(|_| ApiError::InternalError("CLERK_WEBHOOK_SIGNING_SECRET not set".into()))?;

    // Convert axum HeaderMap to http::HeaderMap for svix
    let mut http_headers = http::HeaderMap::new();
    for (name, value) in headers.iter() {
        http_headers.insert(name.clone(), value.clone());
    }

    let wh = Webhook::new(&secret)
        .map_err(|_| ApiError::InternalError("Invalid webhook secret format".into()))?;

    wh.verify(&body, &http_headers)
        .map_err(|_| {
            tracing::warn!("Clerk webhook signature verification failed");
            ApiError::ValidationError("Invalid webhook signature".into())
        })?;

    // Parse event
    let event: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| ApiError::ValidationError(format!("Invalid JSON: {}", e)))?;

    let event_type = event["type"].as_str().unwrap_or("");

    match event_type {
        "user.created" => {
            let clerk_user_id = event["data"]["id"].as_str()
                .ok_or_else(|| ApiError::ValidationError("Missing user id".into()))?;
            let email = event["data"]["email_addresses"][0]["email_address"].as_str()
                .unwrap_or("");
            // INSERT INTO users (clerk_user_id, email) ON CONFLICT DO NOTHING
            // ... db call ...
            tracing::info!(clerk_user_id = %clerk_user_id, "User created via webhook");
        }
        _ => {
            tracing::info!(event_type = %event_type, "Unhandled Clerk webhook event");
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
```

Svix header names required: `svix-id`, `svix-timestamp`, `svix-signature`.
Webhook secret format: starts with `whsec_` — store as-is in env var.

### Pattern 8: CORS Fix for Authorization Header
**What:** Add `axum::http::header::AUTHORIZATION` to the existing `allow_headers` in `main.rs`.

```rust
// Source: tower-http CorsLayer docs
// File: src/main.rs (updated cors setup)
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};

let cors = CorsLayer::new()
    .allow_origin(frontend_url)
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([CONTENT_TYPE, AUTHORIZATION]);  // Added AUTHORIZATION
```

Note: Cannot use wildcard headers (`Any`) when also needing to allow `Authorization`. Must list headers explicitly.

### Pattern 9: Nginx CVE-2025-29927 Mitigation
**What:** Strip `x-middleware-subrequest` header at the Nginx layer so spoofed headers never reach Next.js.

```nginx
# Source: https://www.picussecurity.com/resource/blog/cve-2025-29927-nextjs-middleware-bypass-vulnerability
# File: infrastructure/templates/shipsecure.nginx.conf.j2

# In the /api/ location block:
location /api/ {
    proxy_set_header x-middleware-subrequest "";
    # ... existing proxy_pass and other directives
}

# In the / (frontend) location block:
location / {
    proxy_set_header x-middleware-subrequest "";
    # ... existing proxy_pass and other directives
}
```

Setting the header to an empty string causes Nginx to strip it before forwarding. Must be in both `location /api/` and `location /` blocks since Next.js handles the `/` block.

### Anti-Patterns to Avoid

- **Using `middleware.ts` filename on Next.js 16:** Next.js 16 deprecates `middleware.ts` in favor of `proxy.ts`. The code inside is identical, only the filename changes. Missing this causes confusing Clerk errors.
- **Wildcard allow_headers for CORS:** `CorsLayer::any()` or `.allow_headers(Any)` does NOT cover the Authorization header. Must list AUTHORIZATION explicitly.
- **clerk-rs crate:** Explicitly rejected in prior decisions — stale (v0.4.1, 8+ months), not production-safe.
- **Per-request Clerk API calls for JWT verification:** RemoteJwksDecoder caches keys. Never call Clerk's backend API on every request — violates INFR-04.
- **Passing full currentUser() object to client components:** Clerk's `currentUser()` returns a large object. Pass only the fields needed (e.g., just `firstName`).
- **Missing `[[...sign-in]]` catch-all segment:** Using `sign-in/page.tsx` without the catch-all will break Clerk's OAuth callback flows.
- **Returning 401 with JWT details:** Decision requires generic "Authentication required" message — never expose reason (expired, invalid algorithm, etc.).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JWT verification with JWKS | Custom JWKS fetcher + key cache | axum-jwt-auth RemoteJwksDecoder | Key rotation, concurrent fetches, background refresh, RS256 with correct key ID matching — all edge cases |
| Webhook signature verification | Manual HMAC-SHA256 like Stripe handler | svix crate Webhook::verify() | Svix uses more complex signature scheme (multiple signatures, timestamp comparison, base64 key parsing) |
| Auth middleware for Next.js | Custom NextRequest inspection | clerkMiddleware + createRouteMatcher | Session token validation, redirect logic, OAuth callback handling |
| OAuth flows | Custom OAuth handlers | Clerk dashboard config | Google/GitHub OAuth registration, token exchange, refresh — Clerk handles entirely |
| Session persistence | Custom cookie management | Clerk automatic | Clerk stores session in HttpOnly cookies, handles rotation |

**Key insight:** Clerk's value is handling the entire auth surface (signup flow, email verification, OAuth, session cookies). Resist reimplementing any part of it.

---

## Common Pitfalls

### Pitfall 1: Wrong Middleware Filename (Next.js 16)
**What goes wrong:** File named `middleware.ts` works in dev but Clerk logs warnings; future Next.js 16 minor versions will break it.
**Why it happens:** The Next.js 16 migration renamed `middleware.ts` → `proxy.ts`. Clerk's own docs acknowledge this in the "Next.js ≤15" note.
**How to avoid:** Create file as `frontend/proxy.ts` from the start. Never create `middleware.ts`.
**Warning signs:** Console warning "middleware convention is deprecated" in Next.js 16 build output.

### Pitfall 2: CORS Wildcard Doesn't Cover Authorization
**What goes wrong:** Frontend gets CORS error on `Authorization: Bearer` header even though CORS "seems" configured.
**Why it happens:** The HTTP CORS spec excludes Authorization from the wildcard `*` header. Must be explicitly listed.
**How to avoid:** Use `allow_headers([CONTENT_TYPE, AUTHORIZATION])` with imported header constants from `axum::http::header`.
**Warning signs:** Browser console shows "Request header field Authorization is not allowed by Access-Control-Allow-Headers."

### Pitfall 3: axum-jwt-auth Decoder Not Initialized
**What goes wrong:** All JWT verifications fail silently or panic.
**Why it happens:** `RemoteJwksDecoder` must call `.initialize().await` before the server starts accepting requests. It fetches keys immediately and starts a background refresh task.
**How to avoid:** Call `decoder.initialize().await` in main() before binding the listener. Store the returned shutdown token.
**Warning signs:** All `Claims<ClerkClaims>` extractions return 401 even with valid JWTs.

### Pitfall 4: Svix HeaderMap Type Mismatch
**What goes wrong:** `wh.verify()` fails to compile — Axum uses `axum::http::HeaderMap`, svix expects `http::HeaderMap`.
**Why it happens:** Both are the same underlying type (`http` crate), but if versions don't align, they're different types.
**How to avoid:** Add `http = "1.0"` to Cargo.toml explicitly. The conversion loop `for (name, value) in headers.iter()` handles the copy.
**Warning signs:** Type mismatch compile error on the `verify()` call.

### Pitfall 5: Missing catch-all Route Segment
**What goes wrong:** Sign-in works on direct navigation but OAuth callbacks fail with 404.
**Why it happens:** Clerk's OAuth flow redirects to `/sign-in#/oauth-callback` or similar sub-paths. Without `[[...sign-in]]`, those 404.
**How to avoid:** Always use `app/sign-in/[[...sign-in]]/page.tsx` — the double brackets and triple dots are load-bearing.
**Warning signs:** OAuth (Google/GitHub) sign-in fails with 404 after authorization; email/password works fine.

### Pitfall 6: CVE-2025-29927 Only in One Location Block
**What goes wrong:** The header strip only covers `/api/` but not `/` — Next.js middleware bypass still possible via the frontend path.
**Why it happens:** The CVE affects the Next.js frontend, which is proxied via the `/` location block, not `/api/`.
**How to avoid:** Add `proxy_set_header x-middleware-subrequest ""` to BOTH the `/api/` and `/` location blocks in the Nginx template.
**Warning signs:** Security scanner reports CVE-2025-29927 still present after partial mitigation.

### Pitfall 7: Clerk validate_aud Mismatch
**What goes wrong:** JWT verification rejects all valid Clerk tokens.
**Why it happens:** Clerk's default session tokens do not include an `aud` claim. If `jsonwebtoken`'s `Validation` has `validate_aud = true` (the default), it rejects tokens without `aud`.
**How to avoid:** Set `validation.validate_aud = false` in the Validation struct before passing to the decoder builder.
**Warning signs:** All API calls return 401; Clerk JWT decodes fine in jwt.io but fails server-side.

### Pitfall 8: testProxy Config in next.config.ts
**What goes wrong:** Build warnings or runtime errors after renaming middleware to proxy.
**Why it happens:** The existing `next.config.ts` has `experimental: { testProxy: process.env.PLAYWRIGHT_TEST === '1' }`. This is the correct flag name for proxy.ts (not `testMiddleware`).
**How to avoid:** No action needed — the existing config is already using the correct flag name for Next.js 16.
**Warning signs:** None expected — just awareness that this config already aligns with proxy.ts.

---

## Code Examples

### Full axum-jwt-auth Setup in main.rs

```rust
// Source: https://docs.rs/axum-jwt-auth/latest/axum_jwt_auth/
// Cargo.toml additions:
//   axum-jwt-auth = "0.6"

use axum_jwt_auth::RemoteJwksDecoder;
use jsonwebtoken::{Algorithm, Validation};
use std::sync::Arc;

// In main() after database pool setup:
let jwks_url = std::env::var("CLERK_JWKS_URL")
    .expect("CLERK_JWKS_URL must be set");

let mut validation = Validation::new(Algorithm::RS256);
validation.validate_aud = false;  // Clerk doesn't set aud in default tokens

let decoder = RemoteJwksDecoder::<crate::api::auth::ClerkClaims>::builder()
    .jwks_url(jwks_url)
    .validation(validation)
    .build()
    .expect("Failed to build JWKS decoder");

let decoder = Arc::new(decoder);
let _jwks_shutdown = decoder.initialize().await
    .expect("Failed to initialize JWKS decoder");

// Add decoder to AppState
let state = AppState {
    pool: pool.clone(),
    orchestrator: orchestrator.clone(),
    health_cache,
    metrics_handle: metrics_handle.clone(),
    shutdown_token: shutdown_token.clone(),
    jwt_decoder: decoder,  // new field
};
```

### Svix Verification with Correct Header Names

```rust
// Source: https://docs.svix.com/receiving/verifying-payloads/how
// Required svix headers: svix-id, svix-timestamp, svix-signature
use svix::webhooks::Webhook;

// The secret value from CLERK_WEBHOOK_SIGNING_SECRET starts with "whsec_"
let wh = Webhook::new(&secret).unwrap();
// wh.verify() returns Ok(()) or Err — both are infallible from wrong headers
wh.verify(&body, &http_headers).map_err(|e| {
    tracing::warn!("Webhook verification failed: {:?}", e);
    ApiError::ValidationError("Invalid webhook signature".into())
})?;
```

### Users Table Migration

```sql
-- Source: Project pattern (existing migrations use this style)
-- File: migrations/20260217000001_create_users.sql
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    clerk_user_id TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_clerk_user_id ON users(clerk_user_id);
```

### New 401 ApiError Variant

```rust
// File: src/api/errors.rs (add to ApiError enum)
Unauthorized,

// In IntoResponse impl:
ApiError::Unauthorized => (
    "about:blank".to_string(),
    "Unauthorized".to_string(),
    StatusCode::UNAUTHORIZED,
    "Authentication required".to_string(),
),
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `middleware.ts` in Next.js | `proxy.ts` in Next.js 16 | Next.js 16.0 (late 2025) | Must use proxy.ts for this project; middleware.ts deprecated |
| Per-request Clerk API verification | Local JWKS-cached verification | Clerk guidance since 2024 | No network latency per request; tolerates Clerk API outages |
| `authMiddleware()` in @clerk/nextjs | `clerkMiddleware()` | @clerk/nextjs v5 (Core 2) | clerkMiddleware is the current API; authMiddleware removed in v6 |
| Clerk `NEXT_PUBLIC_CLERK_AFTER_SIGN_IN_URL` | Same env var, still valid | Clerk v6 | No change; still the correct redirect env var |

**Deprecated/outdated:**
- `authMiddleware()`: Removed in @clerk/nextjs v6 — use `clerkMiddleware()`.
- `middleware.ts` filename: Deprecated in Next.js 16 — use `proxy.ts`.
- `clerk-rs` Rust crate: Explicitly rejected — stale v0.4.1.

---

## Open Questions

1. **Clerk JWKS URL format for this instance**
   - What we know: Two options: `https://api.clerk.com/v1/jwks` (Backend API, requires `CLERK_SECRET_KEY`) or `https://<FRONTEND_API_URL>/.well-known/jwks.json` (instance-specific, no auth needed)
   - What's unclear: Whether the instance-specific URL format is known without a Clerk account. The Backend API endpoint requires passing `Authorization: Bearer <CLERK_SECRET_KEY>` when fetching JWKS.
   - Recommendation: Use `https://api.clerk.com/v1/jwks` with axum-jwt-auth configured to pass the `CLERK_SECRET_KEY` as a bearer token in the JWKS fetch, OR use the instance frontend API URL. Verify during implementation which axum-jwt-auth supports. The instance URL `https://clerk.<APP_DOMAIN>.com/.well-known/jwks.json` is the simpler option if no auth header is needed.

2. **AppState extension for jwt_decoder**
   - What we know: The existing `AppState` in `src/api/scans.rs` doesn't have a `jwt_decoder` field. `axum-jwt-auth` requires the decoder in state via `FromRef`.
   - What's unclear: Whether to add to the single `AppState` or use axum's `FromRef` derive with a separate state struct.
   - Recommendation: Add `jwt_decoder: Arc<RemoteJwksDecoder<ClerkClaims>>` to the existing `AppState` struct. Implement `FromRef<AppState>` for `Decoder<ClerkClaims>`. This avoids restructuring the existing state pattern.

3. **Clerk Dashboard OAuth provider setup**
   - What we know: AUTH-02 (Google) and AUTH-03 (GitHub) require OAuth providers to be enabled. This is a Clerk Dashboard configuration step, not code.
   - What's unclear: Whether the Clerk account/app exists yet, and whether OAuth apps (Google, GitHub) are registered.
   - Recommendation: Document as a prerequisite in 29-02 plan. The code integration is complete without it; OAuth buttons appear automatically once Clerk is connected.

---

## Sources

### Primary (HIGH confidence)
- Clerk official docs (clerk.com/docs) — clerkMiddleware, proxy.ts Next.js 16 support, custom sign-in pages, auth(), currentUser(), JWKS endpoints, webhook setup
- Next.js 16 upgrade guide (nextjs.org/docs/app/guides/upgrading/version-16) — middleware → proxy.ts rename confirmed with code examples
- docs.rs/axum-jwt-auth — RemoteJwksDecoder, Claims extractor, AppState pattern
- docs.svix.com — Webhook::new(), verify(), required header names, Cargo.toml dependency

### Secondary (MEDIUM confidence)
- picussecurity.com CVE-2025-29927 analysis — Nginx proxy_set_header mitigation for x-middleware-subrequest (verified: matches other sources)
- tower-http CORS docs (via search) — explicit AUTHORIZATION header required, wildcard doesn't cover it

### Tertiary (LOW confidence)
- Search results for axum-jwt-auth 0.6.3 version number (Jan 25, 2026 release) — cited from WebFetch of GitHub, should be verified with `cargo search axum-jwt-auth` during implementation
- Svix v1.85.0 version number — from docs.rs search, verify with `cargo search svix`

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified from official Clerk docs, axum-jwt-auth docs.rs, svix docs.rs
- Architecture: HIGH — all code patterns verified against official sources; Next.js 16 proxy.ts confirmed from official upgrade guide
- Pitfalls: HIGH for documented patterns (CORS wildcard, catch-all routes, Next.js 16 filename); MEDIUM for Clerk validate_aud behavior (from JWT spec + verified Clerk claims structure)

**Research date:** 2026-02-17
**Valid until:** 2026-03-17 (30 days — Clerk and axum-jwt-auth move fast; re-verify versions before implementation)
