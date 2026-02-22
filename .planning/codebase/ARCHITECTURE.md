# Architecture

**Analysis Date:** 2026-02-21

## Pattern Overview

**Overall:** Layered monorepo with full-stack web application separated into frontend (Next.js 16) and backend (Rust/Axum).

**Key Characteristics:**
- Frontend and backend communicating via REST API
- Async task orchestration with concurrent scanning capability
- JWT-based authentication via Clerk for protected endpoints
- Database-driven state management with PostgreSQL
- Multi-scanner aggregation pattern for security findings
- Graceful shutdown coordination across services

## Layers

**Frontend (Next.js/TypeScript):**
- Purpose: User-facing web application for submitting scans, viewing results, and managing domain verification
- Location: `/home/john/vault/projects/github.com/shipsecure/frontend/`
- Contains: React components, server actions (Zod validation), API client, pages (dashboard, scan results, domain verification)
- Depends on: Clerk SDK for auth, backend API, next/navigation for routing
- Used by: End users via browser

**Backend API (Rust/Axum):**
- Purpose: REST API endpoints for scan creation, result retrieval, quota management, domain verification, webhooks
- Location: `/home/john/vault/projects/github.com/shipsecure/src/api/`
- Contains: Route handlers, JWT auth middleware, error handling, CORS configuration
- Depends on: Database pool, scanner orchestrator, Clerk JWKS decoder, rate limiter
- Used by: Frontend client, webhooks from Clerk

**Orchestrator (Rust):**
- Purpose: Manages concurrent scan task execution with semaphore-based queueing and graceful shutdown coordination
- Location: `/home/john/vault/projects/github.com/shipsecure/src/orchestrator/worker_pool.rs`
- Contains: ScanOrchestrator struct, TaskTracker, CancellationToken, semaphore-based concurrency control
- Depends on: Database pool, scanner modules, task tracking primitives
- Used by: API endpoints creating scans, main.rs for lifecycle management

**Scanners (Rust):**
- Purpose: Security scanning implementations for various vulnerability categories
- Location: `/home/john/vault/projects/github.com/shipsecure/src/scanners/`
- Contains: security_headers, tls, js_secrets, exposed_files, detector (framework detection), vibecode, remediation
- Depends on: HTTP client (reqwest), HTML scraper, regex, DNS lookups
- Used by: Orchestrator during scan execution

**Database Layer (Rust):**
- Purpose: SQLx-based database access with migrations for scans, findings, domains
- Location: `/home/john/vault/projects/github.com/shipsecure/src/db/`
- Contains: Queries for scans, findings, domain verification
- Depends on: PostgreSQL pool, sqlx macros
- Used by: API endpoints, orchestrator, cleanup tasks

**Models (Rust):**
- Purpose: Shared data structures for Scan, Finding, Detection, Domain entities
- Location: `/home/john/vault/projects/github.com/shipsecure/src/models/`
- Contains: Serializable structs with database mappings via sqlx
- Depends on: serde, sqlx, chrono, uuid crates
- Used by: All layers (API, orchestrator, database, scanners)

## Data Flow

**Scan Creation Flow:**

1. Frontend `ScanForm` (client) → `submitScan` server action validates URL/email with Zod
2. Server action sends POST to Backend `/api/v1/scans` with optional Clerk JWT
3. API handler (`src/api/scans.rs::create_scan`) validates:
   - SSRF attack prevention via `src/ssrf/validator.rs`
   - Optional Clerk JWT extraction for authenticated tier
   - Domain verification gate (authenticated users must verify domain)
   - Rate limiting (free: 1/IP/24h, authenticated: 5/month)
4. Creates Scan record in database with `pending` status
5. Spins off background task via `orchestrator.spawn_scan_with_tier()`
6. Orchestrator acquires semaphore permit, runs scanners (detector, headers, TLS, files, secrets, vibecode)
7. Aggregates findings via `scanners::aggregator::deduplicate_findings()` and `compute_score()`
8. Updates Scan status to `completed`, saves findings to database
9. Frontend polls `/api/v1/scans/{id}` or views cached results via `/api/v1/results/{token}`

**Results Retrieval Flow:**

1. Authenticated user views `/dashboard` (protected via `proxy.ts`)
2. Fetches `/api/v1/users/me/scans` with Bearer token
3. Unauthenticated user views `/results?token=...`
4. Fetches `/api/v1/results/{token}` (public, token-protected)
5. Both return ScanResponse with findings, grade, and metadata

**Domain Verification Flow:**

1. User initiates `/api/v1/domains/verify-start` with Bearer token
2. Backend generates verification token and HTML meta tag
3. User adds meta tag to website
4. User confirms at `/api/v1/domains/verify-confirm`
5. Backend checks tag presence, marks domain verified in database
6. Subsequent scans on that domain by user bypass domain verification gate

**State Management:**

- **Scan Status:** `pending` → `in_progress` → `completed` or `failed` (state machine in orchestrator)
- **Database as Source of Truth:** All scan state persisted to PostgreSQL; frontend polls for updates
- **Finding Aggregation:** Deduplicates by title, keeps highest severity, combines scanner names
- **Grace Shutdown:** Orchestrator tracks in-flight scans, drains with timeout on SIGTERM/SIGINT

## Key Abstractions

**ScanOrchestrator:**
- Purpose: Manages concurrent scan execution with bounded parallelism
- Examples: `src/orchestrator/worker_pool.rs`
- Pattern: Semaphore-based worker pool with TaskTracker for lifecycle. Bounded concurrency via `MAX_CONCURRENT_SCANS` env var (default 5).

**AppState:**
- Purpose: Shared state container passed to all route handlers
- Examples: `src/api/scans.rs::AppState`
- Pattern: Axum `State<T>` pattern with database pool, orchestrator, JWKS decoder, metrics handle, shutdown token

**ApiError:**
- Purpose: Centralized HTTP error response mapping
- Examples: `src/api/errors.rs`
- Pattern: Enum-based error type implementing `IntoResponse` for Axum

**HealthCache:**
- Purpose: Caches health status (liveness, readiness) between requests
- Examples: `src/api/health.rs::HealthCache`
- Pattern: Mutex-protected state, separate from traced routes to avoid overhead

**Scanner Trait Pattern:**
- Purpose: Each scanner implements independent vulnerability scanning
- Examples: `src/scanners/security_headers.rs`, `src/scanners/tls.rs`, `src/scanners/js_secrets.rs`
- Pattern: Async functions returning `Vec<Finding>` or error, composed in orchestrator

**Rate Limiter:**
- Purpose: Enforce usage quotas per IP (anonymous) or user (authenticated)
- Examples: `src/rate_limit/` module
- Pattern: Database-backed rate limiting with per-minute and per-month windows

**SSRF Validator:**
- Purpose: Prevents Server-Side Request Forgery attacks on scan target
- Examples: `src/ssrf/validator.rs`
- Pattern: Validates domain against blocklist, checks DNS resolution

## Entry Points

**Backend Main:**
- Location: `/home/john/vault/projects/github.com/shipsecure/src/main.rs`
- Triggers: `cargo run` or Docker container startup
- Responsibilities: Env validation, logging setup, database migrations, JWKS initialization, router construction, graceful shutdown coordination

**Frontend Root Layout:**
- Location: `/home/john/vault/projects/github.com/shipsecure/frontend/app/layout.tsx`
- Triggers: Browser page load
- Responsibilities: Wraps entire app in ClerkProvider, Header/Footer layout, metadata, analytics script injection (Plausible)

**Frontend Proxy Middleware:**
- Location: `/home/john/vault/projects/github.com/shipsecure/frontend/proxy.ts`
- Triggers: Every request (Next.js 16 intercepts)
- Responsibilities: Clerk authentication, protects `/dashboard` and `/verify-domain` routes

**API Route: Create Scan:**
- Location: `src/api/scans.rs::create_scan` → route `/api/v1/scans`
- Triggers: POST from frontend or external client
- Responsibilities: Input validation, SSRF check, rate limiting, domain verification gate, orchestrator spawn

**API Route: Get Scan:**
- Location: `src/api/scans.rs::get_scan` → route `/api/v1/scans/{id}`
- Triggers: GET from frontend (polling)
- Responsibilities: Retrieves scan status and findings by ID

**API Route: Get Results:**
- Location: `src/api/results.rs::get_results_by_token` → route `/api/v1/results/{token}`
- Triggers: GET from unauthenticated users with public token
- Responsibilities: Returns full scan results if scan completed and not expired

**Frontend Dashboard Page:**
- Location: `/home/john/vault/projects/github.com/shipsecure/frontend/app/dashboard/page.tsx`
- Triggers: Authenticated user navigates to `/dashboard`
- Responsibilities: Server-side data fetching (scans, quota, domains), renders history table and domain management UI

**Frontend Domain Verification Page:**
- Location: `/home/john/vault/projects/github.com/shipsecure/frontend/app/verify-domain/page.tsx`
- Triggers: Authenticated user navigates to `/verify-domain`
- Responsibilities: Initiates verification flow, displays meta tag, confirms tag presence, displays verified domains

## Error Handling

**Strategy:** Centralized via `ApiError` enum with automatic HTTP response conversion. Database errors logged but returned as generic 500. Validation errors return 400 with details.

**Patterns:**

- **Validation Errors:** Return 400 with field-level error messages. Zod schema validation in frontend server actions; API-side SSRF/email/URL validation.
- **Authentication Errors:** Return 401 if Clerk JWT missing/invalid for protected endpoints. Return 403 if domain not verified but required.
- **Rate Limit Errors:** Return 429 with retry-after header.
- **Database Errors:** Logged with full context (request ID); returned as generic 500 to client.
- **Orchestrator Errors:** Scan status updated to `failed` with error_message persisted to database.
- **Scanner Failures:** Individual scanner errors collected; if all fail, entire scan marked failed; partial results saved.

## Cross-Cutting Concerns

**Logging:** Structured JSON or text via `tracing` and `tracing-subscriber`. Request ID injected into all HTTP logs via middleware. Configurable via `RUST_LOG` and `LOG_FORMAT` env vars.

**Validation:**
- Frontend: Zod schemas for form inputs (ScanForm, domain verification)
- Backend: Manual validation in handlers (URL, email, domain format); SSRF validator; rate limiter checks

**Authentication:**
- Frontend: Clerk SDK via `useAuth()`, `getToken()` hooks; session token in Authorization header
- Backend: Clerk JWT validation via `axum-jwt-auth` with JWKS decoder; validates RS256 signature
- Protected Routes: Frontend proxy.ts gates `/dashboard` and `/verify-domain`; Backend checks Bearer token for authenticated endpoints

**Metrics:** Prometheus metrics recorded via `metrics` crate (active_scans, scan_queue_depth, scan_duration, http_request_duration). Exposed at `/metrics` endpoint.

