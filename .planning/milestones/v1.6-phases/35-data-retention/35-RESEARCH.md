# Phase 35: Data Retention - Research

**Researched:** 2026-02-18
**Domain:** Tokio interval task, TaskTracker graceful shutdown, SQLx DELETE, tier-based expires_at
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- Hard delete scan rows — CASCADE removes findings, paid_audits FK already SET NULL (Phase 30)
- No soft delete — deleted scans are gone permanently
- Expired result URLs return standard 404 Not Found — no special "expired" messaging
- CASCADE + SET NULL is sufficient for related data — no other tables reference scans
- Researcher should check for any temp files or disk artifacts that scanners may leave behind — clean those up too if they exist (FINDING: no action needed — see Architecture Patterns)
- Deleted scans vanish completely from dashboard history — no tombstone rows
- 24-hour grace period after expires_at before deletion — users see the Phase 34 "Expired" badge (opacity-60) for at least a day before data disappears
- Existing expiry countdown in scan history table is sufficient — no additional warning state or notifications needed
- Per-tier breakdown in structured logs: "Retention cleanup: deleted 8 anonymous + 4 developer scans (12 total)"
- Structured tracing logs only — no Prometheus counter needed
- Always log at INFO level even when zero scans deleted — confirms the task is running
- Do NOT log skipped in-progress count — protection is implicit in the WHERE clause
- On DB failure: log error, wait for next hourly tick — no immediate retry
- Single DELETE query — no batching needed for expected scan volumes
- Register cleanup task with TaskTracker — graceful shutdown waits for current DELETE to complete
- Stuck scan detection (pending/in_progress >6h) is out of scope — future phase concern

### Claude's Discretion

- Exact Tokio interval setup and integration pattern with main.rs
- SQL query structure for the grace period offset
- File cleanup implementation details (if disk artifacts are found)
- Log format and tracing span structure

### Deferred Ideas (OUT OF SCOPE)

- Stuck scan detection — scans in 'pending' or 'in_progress' for abnormally long periods (>6h) should be marked 'failed' so they become eligible for cleanup. Belongs in a future operational health phase.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| RETN-01 | Anonymous scan results expire after 24 hours | expires_at set to NOW() + 24h for tier='free' at scan completion; DELETE WHERE expires_at + INTERVAL '24 hours' < NOW() AND status IN ('completed','failed') |
| RETN-02 | Developer tier scan results expire after 30 days | expires_at set to NOW() + 30 days for tier='authenticated' at scan completion; same DELETE query covers both tiers |
| RETN-03 | Background cleanup task deletes expired completed/failed scans hourly | Tokio interval task registered with TaskTracker; runs hourly; logs per-tier deleted counts |
</phase_requirements>

## Summary

Phase 35 has two distinct concerns that must both be addressed. The first is a prerequisite fix: the orchestrator currently sets `expires_at` to a hardcoded 3 days for all tiers (see `src/orchestrator/worker_pool.rs` line 267: `chrono::Duration::days(3)`). This must change to be tier-based — 1 day (24 hours) for `"free"` tier, 30 days for `"authenticated"` tier — so that the cleanup query can correctly enforce RETN-01 and RETN-02. The second concern is the cleanup task itself: a Tokio interval task spawned in `main.rs` using the existing `task_tracker` and `shutdown_token`, running hourly, executing a DELETE query with the 24-hour grace period offset, and logging per-tier counts.

The disk artifact concern is resolved: temp files created by scanners (Nuclei and testssl.sh in `src/scanners/container.rs`) use `tempfile::NamedTempFile`, which is RAII — the file is automatically deleted when the binding is dropped at function exit. No scanner leaves persistent disk artifacts tied to a scan record. No separate file cleanup is needed.

The graceful shutdown integration uses the existing `TaskTracker` pattern already established in this codebase. The `task_tracker` is created in `main.rs` before the orchestrator, so the cleanup task can be spawned on it with `task_tracker.spawn(...)` in the same way scan tasks are spawned in the orchestrator. When `orchestrator.initiate_shutdown()` calls `task_tracker.close()` and `shutdown_token.cancel()`, the cleanup task's `select!` loop exits immediately, and `task_tracker.wait()` waits for any in-progress DELETE to complete.

**Primary recommendation:** Implement in a single plan: (1) fix tier-based `expires_at` in `worker_pool.rs`, (2) add `delete_expired_scans` function to `src/db/scans.rs`, (3) create `src/cleanup.rs` with the interval task, (4) wire into `main.rs`, (5) expose the module in `lib.rs`.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tokio::time` | 1.49.0 (already in Cargo.toml) | `interval_at`, `Instant` for periodic task | Already in project as `tokio = { version = "1", features = ["full"] }` — no new dep |
| `tokio_util::task::TaskTracker` | 0.7.18 (already in Cargo.toml) | Graceful shutdown coordination | Already used by `ScanOrchestrator` |
| `tokio_util::sync::CancellationToken` | 0.7.18 (already in Cargo.toml) | Shutdown signal propagation | Already established pattern in project |
| `sqlx::PgPool` | 0.8.6 (already in Cargo.toml) | Execute DELETE query | Already used in all db modules |
| `tracing` | 0.1 (already in Cargo.toml) | Structured logs for cleanup run results | Already used throughout |

No new dependencies required. This phase is purely backend Rust using the existing stack.

### Supporting

None — all needed functionality is already present.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `tokio::time::interval_at` (deferred first tick) | `tokio::time::interval` (immediate first tick) | `interval` fires immediately on startup — would DELETE on every deploy; `interval_at` defers to first scheduled hour, cleaner for operations |
| Two separate DELETE queries (per-tier count) | Single DELETE + separate COUNT queries | Two DELETEs is simpler and gives accurate per-tier counts; single DELETE loses per-tier breakdown |
| `MissedTickBehavior::Skip` | `MissedTickBehavior::Delay` (default: Burst) | Skip is correct for a cleanup task — if a tick is missed (e.g., DB was slow), skip to the next scheduled hour; Burst would run missed ticks back-to-back which is unnecessary |

## Architecture Patterns

### Recommended Project Structure

```
src/
├── cleanup.rs           # NEW: retention cleanup task (pub fn spawn_cleanup_task)
├── lib.rs               # MODIFIED: pub mod cleanup;
├── main.rs              # MODIFIED: call cleanup::spawn_cleanup_task(...)
├── db/
│   └── scans.rs         # MODIFIED: add delete_expired_scans_by_tier()
└── orchestrator/
    └── worker_pool.rs   # MODIFIED: tier-based expires_at in execute_scan_internal
```

### Pattern 1: Tier-Based expires_at Fix

**What:** Change `chrono::Duration::days(3)` to be tier-conditional in `execute_scan_internal`.

**Current code** (worker_pool.rs line 267):
```rust
let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::days(3);
```

**Fixed code:**
```rust
let expires_at = chrono::Utc::now().naive_utc() + match tier {
    "authenticated" => chrono::Duration::days(30),
    _ => chrono::Duration::hours(24), // "free" tier: 24h
};
```

The `tier` parameter is already available in `execute_scan_internal` as `&str`. This is a minimal one-line change.

**Confidence:** HIGH — `tier` is already a parameter; `chrono::Duration::hours` and `chrono::Duration::days` are already used in this file.

### Pattern 2: Two-Query DELETE for Per-Tier Count

**What:** Run two DELETE queries — one for `tier = 'free'`, one for `tier = 'authenticated'` — to get per-tier row counts for the structured log.

**Why two queries:** A single DELETE cannot return per-tier counts. `RETURNING tier` would work but requires collecting all rows and grouping in Rust. Two targeted DELETEs is simpler and more readable.

```sql
-- Anonymous (free tier) — 24h expiry, 24h grace period = 48h total from expiry
DELETE FROM scans
WHERE tier = 'free'
  AND status IN ('completed', 'failed')
  AND expires_at + INTERVAL '24 hours' < NOW()

-- Developer (authenticated tier) — 30 day expiry, 24h grace period
DELETE FROM scans
WHERE tier = 'authenticated'
  AND status IN ('completed', 'failed')
  AND expires_at + INTERVAL '24 hours' < NOW()
```

Note: The WHERE clause uses `expires_at + INTERVAL '24 hours' < NOW()` per the phase context decision. This means a scan with `expires_at = T` is deleted when `NOW() > T + 24h`. For free tier scans: `expires_at = created_at + 24h`, so deletion happens at `created_at + 48h`. This satisfies RETN-01 (accessible for at least 24h, then deleted after an additional 24h grace). The grace period is intentional per the locked decision.

**CASCADE behavior:** `findings` FK to `scans` is `ON DELETE CASCADE` (migration 20260204000002). `paid_audits.scan_id` FK was changed to `SET NULL` in Phase 30 (migration 20260218000001). No other tables reference `scans`. PostgreSQL handles the cascade automatically on DELETE — no application-level join deletion needed.

### Pattern 3: Tokio Interval Task with CancellationToken

**What:** Hourly background task that loops with `tokio::select!` on the interval tick and the cancellation token.

**Key design choice: `interval_at` vs `interval`:**
- `tokio::time::interval(Duration::from_secs(3600))` fires immediately at startup (first tick at `Instant::now()`), then every hour.
- `tokio::time::interval_at(Instant::now() + Duration::from_secs(3600), Duration::from_secs(3600))` defers the first tick by one full hour.

**Recommendation:** Use `interval_at` to defer the first tick. Running a DELETE immediately at every deploy is unnecessary noise; the hourly schedule is the correct operational cadence.

**MissedTickBehavior:** Set to `MissedTickBehavior::Skip`. If a tick is missed (e.g., the DELETE took longer than 1 hour, which is essentially impossible given expected scan volumes), skip missed ticks rather than bursting.

```rust
// Source: tokio docs + tokio_util TaskTracker pattern
use tokio::time::{interval_at, Duration, Instant, MissedTickBehavior};
use tokio_util::sync::CancellationToken;
use sqlx::PgPool;

pub fn spawn_cleanup_task(
    pool: PgPool,
    task_tracker: &tokio_util::task::TaskTracker,
    shutdown_token: CancellationToken,
) {
    task_tracker.spawn(async move {
        let period = Duration::from_secs(3600);
        let start = Instant::now() + period; // defer first tick by 1 hour
        let mut interval = interval_at(start, period);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    run_cleanup(&pool).await;
                }
                _ = shutdown_token.cancelled() => {
                    tracing::info!("Retention cleanup task shutting down");
                    break;
                }
            }
        }
    });
}
```

**Graceful shutdown:** When `shutdown_token.cancel()` is called (by `orchestrator.initiate_shutdown()`), the `select!` arm `_ = shutdown_token.cancelled()` resolves and the loop exits. If a DELETE is in progress at shutdown time, it runs to completion before the `tokio::select!` is polled — the `task_tracker.wait()` in `main.rs` then observes the task completing.

**IMPORTANT: task_tracker ownership concern.** Currently in `main.rs`, `task_tracker` is moved into `ScanOrchestrator::new(pool.clone(), max_concurrent, task_tracker, shutdown_token.clone())`. The cleanup task must be spawned on the same tracker _before_ this move, or `task_tracker` must be cloned (it implements `Clone` — it's an `Arc` internally). The correct approach is to clone: `let cleanup_tracker = task_tracker.clone()` before creating the orchestrator, then `cleanup::spawn_cleanup_task(pool.clone(), &cleanup_tracker, shutdown_token.clone())`. Actually, more simply: since `TaskTracker` is `Clone` + `Sync`, we can spawn the cleanup task after creating the orchestrator using a clone of the orchestrator's inner tracker — but the orchestrator doesn't expose the tracker. The cleanest solution: clone `task_tracker` before moving it into the orchestrator, and use the clone for the cleanup spawn.

**Revised wiring in main.rs:**
```rust
// Clone before move — both refer to the same underlying tracker
let task_tracker = TaskTracker::new();
let shutdown_token = CancellationToken::new();

// Spawn cleanup task before orchestrator consumes task_tracker
shipsecure::cleanup::spawn_cleanup_task(
    pool.clone(),
    &task_tracker,
    shutdown_token.clone(),
);

let orchestrator = Arc::new(ScanOrchestrator::new(
    pool.clone(), max_concurrent, task_tracker, shutdown_token.clone()
));
```

This works because `task_tracker.spawn()` takes `&self` (not `&mut self`), so we can call it before the move. The cleanup task is registered on the tracker that is then moved into the orchestrator. When `orchestrator.initiate_shutdown()` calls `task_tracker.close()`, both the orchestrator's scan tasks and the cleanup task are covered by the same tracker.

### Pattern 4: Structured Log Format

**What:** Per-tier counts logged at INFO level after each cleanup run, including zero counts.

```rust
tracing::info!(
    anonymous_deleted = anonymous_count,
    developer_deleted = developer_count,
    total_deleted = anonymous_count + developer_count,
    "retention_cleanup"
);
```

This produces the desired log line format with structured fields. The message `"retention_cleanup"` is a stable event identifier; numeric fields are structured for JSON log aggregation.

### Anti-Patterns to Avoid

- **Immediate first tick:** Don't use `tokio::time::interval()` — it fires at startup, causing a DELETE on every deploy restart.
- **Checking status with a JOIN:** The WHERE clause `status IN ('completed', 'failed')` is sufficient. Never delete `pending` or `in_progress` scans regardless of age (deferred to future phase).
- **Using `expires_at < NOW()`:** The grace period decision requires `expires_at + INTERVAL '24 hours' < NOW()`. Using `expires_at < NOW()` would delete scans as soon as the token expires, before the 24-hour grace period has elapsed.
- **Running DELETE inside the shutdown_token select arm:** The shutdown arm should only `break`. Never start new work when a shutdown signal arrives.
- **Logging skipped in-progress count:** Explicitly excluded from logging per user decision.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Graceful shutdown coordination | Custom channel/flag | `TaskTracker` + `CancellationToken` | Already established in this codebase; proven pattern from tokio-util |
| Periodic scheduling | Custom sleep loop | `tokio::time::interval_at` | Handles missed ticks correctly; cleaner than manual `sleep` in a loop |
| CASCADE deletion | Application-level JOIN DELETE | PostgreSQL CASCADE on FK | DB guarantees atomicity; findings deleted in same transaction |

**Key insight:** The entire solution uses patterns already established in this codebase. No new libraries, no new patterns.

## Common Pitfalls

### Pitfall 1: Deleting in-progress scans
**What goes wrong:** The cleanup query uses `expires_at + INTERVAL '24 hours' < NOW()` without a status filter. A scan submitted near its tier's expiry window (e.g., a free-tier scan submitted 23h ago that is still `in_progress`) gets deleted while the orchestrator holds its scan_id in memory. Subsequent DB writes from the scan (setting `results_token`, updating stage flags) silently succeed with 0 rows updated, and the orchestrator logs an error or completes without persisting results.
**Why it happens:** Missing `status IN ('completed', 'failed')` in the WHERE clause.
**How to avoid:** Always include `AND status IN ('completed', 'failed')` in the DELETE WHERE clause. This is already in the phase context decisions.
**Warning signs:** DB UPDATE on scan returning 0 rows after cleanup runs.

### Pitfall 2: Grace period offset direction
**What goes wrong:** Using `WHERE expires_at < NOW()` instead of `WHERE expires_at + INTERVAL '24 hours' < NOW()`. This deletes scans immediately when `expires_at` passes, ignoring the 24-hour grace period. Users see the "Expired" badge and then find the data gone immediately.
**Why it happens:** Confusing "token is expired" (expires_at < NOW) with "data should be deleted" (expires_at + grace < NOW).
**How to avoid:** Use `expires_at + INTERVAL '24 hours' < NOW()` exactly as specified in the phase context.
**Warning signs:** Scan dashboard shows expired badge and then immediately shows missing scan on next page load.

### Pitfall 3: task_tracker ownership in main.rs
**What goes wrong:** Trying to spawn the cleanup task after `task_tracker` has been moved into `ScanOrchestrator::new()`. This produces a compile error: "use of moved value".
**Why it happens:** `ScanOrchestrator::new` takes `task_tracker: TaskTracker` by value (not by reference).
**How to avoid:** Spawn the cleanup task using `task_tracker.spawn()` before the orchestrator move. Since `TaskTracker::spawn` takes `&self`, this is possible — the cleanup task is registered on the tracker before it is moved.
**Warning signs:** Rust compiler error "use of moved value: `task_tracker`".

### Pitfall 4: expires_at is NULL for old scans
**What goes wrong:** Some scans in the database may have `expires_at IS NULL` (scans created before Phase 33 set tier-based expiry, or scans where `set_results_token` was never called because the scan failed before completion). The DELETE query `WHERE expires_at + INTERVAL '24 hours' < NOW()` is safe — PostgreSQL evaluates `NULL + INTERVAL '24 hours'` as NULL, and `NULL < NOW()` as NULL (not TRUE), so NULL-expires_at scans are never deleted. This is correct behavior.
**Why it happens:** `expires_at` is nullable on the `scans` table.
**How to avoid:** No action needed — PostgreSQL NULL semantics protect against accidental deletion of NULL-expires_at rows.
**Warning signs:** None — this is safe by design.

### Pitfall 5: Disk artifacts — NO ACTION NEEDED
**What was investigated:** The scanners use `tempfile::NamedTempFile` in `src/scanners/container.rs` for Nuclei and testssl.sh JSON output. `NamedTempFile` is RAII — the OS file is deleted when the Rust binding is dropped, which happens when `run_nuclei` and `run_testssl` return. There are no persistent per-scan files on disk, no scan-specific directories, and no artifacts that survive the function call. Cleanup of disk artifacts is explicitly a no-op for this phase.

## Code Examples

Verified patterns from official sources:

### Delete expired scans — two-query approach for per-tier counts

```rust
// src/db/scans.rs — to be added
/// Delete expired completed/failed scans for a specific tier.
///
/// Returns the number of rows deleted.
/// Only targets status IN ('completed', 'failed') — never deletes pending/in_progress.
/// Applies the 24-hour grace period: expires_at + INTERVAL '24 hours' < NOW().
/// findings rows are CASCADE deleted by PostgreSQL automatically.
#[allow(dead_code)]
pub async fn delete_expired_scans_by_tier(pool: &PgPool, tier: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM scans
         WHERE tier = $1
           AND status IN ('completed', 'failed')
           AND expires_at + INTERVAL '24 hours' < NOW()"
    )
    .bind(tier)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
```

Note: `result.rows_affected()` returns `u64` on `PgQueryResult` (confirmed in sqlx-postgres 0.8.6 source).

### Cleanup task — src/cleanup.rs

```rust
// src/cleanup.rs
use sqlx::PgPool;
use tokio::time::{interval_at, Duration, Instant, MissedTickBehavior};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::db::scans;

/// Spawn the hourly retention cleanup task.
///
/// Deletes expired completed/failed scans (with 24h grace period beyond expires_at).
/// Registered with the TaskTracker so graceful shutdown waits for any in-progress DELETE.
/// First tick is deferred by 1 hour (no cleanup on startup).
pub fn spawn_cleanup_task(pool: PgPool, task_tracker: &TaskTracker, shutdown_token: CancellationToken) {
    task_tracker.spawn(async move {
        let period = Duration::from_secs(3600);
        let mut interval = interval_at(Instant::now() + period, period);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    run_cleanup(&pool).await;
                }
                _ = shutdown_token.cancelled() => {
                    tracing::info!("Retention cleanup task shutting down");
                    break;
                }
            }
        }
    });
}

async fn run_cleanup(pool: &PgPool) {
    let anon = match scans::delete_expired_scans_by_tier(pool, "free").await {
        Ok(n) => n,
        Err(e) => {
            tracing::error!(error = %e, "retention_cleanup_failed");
            return;
        }
    };

    let dev = match scans::delete_expired_scans_by_tier(pool, "authenticated").await {
        Ok(n) => n,
        Err(e) => {
            tracing::error!(error = %e, "retention_cleanup_failed");
            return;
        }
    };

    tracing::info!(
        anonymous_deleted = anon,
        developer_deleted = dev,
        total_deleted = anon + dev,
        "retention_cleanup"
    );
}
```

Note: On first DB failure, we log the error and return early (skipping the second query). This matches the "log error, wait for next hourly tick" decision. On developer-tier failure after anonymous succeeds, the anonymous rows are already deleted (no transaction wrapping the two DELETEs). This is acceptable — the two tiers are independent, and the next tick will catch any missed developer rows.

### Wire into main.rs

```rust
// In main() after creating task_tracker and shutdown_token, before orchestrator:
shipsecure::cleanup::spawn_cleanup_task(
    pool.clone(),
    &task_tracker,         // &TaskTracker — spawn() takes &self
    shutdown_token.clone(),
);

let orchestrator = Arc::new(ScanOrchestrator::new(
    pool.clone(), max_concurrent, task_tracker, shutdown_token.clone()
));
```

### lib.rs addition

```rust
pub mod cleanup;
```

### expires_at tier-based fix in worker_pool.rs

```rust
// Line 267 in execute_scan_internal — replace hardcoded days(3):
let expires_at = chrono::Utc::now().naive_utc() + match tier {
    "authenticated" => chrono::Duration::days(30),
    _ => chrono::Duration::hours(24), // free tier: 24 hours
};
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `chrono::Duration::days(3)` hardcoded for all tiers | Tier-conditional `hours(24)` / `days(30)` | This phase (fix carried over from Phase 33) | RETN-01 and RETN-02 become enforceable |
| No cleanup task | Hourly `interval_at` Tokio task | This phase | Expired scans automatically hard-deleted |

**Deprecated/outdated:**
- `expires_at = Utc::now() + Duration::days(3)` at line 267 of `worker_pool.rs`: superseded by tier-conditional logic in this phase.

## Open Questions

1. **Transaction wrapping the two DELETE queries**
   - What we know: The phase context says "single DELETE query" — but per-tier breakdown requires two queries. The current design runs two separate DELETEs without a transaction.
   - What's unclear: Should both DELETEs be in a single transaction? If developer-tier DELETE fails after anonymous-tier DELETE succeeds, anonymous rows are gone but the error is logged and retried next hour. This seems acceptable given the non-critical nature of the cleanup.
   - Recommendation: Do not wrap in a transaction. The queries are independent by tier; no consistency guarantee is needed between them. Next hourly tick catches any missed rows. This matches the "single DELETE query — no batching needed" spirit.

2. **What happens to in-progress scans at their expiry boundary?**
   - What we know: The WHERE clause includes `AND status IN ('completed', 'failed')`, so in-progress scans are never deleted regardless of their `expires_at`. A free-tier scan that somehow stays `in_progress` for > 48h will never be cleaned up by this task. Stuck scan detection is explicitly deferred.
   - What's unclear: Nothing — this is by design and the deferred phase will handle it.
   - Recommendation: No action in this phase.

## Sources

### Primary (HIGH confidence)

- Codebase inspection: `src/orchestrator/worker_pool.rs` lines 86-148, 267 — TaskTracker.spawn pattern, hardcoded expires_at
- Codebase inspection: `src/main.rs` lines 221-225 — TaskTracker and CancellationToken initialization
- Codebase inspection: `src/db/scans.rs` — execute + no-result query pattern
- Codebase inspection: `src/scanners/container.rs` lines 95-98, 151-153 — NamedTempFile RAII, no persistent disk artifacts
- Codebase inspection: `migrations/20260204000002_create_findings.sql` — `ON DELETE CASCADE` confirmed
- Codebase inspection: `migrations/20260218000001_stripe_removal_schema.sql` — `paid_audits.scan_id ON DELETE SET NULL` confirmed
- `/home/john/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.49.0/src/time/interval.rs` — `interval_at`, `MissedTickBehavior::Skip` API
- `/home/john/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-util-0.7.18/src/task/task_tracker.rs` — `TaskTracker::spawn(&self)` takes `&self`, Clone impl
- `/home/john/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sqlx-postgres-0.8.6/src/query_result.rs` — `PgQueryResult::rows_affected() -> u64`

### Secondary (MEDIUM confidence)

- Phase 33 verification report: confirms `expires_at` was NOT updated to be tier-based in Phase 33 — the fix is required in Phase 35

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies already in Cargo.toml, patterns verified in source
- Architecture: HIGH — patterns verified against existing codebase conventions
- Pitfalls: HIGH — verified from codebase inspection (pitfall 5 definitively resolved by reading scanner source)

**Research date:** 2026-02-18
**Valid until:** 2026-03-20 (stable codebase, no fast-moving dependencies)
