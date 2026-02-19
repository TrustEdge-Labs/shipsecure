# Phase 30: Stripe Removal and Schema Cleanup - Research

**Researched:** 2026-02-17
**Domain:** Rust crate removal, PostgreSQL schema migration, frontend component deletion
**Confidence:** HIGH

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CLEN-01 | Remove Stripe checkout flow, paid audit routes, and async-stripe/hmac/sha2/genpdf dependencies | Full inventory of every file that must change is documented below |
| CLEN-02 | Change paid_audits FK to ON DELETE SET NULL; preserve all historical payment records | Migration pattern, FK constraint drop/re-add sequence, and risk of data loss documented |
| CLEN-03 | Add clerk_user_id column to scans; extend tier constraint to include 'authenticated' | Exact ALTER TABLE statements and CHECK constraint replacement pattern documented |
</phase_requirements>

---

## Summary

Phase 30 is a pure deletion/migration task with no new external dependencies. The Stripe integration touches exactly five backend files, one PDF module, two database tables (`paid_audits`, `stripe_events`), and the `spawn_paid_scan` function in the orchestrator. On the frontend, four files must be deleted and two must be edited. The scope is tightly bounded and entirely discoverable from `grep` — no guessing needed.

The primary risk is the `paid_audits` FK migration. The current schema has `ON DELETE CASCADE`, which means deleting any scan row silently deletes its payment record. The new schema must use `ON DELETE SET NULL`. PostgreSQL does not allow modifying an FK action in-place — the constraint must be dropped and re-created. This is a two-statement migration with no downtime risk since the table will have no write traffic after Stripe removal is complete.

The secondary risk is the `tier` CHECK constraint. The current constraint only accepts `'free'` or `'paid'`. Extending it to include `'authenticated'` requires dropping the existing CHECK constraint and adding a new one. PostgreSQL 12+ supports `ALTER TABLE ... DROP CONSTRAINT ... ADD CONSTRAINT ...` in a single migration file. The `clerk_user_id` column add is a simple `ALTER TABLE ADD COLUMN` with a nullable TEXT foreign key referencing `users(clerk_user_id)`.

**Primary recommendation:** Implement as a single plan (30-01) covering all three requirements in sequence: (1) remove Rust code and crates, (2) remove frontend components, (3) run the two-migration database changes, (4) verify compilation and E2E.

---

## Complete Inventory of Files to Change

This is the definitive list derived from `grep` on the actual codebase. The planner should use this as a checklist.

### Backend — Files to DELETE entirely

| File | Why |
|------|-----|
| `src/api/checkout.rs` | Entire file is the Stripe checkout handler. No non-Stripe code. |
| `src/pdf.rs` | Entire file is `genpdf`-based PDF generation. Only called from `webhooks.rs` paid audit path. |

### Backend — Files to EDIT (partial removal)

| File | What to Remove |
|------|---------------|
| `src/api/webhooks.rs` | Remove `handle_stripe_webhook` function (lines 17-131) and `handle_checkout_completed` function (lines 134-348). Keep `handle_clerk_webhook` function (lines 358-421). Remove `use hmac::{Hmac, Mac};`, `use sha2::Sha256;`, `type HmacSha256 = Hmac<Sha256>;`. Keep `use svix::webhooks::Webhook;` and all other imports. |
| `src/api/mod.rs` | Remove `pub mod checkout;` line. Keep all other modules. |
| `src/main.rs` | Remove `use shipsecure::api::{checkout, ...}` — specifically remove `checkout` from the import. Remove route `.route("/api/v1/checkout", post(checkout::create_checkout))`. Remove route `.route("/api/v1/webhooks/stripe", post(webhooks::handle_stripe_webhook))`. Keep clerk webhook route. |
| `src/lib.rs` | Remove `pub mod pdf;` line. Keep all other modules. |
| `src/models/mod.rs` | Remove `pub mod paid_audit;` and `pub use paid_audit::{PaidAudit, PaidAuditStatus, StripeEvent};`. Keep all other exports. |
| `src/db/mod.rs` | Remove `pub mod paid_audits;` line. Keep `scans` and `findings`. |
| `src/orchestrator/worker_pool.rs` | Remove `spawn_paid_scan` method (lines 160-232). Remove the `crate::db::paid_audits::clear_findings_by_scan` call. Remove the `"paid"` branch in `run_scanners` tier match (line 407) — replace with a catch-all defaulting to free-tier config. Remove `use base64::Engine;` import ONLY IF email's `send_paid_audit_email` also gets removed — see note below. |
| `src/email/mod.rs` | Remove `send_paid_audit_email` function (lines 103-224). Keep `send_scan_complete_email`. The `base64` crate usage in `send_paid_audit_email` (line 180) is the ONLY non-test use in the email module. After removing this function, `base64` is still needed in `worker_pool.rs` (line 355: `base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)` — this is the results token generation, NOT Stripe-specific). So `base64` stays in `Cargo.toml`. |

### Backend — Files to DELETE entirely (models)

| File | Why |
|------|-----|
| `src/models/paid_audit.rs` | `PaidAudit`, `PaidAuditStatus`, `StripeEvent` structs — only used by Stripe code |
| `src/db/paid_audits.rs` | All functions only used by Stripe/paid-audit code |

### Cargo.toml — Crates to REMOVE

| Crate | Confirmed Usage | Safe to Remove? |
|-------|-----------------|-----------------|
| `async-stripe` | Only in `src/api/checkout.rs` via `use stripe::...` | YES — confirmed single file |
| `hmac` | Only in `src/api/webhooks.rs` line 5, used in `handle_stripe_webhook` | YES — confirmed single use |
| `sha2` | Only in `src/api/webhooks.rs` line 6, used in `handle_stripe_webhook` | YES — confirmed single use |
| `genpdf` | Only in `src/pdf.rs` | YES — confirmed single file |
| `hex` | Only in `src/api/webhooks.rs` line 83, used in `handle_stripe_webhook` | YES — after removing the stripe webhook handler, no other `hex` usage exists |

**Crates that stay (verified):**
- `base64` — still used in `worker_pool.rs:355` for results token generation
- `svix` — still used in `handle_clerk_webhook` for webhook signature verification

### Frontend — Files to DELETE entirely

| File | Why |
|------|-----|
| `frontend/components/upgrade-cta.tsx` | Entire file is the Stripe checkout CTA |
| `frontend/app/payment/success/page.tsx` | Stripe payment success page |
| `frontend/app/payment/success/layout.tsx` | Layout wrapper for success page |
| `frontend/e2e/paid-audit.spec.ts` | All 4 tests are Stripe/paid-audit E2E tests |
| `frontend/e2e/fixtures/checkout.ts` | Stripe checkout URL fixtures only |
| `frontend/__tests__/components/UpgradeCTA.test.tsx` | All tests are for the UpgradeCTA component |
| `frontend/__tests__/helpers/fixtures/checkout.ts` | Stripe checkout MSW fixtures only |

### Frontend — Files to EDIT (partial removal)

| File | What to Remove |
|------|---------------|
| `frontend/app/results/[token]/page.tsx` | Remove `import { UpgradeCTA } from '@/components/upgrade-cta'`. Remove the block `{data.tier === 'free' && (<div className="mb-6"><UpgradeCTA ...></div>)}`. |
| `frontend/__tests__/helpers/msw/handlers.ts` | Remove `import { checkoutFixtures }`. Remove the checkout handler `http.post('.../checkout', ...)`. Remove the webhook stripe handler `http.post('.../webhooks/stripe', ...)`. Remove `checkoutServerError` from `errorHandlers`. |
| `frontend/__tests__/components/dark-mode.test.tsx` | Remove `import { UpgradeCTA }`. Remove the two tests referencing `UpgradeCTA` (lines 121-127 and 181-187). |
| `frontend/e2e/helpers/route-mocks.ts` | Remove `mockCheckout` function (lines 45-71) and its stripe redirect mock. Keep `mockScanPolling` and `mockNetworkFailure`. |
| `frontend/e2e/free-scan.spec.ts` | Remove assertion on line 55: `await expect(page.locator('text=Upgrade to Deep Audit')).toBeVisible();`. This assertion will fail after UpgradeCTA is deleted. |

---

## Architecture Patterns

### Pattern 1: PostgreSQL CHECK Constraint Replacement

The current `tier` constraint is:
```sql
CHECK (tier IN ('free', 'paid'))
```

PostgreSQL does not support `ALTER TABLE ... ALTER CONSTRAINT`. The only way to change a CHECK constraint is to drop it and add a new one.

```sql
-- Source: PostgreSQL 15 official docs — ALTER TABLE
-- First: discover the constraint name
-- The constraint name was auto-generated by the migration:
-- "scans_tier_check" (PostgreSQL default naming: <table>_<column>_check)

ALTER TABLE scans DROP CONSTRAINT scans_tier_check;
ALTER TABLE scans ADD CONSTRAINT scans_tier_check
  CHECK (tier IN ('free', 'paid', 'authenticated'));
```

**Important:** The constraint name `scans_tier_check` must be verified against the actual database. The naming convention is `<tablename>_<columnname>_check` for inline CHECK constraints added via `ALTER TABLE`. If the constraint was named differently at creation time, the DROP will fail. Use `\d scans` in psql or query `information_schema.check_constraints` to confirm.

**Alternative safe pattern** (avoids name guessing):

```sql
-- Drop all CHECK constraints on 'tier' column then re-add
ALTER TABLE scans DROP CONSTRAINT IF EXISTS scans_tier_check;
ALTER TABLE scans ADD CONSTRAINT scans_tier_check
  CHECK (tier IN ('free', 'paid', 'authenticated'));
```

### Pattern 2: FK Constraint Change (CASCADE to SET NULL)

The current FK in `paid_audits` is:
```sql
scan_id UUID NOT NULL REFERENCES scans(id) ON DELETE CASCADE
```

To change to `ON DELETE SET NULL`:
1. `scan_id` must become nullable (it currently is `NOT NULL`)
2. Drop the existing FK constraint
3. Add a new FK constraint with `ON DELETE SET NULL`

```sql
-- Source: PostgreSQL 15 official docs — ALTER TABLE
-- Step 1: Make scan_id nullable (required for SET NULL behavior)
ALTER TABLE paid_audits ALTER COLUMN scan_id DROP NOT NULL;

-- Step 2: Drop existing FK (auto-named by PostgreSQL)
-- Name will be: paid_audits_scan_id_fkey
ALTER TABLE paid_audits DROP CONSTRAINT IF EXISTS paid_audits_scan_id_fkey;

-- Step 3: Re-add with SET NULL
ALTER TABLE paid_audits ADD CONSTRAINT paid_audits_scan_id_fkey
  FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE SET NULL;
```

**Critical:** The `DROP NOT NULL` on `scan_id` is required. Without it, Postgres cannot set the column to NULL when the referenced scan is deleted — the FK constraint would still fail. This changes the column from `UUID NOT NULL` to `UUID` (nullable).

### Pattern 3: Adding a Nullable FK Column (clerk_user_id on scans)

CLEN-03 requires adding `clerk_user_id` to `scans` with a FK to `users(clerk_user_id)`. The `users` table exists (created in migration `20260217000001_create_users.sql`).

```sql
-- Source: PostgreSQL 15 official docs — ALTER TABLE
-- Nullable so existing free scans (pre-auth) don't need backfill
ALTER TABLE scans ADD COLUMN clerk_user_id TEXT
  REFERENCES users(clerk_user_id) ON DELETE SET NULL;
```

**Why `ON DELETE SET NULL` (not CASCADE or RESTRICT):** If a user account is deleted, we want the scan history preserved but detached — not cascade-deleted. RESTRICT would block user deletion, which is wrong for GDPR compliance.

### Pattern 4: Rust Crate Removal Safety

When removing crates from `Cargo.toml`, the compiler will catch any remaining `use` statements. The safe removal sequence is:

1. Remove all `use` statements and code using the crate in `.rs` files first
2. Then remove the crate from `Cargo.toml`
3. Run `cargo check` — compilation errors catch any missed usages

**Do not** remove from `Cargo.toml` first — the error messages are harder to interpret when the crate is missing entirely.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Verify constraint name before dropping | Custom SQL to look up name | `DROP CONSTRAINT IF EXISTS` + known naming pattern | PostgreSQL auto-naming is deterministic: `<table>_<col>_check` |
| Test that migration is idempotent | Multiple migration runs | sqlx-migrate handles this natively | Migrations are versioned — each runs exactly once |

---

## Common Pitfalls

### Pitfall 1: `hex` Crate Left in Cargo.toml After Webhook Removal
**What goes wrong:** `hex` is imported in `webhooks.rs` and used only by the Stripe signature verification code. If you remove the Stripe handler from `webhooks.rs` but forget to remove `hex` from `Cargo.toml`, the binary will still compile (unused dep is just dead weight). However, the `hex` import in `webhooks.rs` must also be removed or `cargo check` will warn about an unused import.
**How to avoid:** After removing the Stripe handler, run `cargo check` and look for `unused import` warnings.

### Pitfall 2: `base64` Removal Would Break Results Token Generation
**What goes wrong:** The phase requirements list `base64` is NOT in the removal list, but the prior decisions mention removing it. `base64` is used in `worker_pool.rs:355` for generating results tokens (URL-safe encoding of random bytes). Removing it breaks results token generation, which breaks the entire scan completion flow.
**How to avoid:** Do NOT remove `base64` from `Cargo.toml`. It is not listed in CLEN-01 and has active non-Stripe usage.

### Pitfall 3: free-scan E2E Test Fails Because UpgradeCTA Assertion Remains
**What goes wrong:** `frontend/e2e/free-scan.spec.ts` line 55 asserts `'text=Upgrade to Deep Audit'` is visible. After deleting `upgrade-cta.tsx` and removing it from the results page, this assertion will fail, breaking the passing E2E suite.
**How to avoid:** Remove or replace that assertion in `free-scan.spec.ts` as part of the frontend cleanup.

### Pitfall 4: Dark Mode Tests Import UpgradeCTA
**What goes wrong:** `frontend/__tests__/components/dark-mode.test.tsx` imports `UpgradeCTA` and has two test cases for it. After deleting `upgrade-cta.tsx`, vitest will fail to compile the test file.
**How to avoid:** Remove the `UpgradeCTA` import and both test cases from `dark-mode.test.tsx`.

### Pitfall 5: `spawn_paid_scan` Called From Webhook Handler Being Deleted
**What goes wrong:** `spawn_paid_scan` in `worker_pool.rs` is only called from `handle_checkout_completed` in `webhooks.rs`. After `webhooks.rs` is cleaned up, `spawn_paid_scan` becomes dead code. The `paid` branch in `run_scanners`'s tier match also becomes unreachable (no code can set `tier = 'paid'` after Stripe removal). These should be removed to keep the codebase clean.
**How to avoid:** After removing `handle_checkout_completed`, also remove `spawn_paid_scan` and clean up the `run_scanners` tier match to use `_ => free_config` for all non-free tiers (or remove the match entirely, defaulting to free-tier config).

### Pitfall 6: `paid_audits` Table Has `scan_id NOT NULL` — SET NULL Requires Nullable Column
**What goes wrong:** `ON DELETE SET NULL` cannot work if the column is `NOT NULL`. PostgreSQL will raise an error at constraint creation time: `ERROR: column "scan_id" is marked NOT NULL but null values must be allowed`.
**How to avoid:** The migration must `ALTER COLUMN scan_id DROP NOT NULL` before re-adding the FK with `ON DELETE SET NULL`. This changes the column semantics — a `paid_audit` row can now have a null `scan_id`, which is the intended behavior (scan deleted, but payment record preserved).

### Pitfall 7: `paid_audits_scan_id_fkey` Constraint Name Assumption
**What goes wrong:** The FK was created as an inline `REFERENCES` clause in the `CREATE TABLE`. PostgreSQL names such constraints `<table>_<col>_fkey` by convention. However, if that convention was overridden, `DROP CONSTRAINT IF EXISTS paid_audits_scan_id_fkey` will silently do nothing (due to `IF EXISTS`), and the old CASCADE constraint will remain.
**How to avoid:** Before the migration runs, verify with: `SELECT constraint_name FROM information_schema.table_constraints WHERE table_name = 'paid_audits' AND constraint_type = 'FOREIGN KEY';`. The planner should include this as a verification step.

### Pitfall 8: `stripe_events` Table — Should It Be Dropped?
**What goes wrong:** The `stripe_events` table was created for Stripe webhook idempotency. It is no longer used after this phase. If it's not dropped, it's dead schema. If it IS dropped in this migration, and the production DB has rows in it, those rows are simply deleted (no FK references to stripe_events from anything else).
**Recommendation:** Include `DROP TABLE IF EXISTS stripe_events;` in the migration. It has no FK references from other tables, so dropping it is safe.

---

## Code Examples

### Migration File: FK and Tier Changes (30-NNNNN_stripe_removal_schema.sql)

```sql
-- Source: Direct analysis of migrations/20260206000001_add_paid_audits.sql
-- and PostgreSQL 15 official ALTER TABLE docs

-- 1. Change paid_audits FK from CASCADE to SET NULL
--    Must drop NOT NULL first since SET NULL requires a nullable column
ALTER TABLE paid_audits ALTER COLUMN scan_id DROP NOT NULL;
ALTER TABLE paid_audits DROP CONSTRAINT IF EXISTS paid_audits_scan_id_fkey;
ALTER TABLE paid_audits ADD CONSTRAINT paid_audits_scan_id_fkey
  FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE SET NULL;

-- 2. Drop the Stripe events idempotency table (no longer needed)
DROP TABLE IF EXISTS stripe_events;

-- 3. Extend tier CHECK constraint to include 'authenticated'
ALTER TABLE scans DROP CONSTRAINT IF EXISTS scans_tier_check;
ALTER TABLE scans ADD CONSTRAINT scans_tier_check
  CHECK (tier IN ('free', 'paid', 'authenticated'));

-- 4. Add clerk_user_id to scans (nullable FK to users table)
ALTER TABLE scans ADD COLUMN IF NOT EXISTS clerk_user_id TEXT
  REFERENCES users(clerk_user_id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_scans_clerk_user_id ON scans(clerk_user_id);
```

### Cargo.toml After Removal

```toml
# Remove these four lines entirely:
# async-stripe = { version = "0.41", default-features = false, features = ["runtime-tokio-hyper", "checkout"] }
# genpdf = "0.2"
# hex = "0.4"
# hmac = "0.12"
# sha2 = "0.10"

# These stay:
base64 = "0.22"    # used in worker_pool.rs for results token generation
svix = "1"         # used in webhooks.rs for Clerk webhook verification
```

### webhooks.rs After Stripe Removal

The file should retain only:
```rust
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use svix::webhooks::Webhook;
use uuid::Uuid;  // can be removed if not used elsewhere in file

use crate::api::errors::ApiError;
use crate::api::scans::AppState;

/// POST /api/v1/webhooks/clerk - Handle Clerk webhook events
pub async fn handle_clerk_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    // ... (unchanged from current implementation)
}
```

### results page after UpgradeCTA removal

```tsx
// frontend/app/results/[token]/page.tsx
// Remove this import:
// import { UpgradeCTA } from '@/components/upgrade-cta'

// Remove this JSX block:
// {data.tier === 'free' && (
//   <div className="mb-6">
//     <UpgradeCTA scanId={data.id} token={token} />
//   </div>
// )}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| $49 Stripe paid audit | Developer tier via Clerk auth | Phase 30 | Stripe deps removed; `authenticated` tier value needed |
| `paid_audits` FK ON DELETE CASCADE | FK ON DELETE SET NULL | Phase 30 | Historical payment records survive scan deletion |
| `tier` allows only 'free','paid' | 'free','paid','authenticated' | Phase 30 | New auth-gated scan tier supported |

---

## Open Questions

1. **Should `paid` tier value be kept in the CHECK constraint?**
   - What we know: Existing `paid_audits` rows reference scans with `tier = 'paid'`. After Stripe removal, no new scans will be set to `paid`.
   - What's unclear: Whether the retention/cleanup runs will eventually delete all `paid` scans, at which point the `paid` value becomes unreachable.
   - Recommendation: Keep `'paid'` in the CHECK constraint for now. Removing it would require a data migration to update existing rows first. This is out of scope for CLEN-03.

2. **Does `spawn_paid_scan` need any replacement or just deletion?**
   - What we know: It's the orchestrator method for running a paid-tier rescan. It calls `clear_findings_by_scan` and executes a scan with `tier = "paid"` config.
   - What's unclear: Whether authenticated tier scans will eventually need a similar mechanism (they might scan with the same free-tier config initially).
   - Recommendation: Delete `spawn_paid_scan` entirely. The authenticated tier for Phase 30 just stores the `clerk_user_id` on a free-tier scan — no enhanced scanning behavior yet.

3. **`genpdf` confirmation — is it used anywhere besides `src/pdf.rs`?**
   - Research finding: `grep` confirms `genpdf` only appears in `src/pdf.rs`. The module is only called from `handle_checkout_completed` in `webhooks.rs`. After that function is deleted, `pdf.rs` becomes unreachable dead code.
   - Confidence: HIGH (confirmed by direct grep on codebase).

---

## Sources

### Primary (HIGH confidence)
- Direct codebase grep — all file paths and line numbers verified against actual code
- `migrations/20260206000001_add_paid_audits.sql` — confirmed current FK and CHECK constraint definitions
- `migrations/20260217000001_create_users.sql` — confirmed `users` table and `clerk_user_id` column exist
- `Cargo.toml` — confirmed all four crates exist and their exact version strings

### Secondary (MEDIUM confidence)
- PostgreSQL documentation on ALTER TABLE constraint modification — standard pattern, well-established
- PostgreSQL constraint naming convention `<table>_<col>_fkey` / `<table>_<col>_check` — convention-based, verified consistent with auto-generated names in this project

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- File inventory: HIGH — derived from direct `grep` on codebase
- Migration patterns: HIGH — standard PostgreSQL ALTER TABLE
- Frontend changes: HIGH — direct file inspection
- Pitfall list: HIGH — identified from actual code paths

**Research date:** 2026-02-17
**Valid until:** 2026-03-19 (30 days — stable codebase, no external dependency changes needed)
