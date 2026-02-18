# Phase 31: Results Gating - Research

**Researched:** 2026-02-17
**Domain:** Server-side API response gating (Axum/Rust) + Clerk-aware frontend teaser UI (Next.js 16 / React 19)
**Confidence:** HIGH

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| GATE-01 | API strips description/remediation from high/critical findings for anonymous scan tokens | Optional JWT extraction pattern in Axum; `clerk_user_id` must be added to `Scan` struct and `get_scan_by_token` query |
| GATE-02 | API returns `gated: true` flag and `owner_verified` field on results | `owner_verified` computed by comparing `scan.clerk_user_id` with authenticated user's `sub`; `gated` field added to each stripped finding |
| GATE-03 | Frontend renders teaser cards with lock overlay for gated findings | `AuthGate` Client Component reads `gated` field per-finding; lock overlay rendered when `gated === true` and user not `owner_verified` |
| GATE-04 | Teaser cards show severity and category but not details, with "Sign up free" CTA | `FindingAccordion` receives `gated` flag; when gated, expando body replaced with lock overlay; `useClerk().openSignUp()` triggers Clerk modal |
</phase_requirements>

---

## Summary

Phase 31 implements a freemium content gate: anonymous visitors see high/critical findings as teaser cards (severity + title visible, description/remediation stripped), while authenticated owners see everything. The gate is enforced at the API layer — the server strips fields before the response leaves, so `curl` bypasses are impossible. The frontend renders visual lock overlays purely based on the `gated` field the server sends, not by hiding client-side data.

The two implementation domains are cleanly separated. The Rust backend needs (a) optional JWT extraction from `Authorization: Bearer`, (b) `clerk_user_id` field added to the `Scan` model and `get_scan_by_token` DB query, and (c) conditional field stripping in `get_results_by_token`. The Next.js frontend needs a new `AuthGate` Client Component that reads `useAuth()` and `useClerk()` from Clerk and renders a lock overlay on individual findings with `gated: true`.

The key implementation subtlety on the backend is that `axum-jwt-auth`'s `Claims<T>` extractor is mandatory (fails with `MissingToken` when no `Authorization` header is present). Optional authentication requires manually extracting and decoding the `Authorization` header — using `Option<Claims<ClerkClaims>>` is NOT supported by the library's `FromRequestParts` implementation. The correct pattern is to manually extract the bearer token from the request parts and call the JWKS decoder directly, returning `None` if the header is absent or invalid.

**Primary recommendation:** Add `clerk_user_id: Option<String>` to the `Scan` struct, extend `get_scan_by_token` to SELECT it, manually extract the optional JWT in the handler to get `Option<String>` for the caller's user ID, compute `owner_verified = scan.clerk_user_id.as_deref() == authenticated_user_id.as_deref()`, and strip `description`/`remediation` from high/critical findings only when the caller is not the verified owner. On the frontend, use `useAuth()` + `useClerk()` in a `'use client'` `AuthGate` component wrapping each finding.

---

## Standard Stack

### Core (Backend — already in project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| axum | 0.8.8 | HTTP framework, handler signature | Already in use |
| axum-jwt-auth | 0.6.3 | JWKS-based JWT decoder | Already in use; `Decoder<ClerkClaims>` in `AppState` |
| sqlx | 0.8.6 | PostgreSQL query | Already in use; `get_scan_by_token` needs extension |
| serde_json | 1 | JSON response construction | Already in use for `json!()` macro |

### Core (Frontend — already in project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| @clerk/nextjs | ^6.37.5 | Auth state, sign-up modal | Already installed; `useAuth`, `useClerk` |
| next | 16.1.6 | App Router, Server Components | Already in use |
| react | 19.2.3 | Client Component rendering | Already in use |

### No New Dependencies Required
Both plans use only what is already installed. No `npm install` or `cargo add` needed.

---

## Architecture Patterns

### Recommended File Changes
```
src/
├── models/scan.rs          # Add clerk_user_id: Option<String>
├── db/scans.rs             # Extend get_scan_by_token SELECT list
└── api/results.rs          # Optional JWT extraction + gating logic

frontend/
├── lib/types.ts            # Add gated?: boolean to Finding; owner_verified to ScanResponse
├── components/
│   ├── auth-gate.tsx       # NEW: Client Component with lock overlay
│   └── finding-accordion.tsx  # Accept gated prop, delegate to AuthGate
└── app/results/[token]/page.tsx  # Pass owner_verified to ResultsDashboard
```

### Pattern 1: Optional JWT Extraction in Axum (WITHOUT axum-jwt-auth's Claims extractor)

**What:** `Claims<ClerkClaims>` fails the request when no `Authorization` header is present. Optional auth requires manually accessing the bearer token and calling the decoder.

**Why it matters:** The results endpoint must serve anonymous users (no auth header) and authenticated users (with auth header) from the same route. Using `Claims<ClerkClaims>` as a handler parameter would return 401 for all anonymous requests.

**Correct pattern:**

```rust
// Source: axum-jwt-auth 0.6.3 src/axum.rs — BearerTokenExtractor::extract_token
// Source: axum 0.8 FromRequestParts docs

use axum::extract::{Path, State};
use axum::http::header::AUTHORIZATION;
use axum::Json;
use axum_jwt_auth::Claims;
use crate::api::auth::ClerkClaims;
use crate::api::scans::AppState;

pub async fn get_results_by_token(
    State(state): State<AppState>,
    Path(token): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    // 1. Optionally extract Clerk user ID from Authorization header
    let authenticated_user_id: Option<String> = extract_optional_clerk_user(&state, &headers).await;

    // 2. Load scan and check clerk_user_id
    let scan = db::scans::get_scan_by_token(&state.pool, &token).await?
        .ok_or_else(|| ApiError::Custom { ... })?;

    // 3. owner_verified: true only if authenticated user matches scan.clerk_user_id
    let owner_verified = match (&authenticated_user_id, &scan.clerk_user_id) {
        (Some(caller), Some(owner)) => caller == owner,
        _ => false,
    };

    // 4. Gate high/critical findings for non-owners
    // ...
}

async fn extract_optional_clerk_user(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Option<String> {
    let auth_header = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let token = auth_header.strip_prefix("Bearer ")?.trim();
    if token.is_empty() { return None; }
    let token_data = state.jwt_decoder.decode(token).await.ok()?;
    Some(token_data.claims.sub)
}
```

**Key insight:** `state.jwt_decoder` is `Arc<dyn JwtDecoder<ClerkClaims>>`. Its `decode()` method is directly callable. No need for the axum extractor machinery for optional auth.

### Pattern 2: Scan Model Extension

**What:** Add `clerk_user_id: Option<String>` to `Scan` struct and include it in all SELECT queries.

```rust
// src/models/scan.rs
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Scan {
    // ...existing fields...
    pub clerk_user_id: Option<String>,  // ADD THIS
}
```

```sql
-- Extended SELECT in get_scan_by_token (src/db/scans.rs)
SELECT id, target_url, email, submitter_ip::text, request_id, status, score, results_token,
       expires_at::timestamp, detected_framework, detected_platform,
       stage_headers, stage_tls, stage_files, stage_secrets, stage_detection, stage_vibecode,
       tier, error_message, started_at::timestamp, completed_at::timestamp, created_at::timestamp,
       clerk_user_id  -- ADD THIS
FROM scans
WHERE results_token = $1 AND (expires_at IS NULL OR expires_at > NOW())
```

**CRITICAL:** All other `SELECT` queries that use `sqlx::query_as::<_, Scan>` must ALSO add `clerk_user_id` to their column list, or they will fail at runtime with a column count mismatch. Check `get_scan`, `claim_pending_scan`, and `create_scan` RETURNING clauses.

### Pattern 3: Gating Logic in JSON Construction

**What:** When `!owner_verified` AND finding severity is high or critical, replace `description` and `remediation` with `null` and add `"gated": true`.

```rust
// src/api/results.rs — findings_json construction
let findings_json: Vec<serde_json::Value> = findings
    .iter()
    .map(|f| {
        let is_gated = !owner_verified
            && matches!(f.severity, Severity::High | Severity::Critical);

        if is_gated {
            json!({
                "id": f.id,
                "title": f.title,
                "description": null,
                "severity": format!("{:?}", f.severity).to_lowercase(),
                "remediation": null,
                "scanner_name": f.scanner_name,
                "vibe_code": f.vibe_code,
                "gated": true,
            })
        } else {
            json!({
                "id": f.id,
                "title": f.title,
                "description": f.description,
                "severity": format!("{:?}", f.severity).to_lowercase(),
                "remediation": f.remediation,
                "scanner_name": f.scanner_name,
                "vibe_code": f.vibe_code,
                "gated": false,
            })
        }
    })
    .collect();

// Top-level response addition
let response = json!({
    // ...existing fields...
    "owner_verified": owner_verified,
    "findings": findings_json,
    // ...
});
```

### Pattern 4: AuthGate Client Component

**What:** A `'use client'` component that reads `useAuth()` and `useClerk()` to render either the real content (when not gated) or a lock overlay (when gated).

**Key Clerk APIs available in `@clerk/nextjs` 6.x:**
- `useAuth()` — returns `{ isLoaded, isSignedIn, userId, getToken }` — confirmed from `@clerk/clerk-react/dist/useAuth-fq1pQd_y.d.ts`
- `useClerk()` — returns `{ openSignUp, openSignIn, ... }` — confirmed from `@clerk/shared/dist/types/index.d.ts`
- `openSignUp(props?: SignUpModalProps): void` — opens Clerk's SignUp modal inline

```tsx
// frontend/components/auth-gate.tsx
'use client'

import { useClerk } from '@clerk/nextjs'

interface AuthGateProps {
  gated: boolean
  severity: string
  category: string
  children: React.ReactNode
}

export function AuthGate({ gated, severity, category, children }: AuthGateProps) {
  const { openSignUp } = useClerk()

  if (!gated) {
    return <>{children}</>
  }

  return (
    <div className="relative">
      {/* Blurred/obscured content behind overlay */}
      <div className="blur-sm select-none pointer-events-none" aria-hidden="true">
        {children}
      </div>
      {/* Lock overlay */}
      <div className="absolute inset-0 flex flex-col items-center justify-center
                      bg-surface-elevated/80 backdrop-blur-sm rounded-lg
                      border border-border-subtle">
        <div className="text-center px-4">
          <p className="text-sm font-medium text-text-secondary mb-1">
            {severity} finding — details locked
          </p>
          <p className="text-xs text-text-tertiary mb-3">
            {category}
          </p>
          <button
            onClick={() => openSignUp({})}
            className="px-4 py-2 bg-brand-primary text-white text-sm
                       font-semibold rounded-md hover:bg-brand-primary-hover
                       transition-colors"
          >
            Sign up free to view
          </button>
        </div>
      </div>
    </div>
  )
}
```

### Pattern 5: Results Page Passes `owner_verified` Down

**What:** The existing results page is a Server Component that fetches data. It should NOT forward the Clerk session token to the backend. Instead, the server-side fetch happens without the auth header (anonymous), and `owner_verified` will be `false` for everyone on a server-rendered first load.

**IMPORTANT DESIGN DECISION:** There are two possible approaches:

**Option A — Server sends token to backend (complex, full SSR):**
The Next.js Server Component uses `auth()` from `@clerk/nextjs/server` to get `getToken()`, then passes `Authorization: Bearer <token>` in the backend fetch. This gives accurate `owner_verified` on first paint but requires Next.js → Axum JWT forwarding.

**Option B — Client fetches with token (simpler, progressive enhancement):**
The Server Component renders without auth. A Client Component uses `useAuth()` to detect sign-in state and re-fetches with the token if signed in, or reads the `gated` flag from the already-rendered data for anonymous users.

**Recommended: Option A with server-side token forwarding.** The success criteria explicitly requires `curl` to return stripped fields and `gated: true`. This means the server ALWAYS gates for anonymous requests. Authenticated users browsing the results page need to send their token. Since the results page is a Server Component, the cleanest approach is:

1. Server Component calls `auth()` from Clerk → gets `getToken()`
2. Calls `getToken()` to get the current session JWT
3. Passes it in the `Authorization` header on the backend fetch
4. Backend returns `owner_verified: true` for the owner's request
5. Frontend renders full cards for owners, teaser cards for anon

```tsx
// app/results/[token]/page.tsx (Server Component pattern)
import { auth } from '@clerk/nextjs/server'

export default async function ResultsPage({ params }) {
  const { token } = await params
  const { getToken } = await auth()
  const sessionToken = await getToken()

  const headers: Record<string, string> = {}
  if (sessionToken) {
    headers['Authorization'] = `Bearer ${sessionToken}`
  }

  const res = await fetch(`${BACKEND_URL}/api/v1/results/${token}`, {
    cache: 'no-store',
    headers,
  })
  // ...
}
```

**CORS note:** The backend already allows `Authorization` header from `FRONTEND_URL`. Confirmed in `main.rs`:
```rust
.allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION]);
```

### Anti-Patterns to Avoid

- **Frontend-only gating:** Never render gated content from JS and just hide it with CSS/opacity. The server must strip the fields. Confirmed by success criterion 1: `curl` must return stripped fields.
- **Using `Claims<ClerkClaims>` as handler parameter for optional auth:** This will reject all anonymous requests with 401. Use the manual extraction pattern (Pattern 1 above).
- **Forgetting to update ALL Scan SELECT queries:** `sqlx::FromRow` maps by position and column name. Adding `clerk_user_id` to `Scan` struct but not to a SELECT that uses it will cause a runtime panic. Check `create_scan` RETURNING, `get_scan`, `claim_pending_scan`, and `get_scan_by_token`.
- **Reading gated content in AuthGate from DOM:** Do not use `dangerouslySetInnerHTML` or read props from a hidden DOM element. The server has already set `description: null` — the frontend should just not render those fields.
- **Showing lock overlay to authenticated owners:** `owner_verified` from the API response controls whether overlays show. Do NOT show overlays to authenticated users who own the scan.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JWT verification | Custom RS256 verification | `state.jwt_decoder.decode()` (axum-jwt-auth) | Already initialized with JWKS key rotation |
| Sign-up modal | Custom modal/overlay | `useClerk().openSignUp({})` | Clerk handles OAuth, email, magic link, CAPTCHA |
| Auth state detection | Custom session management | `useAuth()` from `@clerk/nextjs` | Returns `isLoaded`, `isSignedIn`, `userId` correctly |
| Authorization header parsing | Custom header splitting | `strip_prefix("Bearer ")` on the raw header value | Simple and already the pattern axum-jwt-auth uses internally |

**Key insight:** The JWT decoder is already wired into `AppState.jwt_decoder`. Call it directly. The Clerk sign-up UI is already part of the installed library. No new infrastructure is needed for either plan.

---

## Common Pitfalls

### Pitfall 1: Scan Struct Column Mismatch After Adding `clerk_user_id`

**What goes wrong:** Adding `clerk_user_id: Option<String>` to the `Scan` struct but not adding it to every `SELECT ... FROM scans` query causes sqlx to fail at runtime (not compile time with the runtime query API style used here). The process panics when fetching any scan via the un-updated query.

**Why it happens:** `sqlx::FromRow` maps columns by name. If the query returns columns that don't match the struct fields, or vice versa, it fails at the query execution point. The existing `create_scan` RETURNING clause, `claim_pending_scan` RETURNING clause, and `get_scan` SELECT all need `clerk_user_id` added.

**How to avoid:** After adding `clerk_user_id` to the struct, grep for all `query_as::<_, Scan>` usages and update every column list. The compiler will NOT catch this.

**Warning signs:** `sqlx::Error::ColumnNotFound("clerk_user_id")` at runtime, or mismatched row.

### Pitfall 2: Clerk `getToken()` Returns `null` When Session Is Absent

**What goes wrong:** `const sessionToken = await getToken()` returns `null` for unauthenticated users. Passing `null` as an Authorization header value will either fail or send `Authorization: Bearer null`.

**Why it happens:** Clerk's `getToken()` returns `string | null`. The code must check for null before setting the header.

**How to avoid:** Guard with `if (sessionToken) { headers['Authorization'] = \`Bearer ${sessionToken}\` }`. Only set the header when non-null.

**Warning signs:** Backend receiving `Authorization: Bearer null` → JWT decode failure → user treated as anonymous even though they're trying to view their own scan.

### Pitfall 3: `owner_verified` False Positive for Unauthenticated Users

**What goes wrong:** `owner_verified` is `true` when both `authenticated_user_id` and `scan.clerk_user_id` are `None`. An anonymous user viewing an anonymous scan (no `clerk_user_id`) would be treated as the owner and see unredacted results.

**Why it happens:** Naive equality check `None == None` is `true` in Rust.

**How to avoid:** Require BOTH sides to be `Some` for `owner_verified = true`:
```rust
let owner_verified = match (&authenticated_user_id, &scan.clerk_user_id) {
    (Some(caller), Some(owner)) => caller == owner,
    _ => false,  // None == None → false
};
```

**Warning signs:** Anonymous scans (scans with `clerk_user_id = NULL`) returning `owner_verified: true`.

### Pitfall 4: `useClerk()` and `useAuth()` Unavailable in Server Components

**What goes wrong:** Using `useClerk()` or `useAuth()` in a Server Component throws "React hooks can only be used in Client Components."

**Why it happens:** Hooks require the React client runtime. Server Components render on the server without a React context.

**How to avoid:** `AuthGate` MUST be a `'use client'` component. The `FindingAccordion` component (already `'use client'`) can import and use `AuthGate`. The results page Server Component does NOT use hooks — it uses `auth()` from `@clerk/nextjs/server` (a server-compatible function).

**Warning signs:** Build error "You're importing a component that needs `useAuth`. It only works in a Client Component."

### Pitfall 5: `gated` Field Missing in TypeScript Types Causes Implicit `undefined`

**What goes wrong:** The `Finding` interface in `lib/types.ts` doesn't include `gated?: boolean`. Code that checks `finding.gated` gets TypeScript's implicit `any` or a type error, and the runtime value is `undefined` (falsy) — which means gated findings would accidentally render without the overlay.

**Why it happens:** TypeScript will allow accessing undefined properties if the type is wrong or `any` is used. The `gated` field from the API would be ignored silently.

**How to avoid:** Add `gated?: boolean` to the `Finding` interface and `owner_verified: boolean` to `ScanResponse`. Make the AuthGate check `finding.gated === true` (strict equality).

### Pitfall 6: Download Endpoint Leaks Gated Content

**What goes wrong:** `GET /api/v1/results/:token/download` renders the full markdown report including description/remediation for all findings, regardless of auth state.

**Why it happens:** `download_results_markdown` in `src/api/results.rs` doesn't have the gating logic.

**How to avoid:** Apply the same optional JWT extraction + gating logic to `download_results_markdown`. Alternatively (simpler), gate the download button entirely — only show it when `owner_verified: true`. The download handler should apply the same `owner_verified` check.

**Phase 31 scope note:** The requirements (GATE-01 through GATE-04) are specifically about `GET /api/v1/results/:token`. Whether `download` also needs gating is an open question — see Open Questions.

---

## Code Examples

Verified patterns from official sources:

### Optional Bearer Token Extraction (Axum + axum-jwt-auth internals)

```rust
// Based on: axum-jwt-auth 0.6.3 src/axum.rs BearerTokenExtractor + JwtDecoder::decode
// Source: /home/john/.cargo/registry/src/.../axum-jwt-auth-0.6.3/src/axum.rs

use axum::http::HeaderMap;
use axum::http::header::AUTHORIZATION;

async fn extract_optional_clerk_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Option<String> {
    // 1. Get Authorization header value
    let auth_value = headers.get(AUTHORIZATION)?.to_str().ok()?;

    // 2. Strip "Bearer " prefix (case-sensitive, matching HTTP convention)
    let token = auth_value.strip_prefix("Bearer ")?.trim();
    if token.is_empty() {
        return None;
    }

    // 3. Decode JWT using the existing JWKS decoder in AppState
    //    Returns None on any error (expired, invalid sig, network, etc.)
    let token_data = state.jwt_decoder.decode(token).await.ok()?;

    // 4. Return the Clerk user ID (sub claim)
    Some(token_data.claims.sub)
}
```

### Clerk Sign-Up Modal (useClerk)

```tsx
// Source: @clerk/shared/dist/types/index.d.ts — openSignUp: (props?: SignUpModalProps) => void
// Source: @clerk/shared/dist/runtime/react/index.d.ts — useClerk hook

'use client'
import { useClerk } from '@clerk/nextjs'

function SignUpButton() {
  const { openSignUp } = useClerk()
  return (
    <button onClick={() => openSignUp({})}>
      Sign up free
    </button>
  )
}
```

### Server Component Auth Token Forwarding

```tsx
// Source: @clerk/nextjs/dist/types/app-router/server/auth.d.ts
// auth() returns SessionAuthWithRedirect which includes getToken()

import { auth } from '@clerk/nextjs/server'

export default async function ResultsPage() {
  const { getToken } = await auth()
  const sessionToken = await getToken()

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  }
  if (sessionToken) {
    headers['Authorization'] = `Bearer ${sessionToken}`
  }

  const res = await fetch(`${BACKEND_URL}/api/v1/results/${token}`, {
    cache: 'no-store',
    headers,
  })
  // ...
}
```

### TypeScript Types Update

```typescript
// frontend/lib/types.ts

export interface Finding {
  id: string
  title: string
  description: string | null   // null when gated
  severity: 'critical' | 'high' | 'medium' | 'low'
  remediation: string | null   // null when gated
  scanner_name: string
  vibe_code: boolean
  gated?: boolean              // NEW: true when description/remediation stripped
}

export interface ScanResponse {
  // ...existing fields...
  owner_verified: boolean      // NEW: true when authenticated caller is the scan owner
  findings: Finding[]
  // ...
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Frontend-only content hiding (opacity:0, display:none) | Server-side field stripping | Always been wrong | `curl` cannot bypass server-side stripping |
| Manual JWT refresh / custom session store | Clerk managed sessions with JWKS | Phase 29 | JWKS decoder already in AppState; just call `decode()` |
| Clerk `<SignIn>` full page redirect | `useClerk().openSignUp()` modal | Clerk v5+ | Keeps user on results page, lower friction |
| `auth()` returning synchronous value | `auth()` is `async` in `@clerk/nextjs` 6.x | Clerk v6 | Must `await auth()` in Server Components |

**Deprecated/outdated:**
- `withAuth()` HOC: replaced by `auth()` server function and `useAuth()` hook in Clerk v6
- `getServerSideProps` with Clerk: replaced by Server Components + `auth()` in Next.js App Router

---

## Open Questions

1. **Should `download_results_markdown` also apply gating?**
   - What we know: The download endpoint renders full markdown including description/remediation for all findings. It is reachable by curl: `GET /api/v1/results/:token/download`
   - What's unclear: GATE-01 says "API strips description/remediation" — does this include the download endpoint?
   - Recommendation: Apply the same optional JWT extraction + gating logic to the download handler to be consistent. Failing that, hide the download button when `owner_verified === false` on the frontend. The plan for 31-01 should clarify.

2. **Behavior for `tier = 'authenticated'` scans with no `clerk_user_id`**
   - What we know: Phase 30 added `clerk_user_id` as nullable with an `authenticated` tier. There may be scans where `tier = 'free'` and `clerk_user_id IS NULL`.
   - What's unclear: Should gating apply to ALL scans regardless of tier, or only free scans?
   - Recommendation: Gate based on the finding's severity and the caller's identity only. Tier is irrelevant to gating logic. If `scan.clerk_user_id IS NULL` (old free scans), `owner_verified = false` for everyone, so high/critical are always gated. This is consistent and requires no tier branching.

3. **`auth()` availability in Server Component without `clerkMiddleware` matchers**
   - What we know: `proxy.ts` runs `clerkMiddleware` on all non-static routes (including `/results/[token]`). The middleware does NOT call `auth.protect()` for results routes. `auth()` in a Server Component should still return the session state if the middleware ran.
   - What's unclear: Whether `auth()` returns a populated session when `clerkMiddleware` doesn't call `auth.protect()` for that route.
   - Recommendation: HIGH confidence this works — `clerkMiddleware` populates the session context for all matched routes even without `protect()`. The matcher in `proxy.ts` covers `/((?!_next|...).*)`  which includes `/results/*`. The `auth()` server function reads from that context.

---

## Sources

### Primary (HIGH confidence)
- axum-jwt-auth 0.6.3 source: `/home/john/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/axum-jwt-auth-0.6.3/src/axum.rs` — `Claims<T>` is mandatory (MissingToken when absent); `JwtDecoder::decode()` is directly callable
- axum-jwt-auth 0.6.3 source: `/home/john/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/axum-jwt-auth-0.6.3/src/lib.rs` — `Decoder<T> = Arc<dyn JwtDecoder<T> + Send + Sync>` with `decode()` method
- Project source: `src/api/auth.rs` — `ClerkUser`, `ClerkClaims`, `FromRef<AppState> for Decoder<ClerkClaims>`
- Project source: `src/api/results.rs` — current `get_results_by_token` handler (no auth)
- Project source: `src/db/scans.rs` — `get_scan_by_token` query (no `clerk_user_id` in SELECT)
- Project source: `src/models/scan.rs` — `Scan` struct (no `clerk_user_id` field yet)
- Project source: `migrations/20260218000001_stripe_removal_schema.sql` — Phase 30 added `clerk_user_id TEXT` to scans
- Project source: `frontend/node_modules/@clerk/clerk-react/dist/useAuth-fq1pQd_y.d.ts` — `useAuth()` returns `UseAuthReturn` with `isLoaded`, `isSignedIn`, `userId`, `getToken`
- Project source: `frontend/node_modules/@clerk/shared/dist/types/index.d.ts` — `openSignUp: (props?: SignUpModalProps) => void` on Clerk object
- Project source: `frontend/node_modules/@clerk/nextjs/dist/types/app-router/server/auth.d.ts` — `auth()` returns `SessionAuthWithRedirect` with `getToken()`
- Project source: `frontend/proxy.ts` — `clerkMiddleware` covers all routes; results page is not `protect()`ed
- Project source: `src/main.rs` — CORS `allow_headers` includes `AUTHORIZATION`

### Secondary (MEDIUM confidence)
- Clerk Next.js 6 App Router pattern: `auth()` is async and available in Server Components; `useClerk().openSignUp()` opens the SignUp modal without full page navigation

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries are already installed; versions confirmed from source files and package.json
- Architecture: HIGH — ClerkClaims, AppState.jwt_decoder, and get_scan_by_token all verified from source; optional extraction pattern verified from axum-jwt-auth internals
- Pitfalls: HIGH — Pitfall 1 (scan struct mismatch) and Pitfall 3 (None == None) are verified from Rust semantics and sqlx behavior; Pitfall 2 (getToken null) verified from Clerk type definitions

**Research date:** 2026-02-17
**Valid until:** 2026-03-17 (stable libraries; Clerk minor versions may change but pattern is stable)
