# Phase 1: Foundation - Research

**Researched:** 2026-02-04
**Domain:** Rust backend API with async job orchestration, PostgreSQL persistence, and security scanning
**Confidence:** MEDIUM-HIGH

## Summary

Phase 1 requires building a Rust/Axum REST API with job orchestration for security scanning. The standard stack is Axum (Tower-based web framework), Tokio (async runtime), SQLx (PostgreSQL async driver), and tower-governor (rate limiting).

The recommended approach uses in-process Tokio task spawning with semaphore-based concurrency limiting (not channels or separate worker services) to match the "in-process worker pool" decision. Database-as-queue pattern with PostgreSQL SELECT...FOR UPDATE is appropriate for MVP scale (3-5 concurrent scans). SSRF protection leverages Rust's std::net built-in methods (`is_private()`, `is_loopback()`, `is_link_local()`) combined with explicit cloud metadata IP blocking.

Key pitfalls: unbounded `tokio::spawn` causes memory overflow under load, tower-governor requires custom key extractors for multi-factor rate limiting (IP + email), and partial results handling demands explicit retry logic since Tokio tasks don't auto-retry.

**Primary recommendation:** Use Axum 0.8+ with tower-governor for rate limiting, SQLx with migrations, Tokio semaphores for concurrency control, and reqwest for HTTP scanning. Implement SSRF checks before DNS resolution using std::net IP classification methods.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| axum | 0.8+ | Web framework | Official Tokio team framework, Tower middleware ecosystem, type-safe extractors, zero macros for routing |
| tokio | 1.53+ | Async runtime | De facto async runtime for Rust, powers Axum, fixes IP validation vulnerability (pre-1.53 had SSRF-prone octal parsing) |
| sqlx | 0.8+ | PostgreSQL driver | Compile-time query verification, async-native, built-in migrations, works with database-as-queue pattern |
| tower-governor | 0.6+ | Rate limiting | Tower middleware for Axum, uses governor GCRA algorithm, supports custom key extraction |
| reqwest | 0.12+ | HTTP client | Most popular async HTTP client, TLS by default, header access for security scanning |
| serde / serde_json | 1.0+ | JSON serialization | Standard serialization, integrates with Axum extractors |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| problem_details | 0.3+ | RFC 7807 errors | Implements RFC 7807 Problem Details with Axum integration (via feature flag) |
| ipnet | 2.9+ | IP network utilities | Optional - for CIDR-based blocklists, extends std::net types with subnet operations |
| tower-http | 0.6+ | HTTP middleware | Timeouts, tracing, compression - built on Tower like Axum |
| tracing / tracing-subscriber | 0.1+ | Structured logging | Standard logging for Tokio/Axum, integrates with async context |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| axum | actix-web | Actix has more maturity but uses actor model; Axum is simpler, Tower-native, official Tokio project |
| tower-governor | Custom middleware | tower-governor provides battle-tested GCRA algorithm, custom middleware risks race conditions |
| sqlx | diesel | Diesel is sync-only (requires spawn_blocking), SQLx is async-native and better for job queue polling |
| reqwest | ureq | ureq is sync-only, reqwest is async-native (required for Tokio integration) |

**Installation:**
```bash
# Core dependencies
cargo add axum@0.8 tokio@1.53 --features tokio/full
cargo add sqlx@0.8 --features runtime-tokio,tls-rustls,postgres,migrate
cargo add tower-governor@0.6 governor@0.7
cargo add reqwest@0.12 --features rustls-tls
cargo add serde@1 serde_json@1 --features serde/derive

# Error handling (RFC 7807)
cargo add problem_details@0.3 --features axum

# Logging
cargo add tracing@0.1 tracing-subscriber@0.3 --features env-filter

# Optional: IP utilities
cargo add ipnet@2.9
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── main.rs              # Axum server setup, router
├── api/                 # HTTP handlers
│   ├── mod.rs
│   ├── scans.rs         # POST /api/v1/scans, GET /api/v1/scans/:id
│   └── errors.rs        # RFC 7807 error types
├── models/              # Database models, domain types
│   ├── mod.rs
│   ├── scan.rs          # Scan struct, status enum
│   └── finding.rs       # Finding struct, severity enum
├── orchestrator/        # Job orchestration
│   ├── mod.rs
│   ├── worker_pool.rs   # Tokio task spawning with semaphore
│   └── timeout.rs       # Per-scanner timeout logic
├── scanners/            # Scanner implementations
│   ├── mod.rs
│   ├── security_headers.rs  # In-process reqwest-based scanner
│   └── aggregator.rs    # Finding deduplication, scoring
├── db/                  # Database operations
│   ├── mod.rs
│   ├── scans.rs         # Scan CRUD, queue polling
│   └── findings.rs      # Findings storage
├── rate_limit/          # Rate limiting
│   ├── mod.rs
│   └── keys.rs          # Custom KeyExtractor for email+IP
└── ssrf/                # SSRF protection
    ├── mod.rs
    └── validator.rs     # IP/URL validation
migrations/              # SQLx migrations
├── 20260204_001_create_scans.sql
└── 20260204_002_create_findings.sql
```

### Pattern 1: In-Process Worker Pool with Semaphore
**What:** Tokio task spawning limited by `Arc<Semaphore>` to cap concurrency
**When to use:** In-process job execution (user decision), low concurrency (3-5 scans)
**Example:**
```rust
// Source: https://medium.com/@adamszpilewicz/building-a-worker-pool-in-rust-scalable-task-execution-with-tokio-abcb4f193a05
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct WorkerPool {
    semaphore: Arc<Semaphore>,
}

impl WorkerPool {
    pub fn new(max_workers: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_workers)),
        }
    }

    pub async fn spawn<F, Fut>(&self, f: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let permit = self.semaphore.clone().acquire_owned().await.unwrap();
        tokio::spawn(async move {
            f().await;
            drop(permit); // Release on completion
        });
    }
}
```

### Pattern 2: Database-as-Queue with SELECT FOR UPDATE
**What:** PostgreSQL table as job queue with row-locking for worker claiming
**When to use:** MVP scale, simplifies deployment (no separate queue service)
**Example:**
```sql
-- Source: https://aminediro.com/posts/pg_job_queue/
-- Schema
CREATE TYPE scan_status AS ENUM ('pending', 'in_progress', 'completed', 'failed');

CREATE TABLE scans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    target_url TEXT NOT NULL,
    email TEXT NOT NULL,
    status scan_status NOT NULL DEFAULT 'pending',
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Worker claiming a job (in transaction)
SELECT id, target_url, email
FROM scans
WHERE status = 'pending'
ORDER BY created_at ASC
FOR UPDATE SKIP LOCKED
LIMIT 1;

UPDATE scans
SET status = 'in_progress', started_at = NOW()
WHERE id = $1;
```

### Pattern 3: Per-Scanner Timeout with tokio::time::timeout
**What:** Wrap scanner calls in `tokio::time::timeout` to enforce 60s limit
**When to use:** Always (prevents hung scanners from blocking workers)
**Example:**
```rust
// Source: https://docs.rs/tokio/latest/tokio/time/fn.timeout.html
use tokio::time::{timeout, Duration};

async fn run_scanner_with_timeout(url: &str) -> Result<ScanResult, ScanError> {
    match timeout(Duration::from_secs(60), run_scanner(url)).await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(e)) => Err(ScanError::ScannerFailed(e)),
        Err(_) => Err(ScanError::Timeout),
    }
}
```

### Pattern 4: Custom Rate Limit Key Extractor (Email + IP)
**What:** Implement `KeyExtractor` trait to combine email (from body) and IP for composite rate limiting
**When to use:** Multi-factor rate limiting (user decision: 3/day per email AND 10/day per IP)
**Example:**
```rust
// Source: https://docs.rs/tower_governor/latest/tower_governor/
use tower_governor::key_extractor::{KeyExtractor, SmartIpKeyExtractor};
use axum::extract::ConnectInfo;
use std::net::SocketAddr;

#[derive(Clone)]
pub struct EmailIpKeyExtractor;

impl KeyExtractor for EmailIpKeyExtractor {
    type Key = String;

    fn extract<T>(&self, req: &axum::http::Request<T>) -> Result<Self::Key, tower_governor::GovernorError> {
        // Extract IP from ConnectInfo
        let ip = req.extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ci| ci.0.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Extract email from request body (requires buffering)
        // In practice, use two separate limiters:
        // 1. IP-based middleware on route
        // 2. Email-based check in handler after parsing body
        Ok(format!("ip:{}", ip))
    }
}
```

### Pattern 5: RFC 7807 Problem Details Error Response
**What:** Structured error responses with type, title, status, detail
**When to use:** All API errors (user decision)
**Example:**
```rust
// Source: https://docs.rs/problem_details/latest/problem_details/
use problem_details::ProblemDetails;
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;

#[derive(Debug)]
pub enum ApiError {
    RateLimited,
    SsrfBlocked,
    NotFound,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, type_uri, title) = match self {
            ApiError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "https://trustEdge.dev/errors/rate-limited",
                "You've reached your daily scan limit",
            ),
            ApiError::SsrfBlocked => (
                StatusCode::BAD_REQUEST,
                "https://trustEdge.dev/errors/ssrf-blocked",
                "Target URL is not allowed",
            ),
            ApiError::NotFound => (
                StatusCode::NOT_FOUND,
                "https://trustEdge.dev/errors/not-found",
                "Scan not found",
            ),
        };

        ProblemDetails::new()
            .with_type(type_uri)
            .with_title(title)
            .with_status(status)
            .into_response()
    }
}
```

### Pattern 6: SSRF Protection with std::net
**What:** Validate resolved IP addresses against private/special ranges before HTTP requests
**When to use:** Always, before any user-supplied URL scan
**Example:**
```rust
// Source: https://doc.rust-lang.org/std/net/struct.Ipv4Addr.html
use std::net::{ToSocketAddrs, IpAddr};

pub fn is_ssrf_safe(url: &str) -> Result<(), SsrfError> {
    let host = extract_host(url)?;

    // Resolve DNS
    let addrs: Vec<IpAddr> = format!("{}:80", host)
        .to_socket_addrs()?
        .map(|sa| sa.ip())
        .collect();

    for addr in addrs {
        match addr {
            IpAddr::V4(ipv4) => {
                if ipv4.is_private() || ipv4.is_loopback() || ipv4.is_link_local()
                    || ipv4.is_unspecified() || ipv4.is_multicast() {
                    return Err(SsrfError::PrivateIp);
                }
                // Explicit cloud metadata check
                if ipv4.octets() == [169, 254, 169, 254] {
                    return Err(SsrfError::CloudMetadata);
                }
            }
            IpAddr::V6(ipv6) => {
                if ipv6.is_loopback() || ipv6.is_unspecified() || ipv6.is_multicast() {
                    return Err(SsrfError::PrivateIp);
                }
                // Check for IPv6 cloud metadata (AWS fd00:ec2::254)
                if ipv6.segments()[0] == 0xfd00 && ipv6.segments()[1] == 0xec2 {
                    return Err(SsrfError::CloudMetadata);
                }
            }
        }
    }
    Ok(())
}
```

### Anti-Patterns to Avoid
- **Unbounded tokio::spawn:** Spawning tasks in a loop without limiting concurrency floods the runtime with tasks, causing memory overflow when producer rate exceeds consumer rate. Always use semaphores or channels with backpressure.
- **Global-only rate limiting:** Using `GlobalKeyExtractor` applies one limit across all users, not per-email or per-IP as required. Must implement custom key extractor or multiple limiters.
- **Checking URLs after DNS resolution:** SSRF attackers can use DNS rebinding (resolve to public IP initially, then private IP on retry). Check is_private() on resolved IPs before HTTP requests.
- **Blocking HTTP in async context:** Using synchronous HTTP libraries (ureq, curl) in Tokio async functions blocks worker threads. Always use async clients (reqwest).

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rate limiting algorithm | Token bucket or leaky bucket | `tower-governor` (GCRA) | GCRA (Generic Cell Rate Algorithm) handles burst correctly and is race-condition-free; custom implementations often have off-by-one errors or allow burst bypass |
| Database migrations | Manual SQL version tracking | `sqlx migrate` CLI | Tracks applied migrations in `_sqlx_migrations` table, handles up/down, prevents re-application, integrates with offline mode |
| Async IP resolution | Manual UDP DNS packets | `std::net::ToSocketAddrs` | Handles DNS failover, IPv4/IPv6 fallback, system resolver configuration, caching |
| HTTP client with TLS | Raw TCP + OpenSSL bindings | `reqwest` with rustls | TLS certificate validation, connection pooling, redirects, timeout handling, header parsing all require thousands of lines to implement correctly |
| Partial results retry logic | Custom retry state machine | Explicit task state + retry counter | Job queue systems like BullMQ pattern: save completed sub-tasks to DB, spawn new task for remaining work with incremented retry count |

**Key insight:** Async Rust has sharp edges around blocking operations and backpressure. Use battle-tested crates (tower-governor, sqlx, reqwest) instead of reinventing, especially for security-critical paths (rate limiting, TLS, SSRF checks).

## Common Pitfalls

### Pitfall 1: Memory Overflow from Unbounded tokio::spawn
**What goes wrong:** Spawning scan tasks in a loop without concurrency limiting causes heap exhaustion when scan submission rate exceeds completion rate.
**Why it happens:** `tokio::spawn` is non-blocking and returns immediately, so the loop can spawn thousands of tasks before any complete. Each task allocates heap for futures, context, and captured variables.
**How to avoid:** Use `Arc<Semaphore>` to cap concurrent tasks at 3-5 (user decision). Acquire permit before spawn, release on completion.
**Warning signs:** Memory usage climbing under load, eventual OOM kill by OS, logs showing thousands of "in_progress" scans.

### Pitfall 2: tower-governor Doesn't Support Multi-Factor Keys Out-of-Box
**What goes wrong:** User decision requires rate limiting by email AND IP (3/day email, 10/day IP), but tower-governor's `KeyExtractor` trait returns one key. Implementing a composite key like "email:foo@bar.com_ip:1.2.3.4" applies one shared limit, not separate limits.
**Why it happens:** GCRA algorithm state is keyed by a single value. Combining factors into one key creates a Cartesian product (each email+IP pair gets separate limit), not independent limits per factor.
**How to avoid:** Use TWO separate rate limiters: (1) IP-based middleware at route level using `SmartIpKeyExtractor`, (2) Email-based check in handler after parsing request body, using in-memory HashMap or Redis with email as key.
**Warning signs:** User with email A from IP 1 hits limit at 3 scans, but same email from IP 2 gets another 3 scans (10 total), exceeding 3/day email limit.

### Pitfall 3: DNS Rebinding Bypass of SSRF Checks
**What goes wrong:** Validating URL string (e.g., "http://evil.com") before DNS resolution allows attacker to set DNS TTL=0 and change resolution from public IP to 127.0.0.1 between check and request.
**Why it happens:** Time-of-check-time-of-use race. Attacker controls DNS server, returns different IPs for sequential lookups.
**How to avoid:** Resolve DNS ONCE using `ToSocketAddrs`, validate ALL returned IPs with `is_private()`/`is_loopback()`/`is_link_local()`, then use resolved IP directly for HTTP request (not hostname).
**Warning signs:** SSRF protection logs showing public IP, but HTTP client connects to localhost. Scan results revealing internal service info.

### Pitfall 4: SQLx Compile-Time Checking Requires DATABASE_URL at Build
**What goes wrong:** `cargo build` fails in CI with "DATABASE_URL not set" because sqlx macros connect to DB at compile time to verify queries.
**Why it happens:** SQLx's compile-time checking feature queries live database schema to validate SQL types. This breaks in CI without DB access.
**How to avoid:** Use SQLx offline mode: run `cargo sqlx prepare` locally to generate `.sqlx/` metadata directory, commit to git, set `SQLX_OFFLINE=true` in CI. Validates queries against cached metadata instead of live DB.
**Warning signs:** `cargo build` works locally but fails in CI/Docker with "connection refused" or "DATABASE_URL required".

### Pitfall 5: Partial Results Require Explicit Retry Logic
**What goes wrong:** User decision: "retry failed scanner once, then return partial results". Tokio tasks don't auto-retry. If scanner task panics or times out, it's lost unless explicitly handled.
**Why it happens:** `tokio::spawn` fire-and-forget semantics. Task handle must be `.await`ed to detect failure. Panic or timeout returns `Err` from join handle, but doesn't re-execute.
**How to avoid:** Wrap scanner in retry logic: on first failure, log error, increment retry counter in DB, spawn second attempt. After second failure or success, mark scanner complete and aggregate available results. Store per-scanner status in findings table.
**Warning signs:** Scan status stuck at "in_progress", findings missing entire scanner output, no retry attempts logged.

### Pitfall 6: Render Free Tier Auto-Sleep Breaks Background Workers
**What goes wrong:** Render free tier spins down instances after 15 minutes of inactivity. In-process worker polling DB for scans wakes on HTTP request, but if worker runs in background loop, it can't prevent sleep.
**Why it happens:** Render treats web services as request-driven. Background work (polling DB every 5s) doesn't count as "activity" for sleep prevention.
**How to avoid:** For MVP, trigger worker poll on scan submission (lazy execution). Or use paid tier ($7/month) for always-on workers. Or deploy worker as separate "background worker" service type (paid only).
**Warning signs:** Scans stuck "pending" until next HTTP request arrives, 15+ minute delays between submission and execution.

## Code Examples

Verified patterns from official sources:

### Axum Route Versioning
```rust
// Source: https://docs.rs/axum/latest/axum/
use axum::{Router, routing::{get, post}};

fn app() -> Router {
    Router::new()
        .route("/api/v1/scans", post(create_scan))
        .route("/api/v1/scans/:id", get(get_scan))
}
```

### SQLx Migration File Example
```sql
-- Source: https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md
-- migrations/20260204_001_create_scans.sql
CREATE TYPE scan_status AS ENUM ('pending', 'in_progress', 'completed', 'failed');

CREATE TABLE scans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    target_url TEXT NOT NULL,
    email TEXT NOT NULL,
    status scan_status NOT NULL DEFAULT 'pending',
    result JSONB,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    INDEX idx_status_created (status, created_at)
);
```

### SQLx Query with Compile-Time Checking
```rust
// Source: https://www.shuttle.dev/blog/2023/10/04/sql-in-rust
use sqlx::PgPool;

async fn create_scan(pool: &PgPool, url: &str, email: &str) -> Result<Uuid, sqlx::Error> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO scans (target_url, email)
        VALUES ($1, $2)
        RETURNING id
        "#,
        url,
        email
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.id)
}
```

### Reqwest Security Headers Scan
```rust
// Source: https://docs.rs/reqwest/latest/reqwest/
use reqwest::Client;
use std::collections::HashMap;

async fn scan_security_headers(url: &str) -> Result<HashMap<String, String>, reqwest::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let response = client.get(url).send().await?;

    let mut headers = HashMap::new();
    for name in ["content-security-policy", "strict-transport-security",
                 "x-frame-options", "x-content-type-options",
                 "referrer-policy", "permissions-policy"] {
        if let Some(value) = response.headers().get(name) {
            headers.insert(name.to_string(), value.to_str().unwrap_or("").to_string());
        }
    }

    Ok(headers)
}
```

### Tower-Governor Rate Limiting Setup
```rust
// Source: https://docs.rs/tower_governor/latest/tower_governor/
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_governor::key_extractor::SmartIpKeyExtractor;

fn rate_limit_layer() -> GovernorLayer<SmartIpKeyExtractor> {
    let config = GovernorConfigBuilder::default()
        .per_second(10) // 10 requests per IP per second
        .burst_size(10)
        .finish()
        .unwrap();

    GovernorLayer {
        config: Arc::new(config),
    }
}

// In router setup:
Router::new()
    .route("/api/v1/scans", post(create_scan))
    .layer(rate_limit_layer())
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| actix-web | Axum (Tower) | 2021-2022 | Simpler mental model (no actors), official Tokio project, Tower middleware ecosystem |
| Diesel ORM | SQLx with compile-time checking | 2020-2021 | Async-native, no runtime query parsing, offline mode for CI |
| Custom rate limiting | tower-governor (GCRA) | 2021+ | Battle-tested algorithm, no race conditions, burst handling |
| URL string validation | IP validation after DNS resolution | Always critical | Prevents DNS rebinding SSRF bypass |
| Channels for worker pool | Semaphore for concurrency limiting | 2023+ pattern | Simpler for in-process tasks, no channel overhead, backpressure via permit acquisition |
| CVSS numerical scores | Letter grades (A-F) | 2023+ trend | More user-friendly for non-technical audiences, used by SecurityScorecard and others |

**Deprecated/outdated:**
- **Actix-web 3.x**: Actix 4.x made breaking changes, but Axum is now preferred for new projects (official Tokio, simpler)
- **tokio 0.2**: Pre-1.0 had different async traits; 1.0+ is stable API
- **sqlx < 0.6**: Older versions had different migration CLI syntax
- **Rust std::net IP methods on nightly**: `is_global()`, `is_documentation()`, etc. are still nightly-only (as of 2026), so can't rely on them for stable builds

## Open Questions

Things that couldn't be fully resolved:

1. **Render Docker Container Support on Free Tier**
   - What we know: Render free tier supports Docker deployments but has 15-minute auto-sleep for inactivity
   - What's unclear: Whether containerized scanners (Nuclei, testssl.sh) can run on free tier or require paid plan. Documentation mentions "750 instance-hours/month" but doesn't clarify if containers count separately or as part of web service hours.
   - Recommendation: Test in Render staging environment. If containers require paid tier, defer INFRA-05 (containerized scanners) to Phase 2 and use in-process scanners only in Phase 1.

2. **Security Score A-F Calculation Algorithm**
   - What we know: Industry uses CVSS 0-10 scale with severity bands (Low 0-3.9, Medium 4-6.9, High 7-8.9, Critical 9-10). Some services (SecurityScorecard) use A-F letter grades but don't publish calculation methodology.
   - What's unclear: Optimal weighting for user decision: "compute A-F score based on findings severity/count". Should one Critical finding = F? Linear scale? Weighted sum?
   - Recommendation: Start simple - weighted sum approach: assign points per severity (Critical=10, High=5, Medium=2, Low=1), sum total, map to grade bands (A: 0-5, B: 6-10, C: 11-20, D: 21-40, F: 41+). Tune thresholds based on user feedback.

3. **Email-Based Rate Limiting Storage**
   - What we know: tower-governor uses in-memory state (HashMap), which resets on restart and doesn't share across instances
   - What's unclear: For MVP with single instance, in-memory is fine. But user decision requires "3 scans/day per email" - should this persist across restarts?
   - Recommendation: For Phase 1, use PostgreSQL table `rate_limits(email, date, count)` queried in handler (not middleware). Simpler than Redis, persists across restarts, acceptable latency for 3-5 concurrent scans. Migrate to Redis in Phase 3 if needed.

## Sources

### Primary (HIGH confidence)
- [Axum Official Documentation](https://docs.rs/axum/latest/axum/) - Error handling, routing, async handlers
- [Tower-Governor Documentation](https://docs.rs/tower_governor/latest/tower_governor/) - Rate limiting configuration, key extraction
- [SQLx GitHub README](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md) - Migration commands, offline mode
- [Rust std::net::Ipv4Addr](https://doc.rust-lang.org/std/net/struct.Ipv4Addr.html) - SSRF protection methods (is_private, is_loopback, is_link_local)
- [Rust std::net::IpAddr](https://doc.rust-lang.org/std/net/enum.IpAddr.html) - IP classification methods

### Secondary (MEDIUM confidence)
- [Shuttle: Axum Production Guide](https://www.shuttle.dev/blog/2023/12/06/using-axum-rust) - Best practices verified with official docs
- [Medium: Building Worker Pool with Tokio](https://medium.com/@adamszpilewicz/building-a-worker-pool-in-rust-scalable-task-execution-with-tokio-abcb4f193a05) - Semaphore pattern (May 2025)
- [AmineDiro: Postgres Job Queue](https://aminediro.com/posts/pg_job_queue/) - Database-as-queue schema patterns
- [Shuttle: Raw SQL in Rust with SQLx](https://www.shuttle.dev/blog/2023/10/04/sql-in-rust) - Query patterns, compile-time checking
- [OWASP: HTTP Security Headers Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/HTTP_Headers_Cheat_Sheet.html) - CSP, HSTS, X-Frame-Options best practices
- [OWASP: SSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html) - Validation strategies
- [OneUpTime: BullMQ Job Timeouts](https://oneuptime.com/blog/post/2026-01-21-bullmq-job-timeouts/view) - Partial results pattern (January 2026)
- [Resecurity: SSRF to AWS Metadata](https://www.resecurity.com/blog/article/ssrf-to-aws-metadata-exposure-how-attackers-steal-cloud-credentials) - Cloud metadata SSRF attack patterns (2026)

### Tertiary (LOW confidence - marked for validation)
- [Render Free Tier Infographic](https://www.freetiers.com/directory/render) - Auto-sleep behavior (needs testing in Render environment)
- [SecurityScorecard](https://securityscorecard.com/why-securityscorecard/security-ratings/) - A-F rating system (methodology not disclosed)
- WebSearch results on "rust-specific best practices" - aggregated from multiple blog posts (2025-2026)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries are well-documented with official docs, widely used in production
- Architecture patterns: MEDIUM-HIGH - Patterns verified with official docs, but custom key extractor and database-as-queue need implementation testing
- Pitfalls: HIGH - Documented in official sources (tokio spawn, DNS rebinding, SQLx offline mode) and recent articles (Render sleep, partial results)
- SSRF protection: HIGH - Rust std::net methods are stable (1.53+), cloud metadata IPs are public knowledge
- Rate limiting: MEDIUM - tower-governor is well-documented, but multi-factor limiting requires custom implementation not shown in official examples
- Security scoring: LOW - No authoritative source for A-F calculation from vulnerability counts/severities

**Research date:** 2026-02-04
**Valid until:** 2026-03-04 (30 days - stable ecosystem, but check for Axum 0.9/SQLx 0.9 releases)
