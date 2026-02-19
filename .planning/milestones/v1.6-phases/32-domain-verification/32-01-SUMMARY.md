---
phase: 32-domain-verification
plan: "01"
subsystem: api
tags: [rust, axum, jwt, clerk, domain-verification, meta-tag, ssrf, scraper, postgres]

# Dependency graph
requires:
  - phase: 31-results-gating
    provides: owner_verified field and gating pattern in results.rs; extract_optional_clerk_user helper
  - phase: 29-auth-foundation
    provides: AppState.jwt_decoder (Decoder<ClerkClaims>), Claims<T> extractor via FromRef
  - phase: 30-stripe-removal-and-schema-cleanup
    provides: users table with clerk_user_id TEXT PRIMARY KEY (FK target for verified_domains)

provides:
  - verified_domains table migration with unique (clerk_user_id, domain) constraint and 30-day TTL
  - Five DB query functions: get_verified_domain, upsert_pending_domain, mark_verified, list_user_domains, is_domain_verified
  - Four domain API endpoints: verify-start, verify-confirm, verify-check, GET list
  - Shared-hosting TLD blocklist (root-only: github.io, vercel.app, netlify.app, pages.dev)
  - SSRF-protected outbound fetch for meta tag verification in verify-confirm and verify-check
  - owner_verified extended to require active domain verification (identity + domain both required)
  - Expired domain re-gates results for scan owner

affects: [32-domain-verification-frontend, 33-rate-limiting]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Claims struct destructuring: Claims { claims, .. }: Claims<ClerkClaims> — axum-jwt-auth 0.6.3 uses struct not tuple"
    - "r#gen() for Rust 2024 edition: rand::thread_rng().r#gen() — gen is a reserved keyword in edition 2024"
    - "sqlx::query_as::<_, VerifiedDomain>() non-macro pattern for rows returning full structs"
    - "sqlx::query_as::<(bool,)> for EXISTS queries returning single bool column"
    - "SSRF validation before outbound fetch: ssrf::validate_scan_target(&target_url).await.map_err() in domain handlers"
    - "Two-step owner_verified: identity match (caller == owner) AND is_domain_verified — both required"

key-files:
  created:
    - migrations/20260218000002_create_verified_domains.sql
    - src/models/domain.rs
    - src/db/domains.rs
    - src/api/domains.rs
  modified:
    - src/models/mod.rs
    - src/db/mod.rs
    - src/api/mod.rs
    - src/api/results.rs
    - src/main.rs

key-decisions:
  - "Claims struct pattern not tuple: axum-jwt-auth 0.6.3 Claims<T> is a struct with .claims field — use Claims { claims, .. } destructuring"
  - "r#gen for Rust 2024: gen is a reserved keyword, use r#gen or explicit trait method call"
  - "TagInBody is soft warning — verification succeeds because meta tag proves HTML control regardless of placement"
  - "unwrap_or(false) on is_domain_verified DB error — fail-safe: never grant access on DB error"
  - "extract_domain_from_url in results.rs matches normalize_domain in domains.rs — consistent www-stripping and lowercasing prevents Pitfall 6 normalization mismatch"

patterns-established:
  - "Two-step gating: identity match (caller == scan owner) AND is_domain_verified — expired domain re-gates results"
  - "Domain normalization: strip www., lowercase, url::Url::parse for consistent host extraction"
  - "Blocklist as hardcoded const &[&str] — exact match only, subdomains of blocked roots are allowed"
  - "SSRF gate before every outbound fetch — validate_scan_target before fetch_and_check_meta_tag"

requirements-completed: [DOMN-01, DOMN-02, DOMN-04, DOMN-05]

# Metrics
duration: 4min
completed: 2026-02-18
---

# Phase 32 Plan 01: Domain Verification Summary

**Full backend domain verification system: verified_domains migration, verify-start/confirm/check/list API with shared-hosting TLD blocklist and meta tag scraping, owner_verified extended to require active domain verification**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-18T14:10:25Z
- **Completed:** 2026-02-18T14:14:33Z
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments

- Created verified_domains migration with unique (clerk_user_id, domain) constraint, verification_token UNIQUE, status CHECK ('pending'/'verified'), and 30-day expiry columns
- Implemented five DB query functions using sqlx non-macro pattern: get_verified_domain, upsert_pending_domain (ON CONFLICT reset), mark_verified, list_user_domains, is_domain_verified (EXISTS query)
- Built four domain API handlers with: normalize_domain helper, shared-hosting root TLD blocklist (exact-match only), opaque 256-bit token generation, SSRF-protected meta tag verification via scraper crate, 30-day expiry on mark_verified
- Extended owner_verified in both results handlers to require identity match AND active domain verification — expired verification re-gates past scan results

## Task Commits

Each task was committed atomically:

1. **Task 1: Create verified_domains migration, model, and DB query module** - `63d7f93` (feat)
2. **Task 2: Create domain API handlers with normalization, blocklist, SSRF protection, and meta tag verification** - `ac19739` (feat)
3. **Task 3: Extend owner_verified in results.rs to require active domain verification** - `f368928` (feat)

**Plan metadata:** _(docs commit pending)_

## Files Created/Modified

- `migrations/20260218000002_create_verified_domains.sql` - verified_domains table with unique (clerk_user_id, domain) constraint, token UNIQUE, status CHECK, verified_at/expires_at columns
- `src/models/domain.rs` - VerifiedDomain struct with sqlx::FromRow + Serialize
- `src/db/domains.rs` - Five DB query functions for verified_domains table
- `src/api/domains.rs` - Four API handlers: verify_start, verify_confirm, verify_check, list_domains; domain normalization; blocklist; token generation; meta tag verification
- `src/models/mod.rs` - Added pub mod domain + pub use domain::VerifiedDomain
- `src/db/mod.rs` - Added pub mod domains
- `src/api/mod.rs` - Added pub mod domains
- `src/api/results.rs` - Added extract_domain_from_url helper; extended owner_verified in both get_results_by_token and download_results_markdown
- `src/main.rs` - Registered four /api/v1/domains/ routes; imported domains module

## Decisions Made

- `Claims { claims, .. }` struct destructuring required — axum-jwt-auth 0.6.3 `Claims<T>` is a struct with a `.claims` field, not a tuple struct; plan's `Claims(claims)` pattern was incorrect
- `r#gen()` raw identifier required — `gen` is a reserved keyword in Rust 2024 edition; plan's `.gen::<[u8; 32]>()` call fails to compile
- `TagInBody` treated as soft warning, verification still succeeds — meta tag anywhere in HTML proves control of the page's output
- `unwrap_or(false)` on `is_domain_verified` — fail-safe: DB errors never grant access, only deny

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed Claims extractor pattern for axum-jwt-auth 0.6.3**
- **Found during:** Task 2 (cargo check)
- **Issue:** Plan specified `Claims(claims): Claims<ClerkClaims>` (tuple destructuring), but `Claims<T>` is a struct with a `.claims` field — not a tuple struct. Compiler error E0532 "expected tuple struct or tuple variant, found struct Claims"
- **Fix:** Changed to `Claims { claims, .. }: Claims<ClerkClaims>` struct pattern in all four handlers
- **Files modified:** src/api/domains.rs
- **Verification:** cargo check passes with zero errors
- **Committed in:** `ac19739` (Task 2 commit)

**2. [Rule 1 - Bug] Fixed `gen` reserved keyword in Rust 2024 edition**
- **Found during:** Task 2 (cargo check)
- **Issue:** Plan specified `rand::thread_rng().gen::<[u8; 32]>()` but `gen` is a reserved keyword in Rust 2024 edition (Cargo.toml specifies `edition = "2024"`). Compiler error: "expected identifier, found reserved keyword `gen`"
- **Fix:** Changed to `rand::thread_rng().r#gen()` using raw identifier syntax
- **Files modified:** src/api/domains.rs
- **Verification:** cargo check passes with zero errors
- **Committed in:** `ac19739` (Task 2 commit)

**3. [Rule 1 - Bug] Removed unused `Serialize` import**
- **Found during:** Task 2 (compiler warning)
- **Issue:** `use serde::{Deserialize, Serialize}` — `Serialize` unused since request types are Deserialize-only
- **Fix:** Changed to `use serde::Deserialize`
- **Files modified:** src/api/domains.rs
- **Verification:** Zero warnings related to this import
- **Committed in:** `ac19739` (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (all Rule 1 — compile-time bugs in plan's Rust syntax)
**Impact on plan:** All fixes were compile errors surfaced by cargo check. No functional changes — the plan's intent was correct; only the Rust syntax for the specific library version and edition needed adjustment.

## Issues Encountered

None — plan executed smoothly. All three compile errors were immediately identifiable from cargo check output and fixed in one round.

## User Setup Required

None — no external service configuration required. Migration runs automatically via `sqlx::migrate!()` on next startup. The `users` table (FK target for `verified_domains.clerk_user_id`) was created in Phase 29.

## Next Phase Readiness

- Backend domain verification complete — Phase 32 Plan 02 (frontend verification wizard) can now build on the four `/api/v1/domains/` endpoints
- `POST /api/v1/domains/verify-start` returns `{ domain, token, meta_tag }` or `{ already_verified: true, expires_in_days }`
- `POST /api/v1/domains/verify-confirm` fetches target URL (SSRF-validated), parses meta tag with scraper, marks verified with 30-day expiry
- `POST /api/v1/domains/verify-check` pre-check without DB write for "Test my tag" button
- `GET /api/v1/domains` returns user's domain list for dashboard display
- Results API `owner_verified` is now `true` ONLY when caller is scan owner AND domain has active (non-expired) verification

## Self-Check: PASSED

All created files exist on disk:
- migrations/20260218000002_create_verified_domains.sql: FOUND
- src/models/domain.rs: FOUND
- src/db/domains.rs: FOUND
- src/api/domains.rs: FOUND
- .planning/phases/32-domain-verification/32-01-SUMMARY.md: FOUND

All task commits exist in git history:
- 63d7f93 (Task 1): FOUND
- ac19739 (Task 2): FOUND
- f368928 (Task 3): FOUND

---
*Phase: 32-domain-verification*
*Completed: 2026-02-18*
