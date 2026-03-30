# Phase 42: Funnel Unlock - Context

**Gathered:** 2026-03-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Reopen anonymous scans for any URL (revert Juice Shop lockdown), implement two-layer rate limiting (3/IP/day + 5/domain/hour with cached results), and remove the domain verification gate for authenticated users. This is a backend rate limiter change + frontend form unlock. No new UI screens.

</domain>

<decisions>
## Implementation Decisions

### Anonymous Scan Unlock
- **D-01:** Remove the frontend Juice Shop lockdown in `scan-form.tsx`. The hidden input (line 75) and disabled URL field (lines 77-89) are removed. All users get an editable URL input field. The backend already accepts any SSRF-validated URL from anonymous users, so no backend URL restriction changes needed.
- **D-02:** Remove the "demo only" messaging (lines 91-100 of scan-form.tsx) and the "sign up for free" upsell text below the URL input.

### Rate Limiting
- **D-03:** Change `ANONYMOUS_IP_DAILY_HARD_CAP` from 10 to 3 in `src/rate_limit/middleware.rs:9`.
- **D-04:** Remove the email+domain fair-use layer entirely (lines 61-80 of middleware.rs). Two layers are sufficient: per-IP (3/day) and per-target (5/domain/hour).
- **D-05:** Add new per-target rate limit: max 5 scans of the same domain per hour from any source IP. When exceeded, return the most recent completed scan results for that domain transparently (redirect to existing results URL). User sees "Last scanned X minutes ago" timestamp but no "cached results" banner.
- **D-06:** Rate limit error message for 3/IP/day: "You've used your 3 free scans today. Sign up for 5 scans/month and scan history." Include signup link.

### Domain Verification
- **D-07:** Remove the domain verification gate in `src/api/scans.rs` (lines 131-148). Authenticated users can scan any URL without verifying ownership. The `/verify-domain` page stays in the codebase (harmless, may be useful later).

### Claude's Discretion
- Per-target rate limit implementation: whether to add a new DB query function or reuse/extend existing query patterns
- Per-target cached results: whether to redirect to the existing results URL or return results inline in the scan creation response

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Backend scan pipeline
- `src/api/scans.rs` -- Scan creation endpoint, domain verification gate (lines 131-148 to remove), tier routing
- `src/rate_limit/middleware.rs` -- Rate limit logic, ANONYMOUS_IP_DAILY_HARD_CAP constant, all three current layers
- `src/ssrf/validator.rs` -- SSRF protection (validated, covers RFC 1918, cloud metadata, DNS rebinding)
- `src/db/scans.rs` -- DB query functions for rate limit checks (count_anonymous_scans_by_ip_today, count_anonymous_scans_by_email_and_domain_today)

### Frontend scan form
- `frontend/components/scan-form.tsx` -- Hidden input (line 75), disabled URL field (lines 77-89), demo messaging (lines 91-100)
- `frontend/app/actions/scan.ts` -- Server action that calls backend API

### E2E tests (need updating)
- `frontend/e2e/error-flows.spec.ts` -- Lines 20-38 enforce Juice Shop lockdown, must be rewritten
- `frontend/e2e/free-scan.spec.ts` -- Happy path test, may need URL update

### Design decisions
- `docs/designs/customer-acquisition-v1.md` -- CEO plan with full scope decisions and cross-model tension resolutions
- `DESIGN.md` -- Design system (Geist, industrial/utilitarian, color palette)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ScanForm` component: already has the URL input, email input, CFAA checkbox, error display, and Plausible event. Needs modification, not rebuild.
- `check_rate_limits()` in middleware.rs: well-structured with metric counters. Easy to modify layers.
- `count_anonymous_scans_by_ip_today()` in db/scans.rs: existing pattern for time-windowed count queries. New per-target query follows same pattern.

### Established Patterns
- Rate limit errors return `ApiError::RateLimitedWithReset` with a `resets_at` timestamp
- Frontend displays rate limit errors via the `RATE_LIMITED:` prefix in form error state
- Scan form uses `useActionState` for form submission with server actions

### Integration Points
- Rate limiter is called in `scans.rs:154-161` (step 6 of scan creation)
- Domain verification gate is `scans.rs:131-148` (step 5, being removed)
- Frontend form error handling switches on error message prefixes (lines 43-66 of scan-form.tsx)

</code_context>

<specifics>
## Specific Ideas

- The Juice Shop lockdown was frontend-only (hidden input). Backend always accepted any SSRF-validated URL. This means the "revert" is purely a frontend change.
- Per-target rate limit should return cached results transparently. No banner, no special messaging. Just redirect to the most recent completed scan for that domain with its real timestamp.
- Rate limit upsell copy: "You've used your 3 free scans today. Sign up for 5 scans/month and scan history."

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 42-funnel-unlock*
*Context gathered: 2026-03-30*
