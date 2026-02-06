---
phase: 04
plan: 01
subsystem: database
completed: 2026-02-06
duration: 2.9 min

tags: [database, models, stripe, paid-audits, migration]

requires:
  - phase: 03
    plan: 05
    why: Builds on existing scan/finding schema from Phase 3

provides:
  - paid_audits and stripe_events database tables
  - PaidAudit, PaidAuditStatus, StripeEvent Rust models
  - CRUD operations for paid audit lifecycle
  - Webhook idempotency tracking
  - Tier column on scans table for free/paid differentiation
  - Finding cleanup before paid rescans

affects:
  - phase: 04
    plan: 02
    why: Stripe integration will use paid_audits table and webhook idempotency
  - phase: 04
    plan: 03
    why: Tier-aware scanning will check tier column and use clear_findings_by_scan
  - phase: 04
    plan: 04
    why: PDF generation will use mark_pdf_generated and check pdf_generated_at

tech-stack:
  added:
    - async-stripe 0.41 (Stripe API client)
    - genpdf 0.2 (PDF generation for Plan 04)
    - hmac 0.12 (webhook signature verification)
    - sha2 0.10 (webhook signature verification)
    - hex 0.4 (webhook signature verification)
  patterns:
    - Webhook idempotency via INSERT ON CONFLICT DO NOTHING
    - Database-as-queue pattern extended for paid audit lifecycle
    - Status tracking via VARCHAR enum (pending, completed, failed, refunded)
    - Tier differentiation (free vs paid) at scan level

key-files:
  created:
    - migrations/20260206000001_add_paid_audits.sql
    - src/models/paid_audit.rs
    - src/db/paid_audits.rs
  modified:
    - Cargo.toml
    - src/models/scan.rs
    - src/models/mod.rs
    - src/db/mod.rs
    - src/db/scans.rs

decisions:
  - slug: varchar-status-not-enum
    text: Use VARCHAR for paid_audits.status instead of PostgreSQL enum
    why: More flexible for future status additions without migrations, matches existing pattern
  - slug: webhook-idempotency-table
    text: Separate stripe_events table for webhook deduplication
    why: Simpler than tracking processed events in paid_audits, clear separation of concerns
  - slug: tier-at-scan-level
    text: Add tier column to scans table instead of inferring from paid_audits
    why: Faster queries, supports future tiers (e.g., enterprise), denormalized for read performance
  - slug: clear-findings-before-rescan
    text: Provide clear_findings_by_scan for paid tier rescans
    why: Prevents duplicate findings when re-running scanners with extended parameters
  - slug: add-genpdf-now
    text: Add genpdf dependency in Plan 01 not Plan 04
    why: Avoids Cargo.toml conflicts during Plan 04 execution
---

# Phase 4 Plan 1: Paid Audit Database Foundation Summary

**One-liner:** PostgreSQL schema and Rust data layer for paid audit lifecycle tracking, Stripe webhook idempotency, and free/paid tier differentiation.

## What Was Built

Added complete database foundation for Phase 4 monetization features:

1. **Database Schema (Migration 20260206000001)**
   - `paid_audits` table: Tracks audit purchases, Stripe session/payment IDs, status, PDF generation
   - `stripe_events` table: Webhook idempotency via event_id primary key
   - `tier` column on `scans`: Differentiates free vs paid scans (CHECK constraint)
   - Indexes on paid_audits: scan_id, stripe_checkout_session_id, status

2. **Rust Models (src/models/paid_audit.rs)**
   - `PaidAudit` struct: Maps to paid_audits table with sqlx::FromRow
   - `PaidAuditStatus` enum: Pending, Completed, Failed, Refunded (though status stored as VARCHAR in DB)
   - `StripeEvent` struct: Maps to stripe_events table for idempotency tracking

3. **Database Access Layer (src/db/paid_audits.rs)**
   - `create_paid_audit`: Insert new paid audit record
   - `update_paid_audit_status`: Update status and payment_intent_id after webhook
   - `get_paid_audit_by_scan_id`: Fetch audit details for a scan
   - `check_and_mark_event`: Idempotent event processing (INSERT ON CONFLICT DO NOTHING)
   - `mark_pdf_generated`: Timestamp when PDF is generated
   - `update_scan_tier`: Set tier to 'paid' after payment
   - `clear_findings_by_scan`: Delete existing findings before paid rescan

4. **Cargo Dependencies**
   - Added async-stripe, genpdf, hmac, sha2, hex for Plans 02-04

5. **Schema Migration for Existing Queries**
   - Updated all 4 scan queries in src/db/scans.rs to include `tier` column
   - Added `pub tier: String` to Scan struct in src/models/scan.rs

## Key Technical Decisions

**VARCHAR status over PostgreSQL enum:**
Used `status VARCHAR(20)` instead of custom enum type for flexibility. Future status additions (e.g., "disputed", "partially_refunded") won't require schema migrations, and it matches the project's pattern of using string-backed enums in Rust.

**Webhook idempotency via separate table:**
Created `stripe_events` table with `event_id` primary key instead of tracking processed events in `paid_audits`. This separates concerns cleanly — `paid_audits` is business logic, `stripe_events` is infrastructure. The `INSERT ON CONFLICT DO NOTHING` pattern in `check_and_mark_event` makes webhook handling truly idempotent.

**Tier as scan column not join:**
Added `tier VARCHAR(10)` directly to `scans` table instead of inferring from `paid_audits` join. This denormalization speeds up queries (no join needed) and supports future tiers (e.g., "enterprise", "agency") without schema changes.

**Pre-rescan finding cleanup:**
Implemented `clear_findings_by_scan` to delete existing findings before paid rescans. Paid scans run scanners with extended parameters (longer timeouts, more templates), so old free-tier findings must be cleared to prevent duplicates.

**Add genpdf in Plan 01:**
Added `genpdf` dependency now (Plan 01) instead of Plan 04 (PDF generation). This avoids Cargo.toml merge conflicts when multiple plans modify dependencies sequentially.

## Deviations from Plan

None — plan executed exactly as written.

## Task Commits

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Database migration and dependencies | 0a044c5 | migrations/20260206000001_add_paid_audits.sql, Cargo.toml, Cargo.lock |
| 2 | Rust models and database access layer | 49b5d77 | src/models/paid_audit.rs, src/models/scan.rs, src/models/mod.rs, src/db/paid_audits.rs, src/db/mod.rs, src/db/scans.rs |

## Verification Results

✅ All verification criteria met:

- `cargo check` compiles successfully with no errors
- Migration file contains `CREATE TABLE paid_audits`, `CREATE TABLE stripe_events`, `ALTER TABLE scans ADD COLUMN tier`
- `src/models/paid_audit.rs` exports PaidAudit, PaidAuditStatus, StripeEvent
- `src/db/paid_audits.rs` exports all 7 required functions
- Scan struct includes `pub tier: String` field
- All existing scan queries updated to include `tier` column in SELECT/RETURNING clauses

## Next Phase Readiness

**Ready to proceed with:**
- Plan 04-02: Stripe Checkout and webhook integration can use `paid_audits` table and `check_and_mark_event` idempotency
- Plan 04-03: Tier-aware scanning can check `tier` column and call `clear_findings_by_scan` before rescans
- Plan 04-04: PDF generation can use `mark_pdf_generated` and check `pdf_generated_at`

**No blockers.** All database infrastructure for monetization is in place.

## Self-Check: PASSED

All files created and commits verified:

✅ migrations/20260206000001_add_paid_audits.sql exists
✅ src/models/paid_audit.rs exists
✅ src/db/paid_audits.rs exists
✅ Commit 0a044c5 exists
✅ Commit 49b5d77 exists
