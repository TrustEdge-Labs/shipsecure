-- Add paid audit tracking table
CREATE TABLE paid_audits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_id UUID NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
    stripe_checkout_session_id VARCHAR(255) NOT NULL UNIQUE,
    stripe_payment_intent_id VARCHAR(255),
    amount_cents INTEGER NOT NULL,
    currency VARCHAR(3) DEFAULT 'usd',
    customer_email VARCHAR(255) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    pdf_generated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for paid_audits
CREATE INDEX idx_paid_audits_scan_id ON paid_audits(scan_id);
CREATE INDEX idx_paid_audits_checkout_session ON paid_audits(stripe_checkout_session_id);
CREATE INDEX idx_paid_audits_status ON paid_audits(status);

-- Stripe webhook idempotency table
CREATE TABLE stripe_events (
    event_id VARCHAR(255) PRIMARY KEY,
    processed_at TIMESTAMPTZ DEFAULT NOW()
);

-- Add tier column to scans table
ALTER TABLE scans ADD COLUMN tier VARCHAR(10) DEFAULT 'free' CHECK (tier IN ('free', 'paid'));
