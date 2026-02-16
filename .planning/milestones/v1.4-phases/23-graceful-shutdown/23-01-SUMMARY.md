---
phase: 23-graceful-shutdown
plan: 01
subsystem: orchestration
tags: [shutdown, concurrency, task-tracking]
dependency_graph:
  requires: [tokio-util-0.7]
  provides: [orchestrator-shutdown-api, task-tracking]
  affects: [scan-lifecycle, main-initialization]
tech_stack:
  added: [tokio-util]
  patterns: [TaskTracker, CancellationToken, graceful-shutdown]
key_files:
  created: []
  modified:
    - Cargo.toml
    - Cargo.lock
    - src/orchestrator/worker_pool.rs
    - src/main.rs
decisions:
  - "tokio-util rt feature (not sync feature) provides both TaskTracker and CancellationToken access"
  - "Shutdown checks placed before queue depth increment and after semaphore acquire for clean metrics"
  - "Inner scanner spawns remain as tokio::spawn (correct - they are joined within scan task)"
  - "TaskTracker and CancellationToken created in main.rs and passed to ScanOrchestrator constructor"
metrics:
  duration_seconds: 208
  tasks_completed: 2
  files_modified: 4
  deviations: 1
completed: 2026-02-16
---

# Phase 23 Plan 01: Task Tracking and Cancellation Integration Summary

Integrated TaskTracker and CancellationToken into ScanOrchestrator for graceful shutdown coordination, replacing fire-and-forget tokio::spawn with tracked spawns that respect shutdown signaling.

## Overview

Added tokio-util dependency and integrated TaskTracker/CancellationToken primitives into the scan orchestration layer. Background scan tasks are now tracked and check for shutdown signals at strategic points (before queuing and after acquiring semaphore permit).

## Tasks Completed

### Task 1: Add tokio-util dependency
**Status:** Complete
**Commit:** 4af51d1

Added tokio-util 0.7 to Cargo.toml with the `rt` feature, which provides both TaskTracker (via `tokio_util::task`) and CancellationToken (via `tokio_util::sync`). Verified dependency resolves correctly with cargo check.

**Files modified:**
- Cargo.toml
- Cargo.lock

### Task 2: Integrate TaskTracker and CancellationToken into ScanOrchestrator
**Status:** Complete
**Commit:** 49414a4

Modified ScanOrchestrator to accept TaskTracker and CancellationToken in constructor, replaced fire-and-forget tokio::spawn with task_tracker.spawn, added shutdown token checks at two critical points (before queuing, after semaphore acquire), and exposed shutdown coordination methods for external use.

**Files modified:**
- src/orchestrator/worker_pool.rs
- src/main.rs

**Key changes:**
- Added `use tokio_util::sync::CancellationToken` and `use tokio_util::task::TaskTracker` imports
- Added `task_tracker: TaskTracker` and `shutdown_token: CancellationToken` fields to ScanOrchestrator struct
- Updated constructor signature to accept task_tracker and shutdown_token parameters
- Replaced `tokio::spawn` with `self.task_tracker.spawn` in both `spawn_scan` and `spawn_paid_scan`
- Added shutdown checks:
  - Before incrementing scan_queue_depth gauge: early return if shutdown in progress
  - After acquiring semaphore permit: early return if shutdown in progress
- Added public shutdown API methods:
  - `is_shutting_down() -> bool`
  - `shutdown_token() -> CancellationToken`
  - `initiate_shutdown()` - closes tracker and cancels token
  - `wait_for_drain() -> async` - waits for all tracked tasks to complete
- Updated main.rs to create TaskTracker and CancellationToken instances and pass to orchestrator

**Verification:**
- cargo check: compiles without errors
- cargo test orchestrator::worker_pool::tests: both tests pass
- grep "task_tracker.spawn": returns 2 (spawn_scan, spawn_paid_scan)
- grep "is_cancelled": returns 5 (4 in spawn methods + 1 in is_shutting_down)
- Inner scanner spawns (headers_handle, tls_handle, etc.) correctly remain as tokio::spawn

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking Issue] Updated main.rs to create TaskTracker and CancellationToken**
- **Found during:** Task 2 verification
- **Issue:** ScanOrchestrator constructor signature changed but main.rs still called the old constructor with 2 arguments instead of 4
- **Fix:** Added TaskTracker::new() and CancellationToken::new() instantiation in main.rs before creating ScanOrchestrator
- **Files modified:** src/main.rs
- **Commit:** 49414a4 (combined with Task 2)
- **Justification:** Required to complete Task 2 - the orchestrator cannot be instantiated without these parameters. This is critical missing functionality that blocks compilation.

## Verification Results

All verification criteria met:

- tokio-util 0.7 with rt feature in Cargo.toml and Cargo.lock
- ScanOrchestrator::new() accepts TaskTracker and CancellationToken parameters
- spawn_scan and spawn_paid_scan use self.task_tracker.spawn() instead of tokio::spawn()
- Both spawn methods check shutdown_token.is_cancelled() before queue depth increment and after semaphore acquire (2 checks each = 4 total)
- initiate_shutdown(), wait_for_drain(), is_shutting_down(), shutdown_token() methods available
- Orchestrator unit tests pass (test_compute_score, test_deduplicate_findings)
- Inner scanner spawns correctly remain as tokio::spawn (they are joined within the scan task via tokio::join!)

## Success Criteria

- [x] tokio-util 0.7 with rt feature in Cargo.toml
- [x] ScanOrchestrator::new() accepts TaskTracker and CancellationToken parameters
- [x] spawn_scan and spawn_paid_scan use self.task_tracker.spawn() instead of tokio::spawn()
- [x] Both spawn methods check shutdown_token.is_cancelled() before queue depth and after semaphore acquire
- [x] initiate_shutdown(), wait_for_drain(), is_shutting_down(), shutdown_token() methods available
- [x] All existing tests pass

## Next Steps

This plan provides the orchestrator-level shutdown primitives. Next plan (23-02) will integrate these primitives with signal handling in main.rs to implement coordinated shutdown flow on SIGTERM/SIGINT.

## Self-Check: PASSED

Verifying all claimed artifacts exist:

**Files modified:**
- Cargo.toml: EXISTS (tokio-util dependency added)
- Cargo.lock: EXISTS (dependency resolved)
- src/orchestrator/worker_pool.rs: EXISTS (TaskTracker integration)
- src/main.rs: EXISTS (TaskTracker/CancellationToken instantiation)

**Commits:**
- 4af51d1: EXISTS (chore(23-01): add tokio-util dependency with rt feature)
- 49414a4: EXISTS (feat(23-01): integrate TaskTracker and CancellationToken for graceful shutdown)

**Key functionality:**
- grep "task_tracker.spawn" src/orchestrator/worker_pool.rs: 2 matches (VERIFIED)
- grep "is_cancelled" src/orchestrator/worker_pool.rs: 5 matches (VERIFIED)
- grep "pub fn initiate_shutdown" src/orchestrator/worker_pool.rs: FOUND
- grep "pub async fn wait_for_drain" src/orchestrator/worker_pool.rs: FOUND
- cargo test orchestrator::worker_pool::tests: 2 passed (VERIFIED)

All claims verified.
