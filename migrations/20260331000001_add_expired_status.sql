-- Add 'expired' value to the scan_status enum.
-- PostgreSQL enums are immutable; ALTER TYPE ... ADD VALUE is the standard approach.
ALTER TYPE scan_status ADD VALUE IF NOT EXISTS 'expired' AFTER 'failed';
