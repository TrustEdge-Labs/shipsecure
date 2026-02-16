# Phase 20: Request Tracing - Research

**Researched:** 2026-02-16
**Domain:** HTTP request tracing and correlation IDs
**Confidence:** HIGH

## Summary

Request tracing in Rust/Axum applications is well-established through tower-http's `TraceLayer` middleware combined with the tracing ecosystem. The pattern involves generating a unique `request_id` (UUID v4) at the HTTP layer via `make_span_with()`, propagating it through tracing spans to background tasks using `.instrument()`, and storing it in the database for queryable correlation. Phase 19's structured logging foundation (tracing-subscriber with JSON output) provides the infrastructure needed for this phase.

The approach is straightforward: tower-http creates a parent span for each request with a `request_id` field, tracing automatically propagates span context to all nested operations (including log statements), and background scan tasks inherit the request span through `.instrument()` on spawned futures. Health check endpoints can be cleanly excluded by adding them after the TraceLayer in the router definition.

**Primary recommendation:** Use tower-http's `TraceLayer::new_for_http().make_span_with()` to generate UUID v4 request IDs, add `request_id` as a shared field on scan spans (not parent-child linking), filter health checks via route ordering, and store `request_id` in the database via a new nullable UUID column.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Request ID visibility:**
- Request ID is internal only — NOT exposed in HTTP response headers
- Always generate a fresh UUID v4 server-side — never honor incoming X-Request-Id headers
- Field name: `request_id` (snake_case, matching scan_id from Phase 19)

**Log detail level:**
- Minimal HTTP details: method, URI path, status code, latency_ms
- Exact paths logged (e.g., /api/scans/550e8400-...), not grouped route patterns
- Filter out health check endpoints (/health, /health/ready) from request logging to reduce noise
- Log levels: INFO for 4xx/5xx errors, DEBUG for 2xx/3xx successes

**Propagation depth:**
- Shared field approach: add request_id as a field on scan spans, NOT parent-child span linking
- Scan tasks only — emails and other background tasks do NOT get request_id
- Pass request_id as an explicit function parameter to spawn_scan/spawn_paid_scan (not span context extraction)
- Store request_id in database scans table (requires migration) for queryable correlation

**Sensitive data policy:**
- Strip query parameters from logged URIs — log path portion only
- Safe headers only: log Content-Type, Accept, etc. but NEVER Authorization, Cookie, or Set-Cookie
- Never log request or response bodies
- No client IP addresses in request logs

### Claude's Discretion

- tower-http TraceLayer configuration specifics
- Exact middleware ordering in the Axum router
- Database migration implementation details for request_id column
- How to wire the request_id from middleware through to the scan spawn call

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope

</user_constraints>

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tower-http | 0.6.8 (current) | TraceLayer middleware for request tracing | Official tower-http middleware, battle-tested in production Axum apps, provides customizable callbacks for request/response logging |
| tracing | 0.1.x | Span context and structured fields | Already used in Phase 19, industry standard for Rust observability, automatic context propagation |
| uuid | 1.x (with v4 feature) | Request ID generation | Already in dependencies, UUID v4 provides cryptographically random IDs with negligible collision probability |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| sqlx | 0.8.6 (current) | Database migration for request_id column | Required for adding nullable UUID column to scans table |
| chrono | 0.4.43 (current) | Already used for timestamps | Not needed for request tracing specifically, but used throughout the codebase |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| UUID v4 | ULID | ULID offers time-ordering and better database performance (50% faster generation, less index fragmentation), but adds external dependency and Phase 19 already uses UUIDs for scan_id — consistency favors UUID v4 |
| UUID v4 | UUID v7 | UUID v7 provides time-ordered sequential IDs reducing database fragmentation, but requires uuid crate upgrade and mixing v4 (scan_id) with v7 (request_id) creates inconsistency |
| tower-http TraceLayer | axum-trace-id crate | Dedicated middleware crate exists but adds external dependency for functionality already available in tower-http with make_span_with() |
| Route ordering (health check filter) | Conditional logic in on_request callback | Custom filtering adds complexity; route ordering is cleaner and leverages Axum's middleware composition model |

**Installation:**

No new dependencies required — tower-http and uuid already present in Cargo.toml.

## Architecture Patterns

### Recommended Project Structure

```
src/
├── main.rs               # TraceLayer middleware added here
├── api/
│   ├── scans.rs         # Extract request_id from span, pass to orchestrator
│   └── ...
├── orchestrator/
│   └── worker_pool.rs   # Receive request_id param, add to scan span
└── db/
    └── scans.rs         # Store request_id in database
migrations/
└── YYYYMMDDHHMMSS_add_request_id_to_scans.sql
```

### Pattern 1: TraceLayer Configuration with make_span_with

**What:** Generate unique request IDs and create tracing spans with structured fields

**When to use:** At the HTTP layer (main.rs router configuration) before routes are defined

**Example:**

```rust
// Source: https://github.com/tokio-rs/axum/discussions/2273
// Verified: https://docs.rs/tower-http/latest/tower_http/trace/index.html

use tower_http::trace::TraceLayer;
use tracing::Level;

let trace_layer = TraceLayer::new_for_http()
    .make_span_with(|request: &http::Request<axum::body::Body>| {
        let request_id = uuid::Uuid::new_v4();
        let uri = request.uri();

        tracing::info_span!(
            "http_request",
            request_id = %request_id,
            method = %request.method(),
            uri = %uri.path(),  // path() excludes query parameters
            status_code = tracing::field::Empty,
            latency_ms = tracing::field::Empty,
        )
    })
    .on_response(|response: &http::Response<_>, latency: Duration, span: &Span| {
        let status = response.status();
        let latency_ms = latency.as_millis() as u64;

        span.record("status_code", status.as_u16());
        span.record("latency_ms", latency_ms);

        let level = if status.is_client_error() || status.is_server_error() {
            Level::INFO
        } else {
            Level::DEBUG
        };

        tracing::event!(level, "http_response");
    });
```

**Key details:**
- `uri.path()` excludes query parameters (satisfies sensitive data policy)
- `field::Empty` reserves fields populated in `on_response` callback
- Conditional log levels: INFO for 4xx/5xx, DEBUG for 2xx/3xx
- Request ID logged as display format (`%request_id`)

### Pattern 2: Health Check Filtering via Route Ordering

**What:** Exclude repetitive health check traffic from TraceLayer

**When to use:** When defining routes in main.rs

**Example:**

```rust
// Source: https://github.com/tokio-rs/axum/discussions/355

let app = Router::new()
    // API routes — these get traced
    .route("/api/v1/scans", post(scans::create_scan))
    .route("/api/v1/scans/{id}", get(scans::get_scan))
    .route("/api/v1/results/{token}", get(results::get_results_by_token))
    // ... other API routes
    .layer(trace_layer)
    .layer(cors)
    .with_state(state)
    // Health checks — added AFTER layer, bypass tracing
    .route("/health", get(|| async { "ok" }))
    .route("/health/ready", get(|| async { "ready" }));
```

**Why this works:** In Axum, middleware applied via `.layer()` only affects routes defined *before* it. Routes added after the layer bypass the middleware entirely.

### Pattern 3: Extracting request_id from Span Context

**What:** Access the current span's request_id field to pass to background tasks

**When to use:** In API handlers before spawning background work

**Example:**

```rust
// Conceptual pattern — implementation requires span context extraction

use tracing::Span;

pub async fn create_scan(
    State(state): State<AppState>,
    Json(req): Json<CreateScanRequest>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    // ... validation, SSRF checks, rate limiting ...

    let scan = db::scans::create_scan(&state.pool, &validated_url, &req.email, Some(&client_ip)).await?;

    // Extract request_id from current span
    let request_id = Span::current()
        .with_subscriber(|(id, dispatch)| {
            dispatch.with_span(id, |span| {
                span.metadata().fields().field("request_id")
                    // Field extraction logic here
            })
        });

    // Pass as explicit parameter (NOT span context)
    state.orchestrator.spawn_scan(scan.id, scan.target_url.clone(), request_id);

    // ...
}
```

**Alternative (simpler):** Store request_id in request extensions during TraceLayer creation, extract via `Extension<RequestId>` in handlers. This avoids span visitor pattern complexity.

### Pattern 4: Propagating request_id to Scan Spans

**What:** Add request_id as a shared field on background task spans

**When to use:** In orchestrator when spawning scan tasks

**Example:**

```rust
// Source: https://tokio.rs/tokio/topics/tracing-next-steps

pub fn spawn_scan(&self, scan_id: Uuid, target_url: String, request_id: Option<Uuid>) {
    let pool = self.pool.clone();
    let semaphore = self.semaphore.clone();

    // Add request_id as a field, not parent-child relationship
    let span = info_span!(
        "scan",
        scan_id = %scan_id,
        target_url = %target_url,
        tier = "free",
        request_id = %request_id.unwrap_or_default(),  // Empty if None
    );

    tokio::spawn(async move {
        let _permit = semaphore.acquire().await.expect("Semaphore closed");
        tracing::info!("scan_started");
        // ... scan execution ...
    }.instrument(span));
}
```

**Key points:**
- `request_id` is a field on the scan span, not a parent span
- `.instrument(span)` attaches span to the future
- All log statements inside the future inherit `request_id` automatically
- Use `Option<Uuid>` — request_id is None for non-HTTP-triggered scans (if any)

### Pattern 5: Database Migration for request_id

**What:** Add nullable UUID column for queryable correlation

**When to use:** Required migration before code changes

**Example:**

```sql
-- Migration: YYYYMMDDHHMMSS_add_request_id_to_scans.sql

ALTER TABLE scans
ADD COLUMN request_id UUID;

CREATE INDEX idx_scans_request_id ON scans (request_id)
WHERE request_id IS NOT NULL;
```

**Why nullable:** Not all scans originate from HTTP requests (e.g., manual database-triggered rescans, testing). Nullable column allows `NULL` for non-HTTP scans.

**Why partial index:** Only indexes non-NULL values, saving space and improving query performance for lookups by request_id.

### Anti-Patterns to Avoid

- **Don't use parent-child span linking:** User decision specifies shared field approach. Parent-child creates nested span hierarchy that complicates queries and filtering.
- **Don't extract request_id from span context in spawned tasks:** Pass it as an explicit parameter. Span visitor pattern is complex and fragile; explicit parameters are clearer.
- **Don't log full URIs with query parameters:** Use `uri.path()` to exclude sensitive query params (API keys, tokens, etc.).
- **Don't log Authorization, Cookie, or Set-Cookie headers:** Even in custom on_request callbacks, never log sensitive headers.
- **Don't filter health checks with conditional logic in on_request:** Route ordering is cleaner and leverages Axum's middleware model.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Request ID generation | Custom ID format or sequential integers | `uuid::Uuid::new_v4()` | UUID v4 provides cryptographic randomness, globally unique IDs, and standard format. Sequential IDs leak business metrics and are guessable. |
| Span context propagation | Manual thread-local storage or context passing | `.instrument(span)` on futures | Tracing library handles async boundaries, thread safety, and automatic field inheritance. Manual propagation is error-prone. |
| Health check filtering | Custom middleware with path matching | Route ordering (add health checks after `.layer()`) | Axum's middleware composition handles this natively. Custom filtering adds complexity for no benefit. |
| Query parameter stripping | Regex or string manipulation | `uri.path()` method | Built-in method is tested, efficient, and handles edge cases (encoded chars, fragments, etc.). |
| Request/response correlation | Custom header injection or global state | TraceLayer with `make_span_with()` | Tower-http's TraceLayer is production-tested, configurable, and integrates with tracing ecosystem. |

**Key insight:** The tracing ecosystem (tracing + tower-http) solves request correlation comprehensively. Building custom solutions duplicates functionality and introduces bugs around async context propagation, thread safety, and structured field inheritance.

## Common Pitfalls

### Pitfall 1: Logging Sensitive Query Parameters

**What goes wrong:** Full URI logging exposes sensitive data in logs (e.g., `/api/endpoint?api_key=secret`)

**Why it happens:** Default `uri.to_string()` includes query parameters; easy to overlook security implications

**How to avoid:** Always use `uri.path()` which excludes query string:

```rust
uri = %uri.path(),  // Correct: /api/endpoint
// NOT: uri = %request.uri(),  // Wrong: /api/endpoint?api_key=secret
```

**Warning signs:** Code reviews flag any `request.uri()` usage without `.path()` method

### Pitfall 2: Holding Span::enter() Across .await Points

**What goes wrong:** Incorrect traces, potential panics, or lost context when holding enter guards across async boundaries

**Why it happens:** `Span::enter()` returns a guard that must be dropped before awaiting; easy mistake in async code

**How to avoid:** Use `.instrument()` on futures instead of manual `enter()`/`exit()`:

```rust
// WRONG:
let span = info_span!("operation");
let _enter = span.enter();  // Guard held across await
some_async_fn().await;      // Incorrect!

// CORRECT:
let span = info_span!("operation");
some_async_fn().instrument(span).await;  // Span attached to future
```

**Warning signs:** Clippy lint `clippy::await_holding_span_guard` detects this error

**Source:** https://docs.rs/tracing/latest/tracing/struct.Span.html

### Pitfall 3: Middleware Ordering Confusion

**What goes wrong:** Middleware applied in unexpected order, causing TraceLayer to not capture requests or health checks to be logged

**Why it happens:** Direct `.layer()` calls execute bottom-to-top, counterintuitive to code reading order

**How to avoid:** Use `tower::ServiceBuilder` which applies middleware top-to-bottom:

```rust
use tower::ServiceBuilder;

let app = Router::new()
    .route("/api/endpoint", get(handler))
    .layer(
        ServiceBuilder::new()
            .layer(trace_layer)    // Executes first
            .layer(cors_layer)     // Then this
    );
```

Alternatively, remember: with direct `.layer()`, last added executes first.

**Warning signs:** Trace logs missing expected fields, middleware behaving unexpectedly

**Source:** https://docs.rs/axum/latest/axum/middleware/index.html

### Pitfall 4: Forgetting to Declare Empty Fields

**What goes wrong:** Attempting to `span.record("field_name", value)` fails silently if field wasn't declared at span creation

**Why it happens:** Tracing requires all field names to be known when span is created; cannot add fields later

**How to avoid:** Declare all potentially-needed fields upfront with `tracing::field::Empty`:

```rust
let span = info_span!(
    "http_request",
    request_id = %uuid,
    status_code = tracing::field::Empty,  // Populated later
    latency_ms = tracing::field::Empty,   // Populated later
);
```

Later: `span.record("status_code", 200);` — works because field was declared.

**Warning signs:** Expected fields missing from JSON logs despite `record()` calls

**Source:** https://docs.rs/tracing/latest/tracing/struct.Span.html

### Pitfall 5: Exposing request_id in HTTP Response Headers

**What goes wrong:** Internal correlation IDs leak to clients, potentially exposing request volume or timing patterns

**Why it happens:** Common observability pattern in other ecosystems (e.g., X-Request-ID header in responses)

**How to avoid:** Never add request_id to response headers. User decision explicitly forbids this. Request ID is internal-only for log correlation.

```rust
// WRONG:
.on_response(|response: &mut Response, _latency, span| {
    response.headers_mut().insert(
        "X-Request-ID",
        request_id.to_string().parse().unwrap()
    );
});

// CORRECT:
// Don't modify response headers at all
```

**Warning signs:** Code reviews must catch any X-Request-ID or similar headers in response handling

### Pitfall 6: UUID Type Mismatch in Database

**What goes wrong:** Runtime errors when inserting UUIDs if Rust's `uuid::Uuid` doesn't match PostgreSQL's `uuid` type

**Why it happens:** SQLx requires explicit type mapping; missing `uuid` feature causes type errors

**How to avoid:** Ensure `sqlx` dependency includes `uuid` feature (already present):

```toml
sqlx = { version = "0.8.6", features = ["runtime-tokio", "tls-rustls", "postgres", "migrate", "uuid", "chrono"] }
```

SQLx automatically maps Rust `uuid::Uuid` ↔ PostgreSQL `uuid` type.

**Warning signs:** Compile-time errors: `the trait bound `Uuid: Type<Postgres>` is not satisfied`

**Source:** https://docs.rs/sqlx/latest/sqlx/types/struct.Uuid.html

## Code Examples

Verified patterns from official sources:

### Middleware Setup with Health Check Filtering

```rust
// Source: https://docs.rs/tower-http/latest/tower_http/trace/index.html
// Source: https://github.com/tokio-rs/axum/discussions/355

use axum::{Router, routing::{get, post}};
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse};
use tower_http::cors::CorsLayer;
use tower_http::LatencyUnit;
use tracing::Level;
use std::time::Duration;

let trace_layer = TraceLayer::new_for_http()
    .make_span_with(|request: &http::Request<axum::body::Body>| {
        let request_id = uuid::Uuid::new_v4();
        tracing::info_span!(
            "http_request",
            request_id = %request_id,
            method = %request.method(),
            uri = %request.uri().path(),
            status_code = tracing::field::Empty,
            latency_ms = tracing::field::Empty,
        )
    })
    .on_response(|response: &http::Response<_>, latency: Duration, span: &tracing::Span| {
        let status = response.status();
        let latency_ms = latency.as_millis() as u64;

        span.record("status_code", status.as_u16());
        span.record("latency_ms", latency_ms);

        if status.is_client_error() || status.is_server_error() {
            tracing::info!("http_response");
        } else {
            tracing::debug!("http_response");
        }
    });

let app = Router::new()
    // Traced routes
    .route("/api/v1/scans", post(scans::create_scan))
    .route("/api/v1/scans/{id}", get(scans::get_scan))
    .layer(trace_layer)
    .layer(cors)
    .with_state(state)
    // Health checks bypass tracing (added after layer)
    .route("/health", get(|| async { "ok" }));
```

### Extracting request_id via Extension

```rust
// Alternative to span visitor pattern complexity
// Store request_id during TraceLayer creation, extract in handlers

use axum::{Extension, extract::State, http::StatusCode, Json};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct RequestId(pub Uuid);

// In TraceLayer setup:
.make_span_with(|request: &http::Request<axum::body::Body>| {
    let request_id = uuid::Uuid::new_v4();
    request.extensions_mut().insert(RequestId(request_id));

    tracing::info_span!(
        "http_request",
        request_id = %request_id,
        // ... other fields
    )
})

// In handler:
pub async fn create_scan(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(req): Json<CreateScanRequest>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    // ...
    state.orchestrator.spawn_scan(scan.id, scan.target_url.clone(), Some(request_id.0));
    // ...
}
```

**Note:** Extension approach is cleaner than span visitor pattern but requires adding `Extension(RequestId)` to all handlers that spawn scans.

### Database Insert with request_id

```rust
// Source: https://docs.rs/sqlx/latest/sqlx/

pub async fn create_scan(
    pool: &PgPool,
    target_url: &str,
    email: &str,
    client_ip: Option<&str>,
    request_id: Option<Uuid>,
) -> Result<Scan, sqlx::Error> {
    let scan = sqlx::query_as!(
        Scan,
        r#"
        INSERT INTO scans (target_url, email, client_ip, request_id, status)
        VALUES ($1, $2, $3, $4, 'pending')
        RETURNING id, target_url, email, client_ip, request_id, status as "status: ScanStatus",
                  score, error_message, started_at, completed_at, created_at
        "#,
        target_url,
        email,
        client_ip,
        request_id,  // Option<Uuid> maps to nullable uuid column
    )
    .fetch_one(pool)
    .await?;

    Ok(scan)
}
```

**Key points:**
- `Option<Uuid>` automatically maps to nullable `uuid` column
- No special handling needed — SQLx handles None → NULL conversion
- Querying by request_id: `WHERE request_id = $1` works with partial index

### Scan Span with request_id Field

```rust
// Source: https://tokio.rs/tokio/topics/tracing

pub fn spawn_scan(&self, scan_id: Uuid, target_url: String, request_id: Option<Uuid>) {
    let pool = self.pool.clone();
    let semaphore = self.semaphore.clone();
    let timeout = self.max_scanner_timeout;

    // Shared field approach: request_id is a field, not parent span
    let span = info_span!(
        "scan",
        scan_id = %scan_id,
        target_url = %target_url,
        tier = "free",
        request_id = request_id.map(|id| id.to_string()).as_deref().unwrap_or(""),
    );

    tokio::spawn(async move {
        let _permit = semaphore.acquire().await.expect("Semaphore closed");
        tracing::info!("scan_started");

        let start = Instant::now();
        match Self::execute_scan_internal(pool, scan_id, target_url, timeout, "free").await {
            Ok(()) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                tracing::info!(duration_ms, "scan_completed");
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                tracing::error!(duration_ms, error = %e, "scan_failed");
            }
        }
    }.instrument(span));
}
```

**Result:** JSON logs include request_id field for all log events within the scan:

```json
{
  "timestamp": "2026-02-16T12:34:56.789Z",
  "level": "INFO",
  "target": "trustedge_audit::orchestrator",
  "span": {
    "scan_id": "550e8400-e29b-41d4-a716-446655440000",
    "target_url": "https://example.com",
    "tier": "free",
    "request_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
  },
  "message": "scan_started"
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Custom correlation ID middleware | tower-http TraceLayer with make_span_with() | ~2021 (Axum launch) | Standardized on tower ecosystem, simplified middleware stack |
| Manual context propagation (thread-locals) | tracing::Instrument for futures | ~2019 (tracing 0.1) | Automatic span propagation across async boundaries, safer and cleaner |
| UUID v4 for all IDs | UUID v7 or ULID for database keys | 2024-2025 | Improved database performance (sequential inserts), but adoption gradual; UUID v4 still valid for correlation IDs |
| Logging all requests including health checks | Selective middleware via route ordering | Axum 0.6+ (2022) | Reduced log noise, cleaner middleware composition |
| Request ID in response headers (X-Request-ID) | Internal-only correlation | Varies by team | Security benefit: doesn't leak request volume patterns to clients |

**Deprecated/outdated:**

- **actix-web-requestid crate (Actix-Web ecosystem):** Actix-Web's approach to request IDs, superseded by tower-http patterns in Axum ecosystem. Not applicable to tower-based apps.
- **Conditional env-based TraceLayer toggling:** Early Axum apps sometimes disabled TraceLayer in production due to performance concerns. Modern tracing-subscriber (0.3+) with JSON output has negligible overhead; always-on tracing is now standard.
- **Extracting trace context via opentelemetry::Context::current():** OpenTelemetry integration adds complexity for simple correlation use cases. Direct tracing span fields are simpler unless full distributed tracing is required.

## Open Questions

### 1. Should request_id be included in email notifications?

**What we know:** Email notifications send scan results to users. Emails are triggered from the orchestrator after scan completion. User decision says "emails do NOT get request_id" (deferred to future phases).

**What's unclear:** Whether this means:
- (A) Email log events don't include request_id field (but they're in the same scan span, so they inherit it automatically)
- (B) Email content (HTML/text) doesn't mention request_id (likely interpretation)

**Recommendation:** Interpret as (B): don't include request_id in email template content. Log events from email sending will inherit request_id from scan span automatically due to `.instrument()`, which is acceptable for debugging.

### 2. How to handle request_id for manually-triggered scans (non-HTTP)?

**What we know:** User decision specifies "scan tasks only" get request_id. Database column is nullable. No current non-HTTP scan triggers exist in codebase (all scans originate from POST /api/v1/scans or webhook-triggered paid rescans).

**What's unclear:** Future-proofing for admin panel, CLI tools, or scheduled rescans

**Recommendation:** Use `Option<Uuid>` for request_id parameter, pass `None` for non-HTTP-triggered scans. Database stores `NULL`. Logs include empty string or omit field. This keeps door open for future triggers without breaking existing patterns.

### 3. Should scanner spans also include request_id as a field?

**What we know:** User decision specifies adding request_id to scan spans. Scanner spans are children of scan spans (via `.instrument(span)` on each scanner task). Tracing propagates parent fields to children automatically in some subscribers.

**What's unclear:** Whether child scanner spans need explicit request_id fields or inherit it from parent scan span

**Recommendation:** Rely on span hierarchy — scanner spans inherit request_id from parent scan span automatically in JSON output. Don't add redundant fields. Test with tracing-subscriber's JSON formatter to confirm nested spans include parent fields.

## Sources

### Primary (HIGH confidence)

- [tower-http TraceLayer documentation](https://docs.rs/tower-http/latest/tower_http/trace/index.html) - TraceLayer API, callbacks, span configuration
- [Axum middleware documentation](https://docs.rs/axum/latest/axum/middleware/index.html) - Middleware ordering, composition patterns
- [Tracing Span documentation](https://docs.rs/tracing/latest/tracing/struct.Span.html) - Span::record(), field::Empty, .instrument()
- [SQLx UUID documentation](https://docs.rs/sqlx/latest/sqlx/types/struct.Uuid.html) - PostgreSQL UUID type mapping
- [Tokio tracing guide](https://tokio.rs/tokio/topics/tracing) - Getting started, async instrumentation

### Secondary (MEDIUM confidence)

- [Axum Discussion #2273](https://github.com/tokio-rs/axum/discussions/2273) - Request ID correlation patterns, verified by maintainers
- [Axum Discussion #355](https://github.com/tokio-rs/axum/discussions/355) - Health check filtering via route ordering
- [Ian Bull's Axum Tracing Tutorial](https://ianbull.com/posts/axum-rust-tracing) - Practical TraceLayer examples
- [Correlation ID patterns overview](https://microsoft.github.io/code-with-engineering-playbook/observability/correlation-id/) - General best practices
- [UUID v4 vs v7 vs ULID comparison](https://thetexttool.com/compare/uuid-v4-vs-v7-vs-ulid) - ID format tradeoffs

### Tertiary (LOW confidence)

- [OneUptime: Correlation ID tracing in ASP.NET Core](https://oneuptime.com/blog/post/2026-01-25-correlation-id-tracing-aspnet-core/view) - Cross-language patterns (ASP.NET, not Rust)
- [Rust Observability with OpenTelemetry (Jan 2026)](https://dasroot.net/posts/2026/01/rust-observability-opentelemetry-tokio/) - Advanced OpenTelemetry integration (out of scope)

## Metadata

**Confidence breakdown:**

- **Standard stack:** HIGH - tower-http and tracing are established standards in Rust/Axum ecosystem, extensively documented, production-proven
- **Architecture:** HIGH - Patterns verified against official docs, GitHub discussions with maintainer input, and current production codebases
- **Pitfalls:** MEDIUM-HIGH - Common issues well-documented in forums and GitHub, but async context propagation edge cases may exist

**Research date:** 2026-02-16

**Valid until:** ~30 days (March 2026) — tracing and tower-http are stable libraries with infrequent breaking changes. Axum 0.8.x is current stable. Monitor for:
- Axum 0.9 (if released) — potential middleware API changes
- tracing 0.2 (if released) — unlikely but would affect span APIs
- tower-http 0.7 (if released) — TraceLayer API evolution
