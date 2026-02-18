# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-17)

**Core value:** Catch security flaws in vibe-coded apps before they become breaches, with remediation guidance anyone can follow.
**Current focus:** v1.6 Auth & Tiered Access — Phase 29: Auth Foundation

## Current Position

Phase: 29 of 35 (Auth Foundation)
Plan: 0 of 3 in current phase
Status: Ready to plan
Last activity: 2026-02-17 — v1.6 roadmap created, 7 phases (29-35), 33 requirements mapped

Progress: [████████░░░░░░░░░░░░] 42% (28/66 plans)

## Performance Metrics

**Velocity:**
- Total plans completed: 75
- Average duration: ~30 min
- Total execution time: ~37 hours

**By Milestone:**

| Milestone | Phases | Plans | Days |
|-----------|--------|-------|------|
| v1.0 MVP | 1-4 | 23 | 3 |
| v1.1 Deployment | 5-7 | 10 | 3 |
| v1.2 Launch | 8-12 | 10 | 2 |
| v1.3 Brand | 13-18 | 10 | 7 |
| v1.4 Observability | 19-24 | 11 | 1 |
| v1.5 Testing | 25-28 | 11 | 2 |

## Accumulated Context

### Decisions

All decisions logged in PROJECT.md Key Decisions table.

**v1.6 key decisions (pending implementation):**
- Clerk over Auth.js/Supabase Auth — managed service, fastest path, pre-built Next.js components
- clerk-rs explicitly rejected — community-maintained, stale (v0.4.1, 8+ months), security risk for JWT path
- jsonwebtoken + axum-jwt-auth for local JWKS-based JWT verification — no per-request Clerk API calls
- Remove $49 Stripe audit — replaced by Developer tier; paid_audits FK must change to SET NULL before retention runs
- Server-side results gating — never frontend-only; curl bypass is a real threat (OWASP A01)
- Meta tag verification only for v1.6 — DNS TXT too opaque for vibe-coder target users
- Rate limit: 5 scans/month for Developer tier (monthly window, not daily) — reconciles FEATURES.md vs ARCHITECTURE.md inconsistency
- Phase 29 must land CORS Authorization header fix before any dashboard routes exist
- CVE-2025-29927 Nginx fix must land in Phase 29 — not as an afterthought

### Pending Todos

None.

### Blockers/Concerns

- Phase 29: Verify axum-jwt-auth current version on crates.io supports async JWKS refresh before writing Cargo.toml
- Phase 30: Confirm genpdf has no usage outside Stripe PDF path before removing (grep needed at plan time)
- Phase 32: Shared-hosting TLD blocklist scope is a policy call — confirm list before implementation (github.io, vercel.app, netlify.app, pages.dev confirmed; others may need addition)
- Phase 33: Rate limit window confirmed as 5/month Developer — ARCHITECTURE.md "10/day" is superseded

## Session Continuity

Last session: 2026-02-17
Stopped at: v1.6 roadmap created — Phase 29 ready to plan
Resume file: None
