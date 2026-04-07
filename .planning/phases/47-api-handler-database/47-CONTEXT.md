# Phase 47: API Handler & Database - Context

**Gathered:** 2026-04-06
**Status:** Ready for planning

<domain>
## Phase Boundary

HTTP endpoint for supply chain scanning, database migration (kind column + JSONB), GitHub URL fetching, request handling for 3 input modes (GitHub URL, upload, paste), result persistence with shareable token, and query audit for kind awareness. This phase wires Phase 46's backend modules to an HTTP API and database.

</domain>

<decisions>
## Implementation Decisions

### Endpoint Design
- **D-01:** Single endpoint `POST /api/v1/scans/supply-chain` handling all 3 input modes via custom Axum extractor with enum dispatch:
  - `Content-Type: application/json` with `github_url` field → GitHub repo scan
  - `Content-Type: application/json` with `lockfile_content` field → pasted lockfile text
  - `Content-Type: multipart/form-data` with `lockfile` field → file upload (max 5MB)
- **D-02:** Route is excluded from Clerk JWT middleware. Uses `extract_optional_clerk_user()` for optional auth. Anonymous scans stored with `user_id = NULL`. Logged-in users auto-associated.
- **D-03:** Synchronous scan — user waits for result. No polling needed. Total handler timeout: 30s via tokio::time::timeout.

### Error-to-HTTP Mapping
- **D-04:** SupplyChainError → HTTP status + RFC 7807 Problem Details JSON (matches existing ApiError pattern):
  | Variant | HTTP | User Message |
  |---|---|---|
  | LockfileParse | 400 | "Invalid lockfile format" |
  | OsvQuery | 502 | "Vulnerability database unavailable" |
  | GitHubFetch | 502 | "Couldn't fetch lockfile from GitHub" |
  | ChunkFailure | 502 | "Vulnerability check failed, try again" |
  | DepCountExceeded | 400 | "Too many dependencies (max 5000)" |
  | Timeout | 504 | "Scan timed out, try a smaller lockfile" |

### GitHub URL Handling
- **D-05:** Parse owner/repo from GitHub URL using strict regex. Construct `https://raw.githubusercontent.com/{owner}/{repo}/{branch}/package-lock.json` server-side. Only `raw.githubusercontent.com` allowed as fetch target.
- **D-06:** Try `main` branch first, fall back to `master`. If both 404, return GitHubFetch error with "No package-lock.json found" message.
- **D-07:** Use `GITHUB_TOKEN` env var if set (5,000 req/hr), fall back to unauthenticated (60 req/hr). GitHub fetch timeout: 5s.

### Abuse Control
- **D-08:** Apply existing per-IP rate limiter from `src/rate_limit/middleware.rs` to supply chain endpoint.
- **D-09:** Dep count cap: 5000 max. Body size limit: 5MB. Both return 400 with clear error message.

### Database Schema
- **D-10:** Migration adds `kind VARCHAR(20) DEFAULT 'web_app'` column to scans table.
- **D-11:** Migration adds `supply_chain_results JSONB` column to scans table (separate from existing web app results).
- **D-12:** Supply chain scan inserts set `expires_at` to NOW() + 30 days explicitly.
- **D-13:** DB write failure returns results inline with "Share link unavailable" warning — never fails the scan itself.

### Query Audit (Critical Paths Only)
- **D-14:** Add `WHERE kind = 'web_app'` filter to these 5 queries only:
  1. Dashboard scan list (user's scan history)
  2. Cleanup task (expired scan deletion)
  3. Per-user quota check (monthly scan count)
  4. Per-target cache check (domain scan frequency)
  5. Stats endpoint (if exists)
  UUID-based lookups (get-by-id, get-by-token) are safe without filtering — one scan, one kind.

### Result Persistence
- **D-15:** Results token generation reuses existing pattern: 256-bit random, base64 URL-safe, stored in `results_token` column.
- **D-16:** Share URLs follow existing `/scan/{uuid}` or equivalent pattern. Supply chain results at `/supply-chain/results/{token}` (separate from web app results page — decided in eng review).

### Claude's Discretion
- GitHub URL regex pattern specifics
- Whether to add SupplyChainError variants to existing ApiError enum or keep separate
- Database migration file naming and numbering
- Exact multipart field names and validation approach

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design & Architecture
- `~/.gstack/projects/TrustEdge-Labs-shipsecure/john-main-design-20260406-133756.md` — Approved supply chain MVP design doc
- `~/.gstack/projects/TrustEdge-Labs-shipsecure/john-main-eng-review-test-plan-20260406-141500.md` — Test plan

### Existing Patterns (MUST READ)
- `src/api/scans.rs` — Existing scan endpoint pattern (create_scan handler, request validation, rate limiting)
- `src/api/errors.rs` — ApiError enum and RFC 7807 Problem Details format
- `src/api/results.rs` — extract_optional_clerk_user() helper, results token handling
- `src/rate_limit/middleware.rs` — Rate limiting implementation
- `src/db/scans.rs` — All scans table queries (audit target for kind filtering)
- `src/main.rs:329-370` — Route registration
- `src/cleanup.rs` — Expired scan cleanup task

### Phase 46 Output (consumed by this phase)
- `src/scanners/lockfile_parser.rs` — parse() function, ParsedDep, DepSource types
- `src/scanners/osv_client.rs` — OsvClient, query_batch()
- `src/scanners/supply_chain.rs` — scan_lockfile(), SupplyChainScanResult, SupplyChainError

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `extract_optional_clerk_user()` in `src/api/results.rs` — optional Clerk auth extraction
- `results_token` generation pattern in `src/orchestrator/worker_pool.rs` — 256-bit random, base64
- `ApiError` enum in `src/api/errors.rs` — RFC 7807 error responses
- `check_rate_limits()` in `src/rate_limit/middleware.rs` — per-IP and per-user limits
- `reqwest` client already in Cargo.toml for GitHub fetching

### Established Patterns
- Scan creation: validate input → rate limit → create DB row → execute → update with results
- Error responses: RFC 7807 Problem Details with `type`, `title`, `status`, `detail`
- Optional auth: `Authorization: Bearer <jwt>` header, extracted but not required

### Integration Points
- New route registered in `src/main.rs` router
- New handler in `src/api/supply_chain.rs`
- New DB functions in `src/db/scans.rs` (or new `src/db/supply_chain.rs`)
- Existing cleanup task in `src/cleanup.rs` needs kind filter

</code_context>

<specifics>
## Specific Ideas

No specific requirements beyond the locked decisions above. Implementation follows existing patterns.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 47-api-handler-database*
*Context gathered: 2026-04-06*
