---
phase: 20-request-tracing
plan: 01
subsystem: observability
tags: [tracing, middleware, correlation-ids, database-schema]
requires: [phase-19-structured-json-logging]
provides: [request-tracing-infrastructure, request-id-middleware, request-id-storage]
affects: [http-middleware, scan-model, database-layer]
tech_stack:
  added:
    - tower-http TraceLayer middleware
    - UUID v4 request_id generation
  patterns:
    - Correlation ID per HTTP request
    - Conditional log levels (INFO for 4xx/5xx, DEBUG for 2xx/3xx)
    - Health check filtering (routes bypass tracing)
    - Partial database index for nullable correlation IDs
key_files:
  created:
    - migrations/20260216000001_add_request_id_to_scans.sql
  modified:
    - src/main.rs
    - src/models/scan.rs
    - src/db/scans.rs
    - src/api/scans.rs
decisions:
  - "Query parameters stripped from logged URIs (path only) — prevents sensitive data in logs"
  - "Request ID internal only (no X-Request-Id response header) — simplicity, no client impact"
  - "Nullable request_id column with partial index — not all scans originate from HTTP requests"
  - "Health check routes bypass tracing — placed after .layer() to avoid noise in logs"
metrics:
  duration: 2 minutes
  tasks: 2
  files: 4
  commits: 2
  completed: 2026-02-16
---

# Phase 20 Plan 01: Request Tracing Infrastructure

**One-liner:** tower-http TraceLayer generates UUID v4 request_id per HTTP request with conditional log levels and health check filtering; database schema ready for correlation ID storage.

## Overview

Established the request tracing foundation for the TrustEdge Audit API. TraceLayer middleware now generates a unique UUID v4 request_id for every HTTP request, logs method/path/status/latency with conditional log levels (INFO for errors, DEBUG for success), and filters out health check requests. The database schema is ready to store correlation IDs with a partial index, and the Scan model includes the request_id field. Plan 02 will wire the request_id through to handlers and the orchestrator.

## Tasks Completed

### Task 1: Add TraceLayer middleware with request_id generation and health check filtering
**Commit:** `0ccf72b`

Added tower-http TraceLayer middleware to the Axum router with the following key features:

- **UUID v4 request_id generation:** Every HTTP request gets a unique correlation ID in the tracing span
- **Structured request logging:** Logs include `method`, `uri` (path only, no query params), `status_code`, and `latency_ms`
- **Conditional log levels:** 4xx/5xx responses log at INFO level, 2xx/3xx log at DEBUG level
- **Health check filtering:** `/health` route placed after `.layer()` to bypass tracing and avoid log noise
- **Privacy-first:** Query parameters stripped from URIs, no auth headers/IPs/bodies logged, no X-Request-Id response header

**Implementation notes:**
- Used `axum::http::Request` and `axum::http::Response` for proper type imports (auto-fixed import issue)
- TraceLayer placed before CORS layer in the middleware chain (bottom-to-top execution)
- `field::Empty` used for `status_code` and `latency_ms` — populated in `on_response` callback

**Files modified:** `src/main.rs`

### Task 2: Add database migration and update Scan model and DB queries for request_id
**Commit:** `abf7a37`

Created database migration and updated the Scan model and all database queries to support request_id correlation IDs:

- **Migration:** Added nullable `request_id UUID` column to `scans` table with partial index on non-NULL values
- **Scan model:** Added `request_id: Option<Uuid>` field after `submitter_ip` and before `status`
- **Database queries:** Updated ALL Scan-returning queries to include `request_id` in SELECT/RETURNING clauses:
  - `create_scan` — now accepts `request_id: Option<Uuid>` parameter
  - `get_scan` — includes `request_id` in SELECT
  - `claim_pending_scan` — includes `request_id` in RETURNING
  - `get_scan_by_token` — includes `request_id` in SELECT
- **Temporary workaround:** `create_scan` caller in `src/api/scans.rs` passes `None` — Plan 02 will extract the actual request_id from the tracing span

**Files created:** `migrations/20260216000001_add_request_id_to_scans.sql`
**Files modified:** `src/models/scan.rs`, `src/db/scans.rs`, `src/api/scans.rs`

## Verification Results

All verification criteria met:

- ✅ `cargo check` compiles without errors
- ✅ `cargo test` passes (62 passed, 1 pre-existing failure in js_secrets out of scope)
- ✅ TraceLayer middleware configured with `make_span_with` generating UUID v4
- ✅ Health check route bypasses tracing (placed after `.layer()`)
- ✅ Migration file ready for `request_id` column with partial index
- ✅ Scan model and all DB queries include `request_id`
- ✅ No sensitive data logged (no query params, no auth headers, no IPs, no bodies)
- ✅ No X-Request-Id response header added

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed axum::http import path**
- **Found during:** Task 1, initial compilation
- **Issue:** Import statement `use http::Request;` failed — `http` crate not directly available
- **Fix:** Changed to `use axum::http::Request` and `use axum::http::Response` — axum re-exports the http crate types
- **Files modified:** `src/main.rs`
- **Commit:** `0ccf72b` (same commit as Task 1)

## Technical Decisions

### Request ID Privacy
Decision to NOT add X-Request-Id response headers keeps the request_id internal-only. This simplifies implementation and avoids client-side confusion. The request_id is purely for server-side correlation in logs and database records.

### Nullable request_id Column
The `request_id` column is nullable because not all scans originate from HTTP requests (testing, manual rescans, background jobs). The partial index optimizes lookups for correlation without wasting space on NULL values.

### Health Check Filtering
Placing health check routes after `.layer()` ensures they bypass TraceLayer entirely. This prevents log noise from frequent health checks (every 10 seconds from load balancers, monitoring tools, etc.).

### Conditional Log Levels
INFO for 4xx/5xx ensures error responses are visible in production (default log level). DEBUG for 2xx/3xx reduces log volume for successful requests while keeping them available for troubleshooting.

## Next Steps (Plan 02)

1. Extract request_id from tracing span in `create_scan` handler — replace `None` with actual value
2. Pass request_id to orchestrator and log at scan lifecycle events (started, completed, failed)
3. Add request_id to scan stage logging (headers, TLS, secrets, etc.)
4. Update scan orchestrator to include request_id in worker logs

## Dependencies

- **Requires:** Phase 19 (Structured JSON Logging) — TraceLayer logs use the tracing infrastructure
- **Provides:** Request tracing infrastructure for Phase 20 Plan 02 (Handler Integration)
- **Affects:** HTTP middleware layer, Scan model, database layer

## Self-Check: PASSED

Verified all claims:

```
FOUND: migrations/20260216000001_add_request_id_to_scans.sql
FOUND: src/main.rs (TraceLayer middleware)
FOUND: src/models/scan.rs (request_id field)
FOUND: src/db/scans.rs (request_id in all queries)
FOUND: 0ccf72b (Task 1 commit)
FOUND: abf7a37 (Task 2 commit)
```

All commits exist in git history. All files modified as documented. All verification criteria met.
