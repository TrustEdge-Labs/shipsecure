# Phase 42: Funnel Unlock - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md -- this log preserves the alternatives considered.

**Date:** 2026-03-30
**Phase:** 42-funnel-unlock
**Areas discussed:** Cached results behavior, Rate limit messaging, Email requirement changes

---

## Cached Results Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Transparent cached results | Redirect to most recent scan results. Show timestamp. User may not realize it's cached. | Yes |
| Cached with notice | Show results with banner: "This domain was recently scanned." Honest, slightly more friction. | |
| You decide | Claude picks during implementation. | |

**User's choice:** Transparent cached results
**Notes:** User prefers seamless experience. No special banner for cached results. Timestamp shows when actually scanned.

---

## Rate Limit Messaging

| Option | Description | Selected |
|--------|-------------|----------|
| Signup upsell | "You've used your 3 free scans today. Sign up for 5 scans/month and scan history." Conversion-focused. | Yes |
| Friendly cooldown | "That's 3 scans for today. Come back tomorrow, or sign up for more." Softer, less pushy. | |
| You decide | Claude picks the copy during implementation. | |

**User's choice:** Signup upsell
**Notes:** Conversion-focused messaging. Rate limit hit is a natural conversion point.

---

## Email Requirement Changes

| Option | Description | Selected |
|--------|-------------|----------|
| Remove email+domain limit | Two layers (IP + per-target) are enough. Simplifies the rate limiter. One less DB query per scan. | Yes |
| Keep all three layers | Belt and suspenders. Email+domain prevents same person from using different browsers/IPs. | |
| You decide | Claude picks based on implementation simplicity. | |

**User's choice:** Remove email+domain limit
**Notes:** IP + per-target is sufficient. Simplifies middleware and removes one DB query per scan.

---

## Claude's Discretion

- Per-target rate limit implementation approach (DB query pattern, caching strategy)
- Per-target cached results delivery mechanism (redirect vs inline response)

## Deferred Ideas

None
