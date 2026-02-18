---
phase: 30-stripe-removal-and-schema-cleanup
plan: "01"
subsystem: database
tags: [stripe, rust, cargo, postgres, migration, clerk, nextjs, vitest]

# Dependency graph
requires:
  - phase: 29-auth-foundation
    provides: users table with clerk_user_id column — FK target for new clerk_user_id column on scans
provides:
  - Cargo.toml without async-stripe, hmac, sha2, genpdf, hex — clean backend dependency list
  - No Stripe route handlers — /api/v1/checkout and /api/v1/webhooks/stripe removed from main.rs
  - migrations/20260218000001_stripe_removal_schema.sql — paid_audits FK ON DELETE SET NULL, stripe_events dropped, tier extended to 'authenticated', clerk_user_id column on scans
  - Frontend builds without UpgradeCTA — results page and all tests clean
affects:
  - 31-results-gating (uses clerk_user_id on scans for ownership; depends on authenticated tier)
  - 33-tiered-access (uses authenticated tier constraint; clerk_user_id for rate limiting)
  - any phase adding scan ownership queries (clerk_user_id column + index ready)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "PostgreSQL FK change pattern: DROP NOT NULL first, then DROP CONSTRAINT IF EXISTS, then ADD CONSTRAINT with ON DELETE SET NULL"
    - "PostgreSQL CHECK constraint replacement: DROP CONSTRAINT IF EXISTS + ADD CONSTRAINT (no ALTER CONSTRAINT in PG)"
    - "Cargo crate removal order: delete Rust code first, then remove from Cargo.toml — compiler errors are clearer this way"

key-files:
  created:
    - migrations/20260218000001_stripe_removal_schema.sql
  modified:
    - Cargo.toml
    - src/api/webhooks.rs
    - src/api/mod.rs
    - src/main.rs
    - src/lib.rs
    - src/models/mod.rs
    - src/db/mod.rs
    - src/orchestrator/worker_pool.rs
    - src/email/mod.rs
    - frontend/app/results/[token]/page.tsx
    - frontend/__tests__/helpers/msw/handlers.ts
    - frontend/__tests__/components/dark-mode.test.tsx
    - frontend/e2e/helpers/route-mocks.ts
    - frontend/e2e/free-scan.spec.ts

key-decisions:
  - "Keep 'paid' in tier CHECK constraint — existing paid_audits rows reference scans with tier='paid'; removing it would require a data migration to update existing rows"
  - "Keep base64 crate — still used in worker_pool.rs for results token generation (URL_SAFE_NO_PAD.encode); not a Stripe dependency"
  - "Keep svix crate — still used in handle_clerk_webhook for signature verification; was never Stripe-specific"
  - "Remove hex crate — only used in Stripe HMAC signature verification; safe to remove after webhook handler deletion"
  - "tier match in run_scanners simplified to tuple assignment — no arms needed after paid arm removal; free-tier config for all tiers"

patterns-established:
  - "Crate removal safety: delete all .rs usage first, then Cargo.toml entries, then cargo check to catch misses"
  - "Migration idempotency: use DROP CONSTRAINT IF EXISTS and ADD COLUMN IF NOT EXISTS to support replay"

requirements-completed:
  - CLEN-01
  - CLEN-02
  - CLEN-03

# Metrics
duration: 6min
completed: 2026-02-18
---

# Phase 30 Plan 01: Stripe Removal and Schema Cleanup Summary

**Removed async-stripe, hmac, sha2, genpdf, hex from Rust backend; deleted 8 Stripe-specific files; created migration for paid_audits FK ON DELETE SET NULL, stripe_events drop, authenticated tier, and clerk_user_id column on scans**

## Performance

- **Duration:** 6 min
- **Started:** 2026-02-18T03:25:07Z
- **Completed:** 2026-02-18T03:31:04Z
- **Tasks:** 2
- **Files modified:** 21 (14 modified + 7 deleted + 1 created migration)

## Accomplishments

- Backend compiles cleanly without 5 Stripe crates (async-stripe, hmac, sha2, genpdf, hex) — `cargo check` clean (CLEN-01)
- Migration file ready: paid_audits FK changes from CASCADE to SET NULL, stripe_events table dropped, tier CHECK extended to include 'authenticated', clerk_user_id TEXT column added to scans with FK to users and index (CLEN-02, CLEN-03)
- Frontend builds without UpgradeCTA — `npm run build` clean, no Stripe references in app/, components/, or tests (CLEN-01)
- 7 Stripe-specific files deleted from backend and frontend; `handle_stripe_webhook`, `handle_checkout_completed`, `spawn_paid_scan`, `send_paid_audit_email` all removed

## Task Commits

Each task was committed atomically:

1. **Task 1: Remove Stripe backend code, crates, and create schema migration** - `a2fca7c` (feat)
2. **Task 2: Remove Stripe frontend components and fix affected tests** - `04ff8b0` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `migrations/20260218000001_stripe_removal_schema.sql` - FK SET NULL for paid_audits, stripe_events DROP, tier constraint with 'authenticated', clerk_user_id column with index
- `Cargo.toml` - Removed async-stripe, genpdf, hex, hmac, sha2 (base64 and svix kept)
- `src/api/webhooks.rs` - Kept handle_clerk_webhook only; removed handle_stripe_webhook, handle_checkout_completed, hmac/sha2/hex imports
- `src/api/mod.rs` - Removed pub mod checkout
- `src/main.rs` - Removed checkout import, /api/v1/checkout and /api/v1/webhooks/stripe routes
- `src/lib.rs` - Removed pub mod pdf
- `src/models/mod.rs` - Removed pub mod paid_audit and PaidAudit/PaidAuditStatus/StripeEvent re-exports
- `src/db/mod.rs` - Removed pub mod paid_audits
- `src/orchestrator/worker_pool.rs` - Removed spawn_paid_scan method; simplified run_scanners tier config to single free-tier tuple
- `src/email/mod.rs` - Removed send_paid_audit_email (only function using base64 in email module)
- `frontend/app/results/[token]/page.tsx` - Removed UpgradeCTA import and conditional JSX block
- `frontend/__tests__/helpers/msw/handlers.ts` - Removed checkoutFixtures import, checkout handler, stripe webhook handler, checkoutServerError error handler
- `frontend/__tests__/components/dark-mode.test.tsx` - Removed UpgradeCTA import and 2 UpgradeCTA test cases
- `frontend/e2e/helpers/route-mocks.ts` - Removed mockCheckout function and Stripe redirect mock
- `frontend/e2e/free-scan.spec.ts` - Removed UpgradeCTA visibility assertion

**Deleted files:**
- `src/api/checkout.rs` - Entire Stripe checkout handler
- `src/pdf.rs` - genpdf PDF generation (only called from paid audit webhook)
- `src/models/paid_audit.rs` - PaidAudit, PaidAuditStatus, StripeEvent structs
- `src/db/paid_audits.rs` - All paid_audit DB functions
- `frontend/components/upgrade-cta.tsx` - Stripe checkout CTA component
- `frontend/app/payment/success/page.tsx` - Stripe payment success page
- `frontend/app/payment/success/layout.tsx` - Payment success layout
- `frontend/e2e/paid-audit.spec.ts` - 4 Stripe/paid-audit E2E tests
- `frontend/e2e/fixtures/checkout.ts` - Stripe checkout URL fixtures
- `frontend/__tests__/components/UpgradeCTA.test.tsx` - UpgradeCTA unit tests
- `frontend/__tests__/helpers/fixtures/checkout.ts` - Checkout MSW fixtures

## Decisions Made

- **Keep 'paid' in tier CHECK:** Existing paid_audits rows reference scans with tier='paid'. Removing it would require a data migration to update all existing rows first — out of scope for this phase.
- **Keep base64 crate:** Used in worker_pool.rs line 355 for URL-safe results token generation (`URL_SAFE_NO_PAD.encode(&bytes)`). Not a Stripe dependency.
- **Keep svix crate:** Used in handle_clerk_webhook for Clerk signature verification. Never Stripe-specific.
- **Remove hex crate:** Only usage was in handle_stripe_webhook for HMAC hex encoding. Safe to remove after that handler is deleted.
- **Simplify tier match to tuple:** After removing the "paid" arm, the match had only `_ => (free_config)`. Replaced with a direct tuple assignment — cleaner, no dead code.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

**Pre-existing Header ClerkProvider test failures (not caused by this phase):**
- `frontend/__tests__/components/Header.test.tsx` — 4 tests fail with `SignedOut can only be used within <ClerkProvider>`
- `dark-mode.test.tsx` — 2 Header tests fail with same error
- These failures existed before Phase 30 began (verified by git stash comparison showing same 6 failures on Task 1 commit)
- Root cause: Phase 29 added Clerk SignedOut/SignedIn to Header without updating test mocks
- Documented in `deferred-items.md` for Phase 30

## User Setup Required

None - no external service configuration required. Migration will apply automatically on next deploy.

## Next Phase Readiness

- Phase 31 (Results Gating) can use `clerk_user_id` column on scans to associate authenticated scans with users
- Phase 33 (Tiered Access) can use `authenticated` tier in the CHECK constraint and `clerk_user_id` for rate limiting
- Backend has no Stripe dead code — clean foundation for authenticated scan ownership
- Migration is ready to run; no data migration needed for existing free/paid scans

---
*Phase: 30-stripe-removal-and-schema-cleanup*
*Completed: 2026-02-18*
