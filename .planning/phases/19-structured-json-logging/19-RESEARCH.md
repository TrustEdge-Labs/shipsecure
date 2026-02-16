# Phase 19: Structured JSON Logging - Research

**Researched:** 2026-02-16
**Domain:** Rust structured logging with tracing/tracing-subscriber
**Confidence:** HIGH

## Summary

Structured JSON logging in Rust is a well-established practice using the `tracing` and `tracing-subscriber` crates maintained by the Tokio project. The ecosystem provides robust built-in support for environment-based configuration, JSON formatting, panic handling, and integration with Axum web frameworks. The current codebase already uses `tracing` (0.1) and `tracing-subscriber` (0.3) with the `env-filter` feature, so this phase primarily involves configuration changes and adding the `json` feature flag.

**Primary recommendation:** Use tracing-subscriber's built-in JSON formatter with `EnvFilter::builder()` for custom defaults that can be overridden by environment variables. Add the `json` feature flag to enable JSON output, and integrate `tracing-panic` for structured panic handling. Configure tower-http's `TraceLayer` for automatic HTTP request span creation with scan lifecycle context.

## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Log verbosity & defaults
- Production default: `info` for our crates, `warn` for third-party crates (hyper, sqlx, tower, etc.)
- Development default: `debug` for our crates, `info` for third-party deps
- Sensible defaults built into the app — no RUST_LOG env var required to run
- RUST_LOG env var overrides the default completely when set
- Individual scanner events logged at `info` level (each scanner start/complete visible in production)

#### Scan event coverage
- Full lifecycle logging: scan_started, scanner_started, scanner_completed, scanner_failed, scan_completed, scan_failed
- Metadata only in log fields: scan_id, target_url, tier, scanner_name, duration
- No scan result summaries (findings count, risk level) in logs — results stay in the database
- External service errors (Stripe, Resend, SSL Labs) use standard `tracing::error!` — no dedicated structured events
- Queue/rate-limit events deferred to Phase 22 (Prometheus metrics) — not log events

#### Sensitive data policy
- Full target URLs are logged (public websites, not sensitive)
- Never log customer email addresses — use customer_id or scan_id for correlation
- Never log scan findings content — only counts and metadata
- Strict no-secrets policy: API keys, tokens, webhook secrets never appear in any log line
- Any log line containing a secret is treated as a bug

#### Development experience
- LOG_FORMAT=json enables JSON output; anything else (or unset) defaults to text mode
- Text mode: no ANSI colors — plain text only
- Startup banner: one info-level event logging key config (log format, filter level, port, DB connection status)
- Zero config for local dev: `cargo run` works out of the box with text mode and debug-level logging
- No .env file required for logging — sensible defaults handle everything

### Claude's Discretion
- JSON field naming conventions (snake_case vs camelCase)
- Exact tracing-subscriber configuration and layer composition
- Panic hook implementation details
- Startup banner field selection (which config values to include)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tracing | 0.1 | Structured logging framework | De facto standard for Rust async applications, maintained by Tokio project |
| tracing-subscriber | 0.3 | Subscriber implementations and utilities | Official subscriber implementation with built-in JSON support |
| tracing-panic | 0.1 | Panic hook integration | Standard way to integrate panics with tracing ecosystem |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tower-http | 0.6+ | HTTP middleware with TraceLayer | Already in use for CORS, adds automatic request span creation |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| tracing-subscriber JSON | json-subscriber | More customization options but requires FormatEvent implementation |
| tracing-subscriber JSON | serde_json directly | Total control but lose structured field handling and span context |
| Built-in EnvFilter | custom Layer | More flexibility but significantly more implementation complexity |

**Installation:**
```bash
# Add to Cargo.toml [dependencies]:
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-panic = "0.1"
# tower-http already installed with "trace" feature
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── main.rs              # Logging initialization
├── logging/
│   ├── mod.rs           # Subscriber configuration logic
│   ├── config.rs        # LogFormat enum, default filter strings
│   └── panic.rs         # Panic hook setup
└── orchestrator/
    └── worker_pool.rs   # Scan lifecycle event logging with spans
```

### Pattern 1: Environment-Based Format Switching
**What:** Use LOG_FORMAT environment variable to toggle between JSON and text output at runtime
**When to use:** Always for production flexibility
**Example:**
```rust
// Source: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html
use tracing_subscriber::fmt;

let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string());

match log_format.as_str() {
    "json" => {
        fmt()
            .json()
            .with_env_filter(build_env_filter())
            .init();
    }
    _ => {
        fmt()
            .with_ansi(false) // No ANSI colors in text mode
            .with_env_filter(build_env_filter())
            .init();
    }
}
```

### Pattern 2: Custom Default EnvFilter
**What:** Builder pattern to set default log levels that can be overridden by RUST_LOG
**When to use:** Always for sensible defaults without requiring env var
**Example:**
```rust
// Source: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html
use tracing_subscriber::filter::EnvFilter;

fn build_env_filter() -> EnvFilter {
    let default_directive = if cfg!(debug_assertions) {
        // Development: debug for our crates, info for third-party
        "debug,hyper=info,sqlx=info,tower=info"
    } else {
        // Production: info for our crates, warn for third-party
        "info,hyper=warn,sqlx=warn,tower=warn"
    };

    EnvFilter::builder()
        .with_default_directive(default_directive.parse().unwrap())
        .from_env_lossy() // Uses RUST_LOG if set, otherwise falls back to default
}
```

### Pattern 3: Scan Lifecycle Spans
**What:** Create spans for scan and scanner operations to automatically include context
**When to use:** Always for scan-related operations
**Example:**
```rust
// Source: https://docs.rs/tracing/latest/tracing/attr.instrument.html
use tracing::{info_span, Instrument};

async fn execute_scan(scan_id: Uuid, target_url: &str, tier: &str) {
    let scan_span = info_span!(
        "scan_execution",
        scan_id = %scan_id,
        target_url = %target_url,
        tier = %tier
    );

    async {
        tracing::info!("scan_started");

        for scanner_name in scanners {
            let scanner_span = info_span!(
                "scanner_execution",
                scanner_name = %scanner_name
            );

            async {
                tracing::info!("scanner_started");
                // Execute scanner
                tracing::info!(duration_ms = elapsed.as_millis(), "scanner_completed");
            }.instrument(scanner_span).await;
        }

        tracing::info!("scan_completed");
    }.instrument(scan_span).await
}
```

### Pattern 4: Panic Hook Integration
**What:** Use tracing-panic to emit panic events through tracing pipeline
**When to use:** Always for consistent error telemetry
**Example:**
```rust
// Source: https://docs.rs/tracing-panic/latest/tracing_panic/fn.panic_hook.html
use tracing_panic::panic_hook;

fn main() {
    // Initialize tracing first
    init_tracing();

    // Set panic hook to emit through tracing
    std::panic::set_hook(Box::new(panic_hook));

    // Rest of application
}
```

### Pattern 5: TraceLayer for HTTP Requests
**What:** Use tower-http's TraceLayer to create automatic spans for HTTP requests
**When to use:** Always for Axum applications
**Example:**
```rust
// Source: https://github.com/tokio-rs/axum/blob/main/examples/tracing-aka-logging/src/main.rs
use tower_http::trace::TraceLayer;
use axum::http::Request;
use tracing::info_span;

let app = Router::new()
    .route("/api/v1/scans", post(create_scan))
    .layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<_>| {
                info_span!(
                    "http_request",
                    method = ?request.method(),
                    uri = %request.uri(),
                )
            })
    );
```

### Anti-Patterns to Avoid
- **High-cardinality span names:** Never put scan_id, user_id, or timestamps in span names. Use them as span fields instead. Span names should be low-cardinality identifiers like "scan_execution" or "scanner_execution".
- **Manual span field threading:** Don't pass scan_id through function parameters just for logging. Use span context and `.instrument()` instead.
- **String formatting in disabled logs:** Don't use `tracing::debug!("Processing {}", expensive_computation())` — the computation runs even if debug is disabled. Use structured fields instead.
- **Logging secrets:** Never log API keys, tokens, or credentials. Use `#[instrument(skip(api_key))]` to exclude sensitive parameters.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON log formatting | Custom serde serialization | tracing-subscriber json feature | Handles span context, metadata, timestamps automatically |
| Environment-based filtering | Custom if/else on env vars | EnvFilter::builder().from_env_lossy() | Supports complex directive syntax, validates filters |
| Panic JSON output | Custom panic hook with serde | tracing-panic crate | Integrates with subscriber pipeline, handles backtraces |
| HTTP request tracing | Manual span creation per route | tower-http TraceLayer | Automatic span creation, lifecycle hooks, consistent metadata |
| Span context propagation | Manual context structs | tracing spans with .instrument() | Automatic field inheritance, async-aware, zero runtime cost when disabled |

**Key insight:** The tracing ecosystem has already solved environment configuration, JSON formatting, async context propagation, and panic handling. Custom solutions miss edge cases like span inheritance in async tasks, filter directive parsing, and efficient disabled-span optimization.

## Common Pitfalls

### Pitfall 1: ANSI Escape Codes in Production Logs
**What goes wrong:** Text logs contain ANSI color codes that break log parsers and create visual noise in log aggregation systems.
**Why it happens:** tracing-subscriber enables ANSI by default when outputting to a terminal. The NO_COLOR environment variable can disable this, but relying on it is fragile.
**How to avoid:** Explicitly call `.with_ansi(false)` in text mode configuration.
**Warning signs:** Log lines contain sequences like `\x1b[32m` or `[32m` when viewed in log aggregation tools.

### Pitfall 2: Requiring RUST_LOG for Application Startup
**What goes wrong:** Application panics or refuses to start if RUST_LOG is not set, creating friction for local development and deployment.
**Why it happens:** Using `EnvFilter::try_from_default_env().expect()` requires RUST_LOG to be set and valid.
**How to avoid:** Use `EnvFilter::builder().with_default_directive(...).from_env_lossy()` to provide fallback defaults.
**Warning signs:** Error messages like "RUST_LOG must be set" or startup failures in environments where env vars aren't configured.

### Pitfall 3: Mixed snake_case and camelCase Field Names
**What goes wrong:** JSON logs have inconsistent field naming (e.g., `scan_id` vs `threadId`), making queries fragile and confusing.
**Why it happens:** tracing-subscriber's built-in fields use snake_case except for `threadName` and `threadId` which use camelCase. Custom fields follow user convention.
**How to avoid:** Be consistent in custom field naming. Document the convention (recommend snake_case to match most built-in fields).
**Warning signs:** Queries that work for `scan_id` fail for `threadId` until camelCase is used.

### Pitfall 4: Expensive Field Computation
**What goes wrong:** Performance degrades because expensive computations run even when log level is disabled.
**Why it happens:** Rust evaluates function arguments before the tracing macro checks if the level is enabled.
**How to avoid:** Use structured field syntax: `tracing::debug!(result = %expensive_fn())` instead of `tracing::debug!("Result: {}", expensive_fn())`. For truly expensive fields, check `tracing::enabled!(Level::DEBUG)` first.
**Warning signs:** CPU profiling shows significant time in logging-related code even at info level.

### Pitfall 5: Logging Sensitive Data in Production
**What goes wrong:** API keys, customer emails, or webhook secrets appear in production logs, creating security and compliance risks.
**Why it happens:** Parameters are automatically logged by `#[instrument]` macro unless explicitly skipped.
**How to avoid:** Use `#[instrument(skip(api_key, email))]` for sensitive parameters. Audit all tracing calls to ensure no secrets in structured fields.
**Warning signs:** Security scans flag secrets in log files. Customer PII appears in log aggregation dashboards.

### Pitfall 6: Span Lifecycle Event Spam
**What goes wrong:** Logs become overwhelmed with span enter/exit events, making it hard to find actual application events.
**Why it happens:** RUST_LOG_SPAN_EVENTS=full enables verbose span lifecycle logging for debugging.
**How to avoid:** Never set RUST_LOG_SPAN_EVENTS in production. Use it only in local development when debugging span issues.
**Warning signs:** Log volume spikes dramatically. Every log line is a span enter/exit event rather than application-level events.

### Pitfall 7: Disabled Span Overhead in Hot Paths
**What goes wrong:** Even disabled spans introduce measurable overhead in tight loops or hot code paths.
**Why it happens:** Creating and dropping Span structs has a small but non-zero cost, even when the subscriber is disabled.
**How to avoid:** For hot paths, guard span creation with `if tracing::enabled!(Level::DEBUG)` or avoid spans entirely in performance-critical inner loops.
**Warning signs:** Profiling shows significant time in `Span::drop` even when logging is disabled.

## Code Examples

Verified patterns from official sources:

### Complete Initialization with Environment Toggle
```rust
// Source: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html
use tracing_subscriber::{fmt, EnvFilter};

fn init_tracing() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(
            if cfg!(debug_assertions) {
                "debug,hyper=info,sqlx=info,tower=info,reqwest=info"
                    .parse()
                    .unwrap()
            } else {
                "info,hyper=warn,sqlx=warn,tower=warn,reqwest=warn"
                    .parse()
                    .unwrap()
            }
        )
        .from_env_lossy();

    let log_format = std::env::var("LOG_FORMAT")
        .unwrap_or_else(|_| "text".to_string());

    match log_format.as_str() {
        "json" => {
            fmt()
                .json()
                .with_env_filter(env_filter)
                .init();
        }
        _ => {
            fmt()
                .with_ansi(false)
                .with_env_filter(env_filter)
                .init();
        }
    }
}
```

### JSON Output Field Structure
```json
// Source: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/format/struct.Json.html
{
  "timestamp": "2022-02-15T18:47:10.821315Z",
  "level": "INFO",
  "fields": {
    "message": "scanner_completed",
    "duration_ms": 1234
  },
  "target": "trustedge_audit::orchestrator::worker_pool",
  "spans": [
    {
      "name": "scan_execution",
      "scan_id": "550e8400-e29b-41d4-a716-446655440000",
      "target_url": "https://example.com",
      "tier": "pro"
    },
    {
      "name": "scanner_execution",
      "scanner_name": "ssl_labs"
    }
  ]
}
```

### Scan Lifecycle Event Logging
```rust
// Source: https://docs.rs/tracing/latest/tracing/attr.instrument.html
use tracing::{info_span, Instrument};
use std::time::Instant;

async fn execute_scanner(
    scanner_name: &str,
    scan_id: Uuid,
    target_url: &str,
    tier: &str,
) -> Result<Vec<Finding>, ScanError> {
    let span = info_span!(
        "scanner_execution",
        scanner_name = %scanner_name,
        scan_id = %scan_id,
        target_url = %target_url,
        tier = %tier
    );

    async {
        tracing::info!("scanner_started");
        let start = Instant::now();

        let result = match run_scanner(scanner_name, target_url).await {
            Ok(findings) => {
                let duration_ms = start.elapsed().as_millis();
                tracing::info!(
                    duration_ms = duration_ms,
                    "scanner_completed"
                );
                Ok(findings)
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis();
                tracing::error!(
                    duration_ms = duration_ms,
                    error = %e,
                    "scanner_failed"
                );
                Err(e)
            }
        };

        result
    }.instrument(span).await
}
```

### Instrument Macro with Skip
```rust
// Source: https://docs.rs/tracing/latest/tracing/attr.instrument.html
use tracing::instrument;

#[instrument(skip(api_key, email), fields(customer_id = %customer_id))]
async fn send_completion_email(
    customer_id: Uuid,
    email: &str,
    api_key: &str,
    scan_results_url: &str,
) -> Result<(), EmailError> {
    tracing::info!("sending_completion_email");
    // api_key and email are NOT logged
    // customer_id and scan_results_url ARE logged
    Ok(())
}
```

### Startup Banner Event
```rust
// Source: https://tokio.rs/tokio/topics/tracing
fn log_startup_banner(port: u16, log_format: &str, db_connected: bool) {
    tracing::info!(
        port = port,
        log_format = log_format,
        database_connected = db_connected,
        version = env!("CARGO_PKG_VERSION"),
        "application_started"
    );
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| log crate | tracing crate | 2019-2020 | Structured fields, span context, async-aware |
| env_logger | tracing-subscriber EnvFilter | 2020-2021 | More flexible filtering, better performance |
| Manual JSON serialization | Built-in json feature | 2020 | Automatic span context, consistent formatting |
| RUST_LOG required | EnvFilter::builder() with defaults | 2021 | Zero-config local development |
| Global default panic hook | tracing-panic integration | 2021 | Panics flow through subscriber pipeline |

**Deprecated/outdated:**
- **pretty_env_logger**: Replaced by tracing-subscriber's Pretty formatter with better async support
- **slog**: Still maintained but tracing has better ecosystem integration with Tokio/Axum
- **RUST_LOG_SPAN_EVENTS=full in production**: High overhead, use only in development for debugging

## Open Questions

None. All implementation details are well-documented in official sources.

## Sources

### Primary (HIGH confidence)
- [tracing-subscriber fmt module](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html) - Configuration methods and formatting options
- [tracing-subscriber Json formatter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/format/struct.Json.html) - JSON output structure and features
- [tracing-subscriber EnvFilter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html) - Environment-based filtering and defaults
- [tracing instrument macro](https://docs.rs/tracing/latest/tracing/attr.instrument.html) - Automatic span creation and field skipping
- [tracing-panic documentation](https://docs.rs/tracing-panic/latest/tracing_panic/fn.panic_hook.html) - Panic hook integration
- [Axum tracing example](https://github.com/tokio-rs/axum/blob/main/examples/tracing-aka-logging/src/main.rs) - TraceLayer configuration
- [tracing-subscriber JSON source code](https://docs.rs/tracing-subscriber/latest/src/tracing_subscriber/fmt/format/json.rs.html) - Field naming conventions

### Secondary (MEDIUM confidence)
- [How to Create Structured JSON Logs with tracing in Rust](https://oneuptime.com/blog/post/2026-01-25-structured-json-logs-tracing-rust/view) - 2026 best practices
- [A Gentle Introduction to Axum, Tracing, and Logging](https://ianbull.com/posts/axum-rust-tracing) - Axum integration patterns
- [Tutorial on Using the tracing::instrument Macro in Rust](https://gist.github.com/oliverdaff/d1d5e5bc1baba087b768b89ff82dc3ec) - Practical instrument examples
- [Comparing logging and tracing in Rust](https://blog.logrocket.com/comparing-logging-tracing-rust/) - Ecosystem overview

### Tertiary (LOW confidence)
- [How to Avoid the Anti-Pattern of Putting High-Cardinality Values in Span Names](https://oneuptime.com/blog/post/2026-02-06-avoid-high-cardinality-span-names/view) - Span design anti-patterns

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - tracing/tracing-subscriber are de facto standard, verified by official docs and current codebase usage
- Architecture: HIGH - All patterns verified in official documentation and examples
- Pitfalls: HIGH - Sourced from official documentation, GitHub issues, and production experience guides

**Research date:** 2026-02-16
**Valid until:** 2026-04-16 (60 days - stable ecosystem with infrequent breaking changes)
