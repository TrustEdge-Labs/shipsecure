# Codebase Structure

**Analysis Date:** 2026-02-21

## Directory Layout

```
/home/john/vault/projects/github.com/shipsecure/
├── Cargo.toml                  # Rust workspace root, backend dependencies
├── Cargo.lock                  # Locked Rust dependencies
├── src/                        # Backend (Rust) source code
├── frontend/                   # Frontend (Next.js 16) source code
├── migrations/                 # PostgreSQL database migrations (sqlx)
├── .planning/                  # GSD orchestrator planning documents
├── .github/workflows/          # CI/CD GitHub Actions
├── infrastructure/             # Ansible playbooks for deployment
├── deploy/                     # Deployment scripts
├── templates/                  # Security scanning templates (Nuclei, etc.)
├── docker-compose.yml          # Local development compose
├── docker-compose.prod.yml     # Production compose
├── Dockerfile                  # Single Dockerfile for Rust backend
├── .env                        # Local development config (not committed)
├── .env.example                # Environment template
└── README.md                   # Project documentation
```

## Directory Purposes

**Backend Source (`src/`):**
- Purpose: Rust implementation of security scanning API and orchestration
- Contains: API handlers, database access, scanner modules, orchestrator, models
- Structure:
  - `main.rs` — Application entry point, server setup, routes, graceful shutdown
  - `lib.rs` — Module declarations (api, db, models, orchestrator, scanners, etc.)

**Backend Modules:**

**`src/api/`:**
- Purpose: HTTP API endpoint handlers and middleware
- Key files:
  - `mod.rs` — Module exports
  - `scans.rs` — POST/GET `/api/v1/scans`, scan creation and status retrieval, AppState definition
  - `results.rs` — GET `/api/v1/results/{token}`, public result access by token
  - `domains.rs` — POST `/api/v1/domains/*`, domain verification workflow
  - `auth.rs` — Clerk JWT claims struct and validation
  - `health.rs` — Liveness/readiness health checks with cached status
  - `metrics.rs` — Prometheus metrics endpoint
  - `stats.rs` — Public statistics (total scan count)
  - `users.rs` — GET `/api/v1/users/me/scans`, authenticated user scan history
  - `webhooks.rs` — POST `/api/v1/webhooks/clerk`, handles Clerk user deletion
  - `errors.rs` — ApiError enum with HTTP response mapping

**`src/db/`:**
- Purpose: Database access layer with sqlx
- Key files:
  - `mod.rs` — Module exports
  - `scans.rs` — Queries for creating, updating, fetching scans
  - `findings.rs` — Queries for inserting and fetching findings
  - `domains.rs` — Queries for domain verification state

**`src/models/`:**
- Purpose: Shared data structures with serialization and database mapping
- Key files:
  - `mod.rs` — Module exports
  - `scan.rs` — Scan struct, ScanStatus enum (pending/in_progress/completed/failed)
  - `finding.rs` — Finding struct, Severity enum (critical/high/medium/low)
  - `detection.rs` — DetectionResult, Framework, Platform for framework detection
  - `domain.rs` — VerifiedDomain struct for domain verification state

**`src/scanners/`:**
- Purpose: Security scanning implementations
- Key files:
  - `mod.rs` — Module exports
  - `detector.rs` — Framework/platform detection (Vibe-Code, React, Next.js, etc.)
  - `security_headers.rs` — HTTP security header checks (CSP, X-Frame-Options, etc.)
  - `tls.rs` — TLS certificate validation, cipher suite checks
  - `js_secrets.rs` — JavaScript secret scanning (API keys, tokens in source)
  - `exposed_files.rs` — Common exposed files (.git, .env, etc.)
  - `container.rs` — Container/Dockerfile detection
  - `vibecode.rs` — Vibe-Code specific vulnerability checks
  - `aggregator.rs` — Finding deduplication, score computation (A+/A/B/C/D/F grading)
  - `remediation.rs` — Remediation suggestions for findings

**`src/orchestrator/`:**
- Purpose: Manages concurrent scan execution lifecycle
- Key files:
  - `mod.rs` — Module exports
  - `worker_pool.rs` — ScanOrchestrator struct, semaphore-based concurrency, scanner execution loop

**`src/rate_limit/`:**
- Purpose: Rate limiting for scans
- Key file: Enforces usage quotas per IP (free tier) and user (authenticated)

**`src/ssrf/`:**
- Purpose: SSRF (Server-Side Request Forgery) prevention
- Key file: `validator.rs` — validates scan target, prevents attacks on internal addresses

**`src/email/`:**
- Purpose: Email sending utilities
- Contains: Email notification templates and sending logic

**`src/metrics/`:**
- Purpose: Prometheus metrics tracking
- Contains: Gauge definitions, middleware for HTTP metrics

**`src/cleanup/`:**
- Purpose: Periodic cleanup tasks
- Contains: Scheduled deletion of expired scan results

**Frontend Source (`frontend/`):**
- Purpose: Next.js 16 web application for users
- Contains: React components, pages, server actions, API client, tests, styles

**`frontend/app/`:**
- Purpose: Next.js App Router pages and layouts
- Key files:
  - `layout.tsx` — Root layout with ClerkProvider, Header, Footer, Plausible analytics
  - `page.tsx` — Homepage with scan form (public)
  - `error.tsx`, `global-error.tsx` — Error boundary components
  - `loading.tsx` — Loading fallback (deprecated in favor of Suspense)
  - `robots.ts`, `sitemap.ts` — SEO metadata generation

**`frontend/app/dashboard/`:**
- Purpose: Authenticated user dashboard
- Key files:
  - `page.tsx` — Displays scan history table, quota usage, verified domains

**`frontend/app/scan/`:**
- Purpose: Scan progress and results pages
- Key files:
  - `[id]/page.tsx` — Scan status polling and results display

**`frontend/app/verify-domain/`:**
- Purpose: Domain ownership verification workflow
- Key files:
  - `page.tsx` — Displays verification form, meta tag, and verified domain list

**`frontend/app/actions/`:**
- Purpose: Next.js server actions for form submission
- Key files:
  - `scan.ts` — submitScan: validates form with Zod, calls backend API, returns scan ID

**`frontend/app/sign-in/` and `frontend/app/sign-up/`:**
- Purpose: Clerk authentication pages
- Delegated to Clerk's hosted UI via redirects

**`frontend/app/privacy/`, `frontend/app/terms/`:**
- Purpose: Legal pages
- Contains: Static markdown-rendered content

**`frontend/components/`:**
- Purpose: Reusable React components
- Key files:
  - `header.tsx` — Navigation bar with logo, sign-in/up buttons, user menu
  - `footer.tsx` — Footer with links and copyright
  - `scan-form.tsx` — Input form for new scans (uses submitScan action)
  - `results-dashboard.tsx` — Findings display with severity grouping
  - `scan-history-table.tsx` — Paginated scan history for authenticated users
  - `finding-accordion.tsx` — Expandable finding details with remediation
  - `grade-summary.tsx` — Security grade (A+/F) display with statistics
  - `domain-badge.tsx` — Verified domain display with management options
  - `auth-gate.tsx` — Conditional rendering for signed-in/out users
  - `progress-checklist.tsx` — Scan stage progress indicator
  - `meta-tag-snippet.tsx` — Domain verification meta tag display

**`frontend/lib/`:**
- Purpose: Shared utilities and types
- Key files:
  - `api.ts` — Fetch-based API client functions (createScan, getScan, getScanByToken, verifyStart, verifyConfirm, verifyCheck, getScanCount)
  - `types.ts` — TypeScript interfaces for API responses and domain models (Scan, Finding, ScanResponse, VerifiedDomain, etc.)

**`frontend/__tests__/`:**
- Purpose: Unit and integration tests
- Structure:
  - `helpers/test-utils.tsx` — renderWithProviders() wrapper for provider-wrapped component tests
  - `helpers/msw/server.ts` — MSW (Mock Service Worker) setup for API mocking
  - `components/` — Unit tests for React components
  - `integration/` — Integration tests combining components and logic

**`frontend/e2e/`:**
- Purpose: End-to-end tests with Playwright
- Key files:
  - `fixtures/` — Custom Playwright fixtures for authentication, test data setup
  - `helpers/` — Test utilities and navigation helpers

**`frontend/public/`:**
- Purpose: Static assets (favicon, Open Graph images, etc.)
- Contains: Images served at root path

**`frontend/scripts/`:**
- Purpose: Build and development scripts
- Contains: Font generation, test setup scripts

**Migrations (`migrations/`):**
- Purpose: Database schema and data migrations
- Pattern: SQLx migrations in `.sql` files, ordered by timestamp
- Key tables: scans, findings, domains, rate_limit_buckets

**CI/CD (`.github/workflows/`):**
- Purpose: GitHub Actions automation
- Key workflows:
  - `ci.yml` — Runs tests on every push/PR
  - `build-push.yml` — Builds Docker images and pushes to ghcr.io

## Key File Locations

**Entry Points:**
- Backend: `src/main.rs` — Server startup, listening on PORT env var
- Frontend: `frontend/app/page.tsx` — Public homepage
- Frontend Auth: `frontend/proxy.ts` — Clerk middleware

**Configuration:**
- Backend environment: `.env.example` (copy to `.env`)
- Frontend config: `frontend/next.config.ts`, `frontend/tsconfig.json`
- Database: `migrations/` (sqlx-managed)
- Docker: `Dockerfile`, `docker-compose.yml`

**Core Logic:**
- Scan creation: `src/api/scans.rs::create_scan`, `frontend/app/actions/scan.ts::submitScan`
- Scan execution: `src/orchestrator/worker_pool.rs::spawn_scan_with_tier`
- Scanner implementations: `src/scanners/*.rs`
- Frontend form: `frontend/components/scan-form.tsx`
- Dashboard: `frontend/app/dashboard/page.tsx`

**Testing:**
- Backend: Tests in `src/**/*.rs` (cfg test modules)
- Frontend unit: `frontend/__tests__/components/`
- Frontend integration: `frontend/__tests__/integration/`
- Frontend E2E: `frontend/e2e/`
- Test setup: `frontend/vitest.setup.ts` (global mocks), `frontend/__tests__/helpers/test-utils.tsx`

## Naming Conventions

**Rust Files:**
- Modules: lowercase, snake_case (e.g., `security_headers.rs`, `worker_pool.rs`)
- Directory structure mirrors module hierarchy (e.g., `src/scanners/security_headers.rs`)

**Rust Types/Functions:**
- Structs: PascalCase (e.g., `ScanOrchestrator`, `ApiError`, `CreateScanRequest`)
- Functions: snake_case (e.g., `create_scan`, `is_domain_verified`, `spawn_scan_with_tier`)
- Enums: PascalCase (e.g., `ScanStatus`, `Severity`, `OrchestratorError`)
- Constants: UPPER_SNAKE_CASE (in code)

**TypeScript/JavaScript Files:**
- Components: PascalCase in `components/` (e.g., `ScanForm.tsx`, `ResultsDashboard.tsx`)
- Utilities: camelCase in `lib/` (e.g., `api.ts`, `types.ts`)
- Pages: lowercase slug-based (e.g., `page.tsx`, `layout.tsx`)
- Server actions: camelCase file (e.g., `scan.ts`)
- Functions in server actions: camelCase (e.g., `submitScan`)

**Database:**
- Tables: snake_case (e.g., `scans`, `findings`, `verified_domains`, `rate_limit_buckets`)
- Columns: snake_case (e.g., `target_url`, `scanner_name`, `created_at`)
- Enum types: snake_case (e.g., `scan_status`)

**API Endpoints:**
- Path format: `/api/v1/{resource}/{action}` (e.g., `/api/v1/scans`, `/api/v1/domains/verify-start`)
- HTTP methods: POST for mutations, GET for queries

**Routes/URLs:**
- Frontend pages: kebab-case (e.g., `/dashboard`, `/scan/[id]`, `/verify-domain`, `/sign-in`)
- Catch-all segments: `[param]` or `[[...optional]]`

## Where to Add New Code

**New Backend Endpoint:**
1. Define request/response types in appropriate `src/models/*.rs` file or inline
2. Add handler function in `src/api/{resource}.rs` (or create new file if category doesn't exist)
3. Register route in `src/main.rs` in the router builder (e.g., `.route("/api/v1/endpoint", post(handler))`)
4. Add tests as module at bottom of handler file (`#[cfg(test)] mod tests`)

**New Frontend Page:**
1. Create directory under `frontend/app/` with `page.tsx` (e.g., `app/new-feature/page.tsx`)
2. For protected pages, add path to `createRouteMatcher` in `frontend/proxy.ts`
3. Export React component as default export
4. Import components from `frontend/components/` as needed

**New Frontend Component:**
1. Create file as `PascalCase.tsx` in `frontend/components/`
2. Mark as `'use client'` if it needs interactivity
3. Export as named export and default export
4. Add `.test.tsx` file alongside with tests using `renderWithProviders`

**New Scanner:**
1. Create `src/scanners/{name}.rs` with async function returning `Vec<Finding>`
2. Export public function from `src/scanners/mod.rs`
3. Add to scanner execution loop in `src/orchestrator/worker_pool.rs`
4. Define findings with appropriate Severity and remediation

**New Database Operation:**
1. Add SQLx query function in `src/db/{resource}.rs` (or create new file)
2. Use compile-time checked queries (#[sqlx::query_as!])
3. Return model types from `src/models/`
4. Export from `src/db/mod.rs`
5. Create migration in `migrations/` with timestamp prefix (e.g., `20260221_add_feature.sql`)

**New Server Action (Frontend):**
1. Create file in `frontend/app/actions/{feature}.ts` with `'use server'` directive
2. Define Zod schema for input validation
3. Define return type interface
4. Export async action function
5. Call from client component via `useActionState` hook

## Special Directories

**`migrations/`:**
- Purpose: PostgreSQL schema evolution
- Generated: No (manually written)
- Committed: Yes
- Format: SQLx-compatible `.sql` files with numbered prefixes
- Usage: `sqlx migrate run` (in `src/main.rs` at startup)

**`.planning/`:**
- Purpose: GSD orchestrator planning documents
- Generated: Yes (by Claude Code via `/gsd` commands)
- Committed: Yes
- Contents: Architecture, structure, testing, conventions, concerns analyses

**`.github/workflows/`:**
- Purpose: CI/CD automation
- Generated: No (manually written)
- Committed: Yes
- Usage: Triggered on push/PR events

**`infrastructure/`:**
- Purpose: Infrastructure-as-Code (Ansible)
- Generated: No (manually written)
- Committed: Yes
- Contents: Playbooks for server setup, updates, monitoring

**`deploy/`:**
- Purpose: Deployment scripts
- Generated: No (manually written)
- Committed: Yes
- Contents: Docker compose production config, deployment shell scripts

**`frontend/public/`:**
- Purpose: Static assets served at root
- Generated: Partially (some images built from templates)
- Committed: Yes (final images committed, templates may be excluded)

**`frontend/.next/`:**
- Purpose: Next.js build output and cache
- Generated: Yes
- Committed: No (in .gitignore)

**`frontend/node_modules/`:**
- Purpose: npm dependencies
- Generated: Yes
- Committed: No (in .gitignore)

**`src/target/`:**
- Purpose: Cargo build output
- Generated: Yes
- Committed: No (in .gitignore)

**`frontend/coverage/`:**
- Purpose: Code coverage reports from Vitest
- Generated: Yes
- Committed: No (in .gitignore)

**`frontend/e2e/` and `frontend/test-results/`:**
- Purpose: Playwright E2E test files and reports
- Generated: Partially (reports generated on test runs)
- Committed: Test files Yes, reports No

