# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-17)

**Core value:** Catch security flaws in vibe-coded apps before they become breaches, with remediation guidance anyone can follow.
**Current focus:** v1.6 Auth & Tiered Access — Phase 29: Auth Foundation

## Current Position

Phase: 29 of 35 (Auth Foundation)
Plan: 1 of 3 in current phase (29-01 complete; 29-02 and 29-03 remain)
Status: In progress
Last activity: 2026-02-18 — Phase 29 plan 01 complete (backend auth: CORS fix, JWKS decoder, Clerk webhook, users migration)

Progress: [████████░░░░░░░░░░░░] 43% (29/66 plans)

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

**Phase 29 Plan 03 decisions:**
- Strip x-middleware-subrequest in BOTH /api/ and / Nginx location blocks — missing either block leaves CVE-2025-29927 exploitable since requests traverse Next.js middleware via both paths
- CLERK_JWKS_URL unconditional in env template (required for JWT verification); CLERK_SECRET_KEY and CLERK_WEBHOOK_SIGNING_SECRET conditional (only needed for webhooks)

**Phase 29 Plan 01 decisions:**
- jsonwebtoken = "10" not "9" — axum-jwt-auth 0.6.3 depends on jsonwebtoken 10.x; using 9 would create incompatible types
- RemoteJwksDecoder has no generic parameter — generics appear at JwtDecoder<T> impl level; use Decoder<ClerkClaims> type annotation on the Arc
- Axum HeaderMap passes directly to svix::Webhook::verify() — both use http 1.x, no conversion loop needed
- sqlx::query() non-macro for users INSERT — avoids compile-time DB connection requirement

### Pending Todos

None.

### Blockers/Concerns

- Phase 29: axum-jwt-auth 0.6.3 confirmed on crates.io (resolved — implemented successfully)
- Phase 30: Confirm genpdf has no usage outside Stripe PDF path before removing (grep needed at plan time)
- Phase 32: Shared-hosting TLD blocklist scope is a policy call — confirm list before implementation (github.io, vercel.app, netlify.app, pages.dev confirmed; others may need addition)
- Phase 33: Rate limit window confirmed as 5/month Developer — ARCHITECTURE.md "10/day" is superseded

## Session Continuity

Last session: 2026-02-18
Stopped at: Completed 29-auth-foundation/29-01-PLAN.md
Resume file: None
