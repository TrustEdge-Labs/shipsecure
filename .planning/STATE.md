# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-17)

**Core value:** Catch security flaws in vibe-coded apps before they become breaches, with remediation guidance anyone can follow.
**Current focus:** v1.6 Auth & Tiered Access — Phase 31: Results Gating complete, Phase 32 next

## Current Position

Phase: 32 of 35 (Domain Verification)
Plan: 0 of 2 in current phase (Phase 31 complete)
Status: Phase 31 complete — Phase 32 (domain verification) is next
Last activity: 2026-02-18 — Phase 31 plan 02 complete (Frontend AuthGate: lock overlay for gated findings, Clerk SignUp CTA, JWT forwarding from Server Component)

Progress: [█████████░░░░░░░░░░░] 50% (33/66 plans)

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

**Phase 29 Plan 02 decisions:**
- proxy.ts (not middleware.ts) for Next.js 16.1.6 middleware convention
- Valid-format placeholder publishable key so npm run build succeeds without real Clerk keys — Clerk validates key format during static page generation
- Force-add .env.example despite .gitignore .env* rule — template file with no secrets
- UserButton uses Clerk defaults only — no custom menu items or tier badge

**Phase 29 Plan 01 decisions:**
- jsonwebtoken = "10" not "9" — axum-jwt-auth 0.6.3 depends on jsonwebtoken 10.x; using 9 would create incompatible types
- RemoteJwksDecoder has no generic parameter — generics appear at JwtDecoder<T> impl level; use Decoder<ClerkClaims> type annotation on the Arc
- Axum HeaderMap passes directly to svix::Webhook::verify() — both use http 1.x, no conversion loop needed
- sqlx::query() non-macro for users INSERT — avoids compile-time DB connection requirement

**Phase 30 Plan 01 decisions:**
- Keep 'paid' in tier CHECK — existing paid_audits rows reference scans with tier='paid'; removing requires data migration first
- Keep base64 crate — still used in worker_pool.rs for results token generation (URL_SAFE_NO_PAD.encode); not Stripe-specific
- Keep svix crate — used in handle_clerk_webhook for Clerk signature verification; never Stripe-specific
- Remove hex crate — only usage was Stripe HMAC hex encoding; safe to remove with webhook handler
- tier match in run_scanners simplified to tuple assignment after paid arm removal

**Phase 31 Plan 02 decisions:**
- AuthGate receives pre-computed gated bool from server-rendered finding data — no client-side JWT check needed
- generateMetadata also forwards session token — consistent auth posture with main page handler
- Spacer div in AuthGate lock overlay maintains accordion height for visual continuity

**Phase 31 Plan 01 decisions:**
- Gate high/critical findings for ALL scans regardless of tier — tier is irrelevant; gating is based on severity + caller identity only
- None == None returns owner_verified: false — anonymous scans (clerk_user_id IS NULL) are always gated for anonymous callers
- download_results_markdown also applies gating — consistent OWASP A01; curl to /download cannot bypass gating
- Optional auth pattern: extract_optional_clerk_user() helper calls state.jwt_decoder.decode() directly; do NOT use Claims<T> extractor for optional auth (it rejects all anonymous requests with 401)

### Pending Todos

None.

### Blockers/Concerns

- Phase 29: axum-jwt-auth 0.6.3 confirmed on crates.io (resolved — implemented successfully)
- Phase 32: Shared-hosting TLD blocklist scope is a policy call — confirm list before implementation (github.io, vercel.app, netlify.app, pages.dev confirmed; others may need addition)
- Phase 33: Rate limit window confirmed as 5/month Developer — ARCHITECTURE.md "10/day" is superseded

## Session Continuity

Last session: 2026-02-18
Stopped at: Phase 32 context gathered
Resume file: .planning/phases/32-domain-verification/32-CONTEXT.md
