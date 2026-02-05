-- Add submitter_ip column to scans table for rate limiting
ALTER TABLE scans ADD COLUMN submitter_ip INET;

-- Index for efficient IP-based rate limiting queries
CREATE INDEX idx_scans_ip ON scans (submitter_ip);
