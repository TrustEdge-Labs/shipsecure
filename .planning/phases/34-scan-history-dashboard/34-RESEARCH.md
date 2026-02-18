# Phase 34: Scan History Dashboard - Research

**Researched:** 2026-02-18
**Domain:** Axum paginated SQL queries, Next.js 16 server components, paginated table UI, severity badge display
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Scan list presentation:**
- Table rows layout — columns for domain, date, severity counts, expiry, action
- Severity counts displayed as colored number badges (red/orange/yellow/green chips with count)
- Default sort: expiring soonest first — drives urgency to review results before expiry
- Entire row is clickable (navigates to /results/:token) AND has an explicit "View" button in last column for accessibility

**Empty & edge states:**
- Zero-scan empty state is context-aware: show verify-domain CTA if no domains verified, otherwise show run-a-scan CTA
- Expired scans remain visible as dimmed rows with an "Expired" badge — user sees history but results are inaccessible
- In-progress scans displayed in a separate "Active scans" section above the history table, with spinner and status text
- Failed scans shown in history with a red "Failed" badge — user knows the attempt happened

**Quota & sidebar:**
- Sidebar card layout (not top banner) — right sidebar alongside account info
- Sidebar contains: quota card AND verified domains list with status badges
- Quota displayed as text only: "3 of 5 scans used — resets Mar 1"
- At quota limit: scan action (New Scan CTA or equivalent) is disabled/grayed out with tooltip explaining when it resets

**Pagination & density:**
- Traditional numbered page navigation (page 1, 2, 3...)
- 10 scans per page
- No filtering — at 5 scans/month Developer tier, volume stays manageable
- Mobile responsive: table stacks into compact cards on narrow screens

### Claude's Discretion

- Exact table column widths and spacing
- Sidebar card styling and layout details
- Active scans section visual treatment
- Pagination component styling
- Expiry countdown format (e.g., "3 days left" vs "Expires Feb 21")

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| DASH-01 | Authenticated user can view paginated scan history (domain, date, severity counts, expiry) | New `GET /api/v1/users/me/scans` endpoint required; severity counts need aggregation JOIN on findings; pagination via `LIMIT`/`OFFSET`; sorted by expiring soonest first |
| DASH-02 | Dashboard shows quota status ("3 of 5 scans used this month, resets Mar 1") | `GET /api/v1/quota` already exists and returns `{used, limit, resets_at}`; dashboard page already fetches it; sidebar layout is the new addition; "resets Mar 1" formatting from the `resets_at` ISO timestamp |
</phase_requirements>

---

## Summary

Phase 34 is a data-display phase: build a paginated scan history list and reorganize the dashboard into a two-column layout with a right sidebar containing quota + verified domains. The backend requires one new endpoint (`GET /api/v1/users/me/scans`) that returns paginated scan history with severity counts computed via aggregation. The frontend replaces the current simple dashboard page with a richer layout.

The most technically interesting piece is the backend query: severity counts require a `LEFT JOIN` to `findings` with a conditional `COUNT CASE` expression per severity level. This is a standard SQL aggregation pattern that SQLx handles well, but it requires a new `ScanHistoryRow` struct (not the full `Scan` model) to hold the aggregated counts. The existing `Scan` model from `src/models/scan.rs` does not have severity count fields, so a purpose-built response struct is the correct approach.

The frontend is a natural evolution of the existing `frontend/app/dashboard/page.tsx` server component. The page already fetches domains and quota server-side with Clerk JWT. The refactor adds a third server-side fetch for scan history, restructures into a two-column layout, and introduces a new `ScanHistoryTable` sub-component. All three API calls can be done in parallel with `Promise.all` since they are independent. Active scans (in-progress) need a separate DB filter to surface in the "Active scans" section above the main table. Pagination is query-parameter driven (`?page=1`) and handled server-side.

No new dependencies are required on either backend or frontend. The Rust side uses `sqlx` patterns already established in Phase 33. The frontend uses Next.js 16 server component patterns established in Phases 32 and 33, `lucide-react` for icons, and Tailwind utility classes already in use.

**Primary recommendation:** Implement in two sub-plans: 34-01 covers the backend endpoint (new DB query function, new handler, route registration), and 34-02 covers the frontend refactor (two-column layout, scan history table component, active scans section, pagination, sidebar quota/domains card, empty states, tier badge on history cards per Phase 33 deferred item).

---

## Standard Stack

### Core (no new dependencies needed)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `sqlx` | 0.8.6 | Paginated scan history query with severity COUNT aggregation | Already in use; pattern established in `db/domains.rs` and `db/findings.rs` |
| `axum-jwt-auth` | 0.6 | Required (non-optional) Clerk authentication on new endpoint | `Claims<ClerkClaims>` extractor — same pattern as `get_quota` in `scans.rs` |
| `serde_json` | 1 | JSON response serialization for scan history list | Already in use throughout |
| `@clerk/nextjs` | ^6.37.5 | Server-side auth in dashboard page component | `auth()` + `getToken()` — identical to existing dashboard pattern |
| `lucide-react` | ^0.563.0 | Icons in scan history table (spinner, badges) | Already used in `domain-badge.tsx` and throughout components |
| `next` | 16.1.6 | Server component with URL query param for page number | `searchParams` prop on page component — established Next.js 16 pattern |

### No New Dependencies Required

All necessary libraries are already present in `Cargo.toml` and `package.json`. Phase 34 adds no new crates or npm packages.

---

## Architecture Patterns

### Recommended Project Structure Changes

```
src/
├── api/
│   └── users.rs          # NEW: GET /api/v1/users/me/scans handler
├── db/
│   └── scans.rs          # EXTEND: add get_user_scan_history() paginated query
└── main.rs               # EXTEND: register /api/v1/users/me/scans route

frontend/app/dashboard/
├── page.tsx              # REFACTOR: two-column layout, parallel fetches, pagination
└── (no new files needed — keep as single server component)

frontend/components/
├── scan-history-table.tsx    # NEW: table component with row-click, View button, status badges
└── domain-badge.tsx          # EXISTING: reuse for sidebar domains list (already used on dashboard)
```

### Pattern 1: Paginated Aggregation Query with Severity Counts

**What:** A single SQL query returns scan rows with severity counts pre-aggregated. Avoids N+1 queries (one per scan to get findings).

**When to use:** Any time a list view needs counts from a child table. This is the standard SQL approach.

**Example:**
```sql
SELECT
    s.id,
    s.target_url,
    s.status,
    s.results_token,
    s.expires_at,
    s.tier,
    s.created_at,
    COUNT(CASE WHEN f.severity = 'critical' THEN 1 END) AS critical_count,
    COUNT(CASE WHEN f.severity = 'high' THEN 1 END) AS high_count,
    COUNT(CASE WHEN f.severity = 'medium' THEN 1 END) AS medium_count,
    COUNT(CASE WHEN f.severity = 'low' THEN 1 END) AS low_count
FROM scans s
LEFT JOIN findings f ON f.scan_id = s.id
WHERE s.clerk_user_id = $1
GROUP BY s.id, s.target_url, s.status, s.results_token, s.expires_at, s.tier, s.created_at
ORDER BY
    CASE WHEN s.expires_at IS NULL THEN 1 ELSE 0 END,
    s.expires_at ASC NULLS LAST,
    s.created_at DESC
LIMIT $2 OFFSET $3
```

**Confidence:** HIGH — standard PostgreSQL pattern, `LEFT JOIN` ensures scans with no findings return 0 counts.

**Sort logic:** Scans without `expires_at` (in-progress or never assigned tokens) go last. Among scans with expiry, soonest expiring appears first. Secondary sort by `created_at DESC` for stable ordering within same expiry day.

**Important:** The main history table should exclude `in_progress` and `pending` scans (those go in the "Active scans" section). Use `WHERE s.clerk_user_id = $1 AND s.status NOT IN ('pending', 'in_progress')` for the history query. A separate query for active scans uses `WHERE s.clerk_user_id = $1 AND s.status IN ('pending', 'in_progress')`.

### Pattern 2: New Rust Struct for Aggregated Response

**What:** A purpose-built `ScanHistoryRow` struct that holds what the endpoint returns. The existing `Scan` model in `src/models/scan.rs` does not have severity count fields — adding them to `Scan` would pollute the model with display-layer concerns.

**When to use:** Whenever a DB query returns columns that don't map to an existing model struct.

**Example:**
```rust
// In src/api/users.rs (new file) or in src/db/scans.rs
#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct ScanHistoryRow {
    pub id: uuid::Uuid,
    pub target_url: String,
    pub status: String,           // Use String not ScanStatus enum to avoid derive complexity
    pub results_token: Option<String>,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub tier: String,
    pub created_at: chrono::NaiveDateTime,
    pub critical_count: i64,
    pub high_count: i64,
    pub medium_count: i64,
    pub low_count: i64,
}
```

**Confidence:** HIGH — SQLx `FromRow` derive handles COUNT() as i64 automatically (PostgreSQL COUNT returns bigint).

**Note on status:** Using `String` instead of `ScanStatus` enum avoids requiring additional `#[sqlx(...)]` derives on this lightweight struct. The existing `ScanStatus` enum in `src/models/scan.rs` can still be used if preferred — just add `#[sqlx(type_name = "scan_status", rename_all = "snake_case")]`.

### Pattern 3: Parallel Server-Side Fetches in Next.js Server Component

**What:** All three backend calls (scan history, quota, domains) are independent and can be awaited in parallel with `Promise.all`.

**When to use:** Any time a server component needs multiple independent API calls.

**Example:**
```typescript
// In frontend/app/dashboard/page.tsx
const [scansRes, quotaRes, domainsRes] = await Promise.all([
  fetch(`${BACKEND_URL}/api/v1/users/me/scans?page=${page}`, {
    cache: 'no-store',
    headers: sessionToken ? { 'Authorization': `Bearer ${sessionToken}` } : {},
  }),
  fetch(`${BACKEND_URL}/api/v1/quota`, {
    cache: 'no-store',
    headers: sessionToken ? { 'Authorization': `Bearer ${sessionToken}` } : {},
  }),
  fetch(`${BACKEND_URL}/api/v1/domains`, {
    cache: 'no-store',
    headers: sessionToken ? { 'Authorization': `Bearer ${sessionToken}` } : {},
  }),
])
```

**Confidence:** HIGH — established Next.js pattern; server components support `await Promise.all` natively.

### Pattern 4: Page Number from searchParams (Next.js 16 Server Component)

**What:** In Next.js 16 App Router, page components receive a `searchParams` prop as a Promise. Pagination is query-param driven — the URL `/dashboard?page=2` renders page 2.

**Example:**
```typescript
interface DashboardPageProps {
  searchParams: Promise<{ page?: string }>
}

export default async function DashboardPage({ searchParams }: DashboardPageProps) {
  const { page: pageParam } = await searchParams
  const page = Math.max(1, parseInt(pageParam ?? '1', 10))
  const offset = (page - 1) * 10
  // ...
}
```

**Confidence:** HIGH — `searchParams` as Promise is the Next.js 16 App Router convention (confirmed in existing results page which uses `params` as Promise).

### Pattern 5: Two-Column Dashboard Layout with Sidebar

**What:** CSS Grid or flexbox for the main content + sidebar layout. Main content (scan history table) takes most width, sidebar (quota + domains) takes fixed width on desktop, stacks below on mobile.

**Example:**
```tsx
<div className="container mx-auto px-4 py-8 max-w-6xl">
  <div className="flex flex-col lg:flex-row gap-8">
    {/* Main content */}
    <div className="flex-1 min-w-0">
      {/* Active scans section + history table */}
    </div>
    {/* Sidebar */}
    <div className="lg:w-72 shrink-0">
      {/* Quota card + verified domains card */}
    </div>
  </div>
</div>
```

**Confidence:** HIGH — standard Tailwind responsive layout; `lg:flex-row` stacks to column on mobile.

### Pattern 6: Mandatory Auth Endpoint (Claims extractor)

**What:** The scan history endpoint requires authentication — anonymous callers get 401. Use `Claims<ClerkClaims>` extractor (same as `get_quota`), not `extract_optional_clerk_user`.

**Example:**
```rust
// In src/api/users.rs
pub async fn get_user_scans(
    State(state): State<AppState>,
    Claims { claims, .. }: Claims<ClerkClaims>,
    axum::extract::Query(params): axum::extract::Query<ScanHistoryParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let page: i64 = params.page.unwrap_or(1).max(1) as i64;
    let limit: i64 = 10;
    let offset: i64 = (page - 1) * limit;
    // ...
}

#[derive(serde::Deserialize)]
pub struct ScanHistoryParams {
    pub page: Option<u32>,
}
```

**Confidence:** HIGH — established in `get_quota` handler in `scans.rs`.

### Pattern 7: Total Count for Pagination

**What:** Numbered pagination needs total count to know how many pages exist. Run a COUNT query alongside the paginated query. Either a separate query or a `COUNT(*) OVER()` window function.

**Recommended approach:** Separate COUNT query — simpler, easier to understand, not a performance concern at 5 scans/month per user.

```sql
SELECT COUNT(*) FROM scans
WHERE clerk_user_id = $1
  AND status NOT IN ('pending', 'in_progress')
```

**Return shape:**
```json
{
  "scans": [...],
  "total": 47,
  "page": 2,
  "per_page": 10,
  "total_pages": 5
}
```

**Confidence:** HIGH — standard pagination API pattern.

### Anti-Patterns to Avoid

- **N+1 queries:** Do NOT fetch findings separately per scan. Use the aggregation JOIN (Pattern 1) instead.
- **Client-side fetch for scan history:** The dashboard is a server component — fetch everything server-side. Do not add `'use client'` or `useEffect` for the scan list.
- **Sharing the Scan model for the response:** The existing `Scan` struct in `src/models/scan.rs` does not have severity count fields. Use a dedicated `ScanHistoryRow` struct.
- **Using `expires_at` as the only sort key:** Scans without `expires_at` (still in progress, never completed) need `NULLS LAST` or explicit handling to avoid sorting incorrectly.
- **Including in_progress scans in the history table:** Active scans go in their own section. The history query must filter them out with `status NOT IN ('pending', 'in_progress')`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Severity count aggregation | Multiple DB queries per scan | `LEFT JOIN findings` with `COUNT(CASE WHEN ...)` per severity | N+1 queries at scale; SQLx handles this cleanly |
| Pagination logic | Custom offset calculation | Standard `LIMIT $limit OFFSET $offset` with separate COUNT | Proven pattern; no library needed at this scale |
| Domain extraction for row display | Re-parsing the URL in Rust | Just pass `target_url` to frontend, parse hostname in TSX | Frontend has URL parsing; keeps backend thin |
| Date formatting | Custom date library | Native `Date` + `toLocaleDateString` in TypeScript | No extra dependency; sufficient for "Feb 18, 2026" format |

---

## Common Pitfalls

### Pitfall 1: COUNT returns NULL for empty groups without LEFT JOIN

**What goes wrong:** Using `INNER JOIN findings` means scans with no findings are excluded from the result entirely.

**Why it happens:** INNER JOIN requires a match — scans with no findings have no matching finding rows.

**How to avoid:** Always use `LEFT JOIN findings f ON f.scan_id = s.id`. Then `COUNT(CASE WHEN ...)` returns 0 (not NULL) for scans with no findings because COUNT ignores NULL values.

**Warning signs:** User has scans that completed with 0 findings but they don't appear in the list.

### Pitfall 2: SQLx cannot map COUNT() to i32 — must use i64

**What goes wrong:** `COUNT()` in PostgreSQL returns `bigint` (64-bit). SQLx will panic or fail at runtime if the struct field is `i32`.

**Why it happens:** Type mismatch between PostgreSQL `bigint` and Rust `i32`.

**How to avoid:** Define severity count fields as `i64` in `ScanHistoryRow`. Confidence: HIGH (verified by SQLx behavior with existing `count_scans_by_user_this_month` which uses `(i64,)` tuple).

### Pitfall 3: NaiveDateTime vs DateTime<Utc> for expires_at in aggregation query

**What goes wrong:** The existing `Scan` model uses `NaiveDateTime` for `expires_at` because the query casts: `expires_at::timestamp`. The aggregation query must apply the same cast or use the same approach for consistency.

**Why it happens:** PostgreSQL `timestamptz` to Rust type requires explicit handling. The project uses `::timestamp` cast to strip timezone and map to `NaiveDateTime` (established in Phase 30 plan 01).

**How to avoid:** In the aggregation query, cast `expires_at` similarly: `s.expires_at::timestamp AS expires_at`. Match the existing `Scan` model pattern exactly.

### Pitfall 4: searchParams is a Promise in Next.js 16

**What goes wrong:** Accessing `searchParams.page` without awaiting it returns a Promise object, not the string value.

**Why it happens:** Next.js 16 App Router changed `searchParams` to return a Promise (same as `params`). The existing `results/[token]/page.tsx` confirms this pattern with `const { token } = await params`.

**How to avoid:** Always `await searchParams` before accessing fields: `const { page } = await searchParams`. Type the prop as `Promise<{page?: string}>`.

### Pitfall 5: Quota resets_at needs human-readable month formatting

**What goes wrong:** The backend returns `resets_at` as an ISO 8601 timestamp (e.g., `2026-03-01T00:00:00Z`). The context decision requires "resets Mar 1" format (not full ISO).

**Why it happens:** `resets_at` is the first instant of the next month in UTC. "resets Mar 1" is the human display.

**How to avoid:** In the TypeScript component, format `resets_at` as short month + day:
```typescript
function formatResetDate(resetsAt: string): string {
  return new Date(resetsAt).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
  })
}
// "2026-03-01T00:00:00Z" → "Mar 1"
```

### Pitfall 6: Expired scans — results_token access still returns 404

**What goes wrong:** Displaying expired scans in the table with dimmed styling but still linking to `/results/:token` will show a 404 since `get_scan_by_token` filters `WHERE expires_at > NOW()`.

**Why it happens:** The results endpoint intentionally excludes expired scans. The decision says expired scans are "visible as dimmed rows" with "results are inaccessible."

**How to avoid:** For expired rows, do NOT render the "View" button as a link to `/results/:token`. The row should be dimmed and the action column should show an "Expired" badge instead of a clickable View button. The entire-row-is-clickable behavior should also be suppressed for expired rows (or navigate to a clear "expired" state). Determine expiry in the frontend by comparing `expires_at` to `Date.now()`.

### Pitfall 7: Active scans section needs polling or a refresh mechanism

**What goes wrong:** In-progress scans displayed in the "Active scans" section above the history table are fetched once on page load (server component). If a user has a scan running, the section won't auto-update.

**Why it happens:** Server components don't re-render without navigation. The scan may complete while the user is on the page.

**How to avoid:** This is acceptable for a server-rendered dashboard — users can refresh the page. Alternatively, the "Active scans" section could be a client component that polls `/api/v1/scans/:id`. However, given the decision context (simple dashboard, no real-time requirements stated), the simplest approach is to accept that users must refresh. Document this as a known limitation and add a "Refresh" button (simple `router.refresh()` from a small client component) if needed. Do not over-engineer this in Phase 34.

---

## Code Examples

Verified patterns from official sources and existing codebase:

### New DB Function: get_user_scan_history

```rust
// In src/db/scans.rs — add alongside existing functions
// Source: Pattern established in db/domains.rs list_user_domains and findings.rs get_findings_by_scan

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct ScanHistoryRow {
    pub id: uuid::Uuid,
    pub target_url: String,
    pub status: String,
    pub results_token: Option<String>,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub tier: String,
    pub created_at: chrono::NaiveDateTime,
    pub critical_count: i64,
    pub high_count: i64,
    pub medium_count: i64,
    pub low_count: i64,
}

#[allow(dead_code)]
pub async fn get_user_scan_history(
    pool: &PgPool,
    clerk_user_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<ScanHistoryRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ScanHistoryRow>(
        "SELECT
             s.id,
             s.target_url,
             s.status::text AS status,
             s.results_token,
             s.expires_at::timestamp AS expires_at,
             s.tier,
             s.created_at::timestamp AS created_at,
             COUNT(CASE WHEN f.severity = 'critical' THEN 1 END) AS critical_count,
             COUNT(CASE WHEN f.severity = 'high' THEN 1 END) AS high_count,
             COUNT(CASE WHEN f.severity = 'medium' THEN 1 END) AS medium_count,
             COUNT(CASE WHEN f.severity = 'low' THEN 1 END) AS low_count
         FROM scans s
         LEFT JOIN findings f ON f.scan_id = s.id
         WHERE s.clerk_user_id = $1
           AND s.status NOT IN ('pending', 'in_progress')
         GROUP BY s.id, s.target_url, s.status, s.results_token,
                  s.expires_at, s.tier, s.created_at
         ORDER BY
             CASE WHEN s.expires_at IS NULL THEN 1 ELSE 0 END ASC,
             s.expires_at ASC NULLS LAST,
             s.created_at DESC
         LIMIT $2 OFFSET $3"
    )
    .bind(clerk_user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

#[allow(dead_code)]
pub async fn count_user_scans_history(
    pool: &PgPool,
    clerk_user_id: &str,
) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)
         FROM scans
         WHERE clerk_user_id = $1
           AND status NOT IN ('pending', 'in_progress')"
    )
    .bind(clerk_user_id)
    .fetch_one(pool)
    .await?;
    Ok(count.0)
}

#[allow(dead_code)]
pub async fn get_user_active_scans(
    pool: &PgPool,
    clerk_user_id: &str,
) -> Result<Vec<ScanHistoryRow>, sqlx::Error> {
    // Active scans have no findings yet — all counts are 0
    let rows = sqlx::query_as::<_, ScanHistoryRow>(
        "SELECT
             s.id,
             s.target_url,
             s.status::text AS status,
             s.results_token,
             s.expires_at::timestamp AS expires_at,
             s.tier,
             s.created_at::timestamp AS created_at,
             0::bigint AS critical_count,
             0::bigint AS high_count,
             0::bigint AS medium_count,
             0::bigint AS low_count
         FROM scans s
         WHERE s.clerk_user_id = $1
           AND s.status IN ('pending', 'in_progress')
         ORDER BY s.created_at DESC"
    )
    .bind(clerk_user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
```

**Note on status casting:** `s.status::text AS status` casts the PostgreSQL `scan_status` enum to text so SQLx maps it to `String` on the `ScanHistoryRow` struct. This mirrors the approach used for `status` in `get_scan` when building the JSON response (`format!("{:?}", scan.status).to_lowercase()`).

### New Handler: GET /api/v1/users/me/scans

```rust
// In src/api/users.rs (NEW FILE)
use axum::extract::{Query, State};
use axum::Json;
use axum_jwt_auth::Claims;
use serde::Deserialize;
use serde_json::json;

use crate::api::auth::ClerkClaims;
use crate::api::errors::ApiError;
use crate::api::scans::AppState;
use crate::db;

#[derive(Deserialize)]
pub struct ScanHistoryQuery {
    pub page: Option<u32>,
}

pub async fn get_user_scans(
    State(state): State<AppState>,
    Claims { claims, .. }: Claims<ClerkClaims>,
    Query(params): Query<ScanHistoryQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let page = params.page.unwrap_or(1).max(1) as i64;
    let per_page: i64 = 10;
    let offset = (page - 1) * per_page;

    let (scans, active_scans, total) = tokio::try_join!(
        db::scans::get_user_scan_history(&state.pool, &claims.sub, per_page, offset),
        db::scans::get_user_active_scans(&state.pool, &claims.sub),
        db::scans::count_user_scans_history(&state.pool, &claims.sub),
    )?;

    let total_pages = (total + per_page - 1) / per_page;

    Ok(Json(json!({
        "scans": scans,
        "active_scans": active_scans,
        "total": total,
        "page": page,
        "per_page": per_page,
        "total_pages": total_pages,
    })))
}
```

**Note on tokio::try_join!:** Runs the three DB queries concurrently. This requires `tokio` to be in scope (already in Cargo.toml). The `?` operator propagates `sqlx::Error` which converts to `ApiError::InternalError` via the existing `From<sqlx::Error>` impl in `errors.rs`.

### Route Registration in main.rs

```rust
// In src/main.rs — add to existing route block alongside other API routes
use shipsecure::api::users;

// In the Router builder:
.route("/api/v1/users/me/scans", get(users::get_user_scans))
```

### New Type in lib.rs / mod.rs

```rust
// In src/api/mod.rs — add the new module
pub mod users;
```

### Frontend: New ScanHistoryRow Type

```typescript
// In frontend/lib/types.ts — add alongside existing types
export interface ScanHistoryItem {
  id: string
  target_url: string
  status: 'pending' | 'in_progress' | 'completed' | 'failed'
  results_token: string | null
  expires_at: string | null
  tier: string
  created_at: string
  critical_count: number
  high_count: number
  medium_count: number
  low_count: number
}

export interface ScanHistoryResponse {
  scans: ScanHistoryItem[]
  active_scans: ScanHistoryItem[]
  total: number
  page: number
  per_page: number
  total_pages: number
}
```

### Frontend: Severity Badge Component Pattern

```typescript
// Inline in scan-history-table.tsx — matches existing DomainBadge pattern from domain-badge.tsx
// Color choices align with existing design tokens: danger/caution/success/info

function SeverityBadge({ count, level }: { count: number; level: 'critical' | 'high' | 'medium' | 'low' }) {
  if (count === 0) return null
  const styles = {
    critical: 'bg-danger-bg text-danger-text border-danger-border',
    high: 'bg-caution-bg text-caution-text border-caution-border',
    medium: 'bg-info-bg text-info-text border-info-border',
    low: 'bg-success-bg text-success-text border-success-border',
  }
  return (
    <span className={`inline-flex items-center px-1.5 py-0.5 rounded text-xs font-semibold border ${styles[level]}`}>
      {count}
    </span>
  )
}
```

### Frontend: Expiry Countdown Display

```typescript
// Inline helper — used in scan row for expiry column
// Per Claude's Discretion: "3 days left" format chosen for urgency

function formatExpiry(expiresAt: string | null, status: string): React.ReactNode {
  if (status === 'failed') {
    return <span className="text-xs px-1.5 py-0.5 rounded bg-danger-bg text-danger-text border border-danger-border">Failed</span>
  }
  if (!expiresAt) return <span className="text-text-secondary text-sm">—</span>

  const expires = new Date(expiresAt)
  const now = new Date()
  const days = Math.ceil((expires.getTime() - now.getTime()) / (1000 * 60 * 60 * 24))

  if (days <= 0) {
    return <span className="text-xs px-1.5 py-0.5 rounded bg-surface-secondary text-text-secondary border border-border-subtle">Expired</span>
  }
  if (days <= 3) {
    return <span className="text-xs text-caution-text">{days} day{days === 1 ? '' : 's'} left</span>
  }
  return <span className="text-sm text-text-secondary">{days} days left</span>
}
```

### Frontend: Quota Text Display in Sidebar

```typescript
// In sidebar quota card — per locked decision: text only, "3 of 5 scans used — resets Mar 1"
// At limit: New Scan CTA is disabled/grayed with tooltip

function formatResetDate(resetsAt: string): string {
  return new Date(resetsAt).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
  })
  // "2026-03-01T00:00:00Z" → "Mar 1"
}

// Usage:
// quota && `${quota.used} of ${quota.limit} scans used — resets ${formatResetDate(quota.resets_at)}`
```

### Frontend: Tier Badge on History Card (Phase 33 deferred item)

```typescript
// Per Phase 33 Plan 02: "Tier history card badge deferred to Phase 34"
// Add to scan history row — same labels as results page badge

function TierBadge({ tier }: { tier: string }) {
  if (tier === 'authenticated') {
    return (
      <span className="text-xs font-medium px-1.5 py-0.5 rounded-full bg-brand-primary/10 text-brand-primary border border-brand-primary/20">
        Enhanced
      </span>
    )
  }
  return (
    <span className="text-xs font-medium px-1.5 py-0.5 rounded-full bg-surface-secondary text-text-secondary border border-border-subtle">
      Basic
    </span>
  )
}
```

### Frontend: Pagination Component

```typescript
// Minimal numbered pagination — no library needed
// Link-based navigation: <Link href={`/dashboard?page=${n}`}>

interface PaginationProps {
  currentPage: number
  totalPages: number
}

function Pagination({ currentPage, totalPages }: PaginationProps) {
  if (totalPages <= 1) return null
  // Render page number links — simple sequential list
  // At low scan volumes (5/month Developer), totalPages rarely exceeds 1
}
```

### Frontend: Mobile Responsive Table → Cards

```typescript
// The locked decision: "table stacks into compact cards on narrow screens"
// Approach: hide table on mobile, show card list instead using Tailwind responsive classes
// <table className="hidden sm:table ...">
// <div className="sm:hidden space-y-3"> ... card-per-scan </div>
```

---

## Key Implementation Decisions

### Where ScanHistoryRow Should Live

**Decision:** Define `ScanHistoryRow` in `src/db/scans.rs` (not in `src/models/scan.rs` and not in `src/api/users.rs`). It is a DB-layer type used only by DB functions. The handler in `users.rs` uses it via `db::scans::ScanHistoryRow`.

**Rationale:** The existing `Scan` model in `src/models/scan.rs` is a full domain model used throughout the codebase. `ScanHistoryRow` is a query-result projection. Keeping it in `db/scans.rs` follows the established pattern where `db/findings.rs` owns `Finding` query logic without a model module.

**Alternative rejected:** Defining it in `src/models/scan.rs` would add display-layer fields (severity counts) to the domain model, which is wrong separation of concerns.

### Where the Handler Should Live

**Decision:** New file `src/api/users.rs`. The endpoint is `GET /api/v1/users/me/scans`, which logically belongs to a "users" resource. Existing files (`scans.rs`, `domains.rs`, `results.rs`) are already focused on their respective resources.

**Registration:** `src/api/mod.rs` gets `pub mod users;` and `src/main.rs` imports and registers the route.

### Frontend Component Decomposition

**Decision:** Keep the dashboard as a single server component (`app/dashboard/page.tsx`) with a new `ScanHistoryTable` component that accepts pre-fetched data as props. Do NOT make the scan history table a client component — no interactivity requires it. The "View" button and row click are plain anchor/link elements, not event handlers.

**The only scenario requiring 'use client':** If a "Refresh" button for active scans is added. That can be a tiny wrapper (`RefreshButton`) that calls `router.refresh()` — isolated to a small client component, keeping the rest server-rendered.

### Empty State Context Logic

**Decision:** The context-aware empty state (verify-domain CTA vs run-a-scan CTA) depends on whether `domains` is empty. This data is already fetched server-side. Pass both `scans` and `domains` to the component — if `scans.length === 0 && domains.length === 0`, show verify-domain CTA; if `scans.length === 0 && domains.length > 0`, show run-a-scan CTA.

### Expired Row Treatment

**Decision:** Expired scans (where `expires_at < now`) are dimmed with `opacity-60` or `text-text-secondary` and the "View" button is replaced with an "Expired" badge. The row is NOT wrapped in an `<a>` or `<Link>` for expired scans. The `results_token` check alone is insufficient — check `expires_at`.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Separate queries per scan for findings | Aggregation JOIN in single paginated query | Phase 34 (new) | Avoids N+1, correct at scale |
| Dashboard page: simple welcome + domains | Two-column layout: scan history + sidebar | Phase 34 (this phase) | Main dashboard purpose realized |
| Quota badge in inline header area | Sidebar card with quota + domains | Phase 34 (refactor) | Cleaner layout, sidebar persists |

**Prior phase additions that Phase 34 builds on:**
- Phase 33 Plan 02: `getQuotaStyle` inline function in dashboard server component — keep, will be used in sidebar
- Phase 33 Plan 02: Tier history card badge deferred — implement here
- Phase 32 Plan 02: Dashboard uses `BACKEND_URL` (server-only env) — maintain this pattern for new scan history fetch
- Phase 32 Plan 02: `DomainBadge` is a server component — reuse in sidebar domains list

---

## Open Questions

1. **Active scans polling**
   - What we know: Server components don't auto-refresh; active scans section is static on page load
   - What's unclear: Whether a "Refresh" button or manual guidance is sufficient for Phase 34
   - Recommendation: Add a simple "Refresh" client component button that calls `router.refresh()`. This is minimal effort and provides good UX without requiring client-side polling.

2. **`status::text` cast vs ScanStatus enum for ScanHistoryRow**
   - What we know: The existing `Scan` model uses `ScanStatus` enum with sqlx derive; the aggregation query uses `s.status::text` to get a String
   - What's unclear: Whether using `String` vs the enum in `ScanHistoryRow` causes any friction downstream
   - Recommendation: Use `String` for `ScanHistoryRow.status`. The frontend typing (`'pending' | 'in_progress' | 'completed' | 'failed'`) is the authoritative contract. No serialization advantage to keeping the enum for this projection type.

3. **Scan count for pagination at quota**
   - What we know: Developer tier is 5 scans/month; total scan history count depends on historical scans across all months
   - What's unclear: Whether `count_user_scans_history` (all historical non-active scans) might differ significantly from what users expect
   - Recommendation: Count all non-active scans regardless of month — this is the total history across all time, which is correct for pagination.

---

## Sources

### Primary (HIGH confidence)

- Existing codebase: `src/db/scans.rs` — established sqlx query patterns, `NaiveDateTime` casting, `#[allow(dead_code)]` convention
- Existing codebase: `src/api/scans.rs` — `get_quota` handler with `Claims<ClerkClaims>` extractor, `Query` param pattern
- Existing codebase: `src/api/domains.rs` — `list_domains` handler pattern with Claims extractor
- Existing codebase: `frontend/app/dashboard/page.tsx` — server component with parallel fetch pattern, `BACKEND_URL`, `getToken()`
- Existing codebase: `frontend/app/results/[token]/page.tsx` — `searchParams` as Promise in Next.js 16 App Router
- Existing codebase: `frontend/components/domain-badge.tsx` — severity/status badge visual pattern with design tokens
- Existing migration: `20260204000002_create_findings.sql` — `finding_severity` enum values confirmed as 'critical'|'high'|'medium'|'low'
- Existing migration: `20260218000001_stripe_removal_schema.sql` — `tier IN ('free', 'paid', 'authenticated')` constraint confirmed

### Secondary (MEDIUM confidence)

- PostgreSQL `COUNT(CASE WHEN ...)` aggregation pattern: standard SQL, well-documented, no verification needed beyond existing COUNT usage in codebase
- Next.js 16 `Promise.all` in server components: confirmed by existing pattern in dashboard page doing sequential fetches; `Promise.all` is standard JavaScript

### Tertiary (LOW confidence)

- None — all findings verified against the actual codebase

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all libraries already in use with known patterns
- Architecture: HIGH — all patterns verified against existing code in prior phases
- Pitfalls: HIGH — most identified from direct inspection of existing code; SQLx COUNT bigint pitfall from existing usage of `(i64,)` tuple

**Research date:** 2026-02-18
**Valid until:** 2026-03-20 (stable stack; Next.js 16 + Axum 0.8 APIs unlikely to change in this window)
