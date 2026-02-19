# Phase 34: Scan History Dashboard - Context

**Gathered:** 2026-02-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Authenticated users can see all their past scans with severity summaries, expiry countdowns, and quota status. Dashboard is a protected route at `/dashboard`. No search, filtering, or bulk actions — just a clear view of scan history and account status.

</domain>

<decisions>
## Implementation Decisions

### Scan list presentation
- Table rows layout — columns for domain, date, severity counts, expiry, action
- Severity counts displayed as colored number badges (red/orange/yellow/green chips with count)
- Default sort: expiring soonest first — drives urgency to review results before expiry
- Entire row is clickable (navigates to /results/:token) AND has an explicit "View" button in last column for accessibility

### Empty & edge states
- Zero-scan empty state is context-aware: show verify-domain CTA if no domains verified, otherwise show run-a-scan CTA
- Expired scans remain visible as dimmed rows with an "Expired" badge — user sees history but results are inaccessible
- In-progress scans displayed in a separate "Active scans" section above the history table, with spinner and status text
- Failed scans shown in history with a red "Failed" badge — user knows the attempt happened

### Quota & sidebar
- Sidebar card layout (not top banner) — right sidebar alongside account info
- Sidebar contains: quota card AND verified domains list with status badges
- Quota displayed as text only: "3 of 5 scans used — resets Mar 1"
- At quota limit: scan action (New Scan CTA or equivalent) is disabled/grayed out with tooltip explaining when it resets

### Pagination & density
- Traditional numbered page navigation (page 1, 2, 3...)
- 10 scans per page
- No filtering — at 5 scans/month Developer tier, volume stays manageable
- Mobile responsive: table stacks into compact cards on narrow screens

### Claude's Discretion
- Exact table column widths and spacing
- Sidebar card styling and layout details
- Active scans section visual treatment
- Pagination component styling
- Expiry countdown format (e.g., "3 days left" vs "Expires Feb 21")

</decisions>

<specifics>
## Specific Ideas

- Row click + View button pattern: entire row navigable for quick access, explicit button for users who expect a clear action target
- "Active scans" section above history creates visual separation between things in flight and completed work
- Sidebar with quota + verified domains gives a persistent account overview without cluttering the scan list

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 34-scan-history-dashboard*
*Context gathered: 2026-02-18*
