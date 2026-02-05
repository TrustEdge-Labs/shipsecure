CREATE TYPE finding_severity AS ENUM ('critical', 'high', 'medium', 'low');

CREATE TABLE findings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_id UUID NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
    scanner_name VARCHAR(100) NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    severity finding_severity NOT NULL,
    remediation TEXT NOT NULL,
    raw_evidence TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_findings_scan_id ON findings (scan_id);
CREATE INDEX idx_findings_severity ON findings (severity);
