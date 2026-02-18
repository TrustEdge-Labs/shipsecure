# Phase 33: Tiered Scan Access and Rate Limiting - Context

**Gathered:** 2026-02-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Differentiate anonymous vs authenticated scan depth configs, enforce per-IP anonymous rate limits and per-user monthly quotas at the API layer, and surface quota/tier information to users. Scan history UI and data retention are separate phases.

</domain>

<decisions>
## Implementation Decisions

### Rate limit responses
- Friendly tone with upgrade nudge for anonymous users hitting 1/24h limit: "You've used your free scan today. Sign up for more scans — resets in 18h."
- Countdown format for resets_at ("in 18h 23m"), not absolute timestamps
- Authenticated users hitting monthly quota get same friendly tone but with upgrade CTA: "5 of 5 scans used this month. Upgrade to Pro for unlimited scans." (placeholder for future tier even if Pro doesn't exist yet)
- Rate limit info in 429 JSON body only — no X-RateLimit headers

### Quota display
- Dashboard header shows text badge with color coding (green/yellow/red): "3/5 scans"
- No pre-warning for anonymous users — they just get the 429 when they hit it
- On monthly reset, badge updates to show "0/5 scans" with fresh/green styling

### Scan tier behavior
- Visible "Enhanced scan" badge for authenticated users — they know they're getting deeper analysis
- Anonymous scans show "Basic scan" label with "Sign up for deeper analysis" upsell link
- Tier badges appear in both scan results header AND scan history cards
- Label only ("Basic" / "Enhanced") — no internal config details exposed (file counts, timeouts hidden)

### Domain verification gate
- Hard block for authenticated users scanning unverified domains — reject with clear error message
- Both client-side and server-side checks: client warns on scan click (better UX), server enforces (security)
- Client-side check triggers on scan button click, not on URL field blur
- Error links to /verify-domain page — no inline verification flow

### Claude's Discretion
- Exact color thresholds for quota badge (green/yellow/red breakpoints)
- Badge placement within dashboard header
- Exact wording of upsell messages
- Error message copy for domain verification rejection

</decisions>

<specifics>
## Specific Ideas

- Upgrade CTA should be forward-looking ("Upgrade to Pro") even if Pro tier doesn't exist yet — sets up future monetization
- Tier label styling should feel like a subtle badge, not a prominent banner — informative without being noisy

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 33-tiered-scan-access-and-rate-limiting*
*Context gathered: 2026-02-18*
