# Phase 48: Frontend - Context

**Gathered:** 2026-04-07
**Status:** Ready for planning

<domain>
## Phase Boundary

New /supply-chain input page with 3-tab form, /supply-chain/results/[token] results page with tiered findings display, loading/error states, Plausible conversion events, and navigation integration (header link + landing page CTA). All following DESIGN.md (Geist, dark industrial).

</domain>

<decisions>
## Implementation Decisions

### Tab Component
- **D-01:** Client component with `useState` for active tab. Three tabs: GitHub URL (text input), Upload Lockfile (file drop zone), Paste Content (textarea). Tab content renders conditionally. No URL-based routing for tabs.
- **D-02:** Each tab submits to the same `POST /api/v1/scans/supply-chain` endpoint but with different Content-Type (JSON for GitHub URL and paste, multipart for upload).

### Results Page
- **D-03:** New dedicated components: `SupplyChainSummary` (summary cards row) and `SupplyChainFindings` (finding list grouped by tier). Do NOT extend existing ResultsDashboard or FindingAccordion.
- **D-04:** Reuse existing `ShareButton` and `PageContainer` components.
- **D-05:** Summary cards row with 5 tiers: Infected (red #ef4444), Vulnerable (orange #f59e0b), Advisory (yellow, dimmed), No Known Issues (green #22c55e), Unscanned (gray #71717a). Colors from DESIGN.md semantic tokens.
- **D-06:** Each finding shows: package name (monospace), version, OSV advisory ID (link-colored), description, and fix action (green accent).
- **D-07:** Results page at `/supply-chain/results/[token]`. Server component for metadata, client component for interactive elements.

### Loading & Error States
- **D-08:** Loading state: spinner with "Scanning N dependencies..." message. Synchronous scan, no polling needed. Frontend shows spinner during fetch, results appear when response arrives.
- **D-09:** Error states map to backend error responses:
  - 400 (LockfileParse): "This doesn't look like a valid package-lock.json file"
  - 400 (DepCountExceeded): "Too many dependencies (max 5,000)"
  - 502 (OsvQuery/GitHubFetch/ChunkFailure): "Something went wrong. Try again or upload your lockfile directly"
  - 504 (Timeout): "Scan timed out. Try a smaller lockfile"
  - Generic: "An unexpected error occurred"

### Plausible Events
- **D-10:** Five events wired to specific user actions:
  - `supply_chain_scan_started` — on form submit
  - `supply_chain_scan_completed` — on results render
  - `infected_found` — on results render if any infected findings
  - `vulnerable_found` — on results render if any vulnerable findings
  - `share_clicked` — on share button click (reuses ShareButton pattern)

### Navigation
- **D-11:** Add "Supply Chain" link to header nav bar (alongside existing links).
- **D-12:** Add a CTA section on the landing page pointing to /supply-chain. Brief description: "Check your dependencies for known compromised packages." Existing "Scan Now" CTA stays for web app scanner.

### Claude's Discretion
- File drop zone implementation details (drag-and-drop vs click-to-browse vs both)
- Exact tab styling (underline, pill, or segmented control — must follow DESIGN.md)
- Whether to use Next.js server actions or client-side fetch for form submission
- OG metadata for /supply-chain and /supply-chain/results/[token] pages
- Mobile responsive layout for summary cards (stack vs scroll)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design System
- `DESIGN.md` — Typography (Geist), colors (dark, semantic), spacing, component vocabulary. MANDATORY before any visual decisions.

### Design Doc + Wireframe
- `~/.gstack/projects/TrustEdge-Labs-shipsecure/john-main-design-20260406-133756.md` — Approved supply chain MVP design doc with wireframe description (Screen 1: input, Screen 2: results, Screen 3: clean state)

### Existing Frontend Patterns (MUST READ)
- `frontend/components/scan-form.tsx` — Existing form pattern (server action, Zod validation)
- `frontend/components/share-button.tsx` — Reusable ShareButton
- `frontend/components/page-container.tsx` — PageContainer layout
- `frontend/components/results-dashboard.tsx` — Existing results layout (reference, not reuse)
- `frontend/components/header.tsx` — Header nav (add supply chain link)
- `frontend/app/page.tsx` — Landing page (add CTA section)
- `frontend/app/results/[token]/page.tsx` — Existing results page (reference for metadata pattern)
- `frontend/app/actions/scan.ts` — Existing server action (reference for API call pattern)

### Backend API (consumed by frontend)
- `src/api/supply_chain.rs` — POST /api/v1/scans/supply-chain endpoint (3 input modes, response shape)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ShareButton` component — copy-to-clipboard + Plausible event. Directly reusable.
- `PageContainer` component — max-width layout wrapper. Directly reusable.
- `scan-form.tsx` — Pattern reference for form submission with server actions + Zod validation.
- Plausible is already wired (script in layout.tsx, `window.plausible()` calls in existing components).

### Established Patterns
- Client components use `"use client"` directive
- Server actions in `app/actions/` with Zod validation
- Dynamic metadata via `generateMetadata` in page.tsx files
- Error states handled inline with conditional rendering
- DESIGN.md tokens applied via CSS variables in globals.css

### Integration Points
- Header nav: add link in `frontend/components/header.tsx`
- Landing page: add CTA section in `frontend/app/page.tsx`
- New pages: `frontend/app/supply-chain/page.tsx` and `frontend/app/supply-chain/results/[token]/page.tsx`
- New components: `frontend/components/supply-chain-form.tsx`, `supply-chain-summary.tsx`, `supply-chain-findings.tsx`

</code_context>

<specifics>
## Specific Ideas

The office-hours wireframe shows three screens:
1. **Input** — tabbed form with GitHub URL as default tab, green "Scan Dependencies" button
2. **Results (issues found)** — 4 summary cards in a row, tiered findings below with severity badges, share bar at bottom
3. **Clean state** — centered shield icon, "No compromised packages found" message, share button

Follow these layouts. The wireframe uses ShipSecure's design system (dark background #0a0a0f, green accent #22c55e, monospace for package names).

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 48-frontend*
*Context gathered: 2026-04-07*
