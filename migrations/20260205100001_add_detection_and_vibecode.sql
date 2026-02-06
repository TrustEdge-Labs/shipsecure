-- Add framework detection columns to scans
ALTER TABLE scans ADD COLUMN detected_framework VARCHAR(50);
ALTER TABLE scans ADD COLUMN detected_platform VARCHAR(50);
ALTER TABLE scans ADD COLUMN stage_detection BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE scans ADD COLUMN stage_vibecode BOOLEAN NOT NULL DEFAULT FALSE;

-- Add vibe_code tag to findings
ALTER TABLE findings ADD COLUMN vibe_code BOOLEAN NOT NULL DEFAULT FALSE;
