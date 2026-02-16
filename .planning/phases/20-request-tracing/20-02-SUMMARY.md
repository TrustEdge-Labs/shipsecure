---
phase: 20-request-tracing
plan: 02
subsystem: observability
tags: [tracing, correlation-ids, request-propagation, middleware]
requires: [phase-20-01-request-tracing-infrastructure]
provides: [end-to-end-request-tracing, request-id-propagation]
affects: [http-handlers, orchestrator, database-layer, scan-logging]
tech_stack:
  added:
    - Axum Extension extractor for RequestId
    - Middleware request_id injection
  patterns:
    - Extension-based request context passing
    - Shared field approach for span correlation (not parent-child)
    - Optional request_id for non-HTTP scan triggers
key_files:
  created: []
  modified:
    - src/lib.rs
    - src/main.rs
    - src/api/scans.rs
    - src/orchestrator/worker_pool.rs
    - src/api/webhooks.rs
decisions:
  - "RequestId defined in lib.rs for library-wide access (not main.rs binary crate)"
  - "Middleware ordering: inject_request_id -> trace_layer -> cors (bottom-to-top with .layer())"
  - "Extension extractor in handler - returns 500 if missing (indicates middleware misconfiguration)"
  - "Webhook-triggered paid scans pass None for request_id (background tasks, not HTTP-originated)"
  - "Shared field approach for request_id in scan spans (not parent-child span linking)"
metrics:
  duration: 2 minutes
  tasks: 2
  files: 5
  commits: 2
  completed: 2026-02-16
---

# Phase 20 Plan 02: Request ID Propagation

**One-liner:** RequestId flows end-to-end from HTTP middleware through handler to database storage and orchestrator scan spans, enabling complete request lifecycle tracing via shared field correlation.

## Overview

Completed the request tracing implementation for TrustEdge Audit by wiring request_id from the TraceLayer middleware through to API handlers, database writes, and orchestrator scan spans. A user-generated HTTP request now produces a single request_id that appears in all logs: the HTTP request span, handler processing, database insert, scan execution, and all scanner operations. This enables operators to search logs by request_id and see the complete lifecycle of a single user request.

The implementation uses Axum's Extension mechanism for clean context passing and a shared field approach for span correlation (request_id is a field on scan spans, not a parent span reference). Paid scans triggered by Stripe webhooks pass None for request_id since they're background tasks re-triggering an existing scan.

## Tasks Completed

### Task 1: Add RequestId extension type and wire request_id through create_scan handler
**Commit:** `a790c1f`

Added RequestId newtype and middleware infrastructure to propagate request_id from TraceLayer to API handlers:

**Key changes:**
- **RequestId newtype in lib.rs:** Defined `pub struct RequestId(pub uuid::Uuid)` in the library crate for use across modules
- **inject_request_id middleware:** New async middleware function generates UUID v4 and inserts RequestId extension before TraceLayer
- **TraceLayer reads from extensions:** Updated make_span_with closure to read request_id from extensions instead of generating it
- **Middleware ordering:** inject_request_id (bottom) -> trace_layer -> cors (top) ensures request_id is available when TraceLayer creates span
- **Handler extraction:** create_scan handler extracts RequestId via `Extension<RequestId>` extractor
- **Database and orchestrator wiring:** Handler passes request_id.0 to both db::scans::create_scan and orchestrator.spawn_scan

**Implementation notes:**
- Placed RequestId in lib.rs (not main.rs) to allow import by both binary and library modules
- Extension extractor returns 500 if missing - appropriate error for middleware misconfiguration
- Middleware uses .layer() bottom-to-top ordering: inject_request_id runs first, then trace_layer sees the extension

**Files modified:** `src/lib.rs`, `src/main.rs`, `src/api/scans.rs`

### Task 2: Update orchestrator spawn methods and webhook handler for request_id propagation
**Commit:** `f842353`

Updated orchestrator spawn methods to accept request_id and include it as a shared field on scan spans:

**Key changes:**
- **spawn_scan signature:** Added `request_id: Option<Uuid>` parameter, added request_id as field on scan span
- **spawn_paid_scan signature:** Added `request_id: Option<Uuid>` parameter, added request_id as field on scan span
- **Shared field approach:** request_id is a field on the span, NOT a parent span reference - honors phase 20 design decision
- **Empty string for None:** `request_id.map(|id| id.to_string()).as_deref().unwrap_or("")` displays empty field cleanly in JSON logs
- **Webhook handler:** Passes None when spawning paid scans - these are background re-scans, not HTTP-originated

**Span inheritance:**
All scanner logs inherit request_id automatically via `.instrument(span)` on the spawned scan task. No changes needed to individual scanner code - the span context propagates through tokio's instrumentation.

**Files modified:** `src/orchestrator/worker_pool.rs`, `src/api/webhooks.rs`

## Verification Results

All verification criteria met:

- ✅ `cargo check` compiles without errors
- ✅ `cargo test` passes (62 passed, 1 pre-existing js_secrets failure out of scope)
- ✅ RequestId flows: middleware generates -> Extension stores -> handler extracts -> DB stores -> orchestrator span field
- ✅ Scan spans include request_id as shared field (visible in JSON logs)
- ✅ Webhook paid scans pass None for request_id (logged as empty string)
- ✅ All spawn_scan and spawn_paid_scan callsites updated
- ✅ No parent-child span linking - shared field approach only

## Deviations from Plan

None - plan executed exactly as written. No auto-fixes required.

## Technical Decisions

### RequestId Location
Decision to define RequestId in lib.rs (not main.rs) enables library modules to import it. Main.rs is a binary crate and cannot be imported by other modules, so lib.rs is the correct location for shared types used across the codebase.

### Middleware Ordering
With `.layer()` bottom-to-top execution, the order is:
1. inject_request_id (outermost middleware, runs first on requests)
2. trace_layer (sees RequestId extension, creates span)
3. cors (innermost middleware, runs last on requests)

Request flow: inject_request_id -> trace_layer -> cors -> handler

### Extension Extractor Error Handling
The Extension extractor returns 500 Internal Server Error if RequestId is missing. Since inject_request_id middleware always adds it for traced routes, this should never fail. If it does, 500 is appropriate - it indicates middleware is misconfigured, which is a server error.

### Webhook request_id = None
Paid scans triggered by Stripe webhooks pass None for request_id. The webhook IS an HTTP request (with its own request_id), but the paid scan it triggers is a background re-scan of an existing scan_id. The correlation is via scan_id (traces back to original free scan), not via the webhook's request_id.

### Shared Field vs Parent-Child Spans
Decision to use shared field approach (request_id as a field on scan span) instead of parent-child span linking honors the phase 20 design decision. This avoids coupling HTTP request lifecycle to scan execution lifecycle - scans can run long after the HTTP request completes.

## Next Steps (Phase 21)

Phase 20 is complete. Phase 21 will add performance metrics collection:
1. Response time histograms by endpoint
2. Scanner execution duration tracking
3. Database query performance metrics
4. Metrics exposition endpoint for Prometheus

## Dependencies

- **Requires:** Phase 20 Plan 01 (Request Tracing Infrastructure)
- **Provides:** End-to-end request tracing for Phase 21 (Performance Metrics)
- **Affects:** HTTP handlers, orchestrator spawn methods, scan logging

## Self-Check: PASSED

Verified all claims:

```
FOUND: src/lib.rs (RequestId definition)
FOUND: src/main.rs (inject_request_id middleware, TraceLayer reads extension)
FOUND: src/api/scans.rs (Extension<RequestId> extractor, passes to DB and orchestrator)
FOUND: src/orchestrator/worker_pool.rs (spawn_scan and spawn_paid_scan accept request_id, add to span)
FOUND: src/api/webhooks.rs (passes None to spawn_paid_scan)
FOUND: a790c1f (Task 1 commit)
FOUND: f842353 (Task 2 commit)
```

All commits exist in git history. All files modified as documented. All verification criteria met.
