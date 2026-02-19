---
phase: 04-monetization
plan: 02
subsystem: payments
tags: [stripe, checkout, webhooks, hmac, payments]

# Dependency graph
requires:
  - phase: 04-01
    provides: paid_audits and stripe_events database tables, CRUD operations
provides:
  - POST /api/v1/checkout - Stripe Checkout Session creation
  - POST /api/v1/webhooks/stripe - Webhook handler with signature verification
  - Payment flow: checkout → webhook → status update → paid scan trigger
affects: [04-03, 04-04, frontend-upgrade-flow]

# Tech tracking
tech-stack:
  added: [async-stripe library already in Cargo.toml]
  patterns:
    - HMAC-SHA256 signature verification for webhooks
    - Idempotency via database-backed event deduplication
    - Stripe Checkout Session with metadata for scan tracking

key-files:
  created:
    - src/api/checkout.rs
    - src/api/webhooks.rs
  modified:
    - src/api/mod.rs
    - src/main.rs

key-decisions:
  - "Manual HMAC signature verification instead of async-stripe built-in for more control"
  - "5-minute timestamp window for replay protection"
  - "Owned strings for Stripe URLs to fix lifetime issues"
  - "Spawn paid scan asynchronously without blocking webhook response"
  - "Return 200 OK immediately for idempotent events to satisfy Stripe retry behavior"

patterns-established:
  - "Webhook signature verification: extract header → parse timestamp and signature → HMAC-SHA256 compute → constant-time compare"
  - "Idempotency pattern: check_and_mark_event returns true for new, false for duplicate"
  - "Stripe metadata: scan_id and email attached to checkout session for webhook correlation"

# Metrics
duration: 2min
completed: 2026-02-06
---

# Phase 4 Plan 2: Stripe Integration Summary

**Stripe Checkout Sessions with HMAC-verified webhooks, idempotent event processing, and async paid scan triggers**

## Performance

- **Duration:** 2 minutes
- **Started:** 2026-02-06T20:40:02Z
- **Completed:** 2026-02-06T20:42:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Checkout endpoint creates Stripe sessions with $49 price and scan metadata
- Webhook handler verifies HMAC-SHA256 signatures with replay protection
- Idempotent event processing via stripe_events table prevents duplicate handling
- Payment completion updates paid_audit status and scan tier to "paid"
- Async paid scan trigger prepared for orchestrator integration in Plan 03

## Task Commits

Each task was committed atomically:

1. **Task 1: Stripe Checkout Session creation endpoint** - `41ec9b6` (feat)
2. **Task 2: Stripe webhook handler with signature verification** - `1425081` (feat)

## Files Created/Modified
- `src/api/checkout.rs` - Creates Stripe Checkout Sessions, validates scan exists and prevents duplicate purchases
- `src/api/webhooks.rs` - Verifies webhook signatures, processes checkout.session.completed events, updates payment status
- `src/api/mod.rs` - Exports checkout and webhooks modules
- `src/main.rs` - Registers /api/v1/checkout and /api/v1/webhooks/stripe routes

## Decisions Made

1. **Manual HMAC signature verification** - Used hmac/sha2/hex crates directly instead of async-stripe's built-in verification for more transparent control over the verification process

2. **5-minute timestamp window** - Stripe webhook timestamps must be within 300 seconds of current time for replay protection (Stripe recommendation)

3. **Owned strings for Stripe URLs** - success_url and cancel_url needed to be owned String values to satisfy Stripe API lifetime requirements, not temporary format! expressions

4. **Async paid scan trigger** - Used tokio::spawn to trigger paid scan asynchronously so webhook returns 200 OK immediately (Stripe best practice: acknowledge fast, process async)

5. **Return 200 for duplicate events** - If check_and_mark_event returns false (duplicate), immediately return 200 OK to satisfy Stripe's retry expectations

## Deviations from Plan

None - plan executed exactly as written. All implementation decisions were within the scope of the plan's guidance.

## Issues Encountered

**Rust lifetime errors in checkout.rs** - Stripe API requires `&str` parameters that must outlive the params struct. Fixed by creating owned String variables (success_url, cancel_url) before passing references to params.

## User Setup Required

**External services require manual configuration.** See [04-USER-SETUP.md](./04-USER-SETUP.md) for:
- STRIPE_SECRET_KEY environment variable (from Stripe Dashboard → API keys)
- STRIPE_WEBHOOK_SECRET environment variable (from Stripe Dashboard → Webhooks)
- Webhook endpoint configuration in Stripe Dashboard
- checkout.session.completed event selection

## Next Phase Readiness

Payment backend complete. Ready for:
- **Plan 04-03**: Tier-aware scanning (trigger paid rescans with extended parameters)
- **Plan 04-04**: PDF report generation (access paid_audits table to determine eligible scans)
- **Frontend integration**: Call POST /api/v1/checkout from "Upgrade" button on free results page

**No blockers.** Webhook placeholder logs "Paid scan triggered" - orchestrator integration happens in Plan 03.

---
*Phase: 04-monetization*
*Completed: 2026-02-06*

## Self-Check: PASSED

All created files verified:
- src/api/checkout.rs ✓
- src/api/webhooks.rs ✓

All commits verified:
- 41ec9b6 (Task 1) ✓
- 1425081 (Task 2) ✓
