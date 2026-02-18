# Phase 32: Domain Verification - Research

**Researched:** 2026-02-18
**Domain:** Domain ownership verification via HTML meta tag — Rust/Axum backend (PostgreSQL migration, reqwest+scraper fetch, JWT-authenticated endpoints) + Next.js 16 / React 19 wizard UI
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Verification wizard flow**
- Dedicated `/verify-domain` page with step-by-step wizard — not a modal or inline section
- Manual "Verify now" button after user places the meta tag — no auto-polling
- One domain at a time — after success, return to dashboard; no "verify another" in the same session
- If user enters an already-verified (non-expired) domain, show existing status ("This domain is already verified. Expires in X days.") — no new token issued

**Blocked domain messaging**
- Allow subdomain verification on shared-hosting TLDs. Block only the root TLD itself (e.g., `vercel.app` blocked, `myapp.vercel.app` allowed)
- Confirmed blocklist roots: github.io, vercel.app, netlify.app, pages.dev

**Verification status display**
- Small inline badge next to domain name: green "Verified" / yellow "Pending" / red "Expired" pill — compact
- Proactive 7-day expiry warning — badge changes to yellow warning state when within 7 days of expiry
- Verified domains list lives as a section within the main dashboard page (not a separate `/dashboard/domains` route)
- **Expired domain re-gates past results:** When a domain's verification expires, past scan results for that domain become gated again. `owner_verified` must also check domain verification status, not just user identity match.

**Meta tag snippet experience**
- Dark code block with one-click copy button showing the full `<meta>` tag
- Optional "Test my tag" pre-check button — verifies the tag is live without consuming the verification attempt
- Specific failure diagnosis on verification failure: tell the user exactly what happened (e.g., "We fetched your page but didn't find the meta tag. Check that it's in `<head>`, not `<body>`.")
- Opaque cryptographically random token — no encoded information, no leakage

### Claude's Discretion
- Error copy tone and wording for blocked root TLDs
- Validation layer choice (frontend-only, backend-only, or both) for TLD blocklist
- Blocklist storage approach (hardcoded constant vs config/env)
- Exact badge styling and color tokens
- Code block framework (syntax highlighting, line numbers, etc.)
- Token length and generation method
- Dashboard section layout relative to scan history

### Deferred Ideas (OUT OF SCOPE)
- DNS TXT verification (explicitly deferred to post-v1.6)
- "Verify another" flow in the same session
- Separate `/dashboard/domains` route
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| DOMN-01 | User can add a domain and receive a unique verification token | `POST /api/v1/domains/verify-start`: validates domain, checks blocklist (root-only), inserts into `verified_domains` table with status `pending`, returns opaque random token |
| DOMN-02 | User can verify domain ownership via HTML meta tag | `POST /api/v1/domains/verify-confirm`: fetches target URL with existing reqwest client, parses `<head>` with scraper `Selector::parse("meta[name='shipsecure-verification']")`, checks `content` attribute matches stored token |
| DOMN-03 | Verified domain displays green badge in dashboard | Frontend dashboard section reads `verified_domains` list from new `GET /api/v1/domains` endpoint; badge color driven by design token `success-*` (green), `caution-*` (yellow 7-day warning), `danger-*` (red expired) |
| DOMN-04 | System blocks verification of shared hosting TLDs (github.io, vercel.app, netlify.app, pages.dev) | Root-only check: parse registered domain (eTLD+1) and compare against hardcoded blocklist; subdomains like `myapp.vercel.app` pass |
| DOMN-05 | Domain verification expires after 30 days requiring re-verification | `verified_at + 30 days = expires_at` stored in DB; `get_results_by_token` checks `verified_domains` for the scan's target domain to extend `owner_verified` logic; badge shows yellow warning when `expires_at < NOW() + 7 days` |
</phase_requirements>

---

## Summary

Phase 32 adds domain ownership verification to the system. Authenticated users prove they control a domain's HTML output by placing a `<meta name="shipsecure-verification" content="<token>">` tag and clicking "Verify now". The backend fetches the target URL using the existing reqwest client, parses the HTML with the existing scraper crate, and confirms the token in the `<head>`. Verified domains are stored in a new `verified_domains` PostgreSQL table with a 30-day TTL.

The most significant architectural decision is **extending the `owner_verified` logic in `results.rs`**. Currently, `owner_verified` is true only when the JWT caller's `sub` matches `scan.clerk_user_id`. Phase 32 adds a second gate: the scan's target domain must also appear in `verified_domains` for that user and not be expired. This means the `get_results_by_token` handler gains a domain verification lookup, and an expired domain re-gates results even for the correct authenticated owner.

The frontend is a dedicated `/verify-domain` page with a three-step wizard: (1) domain input with normalization display, (2) meta tag snippet with copy button and optional "Test my tag" pre-check, (3) verify button that calls `verify-confirm` and shows pass/fail with specific diagnosis. The dashboard gains a "Verified Domains" section above or beside scan history, with compact badge pills. No new npm packages are required; the existing `@clerk/nextjs`, `zod`, and CSS design tokens are sufficient. The backend requires one new migration file and one new `src/api/domains.rs` module.

**Primary recommendation:** Backend: new `verified_domains` table migration + `src/api/domains.rs` with three endpoints (verify-start, verify-confirm, list) using the existing reqwest/scraper pattern from `detector.rs`. Frontend: `/verify-domain/page.tsx` wizard as a `'use client'` component using `useState` for step progression + dashboard section in `dashboard/page.tsx`. The TLD blocklist lives as a hardcoded `const` in `domains.rs` — no env config needed for four known values.

---

## Standard Stack

### Core (Backend — already in project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| axum | 0.8.8 | HTTP routing, handler signatures | Already in use; `State`, `Json`, `Extension` extractors |
| axum-jwt-auth | 0.6.3 | JWT verification for authenticated endpoints | Already in `AppState`; `Claims<ClerkClaims>` is mandatory auth |
| sqlx | 0.8.6 | PostgreSQL migration + queries | Already in use; `query_as`, `query`, migrations system |
| reqwest | 0.13.1 | HTTP fetch for verify-confirm | Already in use in `detector.rs` and `vibecode.rs` scanners |
| scraper | 0.22 | HTML parsing, CSS selector for meta tag | Already in use in `detector.rs` for `Selector::parse` |
| rand | 0.8.5 | Cryptographically random token generation | Already in use in `worker_pool.rs` for results tokens |
| base64 | 0.22.1 | URL-safe base64 encoding of token bytes | Already in use in `worker_pool.rs` |
| url | 2 | URL parsing for domain extraction | Already in use in `ssrf/validator.rs` |
| chrono | 0.4.43 | 30-day TTL, expiry calculation | Already in use |
| uuid | 1 | Primary key for `verified_domains` | Already in use |

### Core (Frontend — already in project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| @clerk/nextjs | ^6.37.5 | JWT token forwarding, `auth()` on server, `useAuth()` on client | Already installed |
| zod | ^4.3.6 | Domain input validation schema | Already installed |
| next | 16.1.6 | App Router, `'use client'` components | Already in use |
| lucide-react | ^0.563.0 | Icons (copy icon, check, warning) | Already installed |

### No New Dependencies Required

Both plans (`32-01` and `32-02`) use only what is already installed. No `cargo add` or `npm install` required.

---

## Architecture Patterns

### Recommended File Changes

```
Backend:
migrations/
└── 20260218000002_create_verified_domains.sql  # NEW

src/
├── api/
│   ├── mod.rs              # Add pub mod domains;
│   ├── domains.rs          # NEW: verify-start, verify-confirm, list endpoints
│   └── results.rs          # Extend owner_verified: add domain check
├── db/
│   ├── mod.rs              # Add pub mod domains;
│   └── domains.rs          # NEW: DB query functions for verified_domains
├── models/
│   ├── mod.rs              # Add pub mod domain; (optional, or inline in api/domains.rs)
│   └── domain.rs           # NEW: VerifiedDomain struct
└── main.rs                 # Register new routes

Frontend:
frontend/
├── app/
│   ├── dashboard/page.tsx  # Add verified domains section
│   └── verify-domain/
│       └── page.tsx        # NEW: wizard page
├── components/
│   ├── domain-badge.tsx    # NEW: Verified/Pending/Expired pill
│   └── meta-tag-snippet.tsx  # NEW: dark code block with copy button
└── lib/
    ├── api.ts              # Add domainApi functions
    └── types.ts            # Add VerifiedDomain, DomainVerifyStartResponse interfaces
```

### Pattern 1: verified_domains Table Migration

**What:** Single new table with the required columns. No FKs to the `scans` table — the domain check at results time is a join/lookup by domain name against `clerk_user_id`.

```sql
-- migrations/20260218000002_create_verified_domains.sql
CREATE TABLE IF NOT EXISTS verified_domains (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    clerk_user_id TEXT NOT NULL REFERENCES users(clerk_user_id) ON DELETE CASCADE,
    domain TEXT NOT NULL,
    verification_token TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'verified')),
    verified_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_verified_domains_clerk_domain
    ON verified_domains(clerk_user_id, domain);

CREATE INDEX idx_verified_domains_domain
    ON verified_domains(domain);

CREATE INDEX idx_verified_domains_token
    ON verified_domains(verification_token);
```

**CRITICAL design note:** The `UNIQUE (clerk_user_id, domain)` index means only one verification record per user per domain. When a user calls `verify-start` for an already-pending domain, `ON CONFLICT DO UPDATE` refreshes the token. When the domain is already verified and non-expired, return the existing status — no new token.

### Pattern 2: Mandatory JWT Auth for Domain Endpoints

**What:** Unlike `results.rs` which uses optional auth, domain endpoints require authentication. Use `Claims<ClerkClaims>` as a handler parameter — this causes axum-jwt-auth to return 401 automatically when no valid token is present.

```rust
// src/api/domains.rs
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_jwt_auth::Claims;

use crate::api::auth::ClerkClaims;
use crate::api::scans::AppState;
use crate::api::errors::ApiError;

/// POST /api/v1/domains/verify-start
pub async fn verify_start(
    State(state): State<AppState>,
    Claims(claims): Claims<ClerkClaims>,
    Json(req): Json<VerifyStartRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let clerk_user_id = &claims.sub;
    // ...
}
```

**Key insight:** `Claims<ClerkClaims>` is the mandatory extractor from axum-jwt-auth. It will reject unauthenticated requests with 401 automatically — no manual check needed. This is the opposite of the optional pattern used in `results.rs`.

### Pattern 3: Domain Normalization and Blocklist Check

**What:** Extract the registered domain (eTLD+1 equivalent) from user input. Block only exact root TLD matches from the blocklist, allow subdomains.

```rust
use url::Url;

const BLOCKED_ROOT_TLDS: &[&str] = &[
    "github.io",
    "vercel.app",
    "netlify.app",
    "pages.dev",
];

/// Normalize user input to a domain (strip scheme, path, trailing slashes).
/// Input: "https://myapp.vercel.app/path" → "myapp.vercel.app"
/// Input: "myapp.vercel.app" → "myapp.vercel.app"
fn normalize_domain(input: &str) -> Result<String, ApiError> {
    let input = input.trim();

    // Try parsing as URL first (handles https:// prefix)
    let domain = if input.contains("://") {
        let parsed = Url::parse(input)
            .map_err(|_| ApiError::ValidationError("Invalid domain format".to_string()))?;
        parsed.host_str()
            .ok_or_else(|| ApiError::ValidationError("No host in URL".to_string()))?
            .to_string()
    } else {
        // Treat as bare domain — prepend https:// to use Url parser for validation
        let parsed = Url::parse(&format!("https://{}", input))
            .map_err(|_| ApiError::ValidationError("Invalid domain format".to_string()))?;
        parsed.host_str()
            .ok_or_else(|| ApiError::ValidationError("No host in URL".to_string()))?
            .to_string()
    };

    // Strip www. prefix if present
    let domain = domain.strip_prefix("www.").unwrap_or(&domain).to_string();

    Ok(domain)
}

/// Check if the domain is a blocked root TLD.
/// Block: "vercel.app" (exact match)
/// Allow: "myapp.vercel.app" (subdomain)
fn is_blocked_root_tld(domain: &str) -> bool {
    BLOCKED_ROOT_TLDS.iter().any(|blocked| domain == *blocked)
}
```

**Why hardcoded constant (Claude's discretion):** The blocklist is four values unlikely to change often. A hardcoded `const` in `domains.rs` is simpler than env config, reduces operational surface area, and is the standard approach for small static lists. If new platforms emerge, it's a one-line code change with a deploy.

**Validation layer (Claude's discretion):** Both frontend and backend. Frontend: prevent premature API calls and give instant feedback. Backend: authoritative check that can't be bypassed. The frontend can do a simple suffix check; the backend does the canonical normalization+check.

### Pattern 4: Token Generation (reuse existing pattern)

**What:** Generate an opaque 32-byte cryptographically random token, encode as URL-safe base64. This is identical to `generate_results_token()` in `worker_pool.rs`.

```rust
fn generate_verification_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
}
```

This produces a 43-character URL-safe base64 string. 32 bytes = 256 bits of entropy — sufficient for a verification token. No JWT encoding, no timestamps embedded — fully opaque.

### Pattern 5: Meta Tag Fetch and Parse (reuse existing scraper pattern)

**What:** Fetch the target URL and parse the `<head>` for a specific meta tag using the scraper crate. The existing `detector.rs` code demonstrates exactly this pattern.

```rust
use scraper::{Html, Selector};
use reqwest::Client;
use std::time::Duration;

#[derive(Debug)]
pub enum VerificationFailureReason {
    FetchFailed(String),
    TagNotFound,
    TagInBody,      // Found but not in <head>
    WrongContent,   // Found in <head> but content doesn't match token
}

async fn fetch_and_check_meta_tag(
    target_url: &str,
    expected_token: &str,
) -> Result<(), VerificationFailureReason> {
    // Build reqwest client — same pattern as detector.rs
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(5))
        .user_agent("ShipSecure-Verifier/1.0")
        .build()
        .map_err(|e| VerificationFailureReason::FetchFailed(e.to_string()))?;

    let response = client
        .get(target_url)
        .send()
        .await
        .map_err(|e| VerificationFailureReason::FetchFailed(e.to_string()))?;

    let html = response.text().await
        .map_err(|e| VerificationFailureReason::FetchFailed(e.to_string()))?;

    let document = Html::parse_document(&html);

    // Select <meta name="shipsecure-verification"> anywhere in document
    let selector = Selector::parse("meta[name='shipsecure-verification']")
        .expect("valid CSS selector");

    let meta_element = document.select(&selector).next();

    match meta_element {
        None => Err(VerificationFailureReason::TagNotFound),
        Some(el) => {
            // Check content attribute matches token
            let content = el.value().attr("content").unwrap_or("");
            if content == expected_token {
                // Optionally verify it's in <head> (not body) — see Pitfall 3
                Ok(())
            } else {
                Err(VerificationFailureReason::WrongContent)
            }
        }
    }
}
```

**SSRF note:** The verify-confirm endpoint fetches a user-supplied URL. It MUST pass the URL through `ssrf::validate_scan_target()` first, exactly as `create_scan` does. This is already in the codebase at `src/ssrf/validator.rs`.

**"Test my tag" pre-check:** The verify-start response includes the token. A separate `POST /api/v1/domains/verify-check` endpoint (or inline in verify-start?) performs the fetch without committing verified status. Simpler implementation: re-use the same fetch-and-check logic, just don't write to the DB. The frontend calls this first when user clicks "Test my tag", then calls verify-confirm when user clicks "Verify now".

### Pattern 6: Extended owner_verified Logic in results.rs

**What:** The current `owner_verified` check in `get_results_by_token` only compares `authenticated_user_id` against `scan.clerk_user_id`. Phase 32 adds: if the scan has a `clerk_user_id` match (user is the owner), ALSO check that the scan's target domain has an active (non-expired) verification in `verified_domains` for that user.

```rust
// src/api/results.rs — extended owner_verified logic

// Phase 31: identity match
let identity_match = match (&authenticated_user_id, &scan.clerk_user_id) {
    (Some(caller), Some(owner)) => caller == owner,
    _ => false,
};

// Phase 32: domain verification check (only if identity matches)
let owner_verified = if identity_match {
    // Extract the domain from scan.target_url
    let domain = extract_domain(&scan.target_url);
    // Check verified_domains table
    if let (Some(domain), Some(user_id)) = (domain, &authenticated_user_id) {
        db::domains::is_domain_verified(&state.pool, user_id, &domain).await
            .unwrap_or(false)
    } else {
        false
    }
} else {
    false
};
```

**CRITICAL:** This change means a user who owns the scan but has an EXPIRED domain verification will get `owner_verified: false` and their high/critical findings will be re-gated. This is the intended behavior from the context decisions.

### Pattern 7: Frontend Wizard with useState Step Progression

**What:** The `/verify-domain` page is a `'use client'` component (needs Clerk auth + interactive state). Three steps using `useState<'input' | 'snippet' | 'verifying' | 'success' | 'failed'>`.

```tsx
// frontend/app/verify-domain/page.tsx
'use client'

import { useState } from 'react'
import { useAuth } from '@clerk/nextjs'
import { useRouter } from 'next/navigation'

type WizardStep = 'input' | 'snippet' | 'verifying' | 'success' | 'failed'

interface VerifyStartResponse {
  domain: string
  token: string
  meta_tag: string  // Full <meta> tag string for display
  already_verified?: boolean
  expires_in_days?: number
}

export default function VerifyDomainPage() {
  const { getToken } = useAuth()
  const router = useRouter()
  const [step, setStep] = useState<WizardStep>('input')
  const [domain, setDomain] = useState('')
  const [verifyData, setVerifyData] = useState<VerifyStartResponse | null>(null)
  const [failureReason, setFailureReason] = useState<string | null>(null)

  // Step 1: call verify-start
  // Step 2: show meta tag snippet + "Test my tag" + "Verify now"
  // Step 3: call verify-confirm → success or failure with diagnosis
  // ...
}
```

**Auth pattern:** `/verify-domain` must be added to the `isProtectedRoute` matcher in `proxy.ts`. The dashboard route is already protected (`/dashboard(.*)`). Adding `/verify-domain(.*)` ensures unauthenticated users are redirected to sign-in.

```ts
// frontend/proxy.ts — update
const isProtectedRoute = createRouteMatcher([
  '/dashboard(.*)',
  '/verify-domain(.*)',
])
```

### Pattern 8: Dashboard Verified Domains Section

**What:** The dashboard `page.tsx` (currently a Server Component) fetches verified domains from `GET /api/v1/domains` with the session token forwarded. Renders a list with badge pills.

```tsx
// frontend/app/dashboard/page.tsx (add fetch call)
import { auth } from '@clerk/nextjs/server'

export default async function DashboardPage() {
  const { userId, getToken } = await auth()
  if (!userId) redirect('/sign-in')

  const sessionToken = await getToken()
  const headers: Record<string, string> = {}
  if (sessionToken) {
    headers['Authorization'] = `Bearer ${sessionToken}`
  }

  // Fetch verified domains
  const domainsRes = await fetch(`${BACKEND_URL}/api/v1/domains`, {
    cache: 'no-store',
    headers,
  })
  const domains: VerifiedDomain[] = domainsRes.ok ? await domainsRes.json() : []

  // ... render domain list with badges
}
```

**Badge design (Claude's discretion using existing tokens):**
- Verified, >7 days: `bg-success-bg text-success-text border-success-border` — green pill
- Verified, ≤7 days: `bg-caution-bg text-caution-text border-caution-border` — yellow pill (expiry warning)
- Expired: `bg-danger-bg text-danger-text border-danger-border` — red pill
- Pending: `bg-info-bg text-info-text border-info-border` — blue pill

### Pattern 9: Meta Tag Snippet Copy Button

**What:** Dark code block with one-click copy. Use `navigator.clipboard.writeText()` — no library needed.

```tsx
// frontend/components/meta-tag-snippet.tsx
'use client'

import { useState } from 'react'
import { Copy, Check } from 'lucide-react'

interface MetaTagSnippetProps {
  metaTag: string
}

export function MetaTagSnippet({ metaTag }: MetaTagSnippetProps) {
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    await navigator.clipboard.writeText(metaTag)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="relative bg-gray-900 rounded-lg p-4 font-mono text-sm text-green-400">
      <pre className="overflow-x-auto whitespace-pre-wrap break-all pr-10">
        {metaTag}
      </pre>
      <button
        onClick={handleCopy}
        className="absolute top-3 right-3 p-1.5 rounded hover:bg-gray-700 transition-colors"
        aria-label="Copy meta tag"
      >
        {copied
          ? <Check className="w-4 h-4 text-green-400" />
          : <Copy className="w-4 h-4 text-gray-400" />
        }
      </button>
    </div>
  )
}
```

**No syntax highlighting library needed.** The meta tag is a single line — a `<pre>` with appropriate dark background styling is sufficient. Lucide's `Copy` and `Check` icons are already installed.

### Anti-Patterns to Avoid

- **Skipping SSRF validation on verify-confirm URL:** The `target_url` sent by the client during verify-confirm must be validated through `ssrf::validate_scan_target()`. An attacker could send `http://169.254.169.254/meta-data` as the "domain to verify". Always call the SSRF validator before making any outbound HTTP request.

- **Issuing a new token when domain is already verified:** The locked decision says: if a domain is already verified and non-expired, return the existing status. Do not issue a new token and do not overwrite the existing `verified_at` / `expires_at`. Only re-issue a token when the previous record is expired or in `pending` state.

- **Using `domain` as-is from query params:** Domain strings from users may include schemes, paths, www prefixes, or unicode characters. Always normalize through the `normalize_domain()` function before storing or comparing.

- **Checking `<head>` at the scraper layer:** The `scraper::Html::parse_document()` parses the full document. A `Selector::parse("meta[name='shipsecure-verification']")` query matches the meta tag anywhere in the document — including if a user placed it in `<body>` by mistake. For the "Test my tag" pre-check and specific failure diagnosis, distinguish between: (a) tag not found, (b) tag found in `<body>` but not `<head>`, (c) tag found in `<head>` but with wrong content. This enables the "Check that it's in `<head>`, not `<body>`" error message.

- **Blocking `verify-domain` redirect from middleware but not protecting the API:** Adding the page to `isProtectedRoute` in `proxy.ts` protects the page route but does NOT protect the API. The API endpoints in `domains.rs` must use `Claims<ClerkClaims>` as a mandatory handler parameter to enforce JWT auth.

- **Conflating `domain` normalization with URL:** The `verified_domains.domain` column stores the bare domain (e.g., `myapp.vercel.app`), not the full URL. When checking domain verification in `results.rs`, extract the host from `scan.target_url` using the `url` crate.

- **Re-using the same domain string comparison at the results layer:** When extending `owner_verified` in `results.rs`, extract the domain consistently: strip `www.`, normalize case, strip port numbers. The stored domain must match the extraction method used at verification time.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTML meta tag parsing | Custom regex or string search | `scraper::Html::parse_document()` + `Selector::parse()` | Already in use in `detector.rs`; handles malformed HTML, case insensitivity, attribute quoting |
| SSRF protection for verify-confirm | Custom IP check | `ssrf::validate_scan_target()` | Already handles private IPs, cloud metadata, link-local, loopback |
| Cryptographic random token | UUID v4 or timestamp-based | `rand::thread_rng().gen::<[u8; 32]>()` + base64 | Same pattern as `generate_results_token()` in `worker_pool.rs`; 256-bit entropy |
| JWT auth enforcement | Manual header check | `Claims<ClerkClaims>` as handler parameter | axum-jwt-auth returns 401 automatically when token absent or invalid |
| Domain URL parsing | Custom string splitting | `url::Url::parse()` | Already in `ssrf/validator.rs`; handles edge cases in URL structure |
| Clipboard copy UI | Custom implementation | `navigator.clipboard.writeText()` + `lucide-react` icons | Browser API + already-installed icons; no library needed |

**Key insight:** The full meta tag verification stack (HTTP fetch + HTML parse + specific element selection) is already present in `detector.rs`. The domain verification logic is approximately 30 lines using existing infrastructure.

---

## Common Pitfalls

### Pitfall 1: SSRF in verify-confirm

**What goes wrong:** User sends `{"url": "http://169.254.169.254/latest/meta-data"}` as the domain to verify. The server fetches it and could leak cloud metadata or access internal services.

**Why it happens:** The verify-confirm endpoint must make an outbound HTTP request — if user input flows directly to `reqwest::get()`, it's a Server-Side Request Forgery vector.

**How to avoid:** Always call `ssrf::validate_scan_target(&req.url).await?` before the reqwest fetch in `verify_confirm`. The existing SSRF validator checks private IPs, loopback, link-local, cloud metadata endpoints, and blocked schemes.

**Warning signs:** Any `reqwest` call in `domains.rs` that doesn't first call the SSRF validator.

### Pitfall 2: Token Collision or Predictability

**What goes wrong:** A short or predictable token allows an attacker to guess the verification token for another user's domain, allowing them to "verify" a domain they don't control.

**Why it happens:** Using UUID v4, sequential IDs, or timestamp-based tokens reduces the search space.

**How to avoid:** Use the same 32-byte random + base64 approach as `generate_results_token()`. 32 bytes = 256 bits of entropy; the probability of collision or brute-force is negligible.

**Warning signs:** Tokens that look like UUIDs, contain predictable patterns, or are shorter than ~40 characters.

### Pitfall 3: Meta Tag Position Ambiguity (Head vs Body)

**What goes wrong:** User places `<meta name="shipsecure-verification" ...>` in `<body>`. The scraper finds it (CSS selector doesn't restrict to `<head>`), verification passes, but this might be unintentional (e.g., a CMS injected it in the wrong location).

**However:** The locked decision says the meta tag proves control of HTML output. Whether it's in `<head>` or `<body>` is secondary to proving control. The "Test my tag" pre-check and the failure diagnosis message should note if the tag is in `<body>` vs `<head>`, but verification should still succeed either way — the goal is proving HTML output control.

**How to implement the body-vs-head diagnosis:** Use scraper's DOM traversal to check the element's ancestors for a `head` element:

```rust
// Check if element is within <head>
// scraper's ElementRef doesn't have a parent() method directly — use the HTML tree
let head_selector = Selector::parse("head meta[name='shipsecure-verification']").expect("valid");
let in_head = document.select(&head_selector).next().is_some();
let body_selector = Selector::parse("body meta[name='shipsecure-verification']").expect("valid");
let in_body = document.select(&body_selector).next().is_some();
```

**For failure diagnosis purposes:** If the tag is absent entirely → "We fetched your page but didn't find the meta tag." If the content doesn't match → "We found the meta tag but the content doesn't match the expected token." These are the messages the locked decision specifies.

### Pitfall 4: UNIQUE Constraint Race on verify-start

**What goes wrong:** Two concurrent `verify-start` requests for the same `(clerk_user_id, domain)` pair both attempt INSERT simultaneously, causing a constraint violation on the second.

**Why it happens:** Without conflict handling, the second request gets a PostgreSQL unique constraint error.

**How to avoid:** Use `INSERT ... ON CONFLICT (clerk_user_id, domain) DO UPDATE SET verification_token = EXCLUDED.verification_token, status = 'pending', updated_at = NOW()`. This atomically refreshes the token if the record already exists.

```sql
INSERT INTO verified_domains (clerk_user_id, domain, verification_token, status, created_at, updated_at)
VALUES ($1, $2, $3, 'pending', NOW(), NOW())
ON CONFLICT (clerk_user_id, domain)
DO UPDATE SET
    verification_token = CASE
        WHEN verified_domains.status = 'pending' THEN EXCLUDED.verification_token
        WHEN verified_domains.expires_at < NOW() THEN EXCLUDED.verification_token
        ELSE verified_domains.verification_token  -- already verified, keep existing
    END,
    status = CASE
        WHEN verified_domains.status = 'verified' AND verified_domains.expires_at > NOW() THEN 'verified'
        ELSE 'pending'
    END,
    updated_at = NOW()
RETURNING *
```

**Alternative (simpler):** Two queries: first SELECT to check existing state, then INSERT or UPDATE based on result. More readable but not atomic. Acceptable for this use case since simultaneous duplicate calls from the same user are unlikely.

### Pitfall 5: Missing Route Protection for /verify-domain

**What goes wrong:** Unauthenticated users navigate to `/verify-domain` and see the page, which calls `verify-start` without a valid JWT, getting 401 errors.

**Why it happens:** The current `proxy.ts` only protects `/dashboard(.*)`. `/verify-domain` is unprotected.

**How to avoid:** Add `/verify-domain(.*)` to `isProtectedRoute` in `proxy.ts`. The Clerk middleware will redirect unauthenticated users to `/sign-in` automatically.

**Warning signs:** The wizard renders for unauthenticated users and shows 401 error messages from the API.

### Pitfall 6: domain Normalization Mismatch Between verify-start and results.rs

**What goes wrong:** `verify-start` stores `myapp.vercel.app` (after stripping `www.`). `results.rs` extracts `www.myapp.vercel.app` from the scan's `target_url`. The domain comparison fails even though it should succeed.

**Why it happens:** The `url` crate's `host_str()` returns the host as stored in the URL, including `www.`. If domain normalization strips `www.` at verify-start time but not at the results lookup time, they never match.

**How to avoid:** Create a single `normalize_domain_from_url(url: &str) -> Option<String>` helper function in `src/api/domains.rs` or a shared module. Call it both when storing the domain during verify-start AND when extracting the domain in results.rs for the verification check.

### Pitfall 7: Expired Domain Check in results.rs Performance

**What goes wrong:** Adding a database lookup to `get_results_by_token` for domain verification adds latency to every authenticated results fetch.

**Why it happens:** A query to `verified_domains` WHERE `clerk_user_id = $1 AND domain = $2 AND status = 'verified' AND expires_at > NOW()` requires an index hit.

**How to avoid:** The migration must create `CREATE INDEX idx_verified_domains_clerk_domain ON verified_domains(clerk_user_id, domain)`. This index makes the lookup a single index scan. The query is lightweight — one row lookup, not a table scan.

---

## Code Examples

### verify-start Handler (Rust)

```rust
// src/api/domains.rs
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_jwt_auth::Claims;
use serde::Deserialize;

use crate::api::auth::ClerkClaims;
use crate::api::scans::AppState;
use crate::api::errors::ApiError;

#[derive(Deserialize)]
pub struct VerifyStartRequest {
    pub domain: String,
}

pub async fn verify_start(
    State(state): State<AppState>,
    Claims(claims): Claims<ClerkClaims>,
    Json(req): Json<VerifyStartRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let clerk_user_id = &claims.sub;

    // 1. Normalize domain
    let domain = normalize_domain(&req.domain)?;

    // 2. Check blocklist (root TLD only)
    if is_blocked_root_tld(&domain) {
        return Err(ApiError::ValidationError(
            format!("'{}' is a shared hosting platform domain. Enter your app's subdomain instead (e.g., myapp.{}).", domain, domain)
        ));
    }

    // 3. Check if domain is already verified (non-expired)
    if let Some(existing) = db::domains::get_verified_domain(&state.pool, clerk_user_id, &domain).await? {
        if existing.status == "verified" && existing.expires_at.map(|e| e > chrono::Utc::now()).unwrap_or(false) {
            let days_left = existing.expires_at.map(|e| {
                (e - chrono::Utc::now()).num_days()
            }).unwrap_or(0);
            return Ok((StatusCode::OK, Json(serde_json::json!({
                "already_verified": true,
                "domain": domain,
                "expires_in_days": days_left,
            }))));
        }
    }

    // 4. Generate token
    let token = generate_verification_token();
    let meta_tag = format!(
        "<meta name=\"shipsecure-verification\" content=\"{}\">",
        token
    );

    // 5. Upsert into verified_domains
    db::domains::upsert_pending_domain(&state.pool, clerk_user_id, &domain, &token).await?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "domain": domain,
        "token": token,
        "meta_tag": meta_tag,
    }))))
}
```

### verify-confirm Handler (Rust)

```rust
pub async fn verify_confirm(
    State(state): State<AppState>,
    Claims(claims): Claims<ClerkClaims>,
    Json(req): Json<VerifyConfirmRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let clerk_user_id = &claims.sub;

    // 1. Normalize domain
    let domain = normalize_domain(&req.domain)?;

    // 2. Load pending verification record
    let record = db::domains::get_verified_domain(&state.pool, clerk_user_id, &domain)
        .await?
        .ok_or_else(|| ApiError::ValidationError(
            "No verification started for this domain. Call verify-start first.".to_string()
        ))?;

    // 3. Build target URL for fetch
    let target_url = format!("https://{}", domain);

    // 4. SSRF validation — REQUIRED before any outbound fetch
    ssrf::validate_scan_target(&target_url).await
        .map_err(|e| ApiError::SsrfBlocked(e.to_string()))?;

    // 5. Fetch and check meta tag
    match fetch_and_check_meta_tag(&target_url, &record.verification_token).await {
        Ok(()) => {
            // 6. Update to verified
            let expires_at = chrono::Utc::now() + chrono::Duration::days(30);
            db::domains::mark_verified(&state.pool, clerk_user_id, &domain, expires_at).await?;

            Ok(Json(serde_json::json!({
                "verified": true,
                "domain": domain,
                "expires_at": expires_at,
            })))
        }
        Err(reason) => {
            let message = match reason {
                VerificationFailureReason::FetchFailed(e) =>
                    format!("We couldn't reach your site: {}. Make sure it's publicly accessible.", e),
                VerificationFailureReason::TagNotFound =>
                    "We fetched your page but didn't find the meta tag. Make sure it's in <head>.".to_string(),
                VerificationFailureReason::WrongContent =>
                    "We found the meta tag but the content doesn't match the expected token. Copy the snippet again and replace any existing tag.".to_string(),
                VerificationFailureReason::TagInBody =>
                    "We found the meta tag in <body>, not <head>. Move it to the <head> section.".to_string(),
            };
            Ok(Json(serde_json::json!({
                "verified": false,
                "domain": domain,
                "failure_reason": message,
            })))
        }
    }
}
```

### Route Registration in main.rs

```rust
// src/main.rs — add to router
.route("/api/v1/domains/verify-start", post(domains::verify_start))
.route("/api/v1/domains/verify-confirm", post(domains::verify_confirm))
.route("/api/v1/domains/verify-check", post(domains::verify_check))  // "Test my tag" pre-check
.route("/api/v1/domains", get(domains::list_domains))
```

**CORS note:** The domain endpoints need the CORS `allow_headers` to include `AUTHORIZATION`. This is already set:
```rust
.allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION]);
```
No CORS changes needed.

### TypeScript Types for Domain API

```typescript
// frontend/lib/types.ts — additions

export interface VerifiedDomain {
  id: string
  domain: string
  status: 'pending' | 'verified'
  verified_at: string | null
  expires_at: string | null
  created_at: string
}

export interface VerifyStartResponse {
  domain: string
  token: string
  meta_tag: string
  already_verified?: boolean
  expires_in_days?: number
}

export interface VerifyConfirmResponse {
  verified: boolean
  domain: string
  expires_at?: string
  failure_reason?: string
}
```

### API Functions for Domain Verification

```typescript
// frontend/lib/api.ts — additions

const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:3000'

export async function verifyStart(
  domain: string,
  token: string
): Promise<VerifyStartResponse> {
  const res = await fetch(`${BACKEND_URL}/api/v1/domains/verify-start`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`,
    },
    body: JSON.stringify({ domain }),
  })
  if (!res.ok) {
    const error = await res.json()
    throw new Error(error.detail || 'Failed to start verification')
  }
  return res.json()
}

export async function verifyConfirm(
  domain: string,
  token: string
): Promise<VerifyConfirmResponse> {
  const res = await fetch(`${BACKEND_URL}/api/v1/domains/verify-confirm`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`,
    },
    body: JSON.stringify({ domain }),
  })
  if (!res.ok) {
    const error = await res.json()
    throw new Error(error.detail || 'Verification failed')
  }
  return res.json()
}

export async function listDomains(token: string): Promise<VerifiedDomain[]> {
  const res = await fetch(`${BACKEND_URL}/api/v1/domains`, {
    cache: 'no-store',
    headers: { 'Authorization': `Bearer ${token}` },
  })
  if (!res.ok) return []
  return res.json()
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| DNS TXT verification for domain ownership | HTML meta tag verification | Phase 32 design decision | More accessible to vibe-coders who don't manage DNS |
| Manual header parsing for JWT | `Claims<ClerkClaims>` extractor (mandatory) or `state.jwt_decoder.decode()` (optional) | Phase 31 | Domain endpoints use mandatory; results uses optional — both patterns already proven |
| Single `owner_verified` check (identity match only) | Dual check: identity match AND active domain verification | Phase 32 | Expired domain verification re-gates results — enforced at API layer |

**Deprecated/outdated:**
- `auth.protect()` in middleware for protecting routes: still valid, but complemented by `Claims<ClerkClaims>` at API layer for direct curl protection

---

## Open Questions

1. **Where does "Test my tag" pre-check live — separate endpoint or inline in verify-confirm?**
   - What we know: The locked decision says there is an optional "Test my tag" button that verifies without consuming the attempt. The concept of "consuming the attempt" is unclear — verify-confirm doesn't decrement a counter, it just checks and writes verified status.
   - What's unclear: Does "without consuming" mean: (a) the meta tag check without writing to the DB, (b) the fetch without any DB write, or (c) something else?
   - Recommendation: Implement `POST /api/v1/domains/verify-check` as a separate endpoint that performs the fetch-and-check logic but writes nothing to the DB. Returns the same success/failure payload as verify-confirm. The frontend calls this when user clicks "Test my tag", calls verify-confirm when user clicks "Verify now".

2. **How should the verify-confirm endpoint handle the target URL construction?**
   - What we know: Users enter a domain like `myapp.vercel.app`. The verify-confirm fetch must check `https://myapp.vercel.app/`. But some apps may redirect `https://myapp.vercel.app/` to a different URL.
   - What's unclear: Should we fetch `https://{domain}/` specifically, or should we allow the user to specify the exact URL to check (e.g., `https://myapp.vercel.app/app/`)?
   - Recommendation: Fetch `https://{domain}/` for the root. The meta tag should be present on every page if using a framework, but document this limitation. Do not allow arbitrary path specification — it increases complexity and the SSRF surface.

3. **Should the `verified_domains` domain column store the scheme+host or just the host?**
   - What we know: The locked decision says "verify a domain" — domain implies host only, not full URL. The meta tag proves HTML output control.
   - What's unclear: If a user verifies `myapp.vercel.app`, should this also apply to `http://myapp.vercel.app` (non-HTTPS)? In practice, all modern Vercel/Netlify apps are HTTPS-only.
   - Recommendation: Store the bare domain (host only, no scheme). When checking against `scan.target_url` in results.rs, extract host from the URL regardless of scheme. Verification always fetches over HTTPS.

4. **Scan submission gating: does Phase 32 block authenticated scan submission if domain is unverified?**
   - What we know: The phase description says "only verified domains can receive authenticated scans." The plan in 32-01 mentions "30-day TTL enforcement in scan submission."
   - What's unclear: The current scan submission (`POST /api/v1/scans`) doesn't require domain verification. Should Phase 32 add that check, or is it just the results gating?
   - Recommendation: The success criteria only mentions results gating (criteria 4). The `owner_verified` extension in results.rs is the primary mechanism. Blocking scan submission at the API layer is a separate, not-yet-planned gate. Recommend treating this as out of scope for Phase 32 unless the 32-01 plan explicitly includes it — the description mentions "TTL enforcement in scan submission" which may mean validating the domain is verified before allowing an authenticated scan to run.

---

## Sources

### Primary (HIGH confidence)
- Project source: `src/scanners/detector.rs` — `scraper::Html::parse_document()` + `Selector::parse()` usage pattern confirmed
- Project source: `src/ssrf/validator.rs` — `validate_scan_target()` for SSRF protection; must be called in verify-confirm
- Project source: `src/orchestrator/worker_pool.rs` (lines 261–266) — `generate_results_token()` using `rand::thread_rng().gen::<[u8; 32]>()` + `base64::URL_SAFE_NO_PAD`
- Project source: `src/api/results.rs` — `extract_optional_clerk_user()` pattern (for reference); `owner_verified` logic that Phase 32 extends
- Project source: `src/api/auth.rs` — `ClerkClaims`, `ClerkUser`, `FromRef<AppState> for Decoder<ClerkClaims>`
- Project source: `src/api/scans.rs` — `AppState` struct; `Claims<ClerkClaims>` as mandatory handler parameter (not present yet, but the infrastructure supports it via `FromRef`)
- Project source: `migrations/20260217000001_create_users.sql` — `users(clerk_user_id)` is the FK target for `verified_domains`
- Project source: `migrations/20260218000001_stripe_removal_schema.sql` — confirms `scans.clerk_user_id TEXT REFERENCES users(clerk_user_id)`
- Project source: `frontend/proxy.ts` — `isProtectedRoute` matcher; must add `/verify-domain(.*)`
- Project source: `frontend/app/dashboard/page.tsx` — Server Component with `auth()` pattern
- Project source: `frontend/app/results/[token]/page.tsx` — `getToken()` forwarding pattern for authenticated backend calls
- Project source: `frontend/app/globals.css` — `success-*`, `caution-*`, `danger-*`, `info-*` semantic tokens for badge colors
- Project source: `Cargo.toml` — confirms `rand = "0.8"`, `base64 = "0.22"`, `scraper = "0.22"`, `reqwest`, `url`, `sqlx` all available

### Secondary (MEDIUM confidence)
- `scraper` 0.22 crate: CSS selector `head meta[name='...']` and `body meta[name='...']` selectors work for head/body distinction — based on standard CSS selector semantics and existing detector.rs usage patterns

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries confirmed from Cargo.toml and Cargo.lock; no new dependencies needed
- Architecture: HIGH — all patterns derived from existing codebase usage; DB schema follows established migration conventions; JWT auth pattern confirmed from axum-jwt-auth source verified in Phase 31 research
- Pitfalls: HIGH — SSRF risk is established (ssrf/validator.rs exists for this exact reason); domain normalization mismatch is a concrete string comparison issue; unique constraint race is a standard PostgreSQL concern
- Frontend wizard: HIGH — pattern follows existing `'use client'` components in codebase; `useState` step machine is idiomatic React; Clerk auth forwarding pattern confirmed from results page

**Research date:** 2026-02-18
**Valid until:** 2026-03-18 (stable libraries; Clerk minor versions may change but pattern is stable)
