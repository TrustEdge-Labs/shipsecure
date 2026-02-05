CREATE TYPE scan_status AS ENUM ('pending', 'in_progress', 'completed', 'failed');

CREATE TABLE scans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    target_url TEXT NOT NULL,
    email TEXT NOT NULL,
    status scan_status NOT NULL DEFAULT 'pending',
    score VARCHAR(2),
    error_message TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_scans_status_created ON scans (status, created_at);
CREATE INDEX idx_scans_email ON scans (email);
