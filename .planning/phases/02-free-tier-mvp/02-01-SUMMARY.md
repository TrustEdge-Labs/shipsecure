---
phase: 02-free-tier-mvp
plan: 01
subsystem: database, frontend
tags: [postgresql, sqlx, nextjs, typescript, tailwind, app-router]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: Base Rust backend, PostgreSQL database, scans table schema
provides:
  - Results token system for public access to scan results without authentication
  - Scan stage tracking (headers, TLS, files, secrets) for progress indicators
  - Next.js 15 frontend scaffold with App Router and TypeScript
  - API client wrapper for Rust backend communication
  - TypeScript types mirroring Rust backend models
affects: [02-02, 02-03, 02-04, 02-05, 02-06, backend-api, frontend-pages]

# Tech tracking
tech-stack:
  added: [next@16.1.6, react@19, zod@3, marked@15, tailwindcss@3, typescript@5]
  patterns: [App Router, server components, TypeScript strict mode, Tailwind utility-first]

key-files:
  created:
    - migrations/20260205000001_add_results_token_and_stages.sql
    - frontend/lib/api.ts
    - frontend/lib/types.ts
    - frontend/app/layout.tsx
    - frontend/app/page.tsx
    - frontend/next.config.ts
  modified:
    - src/models/scan.rs
    - src/db/scans.rs

key-decisions:
  - "Use base64url-encoded 32-byte tokens for results URLs (43 chars)"
  - "3-day expiry for free tier results access"
  - "Stage tracking as individual boolean columns (not JSONB) for simple querying"
  - "Next.js standalone output mode for Docker deployment"
  - "Inter font for consistent brand typography"

patterns-established:
  - "Scan model: All new columns included in SELECT/RETURNING queries for consistency"
  - "Frontend API client: Environment variable NEXT_PUBLIC_BACKEND_URL for backend config"
  - "TypeScript types: Mirror Rust backend models exactly for type safety"
  - "Frontend structure: lib/ for shared utilities, no src/ directory"

# Metrics
duration: 3min
completed: 2026-02-05
---

# Phase 02 Plan 01: Database Schema and Frontend Scaffold Summary

**Results token system with expiry tracking, scan stage booleans, and Next.js 15 frontend with TypeScript API client**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-05T14:01:15Z
- **Completed:** 2026-02-05T14:04:29Z
- **Tasks:** 2
- **Files modified:** 25

## Accomplishments
- Database schema extended with results_token, expires_at, and 4 stage tracking columns
- Rust Scan model and all DB queries updated to include new fields
- 4 new DB functions for token management and scan counting
- Next.js 15 frontend scaffolded with App Router, Tailwind, and TypeScript
- Complete TypeScript type system mirroring Rust backend models
- API client wrapper with environment-based backend URL configuration

## Task Commits

Each task was committed atomically:

1. **Task 1: Database migration for results tokens and scan stages** - `5513584` (feat)
2. **Task 2: Scaffold Next.js 15 frontend with App Router and API client** - `b6fc33a` (feat)

## Files Created/Modified

**Backend (Rust):**
- `migrations/20260205000001_add_results_token_and_stages.sql` - Adds results_token, expires_at, stage tracking columns
- `src/models/scan.rs` - Updated Scan struct with results_token, expires_at, and 4 stage fields
- `src/db/scans.rs` - Added get_scan_by_token, update_scan_stage, set_results_token, count_completed_scans functions

**Frontend (Next.js):**
- `frontend/package.json` - Next.js 15 app with zod, marked dependencies
- `frontend/tsconfig.json` - TypeScript strict mode configuration
- `frontend/next.config.ts` - Standalone output for Docker
- `frontend/app/layout.tsx` - Root layout with Inter font and TrustEdge metadata
- `frontend/app/page.tsx` - Placeholder landing page
- `frontend/app/globals.css` - Tailwind CSS imports
- `frontend/lib/types.ts` - TypeScript interfaces (Scan, Finding, ScanResponse, CreateScanResponse)
- `frontend/lib/api.ts` - API client wrapper (createScan, getScan, getScanByToken, getScanCount)
- `frontend/.env.local` - Development environment configuration

## Decisions Made

1. **Results token format:** 64-char VARCHAR to accommodate base64url-encoded 32 bytes (43 chars) with safety margin
2. **Stage tracking as columns:** Individual boolean columns (stage_headers, stage_tls, stage_files, stage_secrets) instead of JSONB for simpler SQL queries and indexing
3. **3-day expiry for free tier:** expires_at timestamp set 3 days after scan completion for free tier results access
4. **Query consistency:** All existing queries updated to include new columns in SELECT/RETURNING to prevent field mismatch errors
5. **Partial unique index:** `WHERE results_token IS NOT NULL` to avoid indexing NULL values and improve performance
6. **Inter font:** Replaced default Geist fonts with Inter for cleaner, more professional typography
7. **Standalone Next.js output:** Configured for Docker deployment compatibility

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - all tasks completed smoothly with expected compilation and build results.

## User Setup Required

None - no external service configuration required. The `.env.local` file was created for local development with default backend URL.

## Next Phase Readiness

**Ready for Phase 2 development:**
- Database schema supports results tokens, expiry, and stage tracking
- Frontend scaffold is production-ready with TypeScript types and API client
- All 4 new DB functions available: token lookup, stage updates, token setting, scan counting
- Next.js builds successfully with App Router and Tailwind configured

**Available for subsequent plans:**
- Plan 02-02: Landing page can use getScanCount() for social proof
- Plan 02-03: Progress page can use stage_* fields for progress indicator
- Plan 02-04: Results page can use get_scan_by_token() for public access
- Plan 02-05: Expiry system can use expires_at for access control

**No blockers or concerns.**

---
*Phase: 02-free-tier-mvp*
*Completed: 2026-02-05*
