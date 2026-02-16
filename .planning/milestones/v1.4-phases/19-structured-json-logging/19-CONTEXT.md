# Phase 19: Structured JSON Logging - Context

**Gathered:** 2026-02-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Backend emits structured JSON logs in production with scan lifecycle context. Environment-driven toggle between JSON (production) and text (development) output formats. All log events include structured fields; scan events include scan-specific context. Panic handler outputs structured JSON.

</domain>

<decisions>
## Implementation Decisions

### Log verbosity & defaults
- Production default: `info` for our crates, `warn` for third-party crates (hyper, sqlx, tower, etc.)
- Development default: `debug` for our crates, `info` for third-party deps
- Sensible defaults built into the app — no RUST_LOG env var required to run
- RUST_LOG env var overrides the default completely when set
- Individual scanner events logged at `info` level (each scanner start/complete visible in production)

### Scan event coverage
- Full lifecycle logging: scan_started, scanner_started, scanner_completed, scanner_failed, scan_completed, scan_failed
- Metadata only in log fields: scan_id, target_url, tier, scanner_name, duration
- No scan result summaries (findings count, risk level) in logs — results stay in the database
- External service errors (Stripe, Resend, SSL Labs) use standard `tracing::error!` — no dedicated structured events
- Queue/rate-limit events deferred to Phase 22 (Prometheus metrics) — not log events

### Sensitive data policy
- Full target URLs are logged (public websites, not sensitive)
- Never log customer email addresses — use customer_id or scan_id for correlation
- Never log scan findings content — only counts and metadata
- Strict no-secrets policy: API keys, tokens, webhook secrets never appear in any log line
- Any log line containing a secret is treated as a bug

### Development experience
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

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. Use established tracing-subscriber patterns for Rust/Axum applications.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 19-structured-json-logging*
*Context gathered: 2026-02-16*
