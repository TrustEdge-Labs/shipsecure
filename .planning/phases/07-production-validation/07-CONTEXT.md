# Phase 07: Production Validation - Context

**Gathered:** 2026-02-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Verify the deployed ShipSecure application works end-to-end in production. All scan workflows, email delivery, Stripe payment flow, PDF report generation, and service resilience must be validated against real infrastructure. This phase fixes any issues found during validation — it is not a documentation-only checkpoint.

</domain>

<decisions>
## Implementation Decisions

### Validation scope
- Test each of the 5 scanners individually, confirming each returns findings
- Then test the full scan pipeline end-to-end (submit URL → results)
- Email delivery verified by checking actual inbox (not just API success)
- Service recovery tested by actually killing containers and verifying systemd restarts them
- Stripe checkout validated using test-mode keys

### Failure handling
- All failures found during validation must be fixed in this phase — not logged and deferred
- Scanner failures: debug and fix until all 5 scanners pass
- Email delivery: blocker — phase isn't done until emails actually arrive in inbox
- PDF report generation: fix if it fails (install fonts or whatever is needed)
- Stripe webhook: fix here if webhook doesn't fire correctly

### Pre-launch blockers
- Stripe account exists; configure test-mode keys (STRIPE_SECRET_KEY, STRIPE_WEBHOOK_SECRET) on production
- Liberation Sans fonts: not a pre-blocker, but install if PDF generation fails during validation
- Legal review of TOS/consent flow: deferred to next milestone (not needed for v1.1)

### Test targets
- Use a public intentionally-vulnerable test app (e.g., testphp.vulnweb.com or similar)
- Single URL target that triggers multiple scanner findings
- Scanners must return at least some findings to prove they work — zero findings is inconclusive
- Choose a target known to have security issues detectable by header, TLS, secrets, files, and vibe-code scanners

### Claude's Discretion
- Specific public test target selection (must produce findings)
- Order of validation steps
- How to verify email content correctness
- Exact systemd recovery test procedure

</decisions>

<specifics>
## Specific Ideas

- Self-scanning shipsecure.ai was considered but user chose a dedicated test app for more reliable scanner coverage
- Stripe is already set up (account exists) — just needs keys configured on production server
- This is a "fix everything" phase — validation isn't complete until all workflows pass

</specifics>

<deferred>
## Deferred Ideas

- Legal review of TOS/consent flow — deferred to next milestone (v1.2 or later)

</deferred>

---

*Phase: 07-production-validation*
*Context gathered: 2026-02-08*
