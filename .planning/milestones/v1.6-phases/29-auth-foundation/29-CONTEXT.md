# Phase 29: Auth Foundation - Context

**Gathered:** 2026-02-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Clerk authentication integration (frontend + backend) so users can sign up/in and the Axum backend can verify their identity via JWT on every request. Includes CORS fix for Authorization header, CVE-2025-29927 Nginx mitigation, users table, and Clerk webhook sync. No tier display, no scan history, no domain verification — those are later phases.

</domain>

<decisions>
## Implementation Decisions

### Sign-in/Sign-up flow
- Dedicated full pages at /sign-in and /sign-up (not modal, not inline)
- Use Clerk's default appearance — no custom theming to match dark mode
- After sign-in: always redirect to /dashboard
- After sign-up: same destination as sign-in (/dashboard) — no separate onboarding page

### Header auth integration
- UserButton (avatar/dropdown) replaces the 'Scan Now' CTA position (right side of header) when signed in
- When signed out: show a 'Sign In' button in that same position (no 'Scan Now' CTA in header)
- UserButton dropdown uses Clerk defaults only (avatar, email, 'Manage account', 'Sign out') — no custom menu items
- No tier badge or quota display in header — that's Phase 33-34 scope

### Dashboard skeleton
- Protected /dashboard route using same full-width layout as the rest of the site (no sidebar nav)
- Empty state: welcome message greeting user by name ("Welcome, John") + prominent 'Verify your domain' CTA
- CTA links to /verify-domain even though that page won't exist until Phase 32 (will 404 in the interim — acceptable)

### Auth error handling
- Unauthenticated /dashboard access: silent redirect to /sign-in (no flash message)
- Invalid/expired JWT: API returns 401 with generic 'Authentication required' message — don't leak JWT details
- Webhook failure: log the error and return 500 — rely on Clerk/Svix automatic retry with exponential backoff
- Session expiry: no background polling — handle on next user action (API returns 401, frontend redirects to sign-in)

### Claude's Discretion
- Clerk component configuration details (e.g., which OAuth providers to show, form field ordering)
- Loading states during auth redirects
- Exact layout spacing and typography for dashboard welcome page
- proxy.ts middleware configuration specifics

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. Clerk defaults preferred over custom styling for speed.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 29-auth-foundation*
*Context gathered: 2026-02-17*
