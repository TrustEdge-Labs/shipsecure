ALTER TABLE scans
ADD COLUMN request_id UUID;

CREATE INDEX idx_scans_request_id ON scans (request_id)
WHERE request_id IS NOT NULL;
