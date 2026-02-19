---
phase: 29-auth-foundation
plan: "03"
subsystem: infra
tags: [nginx, ansible, clerk, cve, security, jinja2]

# Dependency graph
requires:
  - phase: 29-auth-foundation/29-01
    provides: Clerk account established, JWKS URL known
provides:
  - CVE-2025-29927 mitigation — x-middleware-subrequest header stripped in Nginx
  - Production env template with all Clerk backend variables (JWKS URL, secret key, webhook secret)
  - Production env template with all Clerk frontend variables (publishable key, redirect URLs)
affects:
  - 29-auth-foundation (Nginx config referenced by all traffic routing)
  - deployment (Ansible playbook must supply clerk_jwks_url, clerk_secret_key, clerk_webhook_signing_secret, clerk_publishable_key)
  - 30-stripe-migration (env template used for all production deployments)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Nginx header strip: proxy_set_header header-name \"\" overwrites/strips incoming header before upstream"
    - "Jinja2 conditional blocks: {% if var %} ... {% endif %} for optional Ansible vault variables"
    - "CVE mitigation at infrastructure layer — defense-in-depth above application layer"

key-files:
  created: []
  modified:
    - infrastructure/templates/shipsecure.nginx.conf.j2
    - infrastructure/templates/env.production.j2

key-decisions:
  - "Strip x-middleware-subrequest in both /api/ and / location blocks — missing either block leaves CVE-2025-29927 exploitable since requests traverse Next.js middleware via both paths"
  - "CLERK_JWKS_URL is unconditional (required for backend JWT verification to work at all); CLERK_SECRET_KEY and CLERK_WEBHOOK_SIGNING_SECRET are conditional (optional if webhooks not used)"

patterns-established:
  - "CVE mitigations placed at infrastructure layer with comment referencing CVE number for auditability"
  - "Ansible vault variables documented in template comments — operator knows what to configure"

requirements-completed:
  - INFR-02

# Metrics
duration: 2min
completed: 2026-02-18
---

# Phase 29 Plan 03: Nginx CVE-2025-29927 Mitigation and Clerk Env Wiring Summary

**Nginx strips CVE-2025-29927 x-middleware-subrequest bypass header in both /api/ and / location blocks, and production Jinja2 template wires all Clerk backend and frontend environment variables**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-18T01:58:27Z
- **Completed:** 2026-02-18T02:00:30Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- CVE-2025-29927 mitigated at infrastructure layer — Nginx strips `x-middleware-subrequest` header before forwarding to upstream in both `/api/` and `/` location blocks, preventing attackers from bypassing Next.js proxy.ts auth middleware
- Production env template now includes all Clerk backend variables: CLERK_JWKS_URL (unconditional), CLERK_SECRET_KEY (conditional), CLERK_WEBHOOK_SIGNING_SECRET (conditional)
- Production env template now includes all Clerk frontend variables: NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY, NEXT_PUBLIC_CLERK_SIGN_IN_URL, NEXT_PUBLIC_CLERK_SIGN_UP_URL, NEXT_PUBLIC_CLERK_AFTER_SIGN_IN_URL, NEXT_PUBLIC_CLERK_AFTER_SIGN_UP_URL

## Task Commits

Each task was committed atomically:

1. **Task 1: Nginx CVE-2025-29927 mitigation** - `7b3ed72` (fix)
2. **Task 2: Production environment template — add Clerk variables** - `c857e13` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `infrastructure/templates/shipsecure.nginx.conf.j2` - Added `proxy_set_header x-middleware-subrequest "";` to /api/ and / location blocks (CVE-2025-29927 mitigation)
- `infrastructure/templates/env.production.j2` - Added AUTHENTICATION section with Clerk backend vars and NEXT_PUBLIC_CLERK_* frontend vars to Docker Compose section

## Decisions Made

- Strip `x-middleware-subrequest` in BOTH `/api/` and `/` location blocks — the CVE affects Next.js directly (served via `/`) but the `/api/` block also passes through Next.js middleware when requests arrive at the frontend proxy, so missing either block leaves the vulnerability exploitable
- `CLERK_JWKS_URL` is unconditional (required — backend cannot verify JWTs without it); `CLERK_SECRET_KEY` and `CLERK_WEBHOOK_SIGNING_SECRET` use `{% if %}` conditionals since they are only needed when using webhooks

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

Ansible vault must be populated with these variables before deployment:
- `clerk_jwks_url` — from Clerk Dashboard -> API Keys -> JWKS URL (format: `https://YOUR_FRONTEND_API.clerk.accounts.dev/.well-known/jwks.json`)
- `clerk_secret_key` — from Clerk Dashboard -> API Keys -> Secret keys (starts with `sk_`)
- `clerk_webhook_signing_secret` — from Clerk Dashboard -> Webhooks -> Endpoint -> Signing Secret (starts with `whsec_`)
- `clerk_publishable_key` — from Clerk Dashboard -> API Keys -> Publishable key (starts with `pk_`)

## Next Phase Readiness

- Nginx CVE mitigation is live in the template — will take effect on next deployment
- Production env template is ready for deployment once Ansible vault is populated with Clerk credentials
- Phase 29 backend plans (29-04 onwards) can proceed with confidence that infrastructure-layer auth bypass is closed

## Self-Check: PASSED

- FOUND: infrastructure/templates/shipsecure.nginx.conf.j2 (x-middleware-subrequest appears 2 times)
- FOUND: infrastructure/templates/env.production.j2 (CLERK_JWKS_URL, CLERK_SECRET_KEY, CLERK_WEBHOOK_SIGNING_SECRET, NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY all present)
- FOUND: .planning/phases/29-auth-foundation/29-03-SUMMARY.md
- FOUND: commit 7b3ed72 (Task 1: Nginx CVE mitigation)
- FOUND: commit c857e13 (Task 2: Clerk env variables)
- FOUND: commit f56ba18 (Plan metadata)

---
*Phase: 29-auth-foundation*
*Completed: 2026-02-18*
