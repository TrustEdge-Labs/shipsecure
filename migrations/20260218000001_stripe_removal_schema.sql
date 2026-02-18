-- Phase 30: Stripe Removal and Schema Cleanup
-- CLEN-02: Change paid_audits FK from CASCADE to SET NULL
-- CLEN-03: Extend tier constraint and add clerk_user_id

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
