# Phase 43: Share & Results UX - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md -- this log preserves the alternatives considered.

**Date:** 2026-03-31
**Phase:** 43-share-results-ux
**Areas discussed:** Expired results detection, Share button placement, OG tag content

---

## Expired Results Detection

| Option | Description | Selected |
|--------|-------------|----------|
| Soft delete with tombstone | Mark expired, keep target_url. Results page checks status and shows CTA. | Yes |
| Generic expired page | Show generic "expired" without pre-filling URL. No DB changes. | |
| You decide | Claude picks based on implementation simplicity. | |

**User's choice:** Soft delete with tombstone
**Notes:** Preserves target_url for the re-scan CTA. More DB storage but enables the full funnel re-entry flow.

---

## Share Button Placement

| Option | Description | Selected |
|--------|-------------|----------|
| In the header actions row | Next to existing action buttons. Consistent with current layout. | Yes |
| Sticky floating button | Fixed position, always visible while scrolling. More prominent. | |
| You decide | Claude picks based on layout. | |

**User's choice:** In the header actions row
**Notes:** Consistent with existing download/rescan buttons.

---

## OG Tag Content

| Option | Description | Selected |
|--------|-------------|----------|
| Grade-first format | 'Grade B - Security Scan | ShipSecure' | |
| Domain-first format | 'my-app.vercel.app - Grade B | ShipSecure' | Yes |
| You decide | Claude picks best copy for social previews. | |

**User's choice:** Domain-first format
**Notes:** Domain name first gives context in Slack/Twitter previews. Description includes finding count + severity + CTA.

---

## Claude's Discretion

- Database migration approach (add column vs modify status enum)
- Whether to strip findings from expired scans
- Toast component implementation

## Deferred Ideas

None
