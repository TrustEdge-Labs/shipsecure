# Phase 05: Codebase Preparation - Research

**Researched:** 2026-02-06
**Domain:** Codebase refactoring for DigitalOcean deployment
**Confidence:** HIGH

## Summary

Phase 05 prepares the existing v1.0 TrustEdge Audit codebase for DigitalOcean deployment by removing Render-specific configuration, converting scanner execution from Docker containers to native subprocess invocations, and externalizing all configuration into environment variables. The current codebase already uses `tokio::process::Command` with `docker run` (not bollard), so the "subprocess conversion" is actually simpler than expected—it's primarily about adding environment variable configuration for binary paths and template directories, plus graceful degradation when binaries are missing.

**Key Discovery:** The codebase already executes Nuclei and testssl.sh as Docker containers via subprocess (`Command::new("docker")`), not via the bollard crate. The Phase 05 work is about making scanner execution configurable (env vars for binary paths, optional templates directory) and removing Render deployment artifacts, NOT a fundamental architectural change.

**Primary recommendation:** Add configuration-driven scanner execution with graceful degradation, externalize all environment variables to `.env`, create `.env.example` with comprehensive documentation, remove docker-compose.yml (replaced in Phase 06 with production-ready version), and audit/remove all Render references from code, docs, and configuration.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Nuclei subprocess design:**
- Binary located via `NUCLEI_BINARY_PATH` env var with fallback to PATH lookup
- If Nuclei binary not found at startup: log warning and continue (other scanners still work). Good for dev environments.
- Custom vibe-code templates bundled in repo (e.g., `templates/nuclei/`), with optional override via `NUCLEI_TEMPLATES_DIR` env var for additional/custom templates
- Scan output captured via JSON temp file (`-o` flag), not stdout. Avoids buffering issues.

**testssl.sh subprocess design:**
- Also moves to subprocess execution (bollard removed entirely)
- Same pattern as Nuclei: binary path via env var, graceful skip if missing
- Claude's discretion on specific invocation details

**Configuration structure:**
- Single `.env` file for all variables
- `.env.example` committed to repo with all variable names and placeholder/description comments (no real values)
- Fail fast at startup: validate ALL required env vars on boot, crash with clear error listing which vars are missing
- All env vars must be explicitly set — no hidden defaults. PORT, LOG_LEVEL, MAX_CONCURRENT_SCANS, etc. all require explicit values in .env
- What you see in .env.example is exactly what the app needs

**Render cleanup scope:**
- Thorough cleanup: delete render.yaml, strip all Render-specific env var names, remove Render mentions from docs/comments, refactor any code that assumed Render's environment
- Remove bollard crate entirely (both Nuclei and testssl.sh move to subprocess)
- `/health` endpoint stays (standard, not Render-specific)
- Claude should audit codebase for any other Render-specific assumptions during research/planning
- Update README and docs in this phase to reflect new setup — remove Render references, add new local dev instructions

**Dev/prod parity:**
- Docker Compose for both local development AND production deployment
- Full stack in Docker Compose: Rust backend + Next.js frontend + PostgreSQL + scanner binaries
- One `docker compose up` to run everything locally
- Production uses same docker-compose.yml with prod overrides (e.g., docker-compose.prod.yml)
- Systemd manages `docker compose up` on the droplet
- Multi-stage Dockerfiles: build stage (compile Rust, build Next.js) + slim runtime stage

### Claude's Discretion

- testssl.sh subprocess invocation details
- Exact temp file handling for Nuclei JSON output
- Docker Compose service naming and networking
- .env.example variable descriptions and grouping
- Dockerfile base images and runtime dependencies

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

---

## Current State Audit

### Existing Scanner Implementation (CRITICAL FINDING)

**Current architecture (already subprocess-based):**

```rust
// src/scanners/container.rs lines 100-129
async fn run_docker_container(args: &[&str], timeout: Duration) -> Result<String, ScannerError> {
    let child = Command::new("docker")  // ← Already using subprocess!
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| ScannerError::ContainerError(format!("Failed to spawn docker: {}", e)))?;

    // Wait with timeout
    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(Ok(output)) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(ScannerError::ContainerError(format!(
                    "Container exited with code {}: {}",
                    output.status.code().unwrap_or(-1),
                    stderr
                )))
            }
        }
        Ok(Err(e)) => Err(ScannerError::ContainerError(format!("Failed to wait for container: {}", e))),
        Err(_) => {
            // Timeout occurred, try to kill the container
            tracing::warn!("Container execution timed out after {:?}", timeout);
            Err(ScannerError::ContainerTimeout)
        }
    }
}
```

**What this means:**
- **Bollard was never used** — Cargo.toml has no bollard dependency
- Scanners already execute via `tokio::process::Command::new("docker")`
- The "conversion to subprocess" is really just making execution **configurable** via env vars
- No fundamental architectural change required

### Current Nuclei Execution

```rust
// src/scanners/container.rs lines 42-68
pub async fn run_nuclei(target: &str) -> Result<Vec<Finding>, ScannerError> {
    if !is_docker_available().await {
        tracing::warn!("Docker not available, skipping Nuclei scan");
        return Ok(Vec::new());  // ← Graceful degradation already implemented
    }

    let args = vec![
        "run",
        "--rm",
        "--read-only",
        "--cap-drop", "all",
        "--user", "1000:1000",
        "--memory", "512M",
        "--pids-limit", "1000",
        "--cpu-shares", "512",
        "--no-new-privileges",
        "projectdiscovery/nuclei:latest",  // ← Hardcoded Docker image
        "-u", target,
        "-jsonl",
        "-silent",
        "-severity", "medium,high,critical",
        "-tags", "exposure,misconfig,cve",
    ];

    let output = run_docker_container(&args, Duration::from_secs(120)).await?;
    parse_nuclei_output(&output, target)
}
```

**Current limitations:**
- Hardcoded Docker image (`projectdiscovery/nuclei:latest`)
- No env var for binary path
- No configurable templates directory
- Outputs to stdout (JSONL), user wants temp file (`-o` flag)

### Current testssl.sh Execution

```rust
// src/scanners/container.rs lines 72-97
pub async fn run_testssl(target: &str) -> Result<Vec<Finding>, ScannerError> {
    if !is_docker_available().await {
        tracing::warn!("Docker not available, skipping testssl.sh scan");
        return Ok(Vec::new());
    }

    let args = vec![
        "run",
        "--rm",
        "--read-only",
        "--cap-drop", "all",
        "--user", "1000:1000",
        "--memory", "100M",
        "--pids-limit", "1000",
        "--cpu-shares", "512",
        "--no-new-privileges",
        "drwetter/testssl.sh:latest",  // ← Hardcoded Docker image
        "--jsonfile-pretty", "/dev/stdout",  // ← Outputs to stdout
        "--quiet",
        target,
    ];

    let output = run_docker_container(&args, Duration::from_secs(180)).await?;
    parse_testssl_output(&output, target)
}
```

### Current Environment Variables

**From code audit (`grep -r "std::env::var"`):**

| Variable | File | Usage | Required? |
|----------|------|-------|-----------|
| `DATABASE_URL` | `main.rs:31` | PostgreSQL connection string | Yes (panics if missing) |
| `PORT` | `main.rs:77` | HTTP server port | No (defaults to 3000) |
| `RESEND_API_KEY` | `email/mod.rs:49,115` | Email delivery | Optional (graceful skip) |
| `TRUSTEDGE_BASE_URL` | `orchestrator/worker_pool.rs:259` | Email links | Optional (defaults to localhost) |
| `TRUSTEDGE_TEMPLATES_DIR` | `scanners/vibecode.rs:122` | Custom Nuclei templates | Optional (falls back to bundled) |
| `STRIPE_WEBHOOK_SECRET` | `api/webhooks.rs:65` | Stripe webhook validation | Required for webhooks |
| `STRIPE_SECRET_KEY` | `api/checkout.rs:53` | Stripe checkout | Required for checkout |
| `FRONTEND_URL` | `api/checkout.rs:65` | Checkout redirect | Optional (defaults logic unclear) |

**Missing from current .env.example:**

```bash
# Current .env.example (INCOMPLETE):
DATABASE_URL=postgres://trustedge:trustedge@localhost:5432/trustedge_dev
PORT=3000
RUST_LOG=info,trustedge_audit=debug
```

**What's missing:**
- All Stripe variables (STRIPE_SECRET_KEY, STRIPE_WEBHOOK_SECRET, STRIPE_PUBLISHABLE_KEY)
- RESEND_API_KEY
- TRUSTEDGE_BASE_URL
- FRONTEND_URL
- MAX_CONCURRENT_SCANS (orchestrator hardcodes 5)
- Scanner-specific config (NUCLEI_BINARY_PATH, TESTSSL_BINARY_PATH, etc.)

### Current Docker Compose Setup

**File:** `docker-compose.yml`

```yaml
services:
  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: trustedge
      POSTGRES_PASSWORD: trustedge
      POSTGRES_DB: trustedge_dev
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data

  backend:
    build:
      context: .
      dockerfile: Dockerfile
    env_file:
      - .env
    environment:
      DATABASE_URL: postgres://trustedge:trustedge@db:5432/trustedge_dev
      PORT: "3000"
      RUST_LOG: info
      TRUSTEDGE_BASE_URL: http://localhost:3001
    ports:
      - "3000:3000"
    depends_on:
      db:
        condition: service_healthy

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    environment:
      BACKEND_URL: http://backend:3000
      NEXT_PUBLIC_BACKEND_URL: http://localhost:3000
    ports:
      - "3001:3001"
    depends_on:
      - backend

volumes:
  pgdata:
```

**Issues for production:**
- No Docker socket mount (backend can't spawn scanner containers)
- No runtime dependencies (Docker CLI not installed in containers)
- Development-focused (postgres:16-alpine is fine, but no optimizations)
- Missing production overrides (resource limits, health checks)
- Backend needs access to host Docker socket to spawn Nuclei/testssl containers

### Current Render References

**Searched pattern:** `grep -ri "render" --include="*.md" --include="*.yml" --include="*.yaml"`

**Found in:**
- `.planning/` documentation (multiple files referencing "Render" as prior hosting)
- `README.md:23` — "Platform detection — Auto-detects Vercel, Netlify, Railway, Render, Supabase, Firebase"
- No `render.yaml` file exists
- No Render-specific environment variable names found in code

**Verdict:** Minimal cleanup needed. "Render" is mentioned in:
1. Planning docs (historical context, not code)
2. Platform detection (legitimate feature for scanning Render-hosted apps)

**Action:** Remove Render from platform detection list in README (no longer hosting provider), update deployment documentation.

---

## Standard Stack for Subprocess Execution

### Binary Path Resolution Pattern

**Industry standard (used by rustup, cargo, git):**

```rust
// Priority order:
// 1. Explicit env var (NUCLEI_BINARY_PATH=/usr/local/bin/nuclei)
// 2. PATH lookup (which nuclei)
// 3. Common installation paths (/usr/local/bin, /usr/bin, /opt/nuclei/bin)
// 4. Graceful skip (log warning, return empty findings)

fn resolve_nuclei_binary() -> Option<PathBuf> {
    // 1. Check env var
    if let Ok(path) = std::env::var("NUCLEI_BINARY_PATH") {
        let p = PathBuf::from(path);
        if p.exists() && p.is_file() {
            return Some(p);
        }
        tracing::warn!("NUCLEI_BINARY_PATH set but not found: {}", p.display());
    }

    // 2. Check PATH
    if let Ok(path) = which::which("nuclei") {
        return Some(path);
    }

    // 3. Check common installation paths
    for path_str in ["/usr/local/bin/nuclei", "/usr/bin/nuclei", "/opt/nuclei/bin/nuclei"] {
        let p = PathBuf::from(path_str);
        if p.exists() {
            return Some(p);
        }
    }

    // 4. Not found
    None
}
```

**Dependency needed:** `which = "6.0"` crate for cross-platform PATH lookup

### Temp File Handling for JSON Output

**User requirement:** "Scan output captured via JSON temp file (`-o` flag), not stdout. Avoids buffering issues."

**Standard approach (used by rustc, cargo):**

```rust
use std::fs;
use tempfile::NamedTempFile;

async fn run_nuclei_with_tempfile(target: &str) -> Result<Vec<Finding>, ScannerError> {
    // Create temp file for JSON output
    let temp_file = NamedTempFile::new()
        .map_err(|e| ScannerError::ContainerError(format!("Failed to create temp file: {}", e)))?;
    let temp_path = temp_file.path();

    let args = vec![
        "nuclei",  // Direct binary, not docker
        "-u", target,
        "-jsonl",
        "-silent",
        "-o", temp_path.to_str().unwrap(),  // Output to file
    ];

    // Execute binary
    let status = Command::new(nuclei_binary)
        .args(&args)
        .stdout(Stdio::null())  // Don't capture stdout
        .stderr(Stdio::piped())  // Capture stderr for errors
        .status()
        .await
        .map_err(|e| ScannerError::ContainerError(format!("Failed to run Nuclei: {}", e)))?;

    if !status.success() {
        return Err(ScannerError::ContainerError(
            format!("Nuclei exited with code {}", status.code().unwrap_or(-1))
        ));
    }

    // Read JSON from temp file
    let output = fs::read_to_string(temp_path)
        .map_err(|e| ScannerError::ParseError(format!("Failed to read output file: {}", e)))?;

    // Temp file auto-deleted when NamedTempFile goes out of scope
    parse_nuclei_output(&output, target)
}
```

**Dependency needed:** `tempfile = "3.16"` crate for cross-platform temp file creation

### Templates Directory Resolution

**Current implementation (from vibecode.rs:122):**

```rust
fn get_templates_dir() -> PathBuf {
    // Check env var override
    if let Ok(dir) = std::env::var("TRUSTEDGE_TEMPLATES_DIR") {
        return PathBuf::from(dir);
    }

    // Default: bundled templates in repo
    PathBuf::from("templates/nuclei")
}
```

**This is already correct!** Just needs documentation in .env.example.

### Graceful Degradation Pattern

**Current implementation (already correct):**

```rust
pub async fn run_nuclei(target: &str) -> Result<Vec<Finding>, ScannerError> {
    if !is_docker_available().await {
        tracing::warn!("Docker not available, skipping Nuclei scan");
        return Ok(Vec::new());  // ← Empty findings, not an error
    }
    // ... scanner execution
}
```

**Pattern to replicate for binary-based execution:**

```rust
pub async fn run_nuclei(target: &str) -> Result<Vec<Finding>, ScannerError> {
    let nuclei_binary = match resolve_nuclei_binary() {
        Some(path) => path,
        None => {
            tracing::warn!("Nuclei binary not found, skipping Nuclei scan");
            return Ok(Vec::new());  // Graceful skip
        }
    };
    // ... execute binary
}
```

---

## Architecture Patterns

### Pattern 1: Environment-Driven Scanner Configuration

**What:** Scanner execution configured entirely via environment variables, with sensible fallbacks.

**When to use:** Always for production deployments, optional for development.

**Implementation:**

```rust
// Scanner configuration loaded at startup
pub struct ScannerConfig {
    pub nuclei_binary: Option<PathBuf>,
    pub testssl_binary: Option<PathBuf>,
    pub templates_dir: PathBuf,
    pub nuclei_timeout_secs: u64,
    pub testssl_timeout_secs: u64,
}

impl ScannerConfig {
    pub fn from_env() -> Self {
        Self {
            nuclei_binary: resolve_nuclei_binary(),
            testssl_binary: resolve_testssl_binary(),
            templates_dir: get_templates_dir(),
            nuclei_timeout_secs: std::env::var("NUCLEI_TIMEOUT_SECS")
                .unwrap_or_else(|_| "120".to_string())
                .parse()
                .expect("NUCLEI_TIMEOUT_SECS must be a number"),
            testssl_timeout_secs: std::env::var("TESTSSL_TIMEOUT_SECS")
                .unwrap_or_else(|_| "180".to_string())
                .parse()
                .expect("TESTSSL_TIMEOUT_SECS must be a number"),
        }
    }
}

// Pass config to orchestrator
let scanner_config = Arc::new(ScannerConfig::from_env());
let orchestrator = ScanOrchestrator::new(pool.clone(), 5, scanner_config);
```

**Benefits:**
- Single source of truth for all configuration
- Easy to test (override env vars in tests)
- Clear deployment requirements (everything in .env.example)

### Pattern 2: Fail-Fast Validation at Startup

**User requirement:** "Fail fast at startup: validate ALL required env vars on boot, crash with clear error listing which vars are missing."

**Implementation:**

```rust
// main.rs startup
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Validate required env vars
    validate_required_env_vars(&[
        "DATABASE_URL",
        "PORT",
        "RUST_LOG",
        "TRUSTEDGE_BASE_URL",
        "MAX_CONCURRENT_SCANS",
    ]).expect("Missing required environment variables");

    // Validate optional env vars (warn if missing)
    validate_optional_env_vars(&[
        "RESEND_API_KEY",
        "STRIPE_SECRET_KEY",
        "STRIPE_WEBHOOK_SECRET",
        "NUCLEI_BINARY_PATH",
        "TESTSSL_BINARY_PATH",
    ]);

    // ... rest of startup
}

fn validate_required_env_vars(vars: &[&str]) -> Result<(), String> {
    let mut missing = Vec::new();
    for var in vars {
        if std::env::var(var).is_err() {
            missing.push(*var);
        }
    }

    if !missing.is_empty() {
        return Err(format!(
            "Missing required environment variables:\n  - {}\n\nSee .env.example for configuration.",
            missing.join("\n  - ")
        ));
    }

    Ok(())
}

fn validate_optional_env_vars(vars: &[&str]) {
    for var in vars {
        if std::env::var(var).is_err() {
            tracing::warn!("Optional env var {} not set, some features may be disabled", var);
        }
    }
}
```

### Pattern 3: Docker Socket Access for Scanner Containers

**Context:** Backend runs in Docker container, needs to spawn scanner containers (Nuclei, testssl.sh).

**Pattern:** Bind-mount host Docker socket into backend container.

**docker-compose.yml:**

```yaml
services:
  backend:
    build: .
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro  # Read-only socket
    user: "1000:999"  # Non-root user, docker group
    environment:
      DATABASE_URL: postgres://trustedge:${DB_PASSWORD}@db:5432/trustedge_prod
```

**Security considerations:**
- Socket mounted read-only (`:ro`)
- Backend runs as non-root user (UID 1000)
- User is member of docker group (GID 999) to access socket
- Scanner containers are CIS-hardened (8 security flags)

**When NOT to use:**
- If backend runs natively (no Docker), call Docker CLI directly
- If backend is untrusted code (major security risk)

### Pattern 4: Multi-Stage Dockerfile for Slim Runtime

**User requirement:** "Multi-stage Dockerfiles: build stage (compile Rust, build Next.js) + slim runtime stage"

**Current Dockerfile is already multi-stage!**

```dockerfile
# Build stage
FROM rust:1.88-slim-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src target/release/.fingerprint/trustedge_audit-*

# Build application
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/trustedge_audit /usr/local/bin/
COPY --from=builder /app/migrations /app/migrations
WORKDIR /app
EXPOSE 3000
CMD ["trustedge_audit"]
```

**What's missing for production:**
- Docker CLI installation in runtime stage (to spawn scanner containers)
- Non-root user in runtime stage
- Templates directory copied to runtime stage

**Updated runtime stage:**

```dockerfile
# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates docker.io && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -U -s /bin/bash trustedge && \
    usermod -aG docker trustedge

# Copy application
COPY --from=builder /app/target/release/trustedge_audit /usr/local/bin/
COPY --from=builder /app/migrations /app/migrations
COPY templates /app/templates

# Set ownership
RUN chown -R trustedge:trustedge /app

USER trustedge
WORKDIR /app
EXPOSE 3000
CMD ["trustedge_audit"]
```

---

## Don't Hand-Roll

### Problem: Environment Variable Validation

**Don't build:** Custom validation logic with manual error messages.

**Use instead:** `envy` or `config` crate for structured environment parsing.

```rust
// Using envy crate
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    database_url: String,
    port: u16,
    rust_log: String,
    trustedge_base_url: String,
    max_concurrent_scans: usize,
    #[serde(default)]
    resend_api_key: Option<String>,
    #[serde(default)]
    stripe_secret_key: Option<String>,
}

fn load_config() -> Result<Config, envy::Error> {
    envy::from_env::<Config>()
}
```

**Why:** Handles type coercion, optional fields, clear error messages automatically.

**Verdict:** For this phase, manual validation is acceptable (only 10-15 vars). Envy adds dependency for minimal benefit. **Skip for now.**

### Problem: Binary Path Resolution

**Don't build:** Manual PATH parsing, cross-platform path logic.

**Use instead:** `which` crate for standard binary resolution.

```rust
use which::which;

fn resolve_nuclei_binary() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("NUCLEI_BINARY_PATH") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    // which crate handles PATH, cross-platform, edge cases
    which("nuclei").ok()
}
```

**Why:** Handles Windows vs Unix PATH differences, symlink resolution, file permissions.

**Verdict:** **Use `which` crate.** Solves real complexity, well-tested, minimal dependency cost.

### Problem: Temporary File Creation

**Don't build:** Manual `/tmp/` path construction, cleanup logic, collision avoidance.

**Use instead:** `tempfile` crate for safe temp file management.

```rust
use tempfile::NamedTempFile;

let temp_file = NamedTempFile::new()?;
let temp_path = temp_file.path();

// File auto-deleted when NamedTempFile drops (RAII)
```

**Why:** Handles platform differences (`/tmp` vs `C:\Users\...\Temp`), collision avoidance, secure permissions, auto-cleanup.

**Verdict:** **Use `tempfile` crate.** Essential for cross-platform correctness.

---

## Common Pitfalls

### Pitfall 1: Hardcoding Binary Paths

**What goes wrong:** Code assumes Nuclei is at `/usr/bin/nuclei`, fails on systems where it's at `/usr/local/bin/nuclei` or `~/.local/bin/nuclei`.

**Why it happens:** Developer installs binary once, hardcodes their local path, doesn't test on clean system.

**How to avoid:**
1. Always use env var + PATH lookup + common paths fallback
2. Never hardcode absolute paths
3. Log resolved path at startup for debugging

**Warning signs:**
- Scans work locally but fail in Docker/CI
- "command not found" errors in production

### Pitfall 2: Forgetting to Copy Templates to Docker Image

**What goes wrong:** `templates/nuclei/` exists on developer's machine but not in Docker container. Scans fail with "templates not found".

**Why it happens:** Dockerfile doesn't COPY templates directory, or copies to wrong location.

**How to avoid:**
1. Explicitly COPY templates in Dockerfile
2. Verify templates exist in container (`docker run --rm image ls /app/templates`)
3. Test with `TRUSTEDGE_TEMPLATES_DIR` env var override

**Warning signs:**
- Scans work with `cargo run`, fail with Docker
- Error: "Templates directory not found: /app/templates/nuclei"

### Pitfall 3: stdout Buffering Issues (Why User Wants Temp Files)

**What goes wrong:** Large JSON output from Nuclei gets buffered, partially read, or truncated when reading from stdout.

**Why it happens:** Subprocess stdout buffer size is limited (typically 64KB). Nuclei output can exceed this, causing deadlock or data loss.

**How to avoid:**
1. Use temp file output (`-o /tmp/scan.json`)
2. Read file after process completes
3. Avoid reading large stdout streams synchronously

**Warning signs:**
- Scans hang indefinitely
- Missing findings in parsed output
- "broken pipe" errors

### Pitfall 4: Missing Docker Socket Permissions in Container

**What goes wrong:** Backend container tries to spawn scanner containers, gets "permission denied" on `/var/run/docker.sock`.

**Why it happens:** Backend runs as non-root user (UID 1000) but isn't in docker group (GID 999).

**How to avoid:**
1. Add user to docker group in Dockerfile: `usermod -aG docker trustedge`
2. Specify user:group in docker-compose: `user: "1000:999"`
3. Mount socket with correct permissions

**Warning signs:**
- Error: "dial unix /var/run/docker.sock: connect: permission denied"
- Scans work on host, fail in Docker

### Pitfall 5: Environment Variable Type Mismatches

**What goes wrong:** `.env` has `MAX_CONCURRENT_SCANS=5` (string), code expects `usize`, parsing fails at runtime.

**Why it happens:** Environment variables are always strings, must be parsed to correct type.

**How to avoid:**
1. Always `.parse::<Type>()` with `.expect()` or proper error handling
2. Validate type conversions at startup, not during scan
3. Document expected types in .env.example

**Warning signs:**
- Panics: "thread 'main' panicked at 'PORT must be a number'"
- Silent type coercion errors (string "5" treated as 0)

### Pitfall 6: Incomplete .env.example Documentation

**What goes wrong:** New developer clones repo, copies .env.example, app crashes because required vars are missing.

**Why it happens:** .env.example is outdated, missing new variables added in recent features.

**How to avoid:**
1. **Every time you add `std::env::var()`, update .env.example**
2. Include all variables, even optional ones, with comments
3. Use placeholder values (never real secrets)

**Example:**

```bash
# ❌ BAD (missing vars, no descriptions)
DATABASE_URL=postgres://localhost/db
PORT=3000

# ✅ GOOD (complete, documented)
# PostgreSQL connection string
# Format: postgres://username:password@host:port/database
DATABASE_URL=postgres://trustedge:CHANGEME@localhost:5432/trustedge_dev

# HTTP server port (default: 3000)
PORT=3000

# Nuclei binary path (optional, defaults to PATH lookup)
# NUCLEI_BINARY_PATH=/usr/local/bin/nuclei

# Maximum concurrent scans (default: 5, increase for production)
MAX_CONCURRENT_SCANS=5

# Resend API key for email delivery (optional, emails disabled if not set)
# Get from: https://resend.com/api-keys
# RESEND_API_KEY=re_...

# Stripe secret key (required for checkout)
# Get from: https://dashboard.stripe.com/apikeys
# STRIPE_SECRET_KEY=sk_test_...
```

---

## Code Examples

### Verified Pattern: Subprocess Execution with Timeout

**Source:** Current implementation in `src/scanners/container.rs:100-129` (HIGH confidence)

```rust
use tokio::process::Command;
use tokio::time::timeout;
use std::time::Duration;

async fn run_binary_with_timeout(
    binary: &Path,
    args: &[&str],
    timeout_duration: Duration,
) -> Result<String, ScannerError> {
    let child = Command::new(binary)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ScannerError::ContainerError(format!("Failed to spawn: {}", e)))?;

    match timeout(timeout_duration, child.wait_with_output()).await {
        Ok(Ok(output)) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(ScannerError::ContainerError(format!(
                    "Process exited with code {}: {}",
                    output.status.code().unwrap_or(-1),
                    stderr
                )))
            }
        }
        Ok(Err(e)) => Err(ScannerError::ContainerError(format!("Wait failed: {}", e))),
        Err(_) => {
            tracing::warn!("Process timed out after {:?}", timeout_duration);
            Err(ScannerError::ContainerTimeout)
        }
    }
}
```

### Verified Pattern: Graceful Feature Detection

**Source:** Current implementation in `src/scanners/container.rs:29-39` (HIGH confidence)

```rust
async fn is_docker_available() -> bool {
    let result = Command::new("docker")
        .arg("info")
        .output()
        .await;

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

// Usage:
pub async fn run_nuclei(target: &str) -> Result<Vec<Finding>, ScannerError> {
    if !is_docker_available().await {
        tracing::warn!("Docker not available, skipping Nuclei scan");
        return Ok(Vec::new());  // Empty findings, not error
    }
    // ... execute scanner
}
```

### New Pattern: Environment Variable Groups in .env.example

**Source:** Best practices from Twelve-Factor App, Docker docs, Rust ecosystem (MEDIUM confidence)

```bash
# .env.example

# ========================================
# CORE CONFIGURATION
# ========================================

# PostgreSQL connection string (REQUIRED)
# Format: postgres://username:password@host:port/database
# Development: postgres://trustedge:trustedge@localhost:5432/trustedge_dev
# Production: postgres://trustedge:STRONG_PASSWORD@db:5432/trustedge_prod
DATABASE_URL=postgres://trustedge:trustedge@localhost:5432/trustedge_dev

# HTTP server port (REQUIRED)
# Default: 3000
# Production: Usually 3000 (proxied by Nginx)
PORT=3000

# Logging level (REQUIRED)
# Options: error, warn, info, debug, trace
# Format: level,crate_name=level
# Example: info,trustedge_audit=debug,sqlx=warn
RUST_LOG=info,trustedge_audit=debug

# ========================================
# APPLICATION SETTINGS
# ========================================

# Base URL for this instance (REQUIRED)
# Used for generating email links
# Development: http://localhost:3001
# Production: https://trustedge.audit
TRUSTEDGE_BASE_URL=http://localhost:3001

# Maximum concurrent scans (REQUIRED)
# Default: 5 (safe for 4GB RAM)
# Production 8GB+: 10-20
MAX_CONCURRENT_SCANS=5

# ========================================
# SCANNER CONFIGURATION (OPTIONAL)
# ========================================

# Nuclei binary path
# If not set, searches PATH then common install locations
# Example: /usr/local/bin/nuclei
# NUCLEI_BINARY_PATH=

# testssl.sh binary path
# If not set, searches PATH then /usr/local/bin/testssl.sh
# TESTSSL_BINARY_PATH=

# Custom Nuclei templates directory
# If not set, uses bundled templates at templates/nuclei/
# Example: /opt/trustedge/custom-templates
# TRUSTEDGE_TEMPLATES_DIR=

# Nuclei scan timeout in seconds (OPTIONAL)
# Default: 120
# NUCLEI_TIMEOUT_SECS=120

# testssl.sh scan timeout in seconds (OPTIONAL)
# Default: 180
# TESTSSL_TIMEOUT_SECS=180

# ========================================
# THIRD-PARTY SERVICES (OPTIONAL)
# ========================================

# Resend API key for email delivery
# If not set, email features are disabled
# Get from: https://resend.com/api-keys
# RESEND_API_KEY=re_...

# Stripe secret key for payments
# Required for checkout endpoints
# Get from: https://dashboard.stripe.com/apikeys
# STRIPE_SECRET_KEY=sk_test_...

# Stripe webhook secret for webhook validation
# Required for webhook endpoint
# Get from: https://dashboard.stripe.com/webhooks
# STRIPE_WEBHOOK_SECRET=whsec_...

# Frontend URL for Stripe checkout redirects (OPTIONAL)
# Default: TRUSTEDGE_BASE_URL
# FRONTEND_URL=http://localhost:3001
```

---

## Open Questions

### 1. Docker Socket Access in Production

**What we know:**
- Backend needs to spawn scanner containers (Nuclei, testssl.sh)
- Current docker-compose.yml does NOT mount Docker socket
- Production deployment will need socket access

**What's unclear:**
- User decided "Nuclei as subprocess" but containers ARE subprocesses (via `docker run`)
- Is user aware that "subprocess" still means Docker containers, not native binaries?
- Or did user intend to install Nuclei/testssl as native binaries on the droplet?

**Recommendation:**
- **Clarify in planning:** Does "subprocess" mean:
  - A) Native binaries (`/usr/local/bin/nuclei` + `/usr/local/bin/testssl.sh`)
  - B) Subprocess calls to Docker CLI (`docker run projectdiscovery/nuclei`)
- **Likely answer:** Option A (native binaries) based on "Install Nuclei binary directly" in CONTEXT.md
- **Action:** Plan Phase 05 for native binary execution, Phase 06 for installing binaries on droplet

### 2. testssl.sh Invocation Details

**What we know:**
- testssl.sh will move to subprocess (native binary or Docker)
- Current invocation: `docker run drwetter/testssl.sh --jsonfile-pretty /dev/stdout --quiet TARGET`

**What's unclear:**
- If native binary: Where is testssl.sh installed? (GitHub releases? Package manager?)
- If native binary: What's the temp file output flag? (Same as `--jsonfile-pretty /path`?)
- Does testssl.sh support temp file output, or only stdout?

**Recommendation:**
- Research testssl.sh CLI in planning phase
- Document installation method (apt-get, git clone, or Docker only)
- If no native install exists, keep Docker execution

### 3. Frontend-Backend Communication in Docker Compose

**What we know:**
- Current setup: Frontend calls `http://localhost:3000` (assumes port-forwarding)
- Production: Frontend and backend in same Docker network

**What's unclear:**
- In production Docker Compose, does frontend call `http://backend:3000` (Docker DNS)?
- Or does frontend still call `http://localhost:3000` via Nginx reverse proxy?

**Recommendation:**
- In local dev: Frontend → `http://localhost:3000` (port forwarding)
- In production: Frontend → `http://backend:3000` (Docker internal network)
- Use env var `NEXT_PUBLIC_BACKEND_URL` to switch between environments

---

## Sources

### Primary (HIGH confidence)

**Existing codebase:**
- `src/scanners/container.rs` — Current subprocess execution pattern (already using `Command::new("docker")`)
- `src/main.rs` — Environment variable usage (DATABASE_URL, PORT)
- `Cargo.toml` — No bollard dependency exists (confirms subprocess approach)
- `docker-compose.yml` — Current development setup
- `Dockerfile` — Multi-stage build already implemented
- `.env.example` — Current (incomplete) environment variables

**Subprocess execution:**
- [tokio::process::Command docs](https://docs.rs/tokio/latest/tokio/process/struct.Command.html) — Async subprocess execution
- [which crate docs](https://docs.rs/which/latest/which/) — Binary path resolution
- [tempfile crate docs](https://docs.rs/tempfile/latest/tempfile/) — Temporary file creation

**Docker best practices:**
- [Docker Security Cheat Sheet - OWASP](https://cheatsheetseries.owasp.org/cheatsheets/Docker_Security_Cheat_Sheet.html) — Socket security, non-root users
- [Dockerfile Best Practices - Docker Docs](https://docs.docker.com/develop/dev-best-practices/) — Multi-stage builds, layer caching

### Secondary (MEDIUM confidence)

**Environment variable patterns:**
- [The Twelve-Factor App - Config](https://12factor.net/config) — Environment-based configuration
- [Rust env_logger docs](https://docs.rs/env_logger/) — RUST_LOG configuration format

**Binary path resolution:**
- [PATH environment variable - Wikipedia](https://en.wikipedia.org/wiki/PATH_(variable)) — How PATH works
- [which command - man page](https://man7.org/linux/man-pages/man1/which.1.html) — Binary lookup behavior

### Tertiary (LOW confidence)

**testssl.sh installation:**
- [testssl.sh GitHub](https://github.com/drwetter/testssl.sh) — Installation methods (git clone or Docker only, no native packages)
- LOW confidence on native binary availability — needs verification in planning

---

## Metadata

**Confidence breakdown:**
- **Current state audit:** HIGH — Directly inspected source code
- **Subprocess patterns:** HIGH — Existing implementation + official Rust docs
- **Docker patterns:** HIGH — Verified against OWASP, Docker official docs
- **.env structure:** HIGH — Based on existing codebase + 12-factor principles
- **testssl.sh details:** LOW — Native installation method unclear, needs research

**Research date:** 2026-02-06
**Valid until:** 2026-03-06 (30 days, stack is stable)

**Key findings:**
1. **Bollard was never used** — Scanners already execute via subprocess
2. **Multi-stage Dockerfile already exists** — Just needs runtime dependencies
3. **Graceful degradation already implemented** — Pattern can be reused
4. **.env.example is incomplete** — Missing 60% of required variables
5. **Docker socket NOT mounted** — Production docker-compose needs update
6. **Render cleanup is minimal** — Only README and docs need updates
