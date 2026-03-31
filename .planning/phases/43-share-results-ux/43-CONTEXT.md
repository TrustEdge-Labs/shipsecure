# Phase 43: Share & Results UX - Context

**Gathered:** 2026-03-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Add share button to results page, enrich OG meta tags for social preview cards, and create an expired results page that guides users back into the scan funnel. This phase modifies the existing results page and the backend scan deletion logic.

</domain>

<decisions>
## Implementation Decisions

### Share Button
- **D-01:** Add a "Copy Link" button in the results page header actions row, alongside existing action buttons (download report, rescan). Uses Clipboard API to copy the `/results/{token}` URL. Shows "Copied!" toast for 2 seconds, bottom-center, with `aria-live="polite"` for screen readers.
- **D-02:** Button style: `btn-secondary` from DESIGN.md (bg-elevated, text-primary, border-default). Matches existing action button pattern.

### OG Meta Tags
- **D-03:** Domain-first format for social previews. Title: `{target_domain} - Grade {grade} | ShipSecure`. Description: `Security scan found {total} issues. {high_count} high severity. Scan your app free at shipsecure.ai.`
- **D-04:** OG tags are text-based only (no dynamic image generation). The existing `generateMetadata` function in `results/[token]/page.tsx` already fetches scan data server-side. Enrich the returned metadata with grade and finding counts.
- **D-05:** For in-progress scans (no grade yet), keep current behavior: `Security Scan: In Progress - ShipSecure`.

### Expired Results
- **D-06:** Soft delete with tombstone. Instead of hard-deleting scan rows when they expire, mark them with an `expired` status (or `expired_at` timestamp). Keep the `target_url` column populated. The data retention cleanup task should set expired status instead of `DELETE FROM scans`.
- **D-07:** When a user visits a results page for an expired scan, show a dedicated expired state (not 404): "These results have expired. Scan results are kept for 24 hours." with a scan form pre-filled with the original target URL and a signup upsell: "Sign up for 30-day history."
- **D-08:** The backend `/api/v1/results/{token}` endpoint should return the expired scan with a `status: "expired"` field (not 404) so the frontend can render the CTA with the original URL.

### Claude's Discretion
- Database migration approach for soft delete (add column vs. modify existing status enum)
- Whether to strip findings data from expired scans (save storage) while keeping target_url
- Toast component implementation (inline vs. shared component)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Results page
- `frontend/app/results/[token]/page.tsx` -- Server component with generateMetadata, grade display, findings list, action buttons
- `frontend/components/grade-summary.tsx` -- Grade display component
- `frontend/components/results-dashboard.tsx` -- Findings list component

### Backend results endpoint
- `src/api/results.rs` -- GET /api/v1/results/{token} endpoint
- `src/db/scans.rs` -- Scan DB queries, data retention cleanup task

### Data retention
- `src/orchestrator/worker_pool.rs` -- Contains the cleanup_expired_scans task (or similar)

### Design system
- `DESIGN.md` -- Color palette, component styles, typography (btn-secondary for share button)
- `docs/designs/customer-acquisition-v1.md` -- CEO plan with scope decisions

### Prior phase context
- `.planning/phases/42-funnel-unlock/42-CONTEXT.md` -- Rate limit decisions, domain verification removal

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `generateMetadata` in results page already fetches scan data server-side and returns OG metadata. Just needs enriched content.
- `ScanResponse` type in `frontend/lib/types.ts` has `score`, `target_url`, `findings` fields.
- Existing action buttons pattern in results page header (download, rescan) for share button placement.

### Established Patterns
- Results page is a server component that fetches data with `fetch()` and `no-store` cache policy.
- The data retention cleanup runs as a Tokio interval task integrated with graceful shutdown.
- Error states use `notFound()` from `next/navigation`.

### Integration Points
- Results page header section (~line 155-199) for share button insertion
- `generateMetadata` function (~line 15-60) for OG tag enrichment
- Data retention cleanup task for soft delete migration
- Backend results endpoint for returning expired scan state

</code_context>

<specifics>
## Specific Ideas

- The `DEMO_TARGET_HOST` constant in results page may be stale after Phase 42 changes. Clean up if no longer referenced.
- OG description should end with "Scan your app free at shipsecure.ai" as a subtle CTA in social previews.
- Expired results page should feel warm, not like an error. "These results have expired" not "404 Not Found."

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 43-share-results-ux*
*Context gathered: 2026-03-31*
