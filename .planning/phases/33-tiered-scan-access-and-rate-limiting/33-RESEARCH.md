# Phase 33: Tiered Scan Access and Rate Limiting - Research

**Researched:** 2026-02-18
**Domain:** Axum middleware, SQLx query patterns, scan orchestration tiering, rate limiting, Next.js auth-aware form submission
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Rate limit responses:** Friendly tone with upgrade nudge for anonymous users hitting 1/24h limit: "You've used your free scan today. Sign up for more scans — resets in 18h." Countdown format for resets_at ("in 18h 23m"), not absolute timestamps. Authenticated users hitting monthly quota get same friendly tone but with upgrade CTA: "5 of 5 scans used this month. Upgrade to Pro for unlimited scans." (placeholder for future tier even if Pro doesn't exist yet). Rate limit info in 429 JSON body only — no X-RateLimit headers.
- **Quota display:** Dashboard header shows text badge with color coding (green/yellow/red): "3/5 scans". No pre-warning for anonymous users — they just get the 429 when they hit it. On monthly reset, badge updates to show "0/5 scans" with fresh/green styling.
- **Scan tier behavior:** Visible "Enhanced scan" badge for authenticated users — they know they're getting deeper analysis. Anonymous scans show "Basic scan" label with "Sign up for deeper analysis" upsell link. Tier badges appear in both scan results header AND scan history cards. Label only ("Basic" / "Enhanced") — no internal config details exposed (file counts, timeouts hidden).
- **Domain verification gate:** Hard block for authenticated users scanning unverified domains — reject with clear error message. Both client-side and server-side checks: client warns on scan click (better UX), server enforces (security). Client-side check triggers on scan button click, not on URL field blur. Error links to /verify-domain page — no inline verification flow.

### Claude's Discretion

- Exact color thresholds for quota badge (green/yellow/red breakpoints)
- Badge placement within dashboard header
- Exact wording of upsell messages
- Error message copy for domain verification rejection

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| TIER-01 | Anonymous scans use lighter config (20 JS files, 180s vibecode timeout) | Already partially coded in `run_scanners` with hardcoded free-tier values; need to activate the 3-arm match and wire it properly |
| TIER-02 | Authenticated scans use enhanced config (30 JS files, 300s vibecode timeout, extended exposed files) | `scan_js_secrets(url, max_files)` and `scan_exposed_files(url, extended: bool)` already accept these params; just need the enhanced path called |
| TIER-03 | Anonymous users limited to 1 scan per IP per 24 hours | `count_scans_by_ip_today` DB function exists; `check_rate_limits` needs a new anonymous IP arm |
| TIER-04 | Developer tier users limited to 5 scans per calendar month | Need new DB query `count_scans_by_user_this_month`; `check_rate_limits` needs `Option<clerk_user_id>` parameter |
| TIER-05 | Rate limit exceeded returns 429 with friendly message and `resets_at` timestamp | `ApiError::RateLimited` already returns 429; need to extend the error body to include `resets_at` and friendly message |
| TIER-06 | Authenticated scans require verified domain ownership | `db::domains::is_domain_verified` exists; need to call it in `create_scan` handler when a Clerk user is present |
</phase_requirements>

---

## Summary

Phase 33 is primarily a backend wiring phase on an already well-prepared codebase. The infrastructure for tiering, rate limiting, domain verification, and JWT extraction is all present — the work is connecting these pieces in `create_scan` and `check_rate_limits`, then surfacing the results to the frontend.

The backend has three distinct gaps to close. First, `run_scanners` already has a comment saying all tiers use free-tier config but never activates the enhanced branch — the 3-arm match (anonymous/authenticated/paid) needs to be implemented so `tier == "authenticated"` uses 30 JS files, 300s vibecode timeout, and `extended_files = true`. Second, `check_rate_limits` takes `(pool, email, ip)` and checks email-based and IP-based daily limits — this needs to be replaced with a new signature taking `Option<clerk_user_id>` that routes to per-IP 1/24h for anonymous callers and per-user 5/calendar-month for authenticated callers. Third, `create_scan` has no JWT extraction at all — adding optional JWT extraction (using the same pattern as `results.rs`) enables routing to the correct tier arm, enforcing the domain verification gate, and persisting `clerk_user_id` on the scan record.

The frontend work is smaller: update the scan submission to forward the Clerk token when signed in, add a domain ownership check on scan button click, show "Basic"/"Enhanced" scan tier badges on results pages, and add a quota badge to the dashboard header. The existing Clerk integration (`useAuth`, `auth()`) and component patterns (DomainBadge, etc.) mean these are incremental additions following established patterns.

**Primary recommendation:** Implement in two sub-plans: 33-01 covers the backend (3-arm tier match, create_scan JWT extraction, domain verification gate, enhanced config dispatch) and 33-02 covers rate limiting extension plus frontend quota/tier display.

---

## Standard Stack

### Core (no new dependencies needed)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `sqlx` | 0.8.6 | DB queries for rate limit counting and domain verification | Already in use; `count_scans_by_ip_today` pattern can be cloned for monthly |
| `axum-jwt-auth` | 0.6 | Optional JWT extraction in `create_scan` | Already used in `results.rs` for `extract_optional_clerk_user` — identical pattern |
| `chrono` | 0.4.43 | Calendar-month window calculation for monthly quota | Already used; `Utc::now().date_naive()` and `NaiveDate` for first-of-next-month |
| `@clerk/nextjs` | existing | `useAuth()` for token forwarding in scan form | Already used in `verify-domain/page.tsx` — identical pattern |

### No New Dependencies Required

All necessary libraries are already in `Cargo.toml` and `package.json`. This phase adds no new crates or npm packages.

---

## Architecture Patterns

### Recommended Structure for 33-01 (Scan Orchestration)

```
src/
├── api/
│   └── scans.rs          # create_scan: add JWT extraction, tier match, domain gate
├── orchestrator/
│   └── worker_pool.rs    # spawn_authenticated_scan (new), activate enhanced config in run_scanners
├── rate_limit/
│   └── middleware.rs     # extend check_rate_limits signature (Option<clerk_user_id>)
└── db/
    └── scans.rs          # add count_scans_by_user_this_month, create_scan_with_tier
```

### Pattern 1: Optional JWT Extraction in create_scan

`results.rs` already defines `extract_optional_clerk_user` — the same function can be used or extracted to a shared location. Since `create_scan` has a different signature from `get_results_by_token`, the approach is to either:

1. Import and call the existing `extract_optional_clerk_user` from `results.rs` (preferred — avoids duplication), or
2. Duplicate the pattern inline in `scans.rs`

The function signature already exists:

```rust
// Source: src/api/results.rs
async fn extract_optional_clerk_user(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Option<String> {
    let auth_value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let token = auth_value.strip_prefix("Bearer ")?.trim();
    if token.is_empty() {
        return None;
    }
    let token_data = state.jwt_decoder.decode(token).await.ok()?;
    Some(token_data.claims.sub)
}
```

The `create_scan` handler needs to accept `headers: axum::http::HeaderMap` as an additional extractor (axum supports multiple extractors in any order).

### Pattern 2: 3-Arm Tier Match in create_scan

```rust
// Pseudo-code for the tier match
let clerk_user_id = extract_optional_clerk_user(&state, &headers).await;

let tier = match &clerk_user_id {
    None => "free",           // TIER-01: anonymous
    Some(_) => "authenticated", // TIER-02: authenticated Developer
    // "paid" branch reserved for future monetization
};
```

The `tier` value is stored on the scan record (column already exists with CHECK constraint `IN ('free', 'paid', 'authenticated')`).

### Pattern 3: run_scanners Enhanced Config Activation

`run_scanners` already has the comment "all tiers use free-tier config" with the values hardcoded. The fix is a match on the `tier` parameter:

```rust
// Source: src/orchestrator/worker_pool.rs (current code, needs change)
let (max_js_files, extended_files, vibecode_timeout, other_timeout) =
    (20, false, Duration::from_secs(180), Duration::from_secs(60));

// CHANGE TO:
let (max_js_files, extended_files, vibecode_timeout, other_timeout) = match tier {
    "authenticated" | "paid" => (30, true, Duration::from_secs(300), Duration::from_secs(60)),
    _ => (20, false, Duration::from_secs(180), Duration::from_secs(60)),
};
```

The scanner functions already accept these parameters:
- `scan_js_secrets(url, max_files: usize)` — `max_files` param present, line 146 of `js_secrets.rs`
- `scan_exposed_files(url, extended: bool)` — `extended` param present, line 59 of `exposed_files.rs`

### Pattern 4: spawn_authenticated_scan

`spawn_scan` hard-codes `tier = "free"` in its span and passes `"free"` to `execute_scan_internal`. A new `spawn_authenticated_scan` method should pass `"authenticated"` instead. Rather than duplicating the 150-line `spawn_scan` body, extract the tier as a parameter:

```rust
// Refactor: make spawn_scan take tier as parameter, or add a wrapper
pub fn spawn_authenticated_scan(&self, scan_id: Uuid, target_url: String, request_id: Option<Uuid>) {
    self.spawn_scan_with_tier(scan_id, target_url, request_id, "authenticated")
}

pub fn spawn_scan(&self, scan_id: Uuid, target_url: String, request_id: Option<Uuid>) {
    self.spawn_scan_with_tier(scan_id, target_url, request_id, "free")
}

fn spawn_scan_with_tier(&self, scan_id: Uuid, target_url: String, request_id: Option<Uuid>, tier: &'static str) {
    // ... current spawn_scan body with `tier` as variable
}
```

### Pattern 5: Domain Verification Gate in create_scan

```rust
// After tier match, before spawning scan:
if let Some(ref user_id) = clerk_user_id {
    let domain = extract_domain_from_url(&validated_url)
        .ok_or_else(|| ApiError::ValidationError("Could not parse domain from URL".to_string()))?;

    let verified = db::domains::is_domain_verified(&state.pool, user_id, &domain).await?;
    if !verified {
        return Err(ApiError::Custom {
            status: StatusCode::FORBIDDEN,
            error_type: "https://shipsecure.ai/errors/domain-not-verified".to_string(),
            title: "Domain Not Verified".to_string(),
            detail: "You must verify ownership of this domain before scanning. Visit /verify-domain to get started.".to_string(),
        });
    }
}
```

`extract_domain_from_url` already exists in `results.rs` — same reuse/extraction pattern applies.

### Pattern 6: Extended check_rate_limits

Current signature: `check_rate_limits(pool: &PgPool, email: &str, ip: &str) -> Result<(), ApiError>`

New signature: `check_rate_limits(pool: &PgPool, clerk_user_id: Option<&str>, ip: &str) -> Result<(), ApiError>`

The `email` field is no longer needed as the routing key — for anonymous users, IP is the key; for authenticated users, `clerk_user_id` is the key.

```rust
pub async fn check_rate_limits(
    pool: &PgPool,
    clerk_user_id: Option<&str>,
    ip: &str,
) -> Result<(), ApiError> {
    match clerk_user_id {
        None => {
            // Anonymous: 1 scan per IP per 24h
            let count = count_scans_by_ip_today(pool, ip).await?;
            if count >= 1 {
                let resets_at = next_midnight_utc();
                return Err(rate_limit_error_anonymous(resets_at));
            }
        }
        Some(user_id) => {
            // Authenticated Developer: 5 scans per calendar month
            let count = count_scans_by_user_this_month(pool, user_id).await?;
            if count >= 5 {
                let resets_at = first_of_next_month_utc();
                return Err(rate_limit_error_authenticated(count, resets_at));
            }
        }
    }
    Ok(())
}
```

### Pattern 7: 429 Response with resets_at

The current `ApiError::RateLimited(String)` only stores a message. The 429 response needs `resets_at` in the JSON body. Options:

**Option A:** Add `ApiError::RateLimitedWithReset { message: String, resets_at: DateTime<Utc> }` variant (cleanest, extends existing pattern).

**Option B:** Use existing `ApiError::Custom` with a custom JSON body (avoids enum change but less structured).

**Recommendation:** Option A — add a new variant so `IntoResponse` can serialize `resets_at` properly. The existing `ProblemDetails` struct can gain an `optional_resets_at` field, or a custom serializer can be used for this variant.

The 429 JSON body should look like:
```json
{
  "type": "https://shipsecure.ai/errors/rate-limited",
  "title": "Rate Limit Exceeded",
  "status": 429,
  "detail": "You've used your free scan today. Sign up for more scans — resets in 18h.",
  "resets_at": "2026-02-19T00:00:00Z"
}
```

The `resets_at` is an ISO 8601 timestamp (absolute); the frontend formats it as "in 18h 23m".

### Pattern 8: Calendar Month Window for Monthly Quota

New DB function needed:

```rust
pub async fn count_scans_by_user_this_month(pool: &PgPool, clerk_user_id: &str) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)
         FROM scans
         WHERE clerk_user_id = $1
           AND created_at >= DATE_TRUNC('month', NOW() AT TIME ZONE 'UTC')"
    )
    .bind(clerk_user_id)
    .fetch_one(pool)
    .await?;
    Ok(count.0)
}
```

`DATE_TRUNC('month', NOW() AT TIME ZONE 'UTC')` gives the first instant of the current UTC month. This is the natural calendar month window.

For `resets_at` (first of next month):
```rust
fn first_of_next_month_utc() -> chrono::DateTime<chrono::Utc> {
    let now = chrono::Utc::now();
    let next_month = if now.month() == 12 {
        chrono::Utc.with_ymd_and_hms(now.year() + 1, 1, 1, 0, 0, 0).unwrap()
    } else {
        chrono::Utc.with_ymd_and_hms(now.year(), now.month() + 1, 1, 0, 0, 0).unwrap()
    };
    next_month
}
```

### Pattern 9: create_scan with Tier and clerk_user_id Persistence

The current `db::scans::create_scan` inserts with default `tier = 'free'` (the column default). We need to pass `tier` and `clerk_user_id` explicitly:

```rust
pub async fn create_scan(
    pool: &PgPool,
    target_url: &str,
    email: &str,
    submitter_ip: Option<&str>,
    request_id: Option<Uuid>,
    tier: &str,
    clerk_user_id: Option<&str>,
) -> Result<Scan, sqlx::Error>
```

The SQL changes from:
```sql
INSERT INTO scans (target_url, email, submitter_ip, request_id)
```
to:
```sql
INSERT INTO scans (target_url, email, submitter_ip, request_id, tier, clerk_user_id)
VALUES ($1, $2, $3::inet, $4, $5, $6)
```

**Important:** The scans table already has `tier TEXT NOT NULL DEFAULT 'free'` and `clerk_user_id TEXT REFERENCES users(clerk_user_id) ON DELETE SET NULL`. No migration needed. However, `clerk_user_id` references `users.clerk_user_id` — this means we need the user to exist in the `users` table when they scan. The Clerk webhook already populates `users` on sign-up (Phase 29 work), but if a user signs in without triggering the webhook (e.g., dev environment), the FK will fail. Consider using `ON CONFLICT DO NOTHING` upsert for the user, or use `ON DELETE SET NULL` semantics (already configured) — the FK will reject a `clerk_user_id` that doesn't exist in `users`. This is a pitfall to document.

### Pattern 10: Frontend Scan Form — Auth-Aware Submission

Current `scan.ts` server action calls the backend without auth headers. To support tier routing:

```typescript
// app/actions/scan.ts (server action)
import { auth } from '@clerk/nextjs/server'

export async function submitScan(prevState, formData) {
  // ... validation ...

  const { getToken, userId } = await auth()
  const token = userId ? await getToken() : null

  const headers: Record<string, string> = { 'Content-Type': 'application/json' }
  if (token) headers['Authorization'] = `Bearer ${token}`

  const response = await fetch(`${backendUrl}/api/v1/scans`, {
    method: 'POST',
    headers,
    body: JSON.stringify(validatedFields.data),
  })
  // ...
}
```

### Pattern 11: Client-Side Domain Verification Check in ScanForm

Decision: client checks on scan button click, not on URL field blur.

Since `ScanForm` is a server-action form using `useActionState`, the button click happens via form submission. The domain check needs to happen either:

1. **In the server action** (`submitScan`) — check domain before forwarding to backend. This is simpler but shows the error only after submit.
2. **In the client component** — intercept form submission with an `onClick` handler on the button (not `onSubmit` on the form, to avoid breaking the form action pattern).

**Recommendation:** Handle in the server action. When `userId` is present, extract the domain from the URL, call a new API endpoint `GET /api/v1/domains/check?domain=...` that returns `{ verified: boolean }`, and if not verified return a form error with a link to `/verify-domain`. This avoids client complexity. The decision says "client warns on scan click" — this is satisfied by the server action returning an error with a link, which appears after button click.

Alternatively, a thin client-side check using `listDomains` could pre-validate. But the server action approach is simpler and consistent with existing patterns. The server enforces regardless (TIER-06 backend gate).

### Pattern 12: Quota Badge in Dashboard Header

The dashboard page (`app/dashboard/page.tsx`) is a Next.js Server Component that fetches data server-side. The quota badge requires knowing:
1. How many scans the user has used this month
2. The monthly limit (5 for Developer tier)

This data should come from a new API endpoint or be included in a new backend call from the dashboard page. Options:

**Option A:** New `GET /api/v1/quota` endpoint returning `{ used: 3, limit: 5, resets_at: "..." }`.
**Option B:** Fetch from the DB directly in the dashboard page (not viable — dashboard page has no direct DB access).
**Option C:** Include quota in the existing API response pattern — dashboard already fetches `/api/v1/domains`, add a similar fetch to `/api/v1/quota`.

**Recommendation:** Option A — a dedicated `/api/v1/quota` endpoint. The dashboard fetches it server-side using `getToken()` (same pattern as domains fetch). The response drives the badge.

### Pattern 13: Tier Badges on Results Page

The results API already returns `"tier": scan.tier`. The frontend `ScanResponse` type already includes `tier: string`. The results page (`app/results/[token]/page.tsx`) just needs a tier badge component in the results header section.

Badge display logic:
- `tier === "free"` → "Basic scan" label with upsell link
- `tier === "authenticated"` or `tier === "paid"` → "Enhanced scan" label

### Anti-Patterns to Avoid

- **Checking email for rate limits on authenticated users:** The old `check_rate_limits` takes email. For authenticated users, rate limit should be keyed on `clerk_user_id`, not email — users could change emails or have multiple emails.
- **Blocking authenticated users with the old IP-based rate limit:** The new logic should be mutually exclusive: anonymous → IP limit, authenticated → user monthly quota. Don't apply both.
- **Forgetting to update the scan's tier before spawning:** The DB record must have `tier` set to `"authenticated"` BEFORE `spawn_authenticated_scan` is called, so `execute_scan_internal` reads the correct tier from context (though it also gets tier passed directly).
- **Assuming users table exists for all Clerk users:** The FK `clerk_user_id REFERENCES users(clerk_user_id)` will reject if the user isn't in `users`. Test with a fresh Clerk sign-in that hasn't triggered the webhook.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Monthly calendar window | Custom date arithmetic | `DATE_TRUNC('month', NOW() AT TIME ZONE 'UTC')` in SQL | PostgreSQL handles DST, leap years correctly |
| JWT verification | Manual JWT parsing | Existing `extract_optional_clerk_user` pattern (axum-jwt-auth) | Already battle-tested in results.rs |
| Rate limit storage | In-memory HashMap | DB-backed counts via existing sqlx query pattern | In-memory doesn't survive restarts, doesn't work with multiple instances |
| Countdown formatting | Custom duration formatter | Simple arithmetic in frontend (e.g., `Math.ceil(diff / 3600)` for hours) | The backend sends `resets_at` ISO timestamp; frontend formats it |

---

## Common Pitfalls

### Pitfall 1: clerk_user_id FK Constraint Failure
**What goes wrong:** `create_scan` sets `clerk_user_id` but the user hasn't been inserted into the `users` table yet (webhook not received, dev environment, etc.). The FK `REFERENCES users(clerk_user_id)` will reject the insert with a foreign key violation.
**Why it happens:** The webhook that populates `users` may lag or not fire in all environments.
**How to avoid:** Option 1 — upsert the user into `users` inside `create_scan` when a `clerk_user_id` is present. Option 2 — make `clerk_user_id` on scans reference without FK enforcement (drop FK). Option 3 — catch the FK violation and treat as anonymous. Recommended: upsert user in the handler before inserting the scan.
**Warning signs:** 500 errors on scan creation for authenticated users in dev environments.

### Pitfall 2: Domain URL Normalization Mismatch
**What goes wrong:** `validated_url` might be `https://www.example.com/path` but the verified domain is stored as `example.com`. The domain extraction must use the same `normalize_domain` logic as the domain verification flow.
**Why it happens:** `extract_domain_from_url` in `results.rs` strips `www.` and lowercases — this must be the exact same logic used when checking `is_domain_verified`.
**How to avoid:** Move `extract_domain_from_url` to a shared location (e.g., `src/api/mod.rs` or `src/lib.rs`) and import it in both `results.rs` and `scans.rs`.
**Warning signs:** Domain verification appears to work but authenticated scans always get blocked as "unverified".

### Pitfall 3: Stale IP Count for Anonymous Rate Limits
**What goes wrong:** The current `count_scans_by_ip_today` counts all scans today, including `authenticated` tier scans. An anonymous user on the same IP as an authenticated user might see the count inflated.
**Why it happens:** The existing query doesn't filter by `tier = 'free'`.
**How to avoid:** Add `AND tier = 'free'` or `AND clerk_user_id IS NULL` to the anonymous IP count query to only count anonymous scans.
**Warning signs:** Authenticated users scanning from an IP block anonymous scans at the same IP prematurely.

### Pitfall 4: Double Rate-Limiting — Old Email Check Still Fires
**What goes wrong:** If the old email-based rate limit check is left in place alongside the new logic, authenticated users might get rate-limited by their email even after the new system is in place.
**Why it happens:** Forgetting to fully replace `check_rate_limits` call site in `create_scan`.
**How to avoid:** Remove the old `check_rate_limits` call entirely; the new `check_rate_limits` with `Option<clerk_user_id>` replaces it completely.

### Pitfall 5: resets_at Timezone Confusion
**What goes wrong:** `resets_at` sent as UTC but displayed without timezone context — users in other timezones might be confused.
**Why it happens:** "In 18h" is relative and timezone-agnostic, but "resets Mar 1" implies midnight local time.
**How to avoid:** Always use `resets_at` as a UTC ISO 8601 timestamp; frontend computes the countdown as `resets_at - now()` in milliseconds, formats as "in Xh Ym". Don't show absolute dates in the 429 message body — just the countdown.

### Pitfall 6: Frontend Domain Check Race Condition
**What goes wrong:** User starts typing a URL, then clicks "Scan Now" — the client-side domain check fires before the user has finished typing, or the domain check API call returns after the scan submission has already started.
**Why it happens:** If domain check is wired to `onChange` or fires async before submit.
**How to avoid:** Decision is locked: check triggers on scan button click. The server action pattern ensures sequential execution — the domain check happens inside `submitScan`, which is `async` and awaited before the scan create call.

### Pitfall 7: Anonymous Users Getting "Enhanced scan" Badge
**What goes wrong:** Scan records created before Phase 33 have `tier = 'free'` but might be viewed by a user who is now authenticated. The tier badge should reflect the scan's tier at creation time, not the current user's tier.
**Why it happens:** Displaying tier based on current auth state rather than stored `scan.tier`.
**How to avoid:** Always use `scan.tier` from the API response for badge display — it's stored at scan creation time and never changes.

---

## Code Examples

### Monthly quota count (new DB function needed)

```rust
// Source: Derived from existing count_scans_by_ip_today pattern in src/db/scans.rs
pub async fn count_scans_by_user_this_month(
    pool: &PgPool,
    clerk_user_id: &str,
) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)
         FROM scans
         WHERE clerk_user_id = $1
           AND created_at >= DATE_TRUNC('month', NOW() AT TIME ZONE 'UTC')"
    )
    .bind(clerk_user_id)
    .fetch_one(pool)
    .await?;
    Ok(count.0)
}
```

### Anonymous IP count (add tier filter)

```rust
// Source: Modified from existing count_scans_by_ip_today in src/db/scans.rs
pub async fn count_anonymous_scans_by_ip_today(
    pool: &PgPool,
    ip: &str,
) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)
         FROM scans
         WHERE submitter_ip = $1::inet
           AND clerk_user_id IS NULL
           AND created_at >= CURRENT_DATE"
    )
    .bind(ip)
    .fetch_one(pool)
    .await?;
    Ok(count.0)
}
```

### Next midnight UTC (for anonymous resets_at)

```rust
// Source: Derived using chrono (already in Cargo.toml)
fn next_midnight_utc() -> chrono::DateTime<chrono::Utc> {
    use chrono::{Utc, Timelike, Duration};
    let now = Utc::now();
    let today_midnight = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let tomorrow_midnight = today_midnight + Duration::days(1);
    chrono::DateTime::from_naive_utc_and_offset(tomorrow_midnight, chrono::Utc)
}
```

### First of next month UTC (for authenticated resets_at)

```rust
// Source: Derived using chrono (already in Cargo.toml)
fn first_of_next_month_utc() -> chrono::DateTime<chrono::Utc> {
    use chrono::{Utc, Datelike, TimeZone};
    let now = Utc::now();
    let (year, month) = if now.month() == 12 {
        (now.year() + 1, 1u32)
    } else {
        (now.year(), now.month() + 1)
    };
    chrono::Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0)
        .unwrap()
}
```

### Friendly resets_at countdown formatting (frontend)

```typescript
// Source: Derived from requirements (countdown format "in 18h 23m")
function formatResetsAt(resetsAtIso: string): string {
  const diff = new Date(resetsAtIso).getTime() - Date.now()
  if (diff <= 0) return 'soon'
  const hours = Math.floor(diff / (1000 * 60 * 60))
  const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60))
  if (hours > 0) return `in ${hours}h ${minutes}m`
  return `in ${minutes}m`
}
```

### Quota badge color logic (Claude's discretion)

Recommended thresholds:
- Green: `used / limit < 0.6` (0–2 of 5)
- Yellow: `used / limit >= 0.6 && used / limit < 1.0` (3–4 of 5)
- Red: `used === limit` (5 of 5)

```typescript
function getQuotaBadgeStyle(used: number, limit: number): string {
  const ratio = used / limit
  if (ratio < 0.6) return 'bg-success-bg text-success-text border border-success-border'
  if (ratio < 1.0) return 'bg-caution-bg text-caution-text border border-caution-border'
  return 'bg-danger-bg text-danger-text border border-danger-border'
}
```

### Quota API endpoint (new GET /api/v1/quota)

```rust
// Pseudocode for new handler in src/api/scans.rs or new src/api/quota.rs
pub async fn get_quota(
    State(state): State<AppState>,
    Claims { claims, .. }: Claims<ClerkClaims>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let used = db::scans::count_scans_by_user_this_month(&state.pool, &claims.sub).await?;
    let limit = 5i64; // Developer tier
    let resets_at = first_of_next_month_utc();
    Ok(Json(json!({
        "used": used,
        "limit": limit,
        "resets_at": resets_at,
    })))
}
```

---

## Implementation Sequence

The two sub-plans map naturally to:

### 33-01: Tiered Scan Orchestration (backend)

1. Move `extract_domain_from_url` to shared location (or import from results.rs)
2. Refactor `spawn_scan` into `spawn_scan_with_tier(tier: &'static str)` + thin wrappers
3. Activate tier config in `run_scanners` (3-arm match on `tier`)
4. Extend `db::scans::create_scan` to accept `tier` and `clerk_user_id`
5. Add `extract_optional_clerk_user` call to `create_scan` handler (accept headers extractor)
6. Add domain verification gate to `create_scan` (call `is_domain_verified` when authenticated)
7. Dispatch to `spawn_authenticated_scan` or `spawn_scan` based on tier
8. Add `GET /api/v1/quota` endpoint with `Claims<ClerkClaims>` extractor
9. Register `/api/v1/quota` route in `main.rs`

### 33-02: Rate Limiting and Quota Display (backend + frontend)

1. Add `count_anonymous_scans_by_ip_today` (replaces old `count_scans_by_ip_today` in rate limit)
2. Add `count_scans_by_user_this_month`
3. Add `ApiError::RateLimitedWithReset` variant + `IntoResponse` serialization with `resets_at`
4. Rewrite `check_rate_limits` with `Option<clerk_user_id>` parameter
5. Update `create_scan` call site for `check_rate_limits`
6. Frontend: update `submitScan` server action to forward Clerk token
7. Frontend: add domain check in `submitScan` (call quota/domain endpoint when authenticated)
8. Frontend: update 429 error handling to show `resets_at` countdown
9. Frontend: add `QuotaBadge` component to dashboard header
10. Frontend: add tier badge to results page header and scan history cards
11. Frontend: update `CreateScanResponse` / `ScanResponse` types if needed

---

## State of the Art

| Old Approach | Current Approach | Notes |
|--------------|------------------|-------|
| In-memory rate limiting (maps) | DB-backed count queries | Already using DB approach — correct for multi-instance |
| Redis for rate limiting | PostgreSQL COUNT queries | Acceptable at this scale; Redis adds operational complexity |
| X-RateLimit headers | 429 JSON body with `resets_at` | User decision: body only |
| Email as rate limit key | IP for anonymous, clerk_user_id for auth | More accurate; avoids email spoofing |

---

## Open Questions

1. **users table FK on anonymous scan submission**
   - What we know: If `clerk_user_id` is set on scans but the user isn't in `users`, the FK constraint will reject the insert.
   - What's unclear: Is the Clerk webhook reliable enough that `users` is always populated before the first scan? In prod: probably yes. In dev: no.
   - Recommendation: In `create_scan`, when `clerk_user_id` is `Some`, do an `INSERT INTO users ... ON CONFLICT DO NOTHING` upsert before inserting the scan. The user's email is available from the JWT claims (it isn't — JWT only has `sub`). Use `sub` only, with a placeholder email, or query Clerk API. Simplest: make the `users` email nullable or omit the upsert and document as a known dev limitation.

2. **Error type for domain verification failure in create_scan**
   - What we know: User wants a "clear error message" linking to `/verify-domain`. The `ApiError::Custom` variant exists for this.
   - What's unclear: Should it be 403 Forbidden or 422 Unprocessable Entity? The domain is valid but the user lacks verified ownership. 403 is semantically correct.
   - Recommendation: Use 403 with `ApiError::Custom` including the `/verify-domain` link in `detail`.

3. **Quota endpoint auth: Claims vs optional**
   - What we know: The quota endpoint requires authentication — anonymous users have no quota.
   - What's unclear: What happens if an unauthenticated user hits `/api/v1/quota`? `Claims<ClerkClaims>` returns 401 automatically via axum-jwt-auth.
   - Recommendation: Use mandatory `Claims<ClerkClaims>` extractor — the 401 response is the correct behavior for anonymous callers.

---

## Sources

### Primary (HIGH confidence — code directly inspected)

- `src/rate_limit/middleware.rs` — current `check_rate_limits` signature and logic
- `src/orchestrator/worker_pool.rs` — `spawn_scan`, `run_scanners`, tier config location
- `src/api/scans.rs` — `create_scan` handler, `AppState`
- `src/api/results.rs` — `extract_optional_clerk_user` pattern (reference implementation)
- `src/api/auth.rs` — `ClerkClaims`, `ClerkUser`, `FromRef<AppState>` impl
- `src/api/errors.rs` — `ApiError` variants, `IntoResponse`
- `src/db/scans.rs` — all existing DB functions including `count_scans_by_ip_today`
- `src/db/domains.rs` — `is_domain_verified` EXISTS query
- `src/models/scan.rs` — `Scan` struct with `tier: String`, `clerk_user_id: Option<String>`
- `migrations/20260218000001_stripe_removal_schema.sql` — tier CHECK constraint, clerk_user_id FK
- `migrations/20260218000002_create_verified_domains.sql` — verified_domains schema
- `frontend/app/actions/scan.ts` — current `submitScan` server action
- `frontend/app/dashboard/page.tsx` — dashboard structure, domains fetch pattern
- `frontend/components/scan-form.tsx` — form component structure
- `frontend/lib/api.ts` — token-forwarding pattern (verifyStart, etc.)
- `frontend/lib/types.ts` — `ScanResponse` already includes `tier: string`
- `frontend/app/results/[token]/page.tsx` — results page structure, data.tier available

### Secondary (MEDIUM confidence)

- `Cargo.toml` — confirmed no new dependencies needed for this phase
- Existing `chrono` 0.4.43 docs — `DATE_TRUNC` equivalent using `NaiveDate` (standard approach)

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all required libraries already present in codebase
- Architecture patterns: HIGH — directly derived from existing code patterns in results.rs, domains.rs
- Pitfalls: HIGH — identified by direct code inspection (FK constraint, normalization mismatch, tier filter gap)
- Frontend patterns: HIGH — auth-forwarding already implemented for domain verification

**Research date:** 2026-02-18
**Valid until:** 2026-03-18 (30 days — stable codebase, no fast-moving dependencies)
