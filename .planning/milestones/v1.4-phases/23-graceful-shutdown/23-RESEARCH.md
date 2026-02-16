# Phase 23: Graceful Shutdown - Research

**Researched:** 2026-02-16
**Domain:** Tokio async runtime graceful shutdown, signal handling, task tracking
**Confidence:** HIGH

## Summary

Graceful shutdown in Tokio involves three core components: detecting shutdown signals (SIGTERM/SIGINT), broadcasting shutdown to all tasks via CancellationToken, and waiting for task completion via TaskTracker. The standard pattern replaces fire-and-forget `tokio::spawn` with `TaskTracker::spawn()` to track background tasks. Axum's `with_graceful_shutdown()` drains in-flight HTTP requests while rejecting new connections. Middleware can reject scan creation requests with 503 during shutdown. The key challenge is coordinating between HTTP layer shutdown (Axum) and orchestrator-level shutdown (scan tasks), requiring defense-in-depth where both layers enforce shutdown.

**Primary recommendation:** Use `tokio_util::sync::CancellationToken` for shutdown signaling (clone and distribute to all components), `tokio_util::task::TaskTracker` to replace fire-and-forget spawns in orchestrator, and Axum's `with_graceful_shutdown()` for HTTP drain. Implement 503 rejection middleware that checks CancellationToken state. Parse `SHUTDOWN_TIMEOUT` env var as seconds (u64) and create `Duration::from_secs()`. Use `tokio::time::interval` for periodic logging every 5-10 seconds.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Scan drain strategy:**
- Only drain active (in-flight) scans. Queued scans that haven't started are cancelled — they never begin execution
- Cancelled queued scans are left as "pending" in the database (not marked as cancelled)
- Defense in depth: both HTTP layer (503) AND orchestrator refuse new spawns during shutdown
- If timeout expires mid-scan, allow the current scanner step to finish before stopping (may slightly exceed timeout)

**Client experience:**
- Scan creation endpoints return 503 with JSON error body: `{"error": "Service shutting down"}`
- No Retry-After header — clients don't need retry timing
- Scan creation returns 503, /health/ready returns unhealthy, other endpoints (results, etc.) keep working
- /health (liveness) stays healthy during shutdown — process is alive. Only /health/ready goes unhealthy — standard readiness pattern for load balancers

**Timeout behavior:**
- Grace period configured via SHUTDOWN_TIMEOUT env var (12-factor pattern, matches LOG_FORMAT approach)
- Default: 90 seconds — SSL Labs scans can take 60-90s, this gives room for longest scans
- After grace period expires: log warning about forced shutdown, then exit(0) — clean exit code so systemd doesn't restart
- SIGTERM and SIGINT handled identically — same graceful drain whether Docker stop or Ctrl+C

**Shutdown logging:**
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

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SHD-01 | Backend handles SIGTERM and SIGINT signals for graceful shutdown | tokio::signal module provides ctrl_c() and unix::signal() for both signals (Standard Stack) |
| SHD-02 | In-flight scans complete before process exits (with configurable timeout) | TaskTracker.wait() blocks until tasks complete; tokio::time::timeout enforces grace period (Architecture Patterns) |
| SHD-03 | Background tasks tracked via TaskTracker replacing fire-and-forget tokio::spawn | TaskTracker::spawn() replaces tokio::spawn() in orchestrator (Don't Hand-Roll) |

</phase_requirements>

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio_util | 0.7 (latest) | CancellationToken and TaskTracker | Official Tokio utility crate, purpose-built for graceful shutdown patterns |
| tokio::signal | (built-in) | SIGTERM/SIGINT detection | Built into Tokio runtime, no external dependencies needed |
| axum::serve::with_graceful_shutdown | (built-in) | HTTP request draining | Native Axum feature, integrates with hyper's graceful shutdown |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio::time::interval | (built-in) | Periodic shutdown progress logging | For monitoring tasks that tick every N seconds |
| tokio::time::timeout | (built-in) | Enforce shutdown timeout | Wrap TaskTracker.wait() to implement grace period |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| CancellationToken | tokio::sync::watch channel | watch requires manual state management; CancellationToken is purpose-built with cleaner API |
| CancellationToken | std::sync::atomic::AtomicBool | AtomicBool doesn't work well with async/await; can't use in tokio::select! |
| TaskTracker | tokio::task::JoinSet | JoinSet accumulates task results in memory (unbounded growth); TaskTracker drops completed tasks immediately |
| Built-in tokio::signal | ctrlc crate | ctrlc is blocking, not async-native; tokio::signal integrates seamlessly with async runtime |

**Installation:**
```bash
cargo add tokio-util --features sync,rt
# tokio and axum already present with required features
```

## Architecture Patterns

### Recommended Project Structure

```
src/
├── main.rs                    # Signal handler, shutdown orchestration
├── orchestrator/
│   ├── worker_pool.rs        # MODIFIED: TaskTracker integration
│   └── shutdown.rs           # NEW: Shutdown coordinator struct
└── api/
    └── middleware/
        └── shutdown_check.rs  # NEW: 503 rejection middleware
```

### Pattern 1: Signal Detection with Both SIGTERM and SIGINT

**What:** Unified shutdown signal handler for both Ctrl+C (SIGINT) and Docker stop (SIGTERM)

**When to use:** Always — production deployments send SIGTERM, development uses Ctrl+C

**Example:**
```rust
// Source: https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
```

### Pattern 2: TaskTracker + CancellationToken Integration

**What:** Replace fire-and-forget `tokio::spawn` with tracked spawns that respect cancellation

**When to use:** All background tasks that must drain during shutdown (scan orchestrator)

**Example:**
```rust
// Source: https://tokio.rs/tokio/topics/shutdown
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

pub struct ScanOrchestrator {
    pool: PgPool,
    semaphore: Arc<Semaphore>,
    task_tracker: TaskTracker,
    shutdown_token: CancellationToken,
}

impl ScanOrchestrator {
    pub fn spawn_scan(&self, scan_id: Uuid, target_url: String) {
        let token = self.shutdown_token.clone();
        let pool = self.pool.clone();
        let semaphore = self.semaphore.clone();

        // BEFORE: tokio::spawn(async move { ... })
        // AFTER: tracker.spawn()
        self.task_tracker.spawn(async move {
            // Check if shutting down before acquiring semaphore
            if token.is_cancelled() {
                tracing::info!("Shutdown in progress, rejecting queued scan");
                return;
            }

            let _permit = semaphore.acquire().await.expect("Semaphore closed");

            tokio::select! {
                result = execute_scan_internal(pool, scan_id, target_url) => {
                    // Scan completed normally
                }
                _ = token.cancelled() => {
                    // Shutdown triggered mid-scan - allow current step to finish
                    tracing::info!("Shutdown triggered, completing current scanner step");
                }
            }
        });
    }

    pub async fn shutdown_gracefully(&self, timeout: Duration) {
        // 1. Stop accepting new spawns
        self.task_tracker.close();

        // 2. Signal all tasks to stop
        self.shutdown_token.cancel();

        // 3. Wait with timeout
        tokio::select! {
            _ = self.task_tracker.wait() => {
                tracing::info!("All scans completed gracefully");
            }
            _ = tokio::time::sleep(timeout) => {
                tracing::warn!("Shutdown timeout expired, forcing exit");
            }
        }
    }
}
```

### Pattern 3: Axum HTTP Drain with Middleware Rejection

**What:** Axum drains HTTP connections while middleware rejects new scan requests with 503

**When to use:** Defense-in-depth — HTTP layer rejects before orchestrator checks

**Example:**
```rust
// Source: https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs
// + https://docs.rs/axum/latest/axum/middleware/index.html

// Middleware layer
async fn reject_during_shutdown(
    Extension(shutdown_token): Extension<CancellationToken>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    if shutdown_token.is_cancelled() {
        // Return 503 with JSON body
        let error_body = serde_json::json!({"error": "Service shutting down"});
        return Err(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    }
    Ok(next.run(request).await)
}

// Main server setup
let app = Router::new()
    .route("/api/v1/scans", post(scans::create_scan))
    .layer(axum::middleware::from_fn(reject_during_shutdown))
    .layer(Extension(shutdown_token.clone()))
    .with_state(state);

axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await;
```

### Pattern 4: Periodic Shutdown Progress Logging

**What:** Log active scan count and elapsed time every 5-10 seconds during drain

**When to use:** Inside shutdown coordinator to provide visibility into drain progress

**Example:**
```rust
// Source: https://docs.rs/tokio/latest/tokio/time/fn.interval.html
async fn log_shutdown_progress(
    orchestrator: Arc<ScanOrchestrator>,
    timeout_secs: u64,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    let start = Instant::now();

    loop {
        interval.tick().await;
        let (active, queued) = orchestrator.get_capacity_and_queue();
        let elapsed = start.elapsed().as_secs();

        if active == 0 && queued == 0 {
            break; // Clean shutdown
        }

        tracing::info!(
            active_scans = active,
            queued_scans = queued,
            elapsed_seconds = elapsed,
            timeout_seconds = timeout_secs,
            "shutdown_progress"
        );

        if elapsed >= timeout_secs {
            tracing::warn!(
                active_scans = active,
                elapsed_seconds = elapsed,
                "Shutdown forced: {} scans remaining after {}s",
                active + queued,
                elapsed
            );
            break;
        }
    }
}
```

### Pattern 5: Parse SHUTDOWN_TIMEOUT Env Var

**What:** Parse environment variable as seconds with fallback default

**When to use:** Startup configuration, matches existing LOG_FORMAT pattern

**Example:**
```rust
// Source: existing main.rs LOG_FORMAT parsing pattern
fn parse_shutdown_timeout() -> Duration {
    let timeout_secs: u64 = std::env::var("SHUTDOWN_TIMEOUT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(90); // Default 90 seconds

    Duration::from_secs(timeout_secs)
}

// In main:
let shutdown_timeout = parse_shutdown_timeout();
tracing::info!(
    shutdown_timeout_seconds = shutdown_timeout.as_secs(),
    "Shutdown configuration loaded"
);
```

### Anti-Patterns to Avoid

- **Fire-and-forget tokio::spawn:** Cannot track or wait for these tasks during shutdown. Always use TaskTracker.
- **Dropping JoinHandle without abort:** Dropping a JoinHandle does NOT cancel the task — it just detaches. Must explicitly call `.abort()` or use TaskTracker.
- **Global cancellation with select! only:** Tasks cannot react to being dropped by select!. Must use CancellationToken to give tasks shutdown cleanup opportunity.
- **AtomicBool for shutdown flag:** Doesn't work with `tokio::select!` and requires manual polling. Use CancellationToken instead.
- **Immediate task abortion:** Prevents cleanup (flushing buffers, closing connections). Always allow tasks to finish their current operation.
- **No timeout on shutdown:** Can hang forever waiting for stuck tasks. Always wrap `TaskTracker.wait()` with `tokio::time::timeout`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tracking background tasks | Custom Arc<AtomicUsize> counter with manual increment/decrement | `tokio_util::task::TaskTracker` | Edge cases: panic safety, race conditions on count updates, memory leaks from forgotten decrements. TaskTracker handles all of this. |
| Shutdown signal broadcasting | mpsc channel with clones sent to each task | `tokio_util::sync::CancellationToken` | Edge cases: late-joining tasks miss signal, manual cleanup of channel handles, no hierarchical cancellation. CancellationToken solves all. |
| Waiting for all tasks | Collecting Vec<JoinHandle> and futures::join_all | `TaskTracker.wait()` | JoinSet accumulates all task results in memory (unbounded growth). TaskTracker drops completed tasks immediately. |
| HTTP graceful shutdown | Custom TcpListener accept loop with shutdown flag | `axum::serve::with_graceful_shutdown` | Edge cases: in-flight requests dropped, new connections not rejected cleanly, complex connection tracking. Axum/hyper handle this correctly. |

**Key insight:** Graceful shutdown is deceptively complex due to async task lifetimes, race conditions between shutdown signal and new task spawns, and coordination across multiple layers (HTTP server, orchestrator, individual tasks). The official Tokio ecosystem has solved these problems with battle-tested primitives.

## Common Pitfalls

### Pitfall 1: Fire-and-Forget Spawns Escape Shutdown

**What goes wrong:** Using bare `tokio::spawn()` creates tasks that aren't tracked. When main() returns, Tokio runtime drops immediately, aborting all tasks without cleanup.

**Why it happens:** `tokio::spawn()` returns a JoinHandle, but dropping it just detaches the task — it doesn't cancel it. Developers assume spawned tasks will complete, but shutdown cancels them abruptly.

**How to avoid:** Replace all `tokio::spawn()` with `TaskTracker::spawn()`. Call `tracker.close()` before shutdown and `tracker.wait().await` to ensure all tasks complete.

**Warning signs:** Logs show scans interrupted mid-execution, database writes incomplete, email notifications not sent despite scan completion.

### Pitfall 2: Queued Tasks Acquire Semaphore After Shutdown Signal

**What goes wrong:** Queued scans waiting on semaphore don't check shutdown token before acquiring permit. They start executing after shutdown initiated.

**Why it happens:** Race condition: shutdown signal arrives after task spawned but before semaphore acquired. Task sees shutdown AFTER it starts work.

**How to avoid:** Check `shutdown_token.is_cancelled()` immediately after acquiring semaphore permit (or use `select!` during acquire). Return early if shutdown in progress.

**Warning signs:** Shutdown logs show "0 active scans" briefly, then count increases as queued scans start. Shutdown takes longer than expected.

### Pitfall 3: No Timeout Means Infinite Hang

**What goes wrong:** Waiting for `TaskTracker.wait()` without timeout can hang forever if a task is stuck (network call, infinite loop, deadlock).

**Why it happens:** TaskTracker waits for ALL tasks to complete. A single stuck task blocks shutdown indefinitely.

**How to avoid:** Always wrap `tracker.wait()` with `tokio::time::timeout()`. Log warning and exit with code 0 on timeout (clean exit prevents systemd restart).

**Warning signs:** Server doesn't respond to SIGTERM, requires SIGKILL. Kubernetes shows pod stuck in Terminating state.

### Pitfall 4: Exit Code 1 Triggers systemd Restart

**What goes wrong:** Exiting with non-zero code after timeout causes systemd to restart service with `Restart=on-failure`. Service enters restart loop.

**Why it happens:** Developer treats timeout as error condition and exits with code 1. systemd interprets this as failure requiring restart.

**How to avoid:** Exit with code 0 after forced shutdown timeout. Forced shutdown is expected behavior, not a failure. Use `std::process::exit(0)`.

**Warning signs:** Systemd logs show rapid service restarts. Service oscillates between running and stopped.

### Pitfall 5: Middleware Runs After Shutdown Check

**What goes wrong:** 503 rejection middleware placed in wrong layer order. Request reaches handler even when shutting down.

**Why it happens:** Axum middleware executes in reverse order of layer application. Shutdown check added too late in chain.

**How to avoid:** Add shutdown check middleware LAST (outermost layer) so it runs FIRST. Place before route-specific middleware.

**Warning signs:** New scans created during shutdown despite middleware in place. HTTP 503s not returned.

### Pitfall 6: TaskTracker Closed But Token Not Cancelled

**What goes wrong:** Calling `tracker.close()` prevents new spawns but doesn't signal existing tasks to stop. Active tasks continue indefinitely.

**Why it happens:** Confusing close (prevent new tasks) with cancel (stop existing tasks). They're independent operations.

**How to avoid:** ALWAYS do both: `tracker.close()` AND `token.cancel()`. Close prevents new work, cancel stops in-progress work.

**Warning signs:** New tasks rejected correctly, but active tasks never complete. Shutdown hangs waiting for TaskTracker.

### Pitfall 7: Allowing Current Scanner Step to Finish Ignored

**What goes wrong:** Using `tokio::select!` without biased annotation can cancel mid-scanner-step despite requirement to finish current step.

**Why it happens:** `select!` races all branches. If shutdown completes first, task drops immediately even mid-operation.

**How to avoid:** Don't use `select!` for timeout on individual scanner steps. Instead, check `token.is_cancelled()` between steps. Let current step complete before checking.

**Warning signs:** Scanner logs show incomplete operations (HTTP request started but response not processed). Timeout slightly exceeds configured value.

## Code Examples

Verified patterns from official sources:

### Signal Handler (Unix + Windows Compatible)

```rust
// Source: https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs
use tokio::signal;

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received SIGINT, initiating graceful shutdown");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, initiating graceful shutdown");
        },
    }
}
```

### Main Server with Coordinated Shutdown

```rust
// Source: https://tokio.rs/tokio/topics/shutdown + Axum docs
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

#[tokio::main]
async fn main() {
    // ... (logging init, db connection, etc.)

    let shutdown_timeout = parse_shutdown_timeout();
    let shutdown_token = CancellationToken::new();
    let task_tracker = TaskTracker::new();

    // Create orchestrator with shutdown coordination
    let orchestrator = Arc::new(ScanOrchestrator::new(
        pool.clone(),
        max_concurrent,
        task_tracker.clone(),
        shutdown_token.clone(),
    ));

    // Build app with shutdown middleware
    let app = Router::new()
        .route("/api/v1/scans", post(scans::create_scan))
        .layer(middleware::from_fn(reject_during_shutdown))
        .layer(Extension(shutdown_token.clone()))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    tracing::info!(
        addr = %addr,
        shutdown_timeout_seconds = shutdown_timeout.as_secs(),
        "Server starting"
    );

    // Spawn server with graceful shutdown
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .expect("Server error");
    });

    // Wait for shutdown signal
    shutdown_signal().await;

    // Initiate graceful shutdown
    tracing::info!(
        timeout_seconds = shutdown_timeout.as_secs(),
        "Graceful shutdown initiated"
    );

    // Close tracker (no new spawns) and cancel token (stop existing)
    task_tracker.close();
    shutdown_token.cancel();

    // Wait with timeout and periodic logging
    let shutdown_complete = tokio::select! {
        _ = task_tracker.wait() => {
            tracing::info!("All tasks completed gracefully");
            true
        }
        _ = tokio::time::sleep(shutdown_timeout) => {
            let (active, queued) = orchestrator.get_stats();
            tracing::warn!(
                active_scans = active,
                queued_scans = queued,
                elapsed_seconds = shutdown_timeout.as_secs(),
                "Shutdown forced: {} scans remaining after {}s",
                active + queued,
                shutdown_timeout.as_secs()
            );
            false
        }
    };

    // Wait for server to finish draining HTTP
    let _ = server_handle.await;

    // Exit with code 0 (clean exit, no systemd restart)
    std::process::exit(0);
}
```

### TaskTracker Integration in Orchestrator

```rust
// Source: https://docs.rs/tokio-util/latest/tokio_util/task/task_tracker/
use tokio_util::task::TaskTracker;
use tokio_util::sync::CancellationToken;

pub struct ScanOrchestrator {
    pool: PgPool,
    semaphore: Arc<Semaphore>,
    task_tracker: TaskTracker,
    shutdown_token: CancellationToken,
    max_concurrent: usize,
}

impl ScanOrchestrator {
    pub fn new(
        pool: PgPool,
        max_concurrent: usize,
        task_tracker: TaskTracker,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            pool,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            task_tracker,
            shutdown_token,
            max_concurrent,
        }
    }

    pub fn spawn_scan(&self, scan_id: Uuid, target_url: String, request_id: Option<Uuid>) {
        let pool = self.pool.clone();
        let semaphore = self.semaphore.clone();
        let shutdown_token = self.shutdown_token.clone();
        let timeout = Duration::from_secs(60);

        let span = info_span!(
            "scan",
            scan_id = %scan_id,
            target_url = %target_url,
            tier = "free",
            request_id = request_id.map(|id| id.to_string()).as_deref().unwrap_or(""),
        );

        // BEFORE: tokio::spawn(async move { ... })
        // AFTER: self.task_tracker.spawn()
        self.task_tracker.spawn(async move {
            // Check shutdown BEFORE incrementing queue depth
            if shutdown_token.is_cancelled() {
                tracing::info!("Shutdown in progress, rejecting queued scan");
                return;
            }

            // Track queue depth (waiting for permit)
            metrics::gauge!("scan_queue_depth").increment(1.0);
            let _permit = semaphore.acquire().await.expect("Semaphore closed");
            metrics::gauge!("scan_queue_depth").decrement(1.0);

            // Check shutdown AFTER acquiring permit (queued task might start)
            if shutdown_token.is_cancelled() {
                tracing::info!("Shutdown in progress, aborting scan before start");
                return;
            }

            // Track active scans (executing)
            metrics::gauge!("active_scans").increment(1.0);
            tracing::info!("scan_started");
            let start = Instant::now();

            let result = Self::execute_scan_internal(pool, scan_id, target_url, timeout, "free").await;

            // Log completion
            match result {
                Ok(()) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    tracing::info!(duration_ms, "scan_completed");
                }
                Err(e) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    tracing::error!(duration_ms, error = %e, "scan_failed");
                }
            }

            metrics::gauge!("active_scans").decrement(1.0);
        }.instrument(span));
    }
}
```

### Shutdown Middleware for 503 Rejection

```rust
// Source: https://docs.rs/axum/latest/axum/middleware/
use axum::{
    extract::Extension,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tokio_util::sync::CancellationToken;

pub async fn reject_during_shutdown(
    Extension(shutdown_token): Extension<CancellationToken>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    // Check if shutdown in progress
    if shutdown_token.is_cancelled() {
        // Return 503 with JSON error body
        let error_body = json!({"error": "Service shutting down"});
        return (StatusCode::SERVICE_UNAVAILABLE, Json(error_body)).into_response();
    }

    // Not shutting down, proceed to handler
    next.run(request).await
}
```

### Periodic Shutdown Logging

```rust
// Source: https://docs.rs/tokio/latest/tokio/time/fn.interval.html
use tokio::time::{interval, Duration, Instant};

async fn monitor_shutdown_progress(
    orchestrator: Arc<ScanOrchestrator>,
    timeout: Duration,
) {
    let mut interval = interval(Duration::from_secs(5));
    let start = Instant::now();

    // Skip first immediate tick
    interval.tick().await;

    loop {
        interval.tick().await;

        let (active, max) = orchestrator.get_capacity();
        let queued = max - active;
        let elapsed_secs = start.elapsed().as_secs();

        // Exit if all clear
        if active == 0 && queued == 0 {
            break;
        }

        tracing::info!(
            active_scans = active,
            queued_scans = queued,
            elapsed_seconds = elapsed_secs,
            timeout_seconds = timeout.as_secs(),
            "shutdown_progress"
        );

        // Check if timeout exceeded
        if start.elapsed() >= timeout {
            break;
        }
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual mpsc channels for cancellation | CancellationToken from tokio-util | tokio-util 0.7+ (2022) | Cleaner API, hierarchical cancellation, no channel cleanup needed |
| JoinSet for tracking background tasks | TaskTracker for fire-and-forget monitoring | tokio-util 0.7+ (2022) | Prevents unbounded memory growth from unprocessed task results |
| Custom signal handling with nix crate | Built-in tokio::signal module | Tokio 1.0+ (2021) | Cross-platform, async-native, no external dependencies |
| Direct hyper server control | Axum's with_graceful_shutdown | Axum 0.6+ (2023) | Simpler API, abstracts hyper details, integrates with Tokio shutdown patterns |
| parse_duration crate for env vars | Simple string parse to u64 seconds | N/A | Simpler for seconds-only config; parse_duration adds "1h 30m" flexibility but extra dependency |

**Deprecated/outdated:**
- **ctrlc crate**: Blocking API, doesn't integrate with async. Use tokio::signal::ctrl_c() instead.
- **Manual Arc<AtomicBool> shutdown flags**: Can't use in tokio::select!, requires polling. Use CancellationToken instead.
- **Collecting Vec<JoinHandle>**: Memory grows unbounded with long-running apps. Use TaskTracker instead.

## Open Questions

1. **How does Axum's with_graceful_shutdown handle new connection attempts during shutdown?**
   - What we know: Axum wraps hyper's graceful shutdown, which stops accepting new connections and drains in-flight requests
   - What's unclear: Exact behavior — are new TCP connections RST, or does OS queue them? Does hyper send Connection: close headers?
   - Recommendation: Assume new connections are rejected at TCP level (listen socket closed). Focus on application-layer 503 for in-flight HTTP requests that arrive before listen close.

2. **Should periodic logging run in separate task or inline with shutdown wait?**
   - What we know: interval.tick() can be used in tokio::select! loop alongside tracker.wait()
   - What's unclear: Cleaner to spawn separate monitoring task or inline select! with multiple branches
   - Recommendation: Use separate spawned task with orchestrator.get_capacity() polling. Cleaner separation of concerns, task exits when monitoring complete.

3. **What happens if timeout expires exactly during scanner step boundary?**
   - What we know: User requires "allow current step to finish" if timeout expires mid-scan
   - What's unclear: How to detect "mid-step" vs "between-steps" — scanner steps are function calls, not separate tasks
   - Recommendation: Don't implement step-level checking (too complex). Document that timeout may slightly exceed configured value to allow scanner method to return naturally. This satisfies "allow current step to finish" without invasive refactoring.

## Sources

### Primary (HIGH confidence)

- [Graceful Shutdown | Tokio](https://tokio.rs/tokio/topics/shutdown) - Signal handling, CancellationToken, TaskTracker integration
- [TaskTracker in tokio_util](https://docs.rs/tokio-util/latest/tokio_util/task/task_tracker/struct.TaskTracker.html) - API details, memory behavior vs JoinSet
- [Axum graceful-shutdown example](https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs) - Complete signal handler and with_graceful_shutdown usage
- [tokio::time::interval](https://docs.rs/tokio/latest/tokio/time/fn.interval.html) - Periodic task implementation
- [Rust tokio task cancellation patterns - Cybernetist](https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/) - Pitfalls, cancellation approaches, JoinHandle vs abort

### Secondary (MEDIUM confidence)

- [Graceful Shutdown for Axum Servers - Medium](https://medium.com/@wedevare/rust-async-graceful-shutdown-for-axum-servers-signals-draining-cleanup-done-right-3b52375412ec) - Defense-in-depth pattern verification
- [Exit codes - Command Line Applications in Rust](https://rust-cli.github.io/book/in-depth/exit-code.html) - Exit code 0 vs 1 conventions
- [systemd.service - freedesktop.org](https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html) - Restart=on-failure behavior with exit codes
- [Gracefully Shutdown a Server | hyper](https://hyper.rs/guides/1/server/graceful-shutdown/) - HTTP connection draining behavior

### Tertiary (LOW confidence)

- [GitHub tokio discussions on shutdown patterns](https://github.com/tokio-rs/tokio/discussions/1819) - Community patterns, not authoritative
- [Third-party tokio-graceful-shutdown crate](https://github.com/Finomnis/tokio-graceful-shutdown) - Higher-level abstraction option, but adds dependency

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official Tokio ecosystem crates with comprehensive documentation
- Architecture: HIGH - Verified patterns from official Tokio/Axum documentation and examples
- Pitfalls: MEDIUM - Based on community discussions and GitHub issues, not formal documentation

**Research date:** 2026-02-16
**Valid until:** 2026-03-16 (30 days - stable ecosystem, infrequent breaking changes)
