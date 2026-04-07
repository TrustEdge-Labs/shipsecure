-- Add kind discriminator and supply chain results columns to scans table.
-- kind: distinguishes web_app scans from supply_chain scans. DEFAULT ensures all existing rows get 'web_app'.
-- supply_chain_results: JSONB blob storing SupplyChainScanResult, only populated for supply_chain kind.
ALTER TABLE scans ADD COLUMN kind VARCHAR(20) NOT NULL DEFAULT 'web_app';
ALTER TABLE scans ADD COLUMN supply_chain_results JSONB;
