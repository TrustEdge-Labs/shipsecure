---
phase: 31-results-gating
plan: "01"
subsystem: api
tags: [rust, axum, jwt, clerk, gating, results, authorization]

# Dependency graph
requires:
  - phase: 29-auth-foundation
    provides: AppState.jwt_decoder (Decoder<ClerkClaims>) and JwtDecoder::decode() trait
  - phase: 30-stripe-removal-and-schema-cleanup
    provides: clerk_user_id TEXT column added to scans table

provides:
  - Server-side results gating: high/critical findings stripped for non-owners
  - extract_optional_clerk_user() helper for optional JWT in Axum handlers
  - owner_verified field in results API response
  - gated field per-finding in results API response
  - Markdown download endpoint also applies gating (prevents curl bypass of /download)

affects: [32-domain-verification, 33-rate-limiting, frontend-results-page]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Optional JWT extraction: manually extract Bearer token and call state.jwt_decoder.decode() — do NOT use Claims<T> extractor for optional auth (it rejects all anonymous requests with 401)"
    - "owner_verified via (Some, Some) match: None == None must return false to prevent anonymous scans from appearing as owned"
    - "Server-side field stripping: null description/remediation in JSON response; gated: true flag signals frontend"

key-files:
  created: []
  modified:
    - src/models/scan.rs
    - src/db/scans.rs
    - src/api/results.rs

key-decisions:
  - "Gate high/critical findings for ALL scans regardless of tier — tier is irrelevant; gating is based on severity + caller identity only"
  - "None == None returns owner_verified: false — anonymous scans (clerk_user_id IS NULL) are always gated for anonymous callers"
  - "download_results_markdown also applies gating — consistent with OWASP A01; curl to /download cannot leak gated content"
  - "ClerkClaims import removed from results.rs — not needed; jwt_decoder.decode() returns TokenData<ClerkClaims> and .claims.sub provides the user ID directly"

patterns-established:
  - "Optional auth pattern: extract_optional_clerk_user(&state, &headers) -> Option<String> pattern for routes that serve both anonymous and authenticated callers"
  - "Gating pattern: is_gated = !owner_verified && matches!(severity, High | Critical); two JSON shapes based on is_gated flag"

requirements-completed: [GATE-01, GATE-02]

# Metrics
duration: 2min
completed: 2026-02-18
---

# Phase 31 Plan 01: Results Gating Summary

**Server-side API results gating: high/critical findings stripped for non-owners via optional JWT extraction and owner_verified computation in Axum handlers**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-18T11:47:15Z
- **Completed:** 2026-02-18T11:49:25Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added `clerk_user_id: Option<String>` to Scan struct and all four `query_as::<_, Scan>` queries to prevent runtime column mismatch
- Implemented `extract_optional_clerk_user()` helper that manually extracts Bearer token and calls `state.jwt_decoder.decode()` without using the mandatory `Claims<T>` extractor
- `get_results_by_token` now computes `owner_verified` using `(Some, Some)` match pattern, gates high/critical findings for non-owners (null description/remediation + `gated: true`), and returns `owner_verified` at the top level
- `download_results_markdown` applies the same gating logic, replacing gated fields with sign-up CTAs to prevent `curl /download` bypass

## Task Commits

Each task was committed atomically:

1. **Task 1: Add clerk_user_id to Scan struct and all SELECT queries** - `44441d8` (feat)
2. **Task 2: Implement optional JWT extraction and gating logic in results handlers** - `33c2b82` (feat)

**Plan metadata:** _(docs commit pending)_

## Files Created/Modified

- `src/models/scan.rs` - Added `clerk_user_id: Option<String>` field after `created_at`
- `src/db/scans.rs` - Added `clerk_user_id` to all four `query_as::<_, Scan>` column lists (create_scan RETURNING, get_scan SELECT, claim_pending_scan RETURNING, get_scan_by_token SELECT)
- `src/api/results.rs` - Added `extract_optional_clerk_user()`, updated both handlers with optional JWT, owner_verified computation, gating logic, and `gated`/`owner_verified` fields in responses

## Decisions Made

- Gate ALL scans regardless of tier — tier is irrelevant to gating; only severity + caller identity matter
- `None == None` returns `owner_verified: false` — anonymous scans (no `clerk_user_id`) are always gated for anonymous callers, preventing false-positive access
- `download_results_markdown` also gates — consistent OWASP A01 posture; markdown download cannot bypass gating via `curl`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed unused `ClerkClaims` import from results.rs**
- **Found during:** Task 2 (cargo check warning)
- **Issue:** Plan suggested importing `ClerkClaims` but the decode() return type inference means it is not needed directly in results.rs — `state.jwt_decoder.decode(token).await.ok()?.claims.sub` works without explicit import
- **Fix:** Removed `use crate::api::auth::ClerkClaims;` that would have generated a compiler warning
- **Files modified:** src/api/results.rs
- **Verification:** cargo check passes with zero errors and zero new warnings
- **Committed in:** `33c2b82` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — unused import removed)
**Impact on plan:** Minor cleanup. No functional change; plan executed as designed.

## Issues Encountered

None — plan executed smoothly. The research file had complete, accurate patterns for optional JWT extraction and gating logic.

## User Setup Required

None - no external service configuration required. The `clerk_user_id` column was already added to the database in Phase 30 (migration `20260218000001_stripe_removal_schema.sql`).

## Next Phase Readiness

- Backend results gating complete — Phase 31 Plan 02 (frontend AuthGate component) can now build on the `gated` and `owner_verified` fields
- `curl GET /api/v1/results/:token` without Authorization returns `gated: true` + null description/remediation for high/critical findings
- `curl GET /api/v1/results/:token` with valid owner JWT returns `owner_verified: true` + full details
- `curl GET /api/v1/results/:token/download` also applies gating — consistent with API endpoint

---
*Phase: 31-results-gating*
*Completed: 2026-02-18*
