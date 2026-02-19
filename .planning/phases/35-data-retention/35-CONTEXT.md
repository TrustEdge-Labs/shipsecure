# Phase 35: Data Retention - Context

**Gathered:** 2026-02-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Hourly background cleanup task that automatically hard-deletes expired scans — anonymous after 24 hours, Developer after 30 days — without touching in-progress scans or payment records. This is a backend-only Tokio interval task integrated into the existing task_tracker and graceful shutdown infrastructure.

</domain>

<decisions>
## Implementation Decisions

### Deletion scope
- Hard delete scan rows — CASCADE removes findings, paid_audits FK already SET NULL (Phase 30)
- No soft delete — deleted scans are gone permanently
- Expired result URLs return standard 404 Not Found — no special "expired" messaging
- CASCADE + SET NULL is sufficient for related data — no other tables reference scans
- Researcher should check for any temp files or disk artifacts that scanners may leave behind — clean those up too if they exist

### Expired scan UX
- Deleted scans vanish completely from dashboard history — no tombstone rows
- 24-hour grace period after expires_at before deletion — users see the Phase 34 "Expired" badge (opacity-60) for at least a day before data disappears
- Existing expiry countdown in scan history table is sufficient — no additional warning state or notifications needed

### Logging and metrics
- Per-tier breakdown in structured logs: "Retention cleanup: deleted 8 anonymous + 4 developer scans (12 total)"
- Structured tracing logs only — no Prometheus counter needed
- Always log at INFO level even when zero scans deleted — confirms the task is running
- Do NOT log skipped in-progress count — protection is implicit in the WHERE clause

### Error handling
- On DB failure: log error, wait for next hourly tick — no immediate retry
- Single DELETE query — no batching needed for expected scan volumes
- Register cleanup task with TaskTracker — graceful shutdown waits for current DELETE to complete
- Stuck scan detection (pending/in_progress >6h) is out of scope — future phase concern

### Claude's Discretion
- Exact Tokio interval setup and integration pattern with main.rs
- SQL query structure for the grace period offset
- File cleanup implementation details (if disk artifacts are found)
- Log format and tracing span structure

</decisions>

<specifics>
## Specific Ideas

- The cleanup WHERE clause must account for the 24-hour grace period: `expires_at + INTERVAL '24 hours' < NOW()`
- The cleanup should only target status IN ('completed', 'failed') — never 'pending' or 'in_progress'
- expires_at values are already set correctly at scan creation time (Phase 33): 24h for anonymous, 30 days for Developer

</specifics>

<deferred>
## Deferred Ideas

- Stuck scan detection — scans in 'pending' or 'in_progress' for abnormally long periods (>6h) should be marked 'failed' so they become eligible for cleanup. Belongs in a future operational health phase.

</deferred>

---

*Phase: 35-data-retention*
*Context gathered: 2026-02-18*
