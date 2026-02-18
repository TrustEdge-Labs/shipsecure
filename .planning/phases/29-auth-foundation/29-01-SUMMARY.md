---
phase: 29-auth-foundation
plan: "01"
subsystem: auth
tags: [clerk, jwt, jwks, svix, axum, cors, webhook, postgres, rs256]

# Dependency graph
requires: []
provides:
  - CORS allows Authorization header — frontend can send JWT bearer tokens without preflight errors
  - ClerkClaims struct and Decoder<ClerkClaims> in AppState — all future protected handlers use Claims<ClerkClaims> extractor
  - ApiError::Unauthorized returning 401 "Authentication required" — generic, no JWT details leaked
  - users table with clerk_user_id UNIQUE, email, timestamps, index
  - POST /api/v1/webhooks/clerk — verifies svix signatures, inserts users on user.created
  - JWKS decoder initialized from CLERK_JWKS_URL env var with RS256 and validate_aud=false
affects:
  - 29-auth-foundation/29-02 (frontend auth uses backend JWT endpoints)
  - all future phases with protected Axum handlers (use Claims<ClerkClaims> pattern)

# Tech tracking
tech-stack:
  added:
    - axum-jwt-auth 0.6.3 (RemoteJwksDecoder, Claims extractor, Decoder type alias)
    - svix 1.85.0 (Webhook::new().verify() for Clerk webhook signature verification)
    - http 1.0 (explicit dependency; Axum HeaderMap is the same type, passes directly to svix)
    - jsonwebtoken 10.3.0 (Validation struct with Algorithm::RS256; axum-jwt-auth uses v10 not v9)
  patterns:
    - "RemoteJwksDecoder is NOT generic — stores Validation internally; generic appears at JwtDecoder<T> impl level"
    - "Decoder<T> = Arc<dyn JwtDecoder<T> + Send + Sync> — AppState stores this, FromRef<AppState> extracts it"
    - "Svix HeaderMap trait supports both http 0.2 and http 1.0 — Axum HeaderMap passes directly without conversion"
    - "sqlx::query() (non-macro) used for new tables to avoid compile-time DB connection requirement"

key-files:
  created:
    - src/api/auth.rs
    - migrations/20260217000001_create_users.sql
  modified:
    - Cargo.toml
    - src/api/errors.rs
    - src/api/mod.rs
    - src/api/scans.rs
    - src/api/webhooks.rs
    - src/main.rs

key-decisions:
  - "jsonwebtoken = '10' not '9' — axum-jwt-auth 0.6 depends on jsonwebtoken 10.x; using 9 would create duplicate types"
  - "RemoteJwksDecoder has no generic parameter — plan said RemoteJwksDecoder::<ClerkClaims>::builder() but struct takes 0 generic args; correct usage is RemoteJwksDecoder::builder() with Decoder<ClerkClaims> type annotation on the Arc"
  - "Axum HeaderMap passes directly to svix::Webhook::verify() — both use http 1.x, no conversion loop needed (simplification over plan spec)"
  - "sqlx::query() non-macro for users INSERT — avoids compile-time DB connection requirement; functionally equivalent"

patterns-established:
  - "Protected handler pattern: Claims<ClerkClaims> as first extractor parameter; claims.sub = Clerk user ID"
  - "Auth error pattern: ApiError::Unauthorized returns 401 with generic message, never JWT details"
  - "Webhook pattern: svix Webhook::new(secret).verify(&body, &headers) — 3 lines, no manual HMAC"

requirements-completed:
  - INFR-01
  - INFR-03
  - INFR-04

# Metrics
duration: 5min
completed: 2026-02-18
---

# Phase 29 Plan 01: Auth Foundation Summary

**Axum backend auth layer: CORS Authorization header fix, RemoteJwksDecoder with ClerkClaims extractor, users table migration, and Clerk webhook with svix signature verification**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-18T01:58:28Z
- **Completed:** 2026-02-18T02:04:05Z
- **Tasks:** 2
- **Files modified:** 8 (6 modified + 2 created)

## Accomplishments

- Backend now accepts `Authorization: Bearer <JWT>` from frontend without CORS preflight errors (INFR-01)
- `Claims<ClerkClaims>` extractor available for all future protected Axum handlers — JWKS keys fetched from Clerk and cached with background refresh, RS256 with `validate_aud=false` (INFR-04)
- `POST /api/v1/webhooks/clerk` verifies svix signatures and inserts users on `user.created` events with ON CONFLICT DO NOTHING (INFR-03)
- `users` table with `clerk_user_id UNIQUE`, `email`, timestamps, and index ready for migration
- `ApiError::Unauthorized` returns 401 "Authentication required" — generic message per security decision

## Task Commits

Each task was committed atomically:

1. **Task 1: CORS fix, dependencies, users migration, Unauthorized error variant, and auth module** - `56fba16` (feat)
2. **Task 2: Clerk webhook handler with svix signature verification** - `5b6e3ae` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `src/api/auth.rs` - ClerkClaims struct (sub, exp, iat, nbf, azp, sid), ClerkUser newtype, FromRef<AppState> for Decoder<ClerkClaims>
- `migrations/20260217000001_create_users.sql` - users table with clerk_user_id UNIQUE, email, timestamps, index
- `Cargo.toml` - Added axum-jwt-auth 0.6, svix 1, http 1.0, jsonwebtoken 10
- `src/api/errors.rs` - Added Unauthorized variant returning 401 "Authentication required"
- `src/api/mod.rs` - Added `pub mod auth;`
- `src/api/scans.rs` - Added `jwt_decoder: Decoder<ClerkClaims>` to AppState
- `src/api/webhooks.rs` - Added handle_clerk_webhook with svix verification and users INSERT
- `src/main.rs` - CORS AUTHORIZATION header, JWKS decoder init, jwt_decoder in AppState, /webhooks/clerk route

## Decisions Made

- **jsonwebtoken 10 not 9:** axum-jwt-auth 0.6.3 depends on jsonwebtoken 10.3.0. Plan specified "9" but that would create a version conflict. Used 10 to match the transitive dependency.
- **RemoteJwksDecoder is not generic:** Plan said `RemoteJwksDecoder::<ClerkClaims>::builder()` but the struct takes 0 generic parameters. The generic is at the `JwtDecoder<T>` trait impl level. Fixed by using `RemoteJwksDecoder::builder()` with explicit `Decoder<ClerkClaims>` type annotation on the Arc.
- **No HeaderMap conversion needed:** Plan specified a conversion loop from Axum HeaderMap to http::HeaderMap. Svix's `HeaderMap` trait directly supports http 1.x `HeaderMap` (which is what Axum uses). The conversion is unnecessary.
- **sqlx::query() for users INSERT:** Used non-macro version to avoid compile-time DB connection requirement in this environment.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] jsonwebtoken version mismatch — used v10 not v9**
- **Found during:** Task 1 (dependency addition + cargo check)
- **Issue:** Plan specified `jsonwebtoken = "9"` but axum-jwt-auth 0.6.3 depends on jsonwebtoken 10.x; using v9 would create incompatible Validation/Algorithm types
- **Fix:** Changed to `jsonwebtoken = "10"` in Cargo.toml
- **Files modified:** Cargo.toml
- **Verification:** cargo check passes with no type conflicts
- **Committed in:** 56fba16 (Task 1 commit)

**2. [Rule 1 - Bug] RemoteJwksDecoder takes no generic arguments**
- **Found during:** Task 1 (cargo check)
- **Issue:** `RemoteJwksDecoder::<ClerkClaims>::builder()` compile error — struct has 0 generic params; generics appear at the JwtDecoder<T> impl level
- **Fix:** Changed to `RemoteJwksDecoder::builder()` with explicit `let jwt_decoder: Decoder<ClerkClaims> = jwks_decoder;` type annotation
- **Files modified:** src/main.rs
- **Verification:** cargo check passes
- **Committed in:** 56fba16 (Task 1 commit)

**3. [Rule 2 - Missing Critical] No HeaderMap conversion needed — Axum HeaderMap passes directly to svix**
- **Found during:** Task 2 (implementation)
- **Issue:** Plan specified a 6-line conversion loop from Axum HeaderMap to http::HeaderMap. Svix already implements its HeaderMap trait for http 1.x HeaderMap (which Axum uses) — the conversion is redundant and adds noise
- **Fix:** Pass Axum `&headers` directly to `wh.verify()` — no conversion
- **Files modified:** src/api/webhooks.rs
- **Verification:** cargo check passes; svix Webhook trait confirmed to support http1::HeaderMap directly
- **Committed in:** 5b6e3ae (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (2 bugs, 1 simplification)
**Impact on plan:** All fixes necessary for correctness. The HeaderMap simplification reduces code. No scope creep.

## Issues Encountered

- Pre-existing test failure in `src/scanners/js_secrets.rs` (`test_false_positive_detection`) confirmed pre-existing before our changes via git stash verification. Logged to `deferred-items.md`. 62 other tests pass.

## User Setup Required

External services require manual configuration before this feature is operational:

**Clerk Dashboard:**
1. Create a Clerk application at https://dashboard.clerk.com → Create Application
2. Get JWKS URL: Clerk Dashboard -> API Keys -> Advanced -> JWKS URL
3. Create webhook endpoint: Clerk Dashboard -> Webhooks -> Add Endpoint
   - URL: `https://shipsecure.ai/api/v1/webhooks/clerk`
   - Events: `user.created`
4. Get Signing Secret from the webhook endpoint page

**Environment variables to add:**
- `CLERK_JWKS_URL` — e.g., `https://your-instance.clerk.accounts.dev/.well-known/jwks.json`
- `CLERK_WEBHOOK_SIGNING_SECRET` — starts with `whsec_`

## Next Phase Readiness

- Backend auth foundation complete — Plan 29-02 (frontend Clerk integration) can proceed
- `Claims<ClerkClaims>` extractor ready for protected handlers in any future plan
- users table migration will run automatically on next deploy
- Clerk webhook is registered and will sync users automatically once dashboard is configured

---
*Phase: 29-auth-foundation*
*Completed: 2026-02-18*

## Self-Check: PASSED

- FOUND: src/api/auth.rs
- FOUND: migrations/20260217000001_create_users.sql
- FOUND: .planning/phases/29-auth-foundation/29-01-SUMMARY.md
- FOUND commit: 56fba16 (Task 1)
- FOUND commit: 5b6e3ae (Task 2)
