---
phase: 21-health-checks
plan: 01
subsystem: observability
tags: [health-checks, monitoring, readiness, liveness, caching]
dependency_graph:
  requires: [phase-20-request-tracing]
  provides: [health-endpoints, db-connectivity-checks, scan-capacity-monitoring]
  affects: [api-server, orchestrator]
tech_stack:
  added: [health-cache, readiness-checks]
  patterns: [separate-router-for-middleware-bypass, mutex-without-await, ttl-cache]
key_files:
  created:
    - src/api/health.rs
  modified:
    - src/orchestrator/worker_pool.rs
    - src/api/mod.rs
    - src/api/scans.rs
    - src/main.rs
decisions:
  - key: "HealthCache field in AppState"
    rationale: "Simplest approach for state sharing - avoids nested state extraction complexity"
  - key: "Separate health_router with .merge()"
    rationale: "Enables health routes to bypass tracing layers while maintaining state access"
  - key: "std::sync::Mutex over tokio::sync::Mutex"
    rationale: "Cache operations are synchronous and non-blocking - no await points inside lock"
  - key: "Three-field ReadinessResponse (db_connected, scan_capacity, status)"
    rationale: "Per user decision - exactly three fields for consistent API shape"
  - key: "5-second cache TTL"
    rationale: "Balances DB protection from aggressive polling with freshness requirements"
metrics:
  duration_minutes: 2
  tasks_completed: 2
  files_modified: 5
  commits: 2
  completed_at: "2026-02-16"
---

# Phase 21 Plan 01: Health Checks Summary

**One-liner:** JWT-free health endpoints with DB connectivity validation, scan capacity reporting, 5-second response caching, and latency-based degradation detection.

## What Was Built

Implemented liveness and readiness health check endpoints to support load balancer routing decisions and monitoring system integration:

### Components Created

1. **Health Module** (`src/api/health.rs`)
   - `LivenessResponse` - Simple `{ "status": "ok" }` response
   - `ReadinessResponse` - Three fields: `db_connected`, `scan_capacity`, `status`
   - `ScanCapacity` - Nested struct with `active` and `max` scan counts
   - `HealthCache` - TTL-based cache with `std::sync::Mutex` for thread-safe 5-second caching

2. **Orchestrator Capacity Tracking** (`src/orchestrator/worker_pool.rs`)
   - Added `max_concurrent` field to `ScanOrchestrator` struct
   - Implemented `get_capacity()` method returning `(active, max)` tuple
   - Non-blocking capacity query using `Semaphore::available_permits()`

3. **Router Integration** (`src/main.rs`)
   - Created separate `health_router` with its own state
   - Merged health routes after tracing/CORS layers via `.merge()`
   - Removed old inline `/health` handler

### Health Check Endpoints

**GET /health** (Liveness)
- No I/O, immediate response
- Returns `{ "status": "ok" }`
- Status code: 200

**GET /health/ready** (Readiness)
- Checks cache first (5-second TTL)
- Queries scan capacity from orchestrator (non-blocking)
- Tests DB connectivity with `SELECT 1` query
- Measures DB latency against `HEALTH_DB_LATENCY_THRESHOLD_MS` (default 50ms)
- Returns:
  - 200 "healthy" - DB responsive, latency <= threshold
  - 429 "degraded" - DB responsive, latency > threshold
  - 503 "unhealthy" - DB unreachable
- Response shape:
  ```json
  {
    "db_connected": true,
    "scan_capacity": { "active": 2, "max": 5 },
    "status": "healthy"
  }
  ```

### Middleware Bypass Pattern

Health routes placed after `.layer()` calls but with state access:
```rust
// Separate router for health - bypasses tracing
let health_router = Router::new()
    .route("/health", get(health::health_liveness))
    .route("/health/ready", get(health::health_readiness))
    .with_state(state.clone());

// Main router with traced API routes
let app = Router::new()
    .route("/api/v1/scans", post(scans::create_scan))
    // ... other routes ...
    .layer(cors)
    .layer(trace_layer)
    .layer(middleware::from_fn(inject_request_id))
    .with_state(state)
    .merge(health_router) // Health checks bypass layers
```

This ensures:
- API routes get traced, health checks don't (no log noise)
- Both have access to AppState (pool, orchestrator, health_cache)
- No complex nested state extraction

## Deviations from Plan

None - plan executed exactly as written.

## Verification Results

- ✅ `cargo check` compiles without errors
- ✅ `cargo test` passes (62 pass, 1 pre-existing failure in js_secrets)
- ✅ GET /health returns `{ "status": "ok" }`
- ✅ GET /health/ready returns JSON with 3 fields
- ✅ ReadinessResponse contains `db_connected`, `scan_capacity`, `status`
- ✅ Status codes map correctly: 200 (healthy), 429 (degraded), 503 (unhealthy)
- ✅ HealthCache uses 5-second TTL
- ✅ `HEALTH_DB_LATENCY_THRESHOLD_MS` env var with 50ms default
- ✅ Mutex not held across await points (synchronous cache operations only)
- ✅ Health routes bypass tracing middleware
- ✅ `get_capacity()` method on ScanOrchestrator

## Implementation Notes

### Mutex Safety
The `HealthCache` uses `std::sync::Mutex` (not `tokio::sync::Mutex`) because cache operations are fully synchronous:
```rust
pub fn get_cached(&self, ttl: Duration) -> Option<ReadinessResponse> {
    let guard = self.inner.lock().unwrap();
    // Clone inside lock, no await points
    if let Some((timestamp, response)) = &*guard {
        if timestamp.elapsed() < ttl {
            return Some(response.clone());
        }
    }
    None
} // guard dropped here, before any async work
```

The lock is acquired, data is cloned, and the guard is dropped - all before any `.await` points. This prevents the "cannot hold mutex across await" error.

### Capacity Calculation
```rust
pub fn get_capacity(&self) -> (usize, usize) {
    let available = self.semaphore.available_permits();
    let active = self.max_concurrent - available;
    (active, self.max_concurrent)
}
```

This gives load balancers visibility into scan queue pressure without blocking on semaphore acquisition.

### Cache Behavior
- First request: Hits DB, caches response for 5 seconds
- Subsequent requests within TTL: Return cached response (no DB hit)
- After TTL expires: Next request hits DB, refreshes cache

This protects the database from aggressive health check polling (e.g., every 1-2 seconds from load balancers).

## Testing Recommendations

Manual verification:
```bash
# Liveness (always works)
curl http://localhost:8080/health

# Readiness (requires DB)
curl http://localhost:8080/health/ready

# Test degradation (set low threshold)
export HEALTH_DB_LATENCY_THRESHOLD_MS=1
curl http://localhost:8080/health/ready
# Should return 429 if DB > 1ms

# Test caching (run twice within 5s)
time curl http://localhost:8080/health/ready
time curl http://localhost:8080/health/ready
# Second should be faster (cached)
```

Integration test ideas:
- Mock slow DB to trigger degraded status
- Mock unreachable DB to trigger unhealthy status
- Verify cache TTL by timing multiple requests
- Verify scan capacity accuracy during concurrent scans

## Next Steps

1. Phase 21 Plan 02: Prometheus metrics endpoint with application-level metrics
2. Phase 22: Nginx IP restriction for metrics endpoint security
3. Phase 23: Graceful shutdown coordination with in-flight scans

## Self-Check: PASSED

**Created files verified:**
- ✅ FOUND: src/api/health.rs

**Commits verified:**
- ✅ FOUND: 755ef9b (Task 1: health module and orchestrator capacity)
- ✅ FOUND: a613dea (Task 2: router wiring and state updates)

**Key artifacts verified:**
- ✅ ReadinessResponse has exactly 3 fields
- ✅ HealthCache implements TTL-based caching
- ✅ ScanOrchestrator.get_capacity() returns (active, max)
- ✅ Health routes merged after middleware layers
- ✅ No mutex held across await points
