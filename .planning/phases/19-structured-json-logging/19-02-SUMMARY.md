---
phase: 19-structured-json-logging
plan: 02
subsystem: orchestrator
tags: [observability, structured-logging, tracing, scan-lifecycle, scanner-lifecycle]
dependency_graph:
  requires: [19-01-structured-logging-foundation]
  provides: [scan-lifecycle-events, scanner-lifecycle-events, structured-scan-context]
  affects: [scan-orchestration, scanner-execution, observability]
tech_stack:
  added: []
  patterns: [tracing-spans, structured-events, hierarchical-context, timing-instrumentation]
key_files:
  created: []
  modified: [src/orchestrator/worker_pool.rs]
decisions:
  - Scan-level spans carry scan_id, target_url, tier as context
  - Scanner-level spans carry scanner_name, scan_id as context
  - All lifecycle events (started/completed/failed) log duration_ms
  - No sensitive data in logs (no emails, no finding counts)
  - Spans use .instrument() pattern for automatic context propagation
metrics:
  duration_minutes: 2
  tasks_completed: 1
  files_modified: 1
  commits: 1
  completed_date: 2026-02-16
---

# Phase 19 Plan 02: Scan Lifecycle Logging Summary

**One-liner:** Structured scan and scanner lifecycle events with tracing spans, duration tracking, and hierarchical context propagation for complete scan observability.

## What Was Built

Instrumented the entire scan orchestration pipeline with structured lifecycle events:
- **Scan-level spans** wrap each scan execution with scan_id, target_url, tier context
- **Scanner-level spans** wrap each scanner with scanner_name, scan_id context
- **Lifecycle events** log scan_started, scan_completed, scan_failed with duration_ms
- **Scanner events** log scanner_started, scanner_completed, scanner_failed with duration_ms
- **Context propagation** via .instrument() ensures all nested logs inherit span fields
- **Sensitive data protection** - no emails, no finding counts in logs
- **Clean event structure** - replaced old ad-hoc error messages with structured events

## Tasks Completed

### Task 1: Add structured scan lifecycle spans and events to the orchestrator

**Commit:** c4b7435

**Changes:**
1. **Imports added:**
   - `use std::time::Instant;` for duration tracking
   - `use tracing::{info_span, Instrument};` for span creation and instrumentation

2. **spawn_scan (free tier) instrumentation:**
   - Created info_span with scan_id, target_url, tier="free"
   - Added scan_started event after permit acquisition
   - Added scan_completed/scan_failed events with duration_ms
   - Wrapped async block with .instrument(span)
   - Removed old "Scan {} failed" error message

3. **spawn_paid_scan (paid tier) instrumentation:**
   - Created info_span with scan_id, target_url, tier="paid"
   - Added scan_started event after permit acquisition
   - Added scan_failed events with duration_ms for pre-step failures (clear findings, reset status)
   - Added scan_completed/scan_failed events with duration_ms for main execution
   - Wrapped async block with .instrument(span)
   - Removed old error messages ("Failed to clear findings", "Failed to reset status", "Paid scan {} failed")

4. **Scanner instrumentation (all 5 scanners):**
   - **headers_handle (security_headers):** Added scanner span, scanner_started, scanner_completed/scanner_failed with duration_ms
   - **tls_handle (tls):** Added scanner span, scanner_started, scanner_completed/scanner_failed with duration_ms
   - **files_handle (exposed_files):** Added scanner span, scanner_started, scanner_completed/scanner_failed with duration_ms
   - **secrets_handle (js_secrets):** Added scanner span, scanner_started, scanner_completed/scanner_failed with duration_ms
   - **vibecode_handle (vibecode):** Added scanner span, scanner_started, scanner_completed/scanner_failed with duration_ms

5. **Timing instrumentation:**
   - All spans start timing immediately after span creation
   - Duration calculated as `start.elapsed().as_millis() as u64`
   - Duration included in all completion and failure events

6. **Sensitive data policy enforcement:**
   - Verified no email addresses logged in worker_pool.rs (only scan_id)
   - Verified no finding_count logged anywhere
   - Note: src/email/mod.rs logs email addresses but is out of scope for this plan

**Files Modified:**
- src/orchestrator/worker_pool.rs: Added 233 lines, removed 144 lines (89 net lines added)

**Verification:**
- cargo check: Compiles successfully (3 pre-existing warnings)
- cargo test: 62/63 tests pass (1 pre-existing failure in js_secrets test)
- 2 scan_started events (free + paid tiers)
- 2 scan_completed events (free + paid tiers)
- 5 scanner_started events (one per scanner)
- 5 scanner_completed events (one per scanner)
- 7 info_span! calls (2 scan-level + 5 scanner-level)
- duration_ms logged on all completion/failure events
- 0 finding_count references in logs
- No email addresses logged in orchestrator code

## Deviations from Plan

None - plan executed exactly as written.

**Pre-existing Issue Documented:**
- Test `scanners::js_secrets::tests::test_false_positive_detection` fails (pre-existing)
- Not caused by logging changes (src/scanners/js_secrets.rs not modified)
- Same failure existed before this plan

## Technical Decisions

### 1. Span Context Propagation via .instrument()

**Decision:** Use .instrument(span) to wrap async blocks and propagate context automatically.

**Rationale:**
- Automatic context inheritance - all tracing calls inside the block inherit span fields
- Clean separation of concerns - span creation separate from business logic
- Idiomatic Rust tracing pattern
- Works seamlessly with nested spans (scan span contains scanner spans)
- Zero manual context passing required

**Implementation:**
```rust
let span = info_span!("scan", scan_id = %scan_id, target_url = %target_url, tier = "free");
tokio::spawn(async move {
    tracing::info!("scan_started"); // Automatically includes scan_id, target_url, tier
    // ... scan logic ...
}.instrument(span));
```

### 2. Hierarchical Span Structure

**Decision:** Scan-level spans wrap scanner-level spans for hierarchical context.

**Rationale:**
- Scanner events inherit scan context automatically
- Enables log filtering by scan_id to see all events (scan + all scanners)
- Enables log filtering by scanner_name to see specific scanner across all scans
- Matches actual execution hierarchy (scan spawns scanners)
- Standard distributed tracing pattern

**Structure:**
```
scan span (scan_id, target_url, tier)
  └─ scanner_started (inherits scan context + scanner_name)
     scanner_completed/failed (inherits scan context + scanner_name + duration_ms)
```

### 3. Duration Tracking on All Lifecycle Events

**Decision:** Log duration_ms on all completed and failed events (scan and scanner level).

**Rationale:**
- Essential for performance monitoring and SLA tracking
- Enables alerting on slow scans or scanners
- Helps identify timeout causes (slow network, slow scanner, slow external API)
- Consistent pattern across all lifecycle events
- Structured field enables aggregation in log analysis tools

**Pattern:**
```rust
let start = Instant::now();
// ... execution ...
let duration_ms = start.elapsed().as_millis() as u64;
tracing::info!(duration_ms, "scan_completed");
```

### 4. Event Naming Convention

**Decision:** Use underscore-separated event names (scan_started, scanner_completed, scan_failed).

**Rationale:**
- Consistent with structured logging conventions
- Easy to grep: `grep "scan_started"`
- Easy to parse in log aggregation tools
- Distinguishes events from regular log messages
- Enables event-based alerting and metrics

**Event vocabulary:**
- `scan_started` - Scan execution begins (after semaphore permit)
- `scan_completed` - Scan execution succeeds
- `scan_failed` - Scan execution fails (pre-steps or main execution)
- `scanner_started` - Scanner execution begins
- `scanner_completed` - Scanner execution succeeds
- `scanner_failed` - Scanner execution fails or times out

### 5. Sensitive Data Protection Policy

**Decision:** No customer email addresses, no scan finding counts or results in logs.

**Rationale:**
- Privacy: Email addresses are PII (Personally Identifiable Information)
- Compliance: Avoid logging user data unnecessarily
- Security: Logs may be stored in less secure systems than the database
- Log volume: Finding counts/content would bloat logs significantly
- Separation: Scan results live in database, logs track execution lifecycle only

**What's logged:**
- scan_id (UUID) - safe, no PII
- target_url (domain) - customer-provided, not PII
- tier (free/paid) - business context
- scanner_name - technical context
- duration_ms - performance metric
- error messages - technical failures only

**What's NOT logged:**
- Email addresses (neither in orchestrator nor in lifecycle events)
- Finding counts
- Finding content
- Grade/score

## Testing & Validation

**Build Validation:**
- cargo check: Passes (3 pre-existing warnings, not introduced by this plan)
- cargo test: 62/63 tests pass (1 pre-existing failure documented)
- No new warnings introduced
- No new test failures introduced

**Code Verification:**
- info_span! count: 7 (2 scan-level + 5 scanner-level) ✓
- scan_started count: 2 (free + paid) ✓
- scan_completed count: 2 (free + paid) ✓
- scanner_started count: 5 (all scanners) ✓
- scanner_completed count: 5 (all scanners) ✓
- duration_ms: Present on all completion/failure events ✓
- finding_count: 0 occurrences ✓
- Email addresses: Only in comments, not in log events ✓

**Event Coverage:**
- Free tier scan: scan_started → scanner_started (×5) → scanner_completed/failed (×5) → scan_completed/failed ✓
- Paid tier scan: scan_started → (pre-steps) → scanner_started (×5) → scanner_completed/failed (×5) → scan_completed/failed ✓
- Paid tier pre-step failures: scan_started → scan_failed (with clear findings or reset status error) ✓

## Impact Assessment

**Files Changed:** 1 (src/orchestrator/worker_pool.rs)

**Lines Changed:** +233 insertions, -144 deletions (89 net lines added)

**Breaking Changes:** None
- No API changes
- No database schema changes
- Existing scans continue to work
- Log format changes are additive (new structured fields)

**Performance Impact:** Minimal
- Span creation overhead: ~100ns per span (negligible)
- Duration tracking overhead: single Instant::now() call per event
- Log volume increase: ~12 structured events per scan (2 scan + 10 scanner events)
- No additional allocations in scanner hot paths
- No blocking operations added

**Observability Gains:**
- Can now track scan duration from start to completion
- Can identify which scanner is slowest for a given scan
- Can monitor scanner failure rates independently
- Can correlate all events for a scan via scan_id
- Can detect timeout patterns by scanner type
- Enables SLA monitoring and alerting

## What's Next

This plan completes the scan lifecycle logging foundation. Future work:

**Immediate:**
- Next plan in Phase 19 will add request/response logging with correlation IDs

**Phase 20: Health Checks**
- Will leverage this structured logging for health check diagnostics
- Will use scan lifecycle events to assess system health

**Phase 21: Metrics**
- Will extract metrics from these lifecycle events
- scan_duration_seconds histogram from duration_ms
- scanner_duration_seconds histogram by scanner_name
- scan_failures_total counter from scan_failed events
- scanner_failures_total counter from scanner_failed events

**Out of Scope Follow-up:**
- src/email/mod.rs logs email addresses in completion email success messages
- Consider whether email logging should be removed or redacted in Phase 19 or later
- Not critical for MVP (email service is internal, logs not exposed externally)

## Self-Check: PASSED

**Modified Files:**
- ✓ src/orchestrator/worker_pool.rs (exists, modified)

**Commits:**
- ✓ c4b7435: feat(19-02): add structured scan lifecycle logging with tracing spans

**Created Files:**
- ✓ .planning/phases/19-structured-json-logging/19-02-SUMMARY.md (this file)

All artifacts verified present and correct.
