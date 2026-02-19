# Phase 2: Free Tier MVP - Context

**Gathered:** 2026-02-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can scan any URL for free and receive comprehensive security results via email. Covers: landing page with URL+email form, scan progress page, results dashboard with findings and A-F grade, email notification on completion. All scanners (headers, TLS, exposed files, JS secrets) integrated with containerized execution. No signup required.

</domain>

<decisions>
## Implementation Decisions

### Landing page & submission
- Tone: Developer-friendly and casual ("ship fast, stay safe") with professional credibility underneath — not fear-driven
- Brief one-line mention of what gets scanned ("We check headers, TLS, exposed files, and more") with a "learn more" link to detailed pages
- Stats-based social proof: "X scans completed" counter to build trust through volume
- On submit: Brief "Scan started!" confirmation message on the landing page, then auto-redirect to progress page after 2-3 seconds

### Results dashboard
- Findings organized with toggle: default to severity grouping (Critical > High > Medium > Low), toggle to switch to category view (Headers, TLS, Secrets, etc.)
- A-F grade shown in a summary bar alongside finding counts — visible but not dominant, not a hero element
- Remediation guidance in expandable accordions — finding summary visible, click to expand for full remediation steps
- Results page is token-protected (unique URL like /results/abc123) — not guessable, no login needed
- Token-protected links expire after 3 days for free tier — creates urgency, paid tier can offer permanent links
- Results page includes markdown download option for full findings report (PDF reserved for paid tier)

### Scan progress experience
- Checklist of scan stages showing each scanner completing one by one (Headers ✓, TLS ○, Secrets ○)
- No partial results — all findings shown at once when scan completes, not as individual scanners finish
- No time estimate — the stage checklist is sufficient, avoids inaccurate predictions
- Progress page URL is bookmarkable and the completion email links back — user can close the tab and return

### Email delivery
- Transactional email service (Resend or Postmark) — not raw SMTP
- Email contains: summary with grade, finding counts by severity, and "View Full Results" button linking to results page
- Primary CTA: "Fixed some issues? Scan again to see your new score" — drives re-engagement
- No upgrade mention in email for Phase 2 — keep it clean, add monetization CTAs in Phase 4

### Claude's Discretion
- Exact landing page layout and component structure
- Color scheme and visual design system
- Summary bar exact design for grade + finding counts
- Transactional email service choice (Resend vs Postmark vs similar)
- "Learn more" detail page structure and content
- Exact confirmation message copy and redirect timing
- Accordion animation and interaction details

</decisions>

<specifics>
## Specific Ideas

- Scan counter for social proof should be real (count from database), not a vanity number
- "Learn more" pages about scan categories are informational — they explain what each check does and why it matters
- The 3-day expiry on free results links is a natural upgrade hook for Phase 4 (paid = permanent access)
- Markdown download should include all findings, remediation steps, and the grade — a complete portable report

</specifics>

<deferred>
## Deferred Ideas

- PDF report generation — Phase 4 (paid tier gets professional branded PDF)
- Upgrade CTAs in email — Phase 4 (monetization)
- Permanent results links — Phase 4 (paid tier benefit)

</deferred>

---

*Phase: 02-free-tier-mvp*
*Context gathered: 2026-02-05*
