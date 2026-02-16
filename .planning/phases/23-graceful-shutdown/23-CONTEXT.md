# Phase 23: Graceful Shutdown - Context

**Gathered:** 2026-02-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Backend drains in-flight scans before exiting on SIGTERM/SIGINT to prevent data loss. Background tasks tracked via TaskTracker instead of fire-and-forget tokio::spawn. New scan requests rejected during shutdown drain period.

</domain>

<decisions>
## Implementation Decisions

### Scan drain strategy
- Only drain active (in-flight) scans. Queued scans that haven't started are cancelled — they never begin execution
- Cancelled queued scans are left as "pending" in the database (not marked as cancelled)
- Defense in depth: both HTTP layer (503) AND orchestrator refuse new spawns during shutdown
- If timeout expires mid-scan, allow the current scanner step to finish before stopping (may slightly exceed timeout)

### Client experience
- Scan creation endpoints return 503 with JSON error body: `{"error": "Service shutting down"}`
- No Retry-After header — clients don't need retry timing
- Scan creation returns 503, /health/ready returns unhealthy, other endpoints (results, etc.) keep working
- /health (liveness) stays healthy during shutdown — process is alive. Only /health/ready goes unhealthy — standard readiness pattern for load balancers

### Timeout behavior
- Grace period configured via SHUTDOWN_TIMEOUT env var (12-factor pattern, matches LOG_FORMAT approach)
- Default: 90 seconds — SSL Labs scans can take 60-90s, this gives room for longest scans
- After grace period expires: log warning about forced shutdown, then exit(0) — clean exit code so systemd doesn't restart
- SIGTERM and SIGINT handled identically — same graceful drain whether Docker stop or Ctrl+C

### Shutdown logging
- Periodic progress updates every 5-10s during drain: active scan count, elapsed seconds, timeout seconds
- Structured fields: active_scans, queued_scans, elapsed_seconds, timeout_seconds (no scan IDs)
- Final summary log only on forced shutdown (timeout expired): "Shutdown forced: N scans remaining after Xs"
- Normal clean shutdowns don't get a summary line — periodic logs are sufficient
- Log levels: INFO for initiation and progress, WARN for forced/timeout events
- No new Prometheus metrics — existing active_scans and scan_queue_depth gauges already show drain progress

### Claude's Discretion
- TaskTracker implementation details and integration pattern
- Signal handler implementation (tokio::signal, ctrlc crate, etc.)
- Shutdown state sharing mechanism (AtomicBool, watch channel, CancellationToken, etc.)
- Exact middleware placement for 503 rejection
- Periodic logging interval (5s or 10s — within the 5-10s range)

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. Follow existing patterns from Phase 19-22 (env var config, structured logging, middleware layering).

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 23-graceful-shutdown*
*Context gathered: 2026-02-16*
