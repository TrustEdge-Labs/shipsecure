# Phase 21: Health Checks - Research

**Researched:** 2026-02-16
**Domain:** Kubernetes health probes, database health checks, Axum endpoint patterns
**Confidence:** HIGH

## Summary

Health checks in production Rust/Axum applications require implementing separate liveness and readiness endpoints following Kubernetes probe patterns. The liveness endpoint (`/health`) provides a fast, dependency-free response confirming the process is alive, while the readiness endpoint (`/health/ready`) validates critical dependencies like database connectivity before accepting traffic. The standard approach uses Axum handlers returning JSON responses with appropriate status codes (200 for healthy, 429 for degraded, 503 for unhealthy) and implements response caching to protect the database from aggressive polling by load balancers.

**Primary recommendation:** Implement two handlers (`health_liveness` and `health_readiness`) placed after routing layers to bypass tracing, use tokio's Semaphore `available_permits()` for scan capacity tracking, execute lightweight `SELECT 1` queries for DB health, measure latency with `std::time::Instant`, cache readiness results with timestamp-based TTL (5 seconds), and return JSON responses with serde-serializable structs via Axum's `Json` extractor.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Readiness response shape
- scan_capacity is an object: `{ active: N, max: N }` showing in-flight scans against limit
- Response body is exactly three fields: `db_connected`, `scan_capacity`, `status` — no extras
- Status field is a string enum: "healthy", "degraded", or "unhealthy"
- Liveness endpoint (/health) returns JSON `{ "status": "ok" }` for consistency across both endpoints

#### Degraded state handling
- Three states: healthy (200), degraded (429), unhealthy (503)
- Degraded triggers on DB latency — DB responds but slowly (above threshold)
- Unhealthy triggers when DB is unreachable
- DB latency threshold configurable via `HEALTH_DB_LATENCY_THRESHOLD_MS` env var, default 50ms

#### Endpoint access control
- /health (liveness) is publicly accessible — external monitoring tools can reach it
- /health/ready (readiness) restricted to localhost via Nginx (same pattern as /metrics — 403 for external)
- Health check requests bypass logging/tracing entirely — consistent with Phase 20 decision
- Readiness check result cached for 5 seconds to protect DB from aggressive polling

### Claude's Discretion
- Exact caching implementation (in-memory timestamp check, tokio mutex, etc.)
- DB connectivity check query (SELECT 1 or equivalent)
- How to measure DB latency for degraded threshold
- Error response body structure when unhealthy

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope

</user_constraints>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| axum | 0.8.8 (installed) | HTTP routing and handlers | Official Tokio project, ecosystem standard for async Rust web APIs |
| serde | 1.x (installed) | JSON serialization | De facto standard for serialization in Rust |
| serde_json | 1.x (installed) | JSON response formatting | Standard companion to serde for JSON |
| sqlx | 0.8.6 (installed) | Database queries | Production-ready async SQL with compile-time verification |
| tokio | 1.x (installed) | Async runtime, Semaphore | Industry standard async runtime with sync primitives |
| std::time::Instant | stdlib | Latency measurement | Zero-cost standard library timing primitive |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| std::sync::Mutex | stdlib | Caching state guard | Protecting cached readiness result (synchronous, lightweight) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom cache | ttl_cache crate | Library adds dependency; hand-rolled timestamp check is simpler for single value |
| tokio::sync::Mutex | std::sync::Mutex | Async mutex is more expensive; synchronous mutex sufficient for non-async cache operations |
| Custom JSON | axum::Json<T> | Axum's Json extractor handles serialization/headers automatically |

**Installation:**
No new dependencies required — all necessary crates already in Cargo.toml.

## Architecture Patterns

### Recommended File Structure
```
src/
├── api/
│   ├── mod.rs              # Add health.rs export
│   └── health.rs           # NEW: Health check handlers
├── lib.rs                  # Export HealthCache if needed in multiple modules
└── main.rs                 # Add health routes AFTER .layer() middleware
```

### Pattern 1: Liveness Handler (Stateless)
**What:** Fast, dependency-free JSON response confirming process is alive
**When to use:** Kubernetes liveness probes, external monitoring tools
**Example:**
```rust
// Source: User decisions + Kubernetes best practices
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
struct LivenessResponse {
    status: String,
}

async fn health_liveness() -> Json<LivenessResponse> {
    Json(LivenessResponse {
        status: "ok".to_string(),
    })
}
```

### Pattern 2: Readiness Handler (Stateful with Cache)
**What:** Deep dependency validation with cached results and latency measurement
**When to use:** Kubernetes readiness probes, load balancer routing decisions
**Example:**
```rust
// Source: Kubernetes readiness patterns + Rust best practices
use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct HealthCache {
    last_check: Mutex<Option<(Instant, ReadinessResponse)>>,
}

#[derive(Serialize, Clone)]
struct ScanCapacity {
    active: usize,
    max: usize,
}

#[derive(Serialize, Clone)]
struct ReadinessResponse {
    db_connected: bool,
    scan_capacity: ScanCapacity,
    status: String, // "healthy", "degraded", "unhealthy"
}

async fn health_readiness(
    State(state): State<AppState>,
    State(cache): State<HealthCache>,
) -> (StatusCode, Json<ReadinessResponse>) {
    // Check cache first
    let now = Instant::now();
    let cache_ttl = Duration::from_secs(5);

    if let Ok(guard) = cache.last_check.lock() {
        if let Some((timestamp, cached_response)) = &*guard {
            if now.duration_since(*timestamp) < cache_ttl {
                let status_code = match cached_response.status.as_str() {
                    "healthy" => StatusCode::OK,
                    "degraded" => StatusCode::TOO_MANY_REQUESTS,
                    _ => StatusCode::SERVICE_UNAVAILABLE,
                };
                return (status_code, Json(cached_response.clone()));
            }
        }
    }

    // Perform actual health check
    let response = perform_health_check(&state).await;

    // Cache result
    if let Ok(mut guard) = cache.last_check.lock() {
        *guard = Some((now, response.clone()));
    }

    let status_code = match response.status.as_str() {
        "healthy" => StatusCode::OK,
        "degraded" => StatusCode::TOO_MANY_REQUESTS,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(response))
}

async fn perform_health_check(state: &AppState) -> ReadinessResponse {
    // Check scan capacity
    let available = state.orchestrator.semaphore.available_permits();
    let max = state.orchestrator.max_concurrent;
    let scan_capacity = ScanCapacity {
        active: max - available,
        max,
    };

    // Check DB connectivity with latency measurement
    let latency_threshold = std::env::var("HEALTH_DB_LATENCY_THRESHOLD_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(50);

    let start = Instant::now();
    let db_result = sqlx::query("SELECT 1")
        .fetch_one(&state.pool)
        .await;
    let latency_ms = start.elapsed().as_millis() as u64;

    let (db_connected, status) = match db_result {
        Ok(_) if latency_ms <= latency_threshold => (true, "healthy"),
        Ok(_) => (true, "degraded"), // Slow but responsive
        Err(_) => (false, "unhealthy"), // Unreachable
    };

    ReadinessResponse {
        db_connected,
        scan_capacity,
        status: status.to_string(),
    }
}
```
**Source:** Synthesized from [Kubernetes probe patterns](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/), [Rust Kubernetes health checks](https://oneuptime.com/blog/post/2026-01-07-rust-kubernetes-health-checks/view), and [Tokio shared state patterns](https://tokio.rs/tokio/tutorial/shared-state)

### Pattern 3: Router Configuration (Bypass Tracing)
**What:** Health check routes placed after middleware layers to avoid log noise
**When to use:** Always — Phase 20 decision to bypass tracing
**Example:**
```rust
// Source: main.rs Phase 20 implementation
let app = Router::new()
    // API routes — these get traced
    .route("/api/v1/scans", post(scans::create_scan))
    // ... other API routes ...
    .layer(cors)
    .layer(trace_layer)
    .layer(middleware::from_fn(inject_request_id))
    .with_state(app_state)
    // Health checks — added AFTER layers, bypass tracing
    .route("/health", get(health::health_liveness))
    .route("/health/ready", get(health::health_readiness))
    .with_state(health_cache);
```
**Source:** Existing pattern in src/main.rs lines 209-210

### Pattern 4: Scan Capacity Tracking
**What:** Use Semaphore's `available_permits()` for non-blocking capacity check
**When to use:** Readiness checks needing current active scan count
**Example:**
```rust
// Source: https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html
use tokio::sync::Semaphore;

// ScanOrchestrator already has:
// semaphore: Arc<Semaphore>
// created with: Semaphore::new(max_concurrent)

// In health check:
let available = semaphore.available_permits();
let max_concurrent = 5; // From config
let active_scans = max_concurrent - available;
```

### Anti-Patterns to Avoid
- **Heavy liveness checks:** Don't check database in liveness probe — causes cascading failures when DB is slow
- **No caching:** Aggressive probe polling (every 1-5 seconds) hammers database without cache
- **Holding mutex across .await:** Never hold std::sync::Mutex guard across await points — causes Send errors
- **Using tokio::sync::Mutex for cache:** Async mutex is expensive for synchronous cache operations
- **Wrong status codes:** Don't return 200 for degraded state — load balancers need 429/503 to route traffic properly

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Semaphore permit counting | Custom atomic counter tracking spawned tasks | `semaphore.available_permits()` | Tokio's Semaphore already maintains accurate permit state, thread-safe and zero-cost |
| JSON serialization | Manual string formatting for JSON | `axum::Json<T>` + `#[derive(Serialize)]` | Axum handles serialization, Content-Type headers, and type safety automatically |
| Cache invalidation | Complex expiry queue or background task | Timestamp + Duration check on read | Simple `Instant::now().duration_since(timestamp) < ttl` is correct and efficient |
| Database latency measurement | Custom query wrapper or middleware | `std::time::Instant` before/after query | Standard library timing is precise and zero-overhead |

**Key insight:** Health checks are well-trodden ground. Use proven patterns from Kubernetes docs, Tokio sync primitives, and Axum's type-safe extractors. Custom solutions introduce bugs (race conditions, memory leaks, incorrect metrics) that standard approaches already solved.

## Common Pitfalls

### Pitfall 1: Holding Mutex Across Await
**What goes wrong:** `std::sync::Mutex` guard held while calling `.await` causes "future cannot be sent between threads safely" errors
**Why it happens:** MutexGuard is not Send, but async functions must be Send to work with Tokio runtime
**How to avoid:** Clone cached value inside lock, drop guard, then return cloned value
**Warning signs:** Compile error: "future created by async block is not Send"
**Example:**
```rust
// WRONG: Guard held across await
let guard = cache.lock().unwrap();
some_async_operation().await; // ERROR: Send not satisfied

// CORRECT: Clone inside lock, drop guard
let cached = {
    let guard = cache.lock().unwrap();
    guard.clone() // Clone before await
}; // Guard dropped here
some_async_operation().await; // OK
```
**Source:** [Tokio shared state tutorial](https://tokio.rs/tokio/tutorial/shared-state)

### Pitfall 2: Database Health Check Query Choice
**What goes wrong:** Using `SELECT version()` or complex queries in health checks adds unnecessary latency
**Why it happens:** Developers overthink validation — they want to "deeply" check database
**How to avoid:** Use `SELECT 1` for lightweight validation — confirms query execution without data retrieval
**Warning signs:** Health check latency >10ms consistently
**Source:** [PostgreSQL health check best practices](https://last9.io/blog/docker-compose-health-checks/), [Rust health check pattern](https://tjmaynes.com/posts/implementing-the-health-check-api-pattern-with-rust/)

### Pitfall 3: Liveness Probe Checking Dependencies
**What goes wrong:** Liveness probe queries database → database slow → liveness fails → Kubernetes restarts pod → load increases on remaining pods → cascading failure
**Why it happens:** Conflating "process alive" with "dependencies available"
**How to avoid:** Liveness returns 200 immediately with no I/O. Readiness checks dependencies.
**Warning signs:** Pods restarting frequently during high database load
**Source:** [Kubernetes liveness probe best practices](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/), [Kubernetes health check differences](https://cloud.google.com/blog/products/containers-kubernetes/kubernetes-best-practices-setting-up-health-checks-with-readiness-and-liveness-probes)

### Pitfall 4: No Cache TTL on Readiness Endpoint
**What goes wrong:** Load balancer polls `/health/ready` every second → 1 query/second to database → database query load spikes
**Why it happens:** Health checks run frequently (1-10 second intervals), database queries add up at scale
**How to avoid:** Cache readiness result for 5 seconds (user decision) — balances freshness with database protection
**Warning signs:** High `SELECT 1` query count in database logs during normal operation
**Source:** [API health check caching](https://testfully.io/blog/api-health-check-monitoring/), [Health check performance](https://api7.ai/blog/10-best-practices-of-api-gateway-health-checks)

### Pitfall 5: Wrong Status Codes for Degraded State
**What goes wrong:** Returning 200 OK when database is slow → load balancer routes traffic → requests timeout → poor user experience
**Why it happens:** Developer thinks "it's working, just slow" counts as healthy
**How to avoid:** Return 429 (Too Many Requests) for degraded state — signals to load balancer to reduce traffic
**Warning signs:** User-facing timeouts while health checks pass
**Source:** User decision (consistent with [degraded health check status codes](https://kubernetesquestions.com/questions/53985294))

## Code Examples

Verified patterns from official sources:

### Liveness Endpoint (Minimal)
```rust
// Source: User decision + Kubernetes liveness pattern
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
struct LivenessResponse {
    status: String,
}

/// GET /health - Fast liveness check (no I/O)
async fn health_liveness() -> Json<LivenessResponse> {
    Json(LivenessResponse {
        status: "ok".to_string(),
    })
}
```

### Database Connectivity Check with Latency
```rust
// Source: https://tjmaynes.com/posts/implementing-the-health-check-api-pattern-with-rust/
// Modified with latency measurement from std::time::Instant
use sqlx::PgPool;
use std::time::Instant;

async fn check_database_health(pool: &PgPool, threshold_ms: u64) -> (bool, bool) {
    let start = Instant::now();
    let result = sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await;
    let latency_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(_) => {
            let is_fast = latency_ms <= threshold_ms;
            (true, is_fast) // (connected, healthy)
        }
        Err(_) => (false, false), // (disconnected, unhealthy)
    }
}
```

### Semaphore Capacity Tracking
```rust
// Source: https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html
use std::sync::Arc;
use tokio::sync::Semaphore;

fn get_scan_capacity(semaphore: &Arc<Semaphore>, max_concurrent: usize) -> (usize, usize) {
    let available = semaphore.available_permits();
    let active = max_concurrent - available;
    (active, max_concurrent)
}
```

### Cache with Timestamp Expiry
```rust
// Source: Tokio shared state + std::time patterns
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct HealthCache {
    last_check: Mutex<Option<(Instant, ReadinessResponse)>>,
}

impl HealthCache {
    fn new() -> Self {
        Self {
            last_check: Mutex::new(None),
        }
    }

    fn get_cached(&self, ttl: Duration) -> Option<ReadinessResponse> {
        let guard = self.last_check.lock().ok()?;
        let (timestamp, response) = guard.as_ref()?;

        if Instant::now().duration_since(*timestamp) < ttl {
            Some(response.clone()) // Clone inside lock
        } else {
            None
        }
    }

    fn update(&self, response: ReadinessResponse) {
        if let Ok(mut guard) = self.last_check.lock() {
            *guard = Some((Instant::now(), response));
        }
    }
}
```

### Axum Status Code + JSON Response
```rust
// Source: https://docs.rs/axum/latest/axum/error_handling/index.html
use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn example_handler() -> (StatusCode, Json<ErrorResponse>) {
    // Can return different status codes with JSON body
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorResponse {
            error: "Database unreachable".to_string(),
        }),
    )
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single /health endpoint | Separate /health (liveness) and /health/ready (readiness) | ~2019 (Kubernetes 1.16) | Prevents cascading failures from dependency checks in liveness probes |
| Plain text "ok" response | JSON responses with structured data | ~2020 (observability trend) | Enables programmatic parsing, monitoring dashboards |
| No caching | TTL-based response caching | Ongoing best practice | Reduces database query load at scale |
| 200/500 only | 200/429/503 status codes | ~2021 (degraded state awareness) | Load balancers can differentiate "slow but working" from "down" |
| Background health monitoring | On-demand health checks | Still current (2026) | Simpler implementation, acceptable for most use cases |

**Deprecated/outdated:**
- **Single /healthz endpoint:** Kubernetes now recommends separate liveness/readiness/startup probes
- **Synchronous blocking health checks:** Modern async Rust uses tokio for non-blocking I/O
- **String concatenation for JSON:** serde + axum::Json replaced manual formatting

## Open Questions

1. **Should readiness check include orchestrator availability?**
   - What we know: Orchestrator is in-memory, always available if process is alive
   - What's unclear: Whether to expose orchestrator state in readiness (queue depth, worker pool status)
   - Recommendation: Current phase scope is DB + scan capacity — defer orchestrator internals to Phase 22 (Prometheus metrics)

2. **Should cache be per-request or global singleton?**
   - What we know: Global cache (Arc) shared across all requests reduces checks
   - What's unclear: Whether per-request caching provides value
   - Recommendation: Use global singleton cache — simpler, effective, matches health check semantics

3. **How to expose semaphore internals for capacity tracking?**
   - What we know: Semaphore is `pub(crate)` in orchestrator, not publicly accessible
   - What's unclear: Whether to add public method or make field public
   - Recommendation: Add `pub fn get_capacity(&self) -> (usize, usize)` method to ScanOrchestrator for encapsulation

## Sources

### Primary (HIGH confidence)
- [Kubernetes Liveness/Readiness Probe Configuration](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/) - Official Kubernetes documentation on probe patterns
- [Tokio Semaphore Documentation](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html) - Available permits tracking, capacity limits
- [Tokio Shared State Tutorial](https://tokio.rs/tokio/tutorial/shared-state) - Mutex patterns, avoiding Send errors
- [Axum Error Handling Documentation](https://docs.rs/axum/latest/axum/error_handling/index.html) - Status code + JSON response patterns
- Existing codebase (src/main.rs, src/orchestrator/worker_pool.rs) - Current architecture patterns

### Secondary (MEDIUM confidence)
- [Rust Kubernetes Health Checks (2026)](https://oneuptime.com/blog/post/2026-01-07-rust-kubernetes-health-checks/view) - Recent implementation guide with Axum examples
- [Kubernetes Health Check Best Practices (Google Cloud)](https://cloud.google.com/blog/products/containers-kubernetes/kubernetes-best-practices-setting-up-health-checks-with-readiness-and-liveness-probes) - Liveness vs readiness differences, common mistakes
- [Implementing Health Check API Pattern with Rust](https://tjmaynes.com/posts/implementing-the-health-check-api-pattern-with-rust/) - Database connectivity check pattern with SELECT 1
- [PostgreSQL Health Check Best Practices](https://last9.io/blog/docker-compose-health-checks/) - SELECT 1 vs pg_isready, latency considerations
- [Rust Axum Error Handling (LogRocket)](https://blog.logrocket.com/rust-axum-error-handling/) - Custom error types, status code patterns

### Tertiary (LOW confidence)
- [API Health Check Monitoring](https://testfully.io/blog/api-health-check-monitoring/) - General best practices for health check endpoints (not Rust-specific)
- [Rust Caching Strategies (2026)](https://oneuptime.com/blog/post/2026-02-01-rust-caching-strategies/view) - TTL cache patterns (generic, not health-check specific)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All dependencies already installed, proven ecosystem choices
- Architecture: HIGH - Patterns verified from Kubernetes docs, Tokio official docs, existing codebase
- Pitfalls: HIGH - Documented in official Kubernetes/Tokio sources, confirmed by multiple articles
- Implementation details: HIGH - User decisions provide exact specifications, reducing ambiguity

**Research date:** 2026-02-16
**Valid until:** 2026-03-18 (30 days) - Kubernetes probe patterns and Tokio APIs are stable
