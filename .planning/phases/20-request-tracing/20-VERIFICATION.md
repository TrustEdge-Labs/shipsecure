---
phase: 20-request-tracing
verified: 2026-02-16T16:50:01Z
status: human_needed
score: 5/6 must-haves verified
re_verification: false
human_verification:
  - test: "Verify request_id appears in scanner logs in JSON output"
    expected: "Run a scan with LOG_FORMAT=json, search logs for a request_id, confirm all scanner events (scanner_started, scanner_completed, scanner_failed) include that request_id field"
    why_human: "Scanner spans are created as children of scan span, and whether child span events inherit parent fields in JSON output depends on tracing-subscriber runtime behavior - cannot verify from code inspection alone"
  - test: "Verify request_id propagates end-to-end in real HTTP request"
    expected: "POST /api/v1/scans, capture logs, confirm same request_id appears in http_request span, handler logs, database insert logs, scan span, and all scanner spans"
    why_human: "End-to-end correlation requires runtime verification with actual HTTP request flow"
  - test: "Verify webhook-triggered paid scans show empty request_id"
    expected: "Trigger Stripe webhook, check logs, confirm scan span has request_id field with empty string value"
    why_human: "Webhook flow requires Stripe integration testing"
---

# Phase 20: Request Tracing Verification Report

**Phase Goal:** Every HTTP request gets traced with correlation IDs propagated to background tasks
**Verified:** 2026-02-16T16:50:01Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Request_id is extracted from the tracing span in create_scan handler and passed to the orchestrator | ✓ VERIFIED | Extension(request_id): Extension<RequestId> on line 25 of src/api/scans.rs, passed to spawn_scan on line 59 |
| 2 | spawn_scan and spawn_paid_scan accept Option<Uuid> request_id and add it as a shared field on the scan span | ✓ VERIFIED | Both methods have request_id: Option<Uuid> parameter (lines 69, 108 of worker_pool.rs), added to span on lines 78 and 117 |
| 3 | request_id appears in all logs associated with a single request lifecycle (HTTP → handler → scan → scanners) | ? NEEDS HUMAN | Scanner spans are children of scan span (info_span! creates child spans), but whether scanner log events include parent request_id field in JSON output requires runtime testing |
| 4 | Paid scan triggered by Stripe webhook passes None as request_id (per decision: scan tasks only from HTTP) | ✓ VERIFIED | Line 211 of webhooks.rs: orchestrator.spawn_paid_scan(scan_id, target_url.clone(), None) |
| 5 | request_id is stored in the database when creating a scan | ✓ VERIFIED | db::scans::create_scan accepts request_id parameter (line 49-54 of scans.rs), database migration adds request_id column with index |
| 6 | Background scan tasks inherit request_id via shared field on scan span (NOT parent-child linking) | ✓ VERIFIED | Scan spans include request_id as a field (not as parent span reference), attached via .instrument(span) on tokio::spawn |

**Score:** 5/6 truths verified (1 requires human verification)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| src/lib.rs | RequestId newtype for Axum Extension | ✓ VERIFIED | Lines 11-13: pub struct RequestId(pub uuid::Uuid) defined in library crate |
| src/main.rs | RequestId stored in request extensions | ✓ VERIFIED | Lines 84-91: inject_request_id middleware inserts RequestId extension; lines 167-170: make_span_with reads from extensions |
| src/api/scans.rs | Handler extracts RequestId from extensions, passes to orchestrator and DB | ✓ VERIFIED | Line 25: Extension<RequestId> extractor; line 54: passed to create_scan; line 59: passed to spawn_scan |
| src/orchestrator/worker_pool.rs | Orchestrator adds request_id as shared field on scan spans | ✓ VERIFIED | Lines 69-79 (spawn_scan) and 108-118 (spawn_paid_scan): request_id field added to scan span |
| src/api/webhooks.rs | Webhook handler passes None for request_id on paid scan spawn | ✓ VERIFIED | Line 211: spawn_paid_scan called with None |

**All artifacts verified at all three levels:**
- Level 1 (Exists): All files present
- Level 2 (Substantive): All contain expected implementations (not stubs)
- Level 3 (Wired): All properly imported and used

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| src/main.rs | src/api/scans.rs | RequestId inserted into request extensions in make_span_with, extracted via Extension<RequestId> | ✓ WIRED | inject_request_id middleware inserts RequestId (line 89 of main.rs), Extension<RequestId> extractor on line 25 of scans.rs |
| src/api/scans.rs | src/orchestrator/worker_pool.rs | request_id passed as explicit parameter to spawn_scan | ✓ WIRED | Line 59 of scans.rs calls spawn_scan with Some(request_id.0), worker_pool.rs lines 69-79 accept and use parameter |
| src/api/scans.rs | src/db/scans.rs | request_id passed to create_scan DB function | ✓ WIRED | Lines 49-54 of scans.rs pass Some(request_id.0) to db::scans::create_scan, which inserts into database |

**All key links verified as WIRED**

### Success Criteria Coverage (from ROADMAP.md)

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Every HTTP request receives a unique request_id via tower-http TraceLayer | ✓ VERIFIED | TraceLayer middleware with make_span_with generates UUID v4 on line 166-179 of main.rs |
| Request and response logs include method, URI, status code, and latency_ms | ✓ VERIFIED | make_span_with includes method and uri fields (lines 174-175), on_response records status_code and latency_ms (lines 183-184) |
| Background scan tasks inherit request span context via .instrument() | ✓ VERIFIED | spawn_scan and spawn_paid_scan use .instrument(span) on tokio::spawn (lines 96, 149 of worker_pool.rs) |
| Request_id appears in all logs associated with a single request lifecycle | ? NEEDS HUMAN | Scan span includes request_id field, scanner spans are children of scan span, but runtime verification needed to confirm JSON log output includes parent fields |

**Score:** 3/4 success criteria verified (1 requires runtime testing)

### Anti-Patterns Found

No anti-patterns detected. All modified files scanned for:
- TODO/FIXME/XXX/HACK/PLACEHOLDER comments: None found
- Placeholder implementations: None found
- Empty handlers: None found
- Console.log-only implementations: None found

**Commits verified:**
- a790c1f: "feat(20-02): add RequestId extension and wire through create_scan handler"
- f842353: "feat(20-02): propagate request_id to orchestrator and scan spans"

Both commits exist in git history. All file modifications documented in SUMMARY.md key_files section confirmed.

### Human Verification Required

#### 1. Scanner Log request_id Inheritance

**Test:** Run application with LOG_FORMAT=json, create a scan, filter logs by a request_id value
**Expected:**
1. HTTP request span includes request_id field
2. Scan span includes same request_id field
3. Scanner spans (security_headers, tls, files, secrets, vibecode) events include request_id field from parent scan span
4. All scanner_started, scanner_completed, scanner_failed events show the same request_id

**Why human:** Scanner spans are created as children of scan spans via `info_span!()` (which makes current span the parent). Whether tracing-subscriber's JSON formatter includes parent span fields in child span events is a runtime behavior that cannot be verified from code inspection. The implementation follows the pattern recommended in RESEARCH.md (rely on span hierarchy), but actual JSON output needs verification.

**Verification command:**
```bash
LOG_FORMAT=json cargo run &
sleep 5
curl -X POST http://localhost:8000/api/v1/scans \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "email": "test@example.com"}'
# Extract request_id from first log line, then grep all logs for that request_id
# Confirm scanner events include the request_id field
```

#### 2. End-to-End Request Lifecycle Tracing

**Test:** Full request lifecycle from HTTP → handler → DB → orchestrator → scanners
**Expected:**
1. Single request_id value generated at HTTP layer
2. Same request_id in all log events: http_request, handler processing, database insert, scan_started, scanner_started, scanner_completed, scan_completed
3. Can filter logs by request_id and see complete lifecycle
4. request_id stored in database scans table for scan record

**Why human:** Requires actual HTTP request with full scan execution, database query verification, and log correlation analysis.

**Verification command:**
```bash
# Start with JSON logging
LOG_FORMAT=json cargo run &
# Send request, capture response scan_id
SCAN_ID=$(curl -X POST http://localhost:8000/api/v1/scans \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "email": "test@example.com"}' | jq -r '.id')
# Wait for scan completion
sleep 30
# Extract request_id from logs
REQUEST_ID=$(grep "http_request" logs.json | jq -r '.fields.request_id' | head -1)
# Verify all lifecycle events have same request_id
grep "$REQUEST_ID" logs.json | jq '.message, .fields.scanner_name'
# Verify database storage
psql $DATABASE_URL -c "SELECT request_id FROM scans WHERE id = '$SCAN_ID';"
```

#### 3. Webhook Paid Scan request_id = None

**Test:** Trigger Stripe webhook for paid scan, verify request_id handling
**Expected:**
1. Webhook handler logs include request_id (the webhook's own HTTP request_id)
2. Spawned paid scan has request_id field with empty string value (not the webhook's request_id)
3. Database record for paid scan has request_id = NULL (not updated from webhook)

**Why human:** Requires Stripe webhook integration testing, either with Stripe CLI or test webhook events.

**Verification command:**
```bash
# Use Stripe CLI to send test webhook
stripe trigger checkout.session.completed
# Check logs for webhook handling
grep "checkout.session.completed" logs.json | jq '.fields.request_id'
# Check spawned paid scan logs (should have empty request_id field)
grep "tier.*paid" logs.json | jq '.fields.request_id'
```

### Implementation Notes

**Middleware Ordering Verified:**
Router uses bottom-to-top layering (line 205-207 of main.rs):
1. inject_request_id (bottom layer, runs first)
2. trace_layer (middle layer, sees RequestId extension)
3. cors (top layer, runs last)

Request flow: inject_request_id → trace_layer → cors → handler

**Health Check Filtering Verified:**
/health route added after .with_state() (line 210 of main.rs), bypasses all middleware including TraceLayer.

**Database Schema Verified:**
Migration 20260216000001_add_request_id_to_scans.sql adds:
- Nullable UUID column: `ADD COLUMN request_id UUID`
- Partial index: `CREATE INDEX idx_scans_request_id ON scans (request_id) WHERE request_id IS NOT NULL`

**Scan Model Verified:**
src/models/scan.rs line 21: `pub request_id: Option<Uuid>` field present.

**Compilation Verified:**
`cargo check` completes successfully with only 3 pre-existing warnings (unused function, unused field) - no errors related to Phase 20 changes.

---

**Overall Assessment:**

All must-haves verified at code level. Implementation follows the architectural patterns from RESEARCH.md. The one uncertainty is whether scanner log events include the parent scan span's request_id field in JSON output, which depends on tracing-subscriber's runtime behavior. The pattern used (child spans via info_span!) is correct according to RESEARCH.md recommendation, but actual JSON output needs human verification.

The phase achieves its goal of establishing end-to-end request tracing infrastructure. The verification gap is not a code defect - it's a need to confirm runtime behavior matches design expectations.

**Recommendation:** Execute human verification tests 1-2 to confirm request_id appears in scanner logs. If scanner logs do NOT include request_id, the fix would be to add request_id as an explicit field to scanner spans (pattern already used for scan spans).

---

_Verified: 2026-02-16T16:50:01Z_
_Verifier: Claude (gsd-verifier)_
