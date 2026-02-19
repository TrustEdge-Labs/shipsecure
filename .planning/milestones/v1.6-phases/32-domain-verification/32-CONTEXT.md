# Phase 32: Domain Verification - Context

**Gathered:** 2026-02-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Authenticated users can prove they own a domain (including shared-hosting subdomains like myapp.vercel.app), and only verified domains can receive authenticated scans. Covers: verified_domains table, verify-start/verify-confirm API endpoints, meta tag verification, shared-hosting TLD blocklist (root only — subdomains allowed), 30-day TTL, domain wizard UI with troubleshooting.

</domain>

<decisions>
## Implementation Decisions

### Verification wizard flow
- Dedicated `/dashboard/verify` page with step-by-step wizard — not a modal or inline section
- Manual "Verify now" button after user places the meta tag — no auto-polling
- One domain at a time — after success, return to dashboard; no "verify another" in the same session
- If user enters an already-verified (non-expired) domain, show existing status ("This domain is already verified. Expires in X days.") — no new token issued

### Blocked domain messaging
- **Critical design change from DOMN-04:** Allow subdomain verification on shared-hosting TLDs. Block only the root TLD itself (e.g., `vercel.app` blocked, `myapp.vercel.app` allowed). Vibe-coders — the target users — typically deploy on shared hosting without custom domains. The meta tag proves they control the app's HTML output, which is sufficient proof of ownership.
- Confirmed blocklist roots: github.io, vercel.app, netlify.app, pages.dev
- Claude's discretion: error copy tone, validation layer (frontend vs backend vs both), blocklist storage approach (hardcoded vs config)

### Verification status display
- Small inline badge next to domain name: green "Verified" / yellow "Pending" / red "Expired" pill — compact, doesn't dominate
- Proactive 7-day expiry warning in dashboard — badge changes to yellow warning state when within 7 days of expiry
- Verified domains list lives as a section within the main dashboard page (not a separate `/dashboard/domains` route) — above or beside scan history
- **Expired domain re-gates past results:** When a domain's verification expires, past scan results for that domain become gated again (high/critical findings hidden) until the user re-verifies. This extends the Phase 31 gating logic — `owner_verified` must also check domain verification status, not just user identity match.

### Meta tag snippet experience
- Dark code block with one-click copy button showing the full `<meta>` tag — standard dev tool pattern
- Optional "Test my tag" pre-check button that verifies the tag is live without consuming the verification attempt — helps nervous users
- Specific failure diagnosis on verification failure: tell the user exactly what happened ("We fetched your page but didn't find the meta tag. Check that it's in `<head>`, not `<body>`."), show what was found if possible
- Opaque cryptographically random token — no encoded information, no leakage

### Claude's Discretion
- Error copy tone and wording for blocked root TLDs
- Validation layer choice (frontend-only, backend-only, or both) for TLD blocklist
- Blocklist storage approach (hardcoded constant vs config/env)
- Exact badge styling and color tokens
- Code block framework (syntax highlighting, line numbers, etc.)
- Token length and generation method
- Dashboard section layout relative to scan history

</decisions>

<specifics>
## Specific Ideas

- The shared-hosting subdomain allowance is the key insight: vibe-coders deploy to `myapp.vercel.app` without custom domains, and meta tag verification proves app control regardless of hosting provider
- Re-gating on domain expiry creates a natural re-verification incentive — users stay engaged to keep their results accessible
- The "Test my tag" pre-check reduces support burden from confused users who deployed but made a placement error

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

Note: The "expired domain re-gates results" decision touches Phase 31's gating logic. Implementation should extend the existing `owner_verified` check in `results.rs` to also verify domain status — this is within Phase 32's scope since domain verification is the trigger.

</deferred>

---

*Phase: 32-domain-verification*
*Context gathered: 2026-02-18*
