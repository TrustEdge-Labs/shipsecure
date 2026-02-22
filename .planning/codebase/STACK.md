# Technology Stack

**Analysis Date:** 2026-02-21

## Languages

**Primary:**
- TypeScript 5.x - Frontend (React/Next.js) application
- Rust 1.88 - Backend API and security scanning orchestration
- SQL - Database queries and migrations

**Secondary:**
- Bash - E2E test scripts, deployment automation
- YAML - GitHub Actions CI/CD configuration

## Runtime

**Environment:**
- Node.js 20 / 22 LTS - Frontend development, Next.js server, E2E testing
- Rust 1.88 - Backend compilation and execution
- PostgreSQL 16 - Data persistence

**Package Manager:**
- npm (Node) - Lockfile: `frontend/package-lock.json` (present)
- Cargo (Rust) - Lockfile: `Cargo.lock` (present)

## Frameworks

**Frontend:**
- Next.js 16.1.6 - Full-stack React framework with App Router
- React 19.2.3 - UI components and state
- Tailwind CSS 4 - Utility-first CSS styling
- PostCSS 4 - CSS processing pipeline

**Backend:**
- Axum 0.8.8 - Async Rust web framework with tower middleware
- Tokio 1.x - Async runtime for concurrent I/O and task scheduling

**Testing:**
- Vitest 4.0.18 - Unit test runner (frontend)
- @testing-library/react 16.3.2 - React component testing utilities
- MSW 2.12.10 - Mock Service Worker for API mocking
- Playwright 1.58.2 - E2E browser testing (chromium)
- Happy DOM 20.6.1 - Lightweight DOM implementation for unit tests

**Build/Dev:**
- TypeScript 5.x - Static type checking
- ESLint 9 - Code linting
- Vite 4.x - Build tooling (via Vitest)
- Sharp 0.34.5 - Image optimization
- Tailwind CSS 4 - Compiled CSS framework

**Backend Build:**
- Cargo - Rust package manager and build system
- SQLx 0.8.6 - SQL query builder with compile-time checking
- Tokio - Async executor for background tasks

## Key Dependencies

**Critical (Frontend):**
- @clerk/nextjs 6.37.5 - Authentication and user management integration
- lucide-react 0.563.0 - Icon library
- marked 17.0.1 - Markdown parser for results rendering
- zod 4.3.6 - Schema validation library

**Critical (Backend):**
- axum-jwt-auth 0.6 - JWT authentication middleware for Clerk tokens
- sqlx 0.8.6 - Type-safe PostgreSQL client with migration support
- jsonwebtoken 10 - JWT encoding/decoding and verification
- reqwest 0.13.1 - HTTP client for external APIs (Resend, GitHub)
- tower-http 0.6.8 - HTTP middleware for CORS, tracing, timeout
- tracing 0.1 - Structured logging and observability
- tracing-subscriber 0.3 - Log formatting (JSON and plaintext)
- metrics 0.24 - Prometheus metrics collection
- metrics-exporter-prometheus 0.17 - Prometheus endpoint exporter

**Scanner Integration:**
- svix 1.x - Webhook signature verification for Clerk webhooks
- which 7 - Binary resolution for Nuclei and testssl.sh
- tempfile 3 - Temporary file management for scanner output
- uuid 1.x - UUID generation for request tracking
- chrono 0.4.43 - Timestamp handling and date formatting
- scraper 0.22 - HTML parsing for security header detection
- regex 1 - Pattern matching for finding extraction
- serde/serde_json 1.x - JSON serialization/deserialization
- base64 0.22 - Base64 encoding for data serialization
- dotenvy 0.15.7 - Environment variable loading

**Infrastructure:**
- lazy_static 1.4 - Static initialization for connection pools
- futures 0.3 - Async composition utilities
- tokio-util 0.7 - Tokio extensions
- http 1.0 - HTTP types

## Configuration

**Environment:**
- Configured via `.env` file (development) or environment variables (production)
- See `.env.example` for comprehensive list of required and optional settings
- Development: `frontend/.env.local`, `frontend/.env.test`, root `.env`
- Production: Docker environment variables in `docker-compose.prod.yml`

**Build:**
- Next.js: `frontend/next.config.ts` - Standalone output mode, experimental testProxy for E2E
- TypeScript: `frontend/tsconfig.json` - Path aliases `@/*` mapping to root
- Vitest: `frontend/vitest.config.ts` - Happy DOM environment, setupFiles, coverage thresholds
- Playwright: `frontend/playwright.config.ts` - Next.js testmode, port 3001 for E2E, chromium only
- ESLint: `frontend/eslint.config.mjs` - Next.js preset
- Rust: `Cargo.toml` with sqlx feature flags for PostgreSQL and JSON migrations

**Key Configuration Files:**
- `frontend/tsconfig.json` - TS compiler options, path resolution
- `frontend/vitest.config.ts` - Test environment setup (happy-dom), 80% coverage target
- `frontend/playwright.config.ts` - E2E test port (3001), fixtures, retry logic
- `Dockerfile` - Multi-stage Rust build, Nuclei + testssl.sh binary installation
- `frontend/Dockerfile` - Multi-stage Node.js build, standalone output

## Platform Requirements

**Development:**
- Node.js 20+ (LTS)
- Rust 1.88+
- PostgreSQL 16 (via Docker Compose)
- Docker & Docker Compose (recommended for database)
- Git (2.0+)

**Production:**
- Docker & Docker Compose
- PostgreSQL 16 (managed separately or containerized)
- Linux host (tested on Ubuntu/Debian)
- Minimum 2 CPUs, 2GB RAM for backend; 1 CPU, 1GB RAM for frontend (per docker-compose.prod.yml)
- HTTPS reverse proxy (Nginx or similar) in front of frontend/backend

**Deployment:**
- GitHub Actions (CI/CD)
- Docker images pushed to GHCR (GitHub Container Registry)
- SSH deployment to production host `shipsecure.ai`
- Systemd service management (via `deploy/shipsecure.service`)

---

*Stack analysis: 2026-02-21*
