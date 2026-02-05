-- Add results token, expiry, and scan stage tracking columns to scans table

-- Add results token for public access to results (without auth)
ALTER TABLE scans ADD COLUMN results_token VARCHAR(64) UNIQUE;

-- Add expiry timestamp for results token (3 days for free tier)
ALTER TABLE scans ADD COLUMN expires_at TIMESTAMPTZ;

-- Add stage tracking booleans for individual scanner completion
ALTER TABLE scans ADD COLUMN stage_headers BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE scans ADD COLUMN stage_tls BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE scans ADD COLUMN stage_files BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE scans ADD COLUMN stage_secrets BOOLEAN NOT NULL DEFAULT false;

-- Add unique index on results_token for fast lookups (partial index excludes NULLs)
CREATE UNIQUE INDEX idx_scans_results_token ON scans (results_token) WHERE results_token IS NOT NULL;

-- Add missing submitter_ip column from Phase 1 (needed for rate limiting)
ALTER TABLE scans ADD COLUMN IF NOT EXISTS submitter_ip INET;
