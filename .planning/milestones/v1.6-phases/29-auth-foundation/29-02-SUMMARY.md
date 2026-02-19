---
phase: 29-auth-foundation
plan: 02
subsystem: auth
tags: [clerk, nextjs, clerkprovider, oauth, middleware, userbutton, dashboard]

# Dependency graph
requires:
  - phase: 29-auth-foundation (plan 01)
    provides: "users table, ClerkUser Axum extractor, JWKS-based JWT verification"
provides:
  - "@clerk/nextjs integration with ClerkProvider wrapping root layout"
  - "proxy.ts middleware protecting /dashboard routes via clerkMiddleware"
  - "/sign-in and /sign-up pages with catch-all routes for OAuth callback handling"
  - "Auth-aware header with SignedIn/SignedOut/UserButton conditional rendering"
  - "Protected /dashboard page with server-side auth() + currentUser() greeting"
  - ".env.example documenting all Clerk environment variables"
affects: [30-stripe-removal, 31-results-gating, 32-domain-verification, 34-scan-history-dashboard]

# Tech tracking
tech-stack:
  added: ["@clerk/nextjs@6.37.5"]
  patterns: ["ClerkProvider at root layout level", "proxy.ts (Next.js 16 middleware) with createRouteMatcher", "Server component auth with auth() + currentUser() from @clerk/nextjs/server", "SignedIn/SignedOut conditional rendering in shared components"]

key-files:
  created:
    - "frontend/proxy.ts"
    - "frontend/app/sign-in/[[...sign-in]]/page.tsx"
    - "frontend/app/sign-up/[[...sign-up]]/page.tsx"
    - "frontend/app/dashboard/page.tsx"
    - "frontend/.env.example"
  modified:
    - "frontend/package.json"
    - "frontend/app/layout.tsx"
    - "frontend/components/header.tsx"
    - "frontend/.env.local"

key-decisions:
  - "proxy.ts (not middleware.ts) for Next.js 16.1.6 middleware convention"
  - "Valid-format placeholder publishable key so npm run build succeeds without real Clerk keys"
  - "Force-add .env.example despite frontend/.gitignore .env* rule — template file with no secrets"
  - "UserButton with Clerk defaults only — no custom menu items or tier badge"

patterns-established:
  - "proxy.ts route protection: createRouteMatcher(['/dashboard(.*)']) with auth.protect()"
  - "Catch-all route segments [[...param]] for Clerk OAuth callback handling"
  - "Server component dashboard pattern: auth() guard + currentUser() for personalization"
  - "Header auth state: SignedOut shows Sign In link, SignedIn shows UserButton"

requirements-completed: [AUTH-01, AUTH-02, AUTH-03, AUTH-04, AUTH-05, AUTH-06]

# Metrics
duration: 44min
completed: 2026-02-18
---

# Phase 29 Plan 02: Frontend Auth Integration Summary

**Clerk NextJS integration with ClerkProvider, proxy.ts middleware, sign-in/sign-up OAuth pages, UserButton header, and protected dashboard with personalized greeting**

## Performance

- **Duration:** 44 min
- **Started:** 2026-02-18T01:58:47Z
- **Completed:** 2026-02-18T02:42:45Z
- **Tasks:** 3 (2 auto + 1 human-verify checkpoint)
- **Files modified:** 9

## Accomplishments
- Installed @clerk/nextjs and wrapped root layout with ClerkProvider for full app auth state
- Created proxy.ts middleware with clerkMiddleware protecting /dashboard routes — uses Next.js 16 proxy.ts convention (not middleware.ts)
- Built /sign-in and /sign-up pages with catch-all route segments for OAuth callback handling (Google, GitHub)
- Replaced "Scan Now" header CTA with auth-aware SignedIn/SignedOut conditional rendering and UserButton
- Created protected /dashboard server component with auth() + currentUser() greeting and "Verify your domain" CTA
- All 12 end-to-end verification steps passed: sign-up, sign-in (email + OAuth), session persistence, header state, dashboard protection

## Task Commits

Each task was committed atomically:

1. **Task 1: Install Clerk, create proxy.ts middleware, wrap layout, build sign-in/sign-up pages** - `208ac10` (feat) + `441c78a` (chore: .env.example)
2. **Task 2: Update header with auth state and create protected dashboard page** - `f35ae66` (feat)
3. **Task 3: Verify auth flow end-to-end** - Checkpoint: APPROVED (all 12 verification steps passed)

## Files Created/Modified
- `frontend/proxy.ts` — clerkMiddleware with createRouteMatcher protecting /dashboard routes
- `frontend/app/layout.tsx` — ClerkProvider wrapping entire app (outside html element)
- `frontend/app/sign-in/[[...sign-in]]/page.tsx` — Clerk SignIn component with catch-all route
- `frontend/app/sign-up/[[...sign-up]]/page.tsx` — Clerk SignUp component with catch-all route
- `frontend/components/header.tsx` — SignedOut/Sign In button + SignedIn/UserButton replacing Scan Now
- `frontend/app/dashboard/page.tsx` — Server component with Welcome greeting and verify domain CTA
- `frontend/.env.example` — Clerk env vars documentation template
- `frontend/.env.local` — Clerk env vars with placeholder keys (user replaces with real keys)
- `frontend/package.json` — @clerk/nextjs@6.37.5 added to dependencies

## Decisions Made
- **proxy.ts naming:** Next.js 16.1.6 uses `proxy.ts` (not `middleware.ts`) for the middleware file convention
- **Valid-format placeholder key:** Generated a syntactically valid Clerk publishable key (`pk_test_cGxhY2Vob2xkZXIuY2xlcmsuYWNjb3VudHMuZGV2JA`) so `npm run build` succeeds without real keys — Clerk validates key format at static page generation time
- **.env.example force-added:** Frontend `.gitignore` excludes `.env*` but `.env.example` is a template with no secrets; force-added with `git add -f`
- **UserButton defaults only:** No custom menu items, no tier badge — Clerk defaults per user decision

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Valid-format placeholder Clerk publishable key**
- **Found during:** Task 1 (build verification)
- **Issue:** `pk_test_REPLACE_ME` fails Clerk's publishable key validation during Next.js static page generation, causing `npm run build` to error with "The publishableKey passed to Clerk is invalid"
- **Fix:** Generated a syntactically valid placeholder key by base64-encoding a fake frontend API domain (`placeholder.clerk.accounts.dev$`), which passes Clerk's format validation but does not connect to any real Clerk instance
- **Files modified:** `frontend/.env.local`
- **Verification:** `npm run build` succeeds with all pages generating correctly
- **Committed in:** `208ac10` (part of Task 1 commit)

**2. [Rule 3 - Blocking] Force-add .env.example past .gitignore**
- **Found during:** Task 1 (commit)
- **Issue:** Frontend `.gitignore` has `.env*` glob which catches `.env.example`, preventing `git add`
- **Fix:** Used `git add -f` to force-track the template file — `.env.example` contains no secrets, only documentation
- **Files modified:** `.env.example`
- **Verification:** File tracked in git, visible in repository
- **Committed in:** `441c78a` (separate chore commit)

---

**Total deviations:** 2 auto-fixed (1 bug fix, 1 blocking issue)
**Impact on plan:** Both fixes necessary for build verification and developer onboarding. No scope creep.

## Issues Encountered
None beyond the deviations documented above.

## User Setup Required

Real Clerk API keys must be configured before auth works in production/development:

1. Create a Clerk application at https://dashboard.clerk.com
2. Enable Google OAuth: Clerk Dashboard -> Social Connections -> Google
3. Enable GitHub OAuth: Clerk Dashboard -> Social Connections -> GitHub
4. Enable Email/Password: Clerk Dashboard -> Email, Phone, Username
5. Copy Publishable Key (`pk_test_...`) and Secret Key (`sk_test_...`) from API Keys
6. Replace placeholder values in `frontend/.env.local`

## Next Phase Readiness
- Frontend auth complete — all 6 AUTH requirements verified end-to-end
- Dashboard route protection in place via proxy.ts — future dashboard features (Phase 34) inherit this protection
- Backend JWT verification (Phase 29 Plan 01) + frontend Clerk integration (this plan) = complete auth stack
- Phase 29 Plan 03 (Nginx CVE mitigation, production env wiring) is the final plan in Phase 29
- Phase 30 (Stripe removal) can begin once Phase 29 completes — needs the users table and clerk_user_id column from Plan 01

## Self-Check: PASSED

- All 7 created/modified files verified on disk
- All 3 task commits verified in git history (208ac10, 441c78a, f35ae66)
- Checkpoint Task 3 approved by human verifier

---
*Phase: 29-auth-foundation*
*Completed: 2026-02-18*
