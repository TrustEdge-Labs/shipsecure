---
phase: 48-frontend
verified: 2026-04-07T15:30:00Z
status: human_needed
score: 9/11 must-haves verified
gaps: []
human_verification:
  - test: "Loading spinner message — 'Scanning N dependencies...' vs 'Scanning dependencies...'"
    expected: "Roadmap SC-2 says spinner shows 'Scanning N dependencies...' with a count. Implementation shows 'Scanning dependencies...' (no count). Plan-01 explicitly decided N is only available post-response so count can only appear on the results page. Confirm this deviation is acceptable."
    why_human: "Functional decision recorded in PLAN, but roadmap SC wording says N is included. Business owner must confirm the UX tradeoff is acceptable."
  - test: "Plausible event name: share_clicked vs Share Clicked"
    expected: "FE-05 requires event name 'share_clicked'. The ShareButton (pre-existing component) fires 'Share Clicked' (capitalized, space-separated). Supply chain results page uses this same ShareButton. Confirm whether the naming mismatch is acceptable or if a supply-chain-specific share event with lowercase_underscore naming is needed."
    why_human: "Analytics event naming affects Plausible dashboard configuration. Product owner must decide if the pre-existing 'Share Clicked' event counts as fulfilling 'share_clicked' in FE-05."
---

# Phase 48: Frontend Verification Report

**Phase Goal:** Users can submit a lockfile by any supported method, see tiered findings on a dedicated results page, and track interactions in Plausible
**Verified:** 2026-04-07T15:30:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (Roadmap Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| SC-1 | /supply-chain loads with three input tabs (GitHub URL, Upload, Paste) | VERIFIED | `supply-chain-form.tsx` renders three tab buttons (github/upload/paste), activeTab default='github', conditional rendering per tab |
| SC-2 | After submitting, spinner shows "Scanning N dependencies..." | PARTIAL | Spinner exists with `animate-spin border-brand-primary` class. Message is "Scanning dependencies..." (no N). Plan-01 explicitly decided N is unavailable during fetch (synchronous endpoint returns N only in response). Deviation documented in plan. |
| SC-3 | /supply-chain/results/[token] shows summary cards for each tier with counts | VERIFIED | `supply-chain-summary.tsx` renders 5 cards (Infected/Vulnerable/Advisory/No Known Issues/Unscanned) with counts from results, correct DESIGN.md colors |
| SC-4 | Each finding row shows package name, version, OSV advisory ID, description, and fix action | VERIFIED | `supply-chain-findings.tsx` FindingRow: name (font-mono), version (font-mono text-sm), osv_id (link to osv.dev), description (line-clamp-2), fix action label per tier |
| SC-5 | GitHub 404, OSV down, invalid lockfile, zero-dep each show distinct actionable error | PARTIAL | 400 invalid-lockfile, 400 too-many-deps, 502, 504, 429, generic mapped in `mapErrorResponse()`. Zero-dep lockfiles return HTTP 200 with empty findings — shown as clean state ("No compromised packages found"), not an error message. Plan explicitly documents this as correct behavior. |

**Must-Haves from Plan Frontmatter:**

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| P01-1 | /supply-chain loads with 3 input tabs (GitHub URL, Upload, Paste) | VERIFIED | Three tabs rendered, GitHub URL default |
| P01-2 | Selecting a tab reveals the correct input control | VERIFIED | Conditional render: text input / drop zone / textarea per tab |
| P01-3 | Submitting shows spinner with "Scanning N dependencies..." | PARTIAL | Spinner shown, message is "Scanning dependencies..." (N omitted per plan decision) |
| P01-4 | Backend errors (400/502/504) display distinct, actionable error messages | VERIFIED | `mapErrorResponse()` maps all error types to user-friendly strings; error rendered in danger-bg banner with dismiss |
| P01-5 | Results endpoint returns supply_chain_results and kind for supply chain scans | VERIFIED | `src/api/results.rs` lines 69+77 (expired) and 179+194 (active): both branches include `kind` and `supply_chain_results` |
| P02-1 | /supply-chain/results/[token] shows summary cards for all 5 tiers with counts | VERIFIED | All 5 tiers present with real counts |
| P02-2 | Each finding row shows package name (monospace), version, OSV ID, description, fix action | VERIFIED | Full FindingRow implementation confirmed |
| P02-3 | Header contains a Supply Chain nav link | VERIFIED | `header.tsx` line 37-42: Link href="/supply-chain" outside SignedIn/SignedOut blocks |
| P02-4 | Landing page has a CTA section pointing to /supply-chain | VERIFIED | `page.tsx` lines 149-168: "npm Supply Chain Scanner" CTA with Link href="/supply-chain" |
| P02-5 | Plausible events fire on scan start, completion, infected found, vulnerable found, share click | PARTIAL | supply_chain_scan_started (form.tsx:91), supply_chain_scan_completed (summary.tsx:51), infected_found (summary.tsx:54), vulnerable_found (summary.tsx:58) all verified. share_clicked: ShareButton fires 'Share Clicked' (capitalized) not 'share_clicked' as specified in FE-05. |
| P02-6 | Clean scan shows centered success message | VERIFIED | `supply-chain-findings.tsx` lines 111-135: shield SVG + "No compromised packages found" + "{count} packages checked, all clear" |

**Score:** 9/11 truths fully verified (2 partial: SC-2/P01-3 and SC-5/P02-5 share_clicked naming)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `frontend/lib/supply-chain-types.ts` | TypeScript types for supply chain API response | VERIFIED | 57 lines, all 5 interfaces present: SupplyChainFinding, UnscannedDep, SupplyChainResults, SupplyChainScanResponse, SupplyChainResultsPageData, ApiErrorResponse |
| `frontend/components/supply-chain-form.tsx` | 3-tab form component with loading and error states | VERIFIED | 269 lines, full implementation: tabs, drag-drop, loading spinner, error banner, validation, Plausible event |
| `frontend/app/supply-chain/page.tsx` | Supply chain input page route | VERIFIED | 63 lines, metadata + PageContainer + SupplyChainForm |
| `frontend/app/actions/supply-chain-scan.ts` | Server action for supply chain scan submission | VERIFIED | 87 lines, handles github/paste/upload modes with correct Content-Types, error mapping |
| `src/api/results.rs` | Extended results endpoint with kind + supply_chain_results | VERIFIED | Lines 69/77 (expired path): kind + supply_chain_results=null. Lines 179/194 (active path): kind + supply_chain_results |
| `frontend/app/supply-chain/results/[token]/page.tsx` | Supply chain results page route with metadata | VERIFIED | 265 lines, server-side fetch, generateMetadata, expired/notfound/success paths |
| `frontend/components/supply-chain-summary.tsx` | 5-tier summary cards component | VERIFIED | 83 lines, 5 cards with correct tier colors from DESIGN.md, Plausible events on mount |
| `frontend/components/supply-chain-findings.tsx` | Tiered findings list component | VERIFIED | 170 lines, TierSection per tier, FindingRow with all required fields, clean state, unscanned collapsible |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `supply-chain-form.tsx` | `app/actions/supply-chain-scan.ts` | form submission | VERIFIED | `import { submitSupplyChainScan }` line 5, called on handleSubmit lines 96/99/101 |
| `app/actions/supply-chain-scan.ts` | `/api/v1/scans/supply-chain` | fetch POST | VERIFIED | Line 49: `const url = \`${baseUrl}/api/v1/scans/supply-chain\``, fetch POSTed lines 55/62/69 |
| `results/[token]/page.tsx` | `/api/v1/results/:token` | server-side fetch | VERIFIED | Line 203: `fetch(\`${BACKEND_URL}/api/v1/results/${token}\`)` with cache: "no-store" |
| `supply-chain-findings.tsx` | `frontend/lib/supply-chain-types.ts` | type imports | VERIFIED | Line 3: `import { SupplyChainResults, SupplyChainFinding, UnscannedDep }` |
| `header.tsx` | `/supply-chain` | Link component | VERIFIED | Line 38: `href="/supply-chain"` outside auth blocks |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `supply-chain-form.tsx` | `result` (scan response) | `submitSupplyChainScan()` → `fetch /api/v1/scans/supply-chain` | Yes — real POST returning SupplyChainScanResponse | FLOWING |
| `results/[token]/page.tsx` | `data` (SupplyChainResultsPageData) | server-side `fetch /api/v1/results/${token}` | Yes — real DB query via backend | FLOWING |
| `supply-chain-summary.tsx` | `results` prop | Passed from ResultsView from fetched data | Yes — from real server fetch | FLOWING |
| `supply-chain-findings.tsx` | `results` prop | Passed from ResultsView from fetched data | Yes — from real server fetch | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| TypeScript compiles cleanly | `npx tsc --noEmit` in frontend/ | exit 0 (no errors) | PASS |
| supply-chain-form.tsx imports submitSupplyChainScan | grep import supply-chain-form.tsx | Found line 5 | PASS |
| Results endpoint includes kind field | grep "kind.*scan.kind" src/api/results.rs | Lines 69, 179 | PASS |
| Results endpoint includes supply_chain_results | grep "supply_chain_results" src/api/results.rs | Lines 77, 194 | PASS |
| Header link outside auth blocks | grep href=/supply-chain in header.tsx before SignedIn | Line 38, before line 43 SignedIn | PASS |
| Clean state shield renders | grep "No compromised packages found" supply-chain-findings.tsx | Line 128 | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| FE-01 | 48-01 | /supply-chain page with tabbed input (GitHub URL / Upload / Paste) | SATISFIED | `app/supply-chain/page.tsx` + `supply-chain-form.tsx` with 3 tabs |
| FE-02 | 48-01 | Loading state with "Scanning N dependencies..." spinner | PARTIAL | Spinner exists; message is "Scanning dependencies..." (no N). Intentional plan decision — N only known post-response. |
| FE-03 | 48-02 | /supply-chain/results/[token] page with tiered results display | SATISFIED | `results/[token]/page.tsx` with SupplyChainSummary + SupplyChainFindings |
| FE-04 | 48-01 | Error states for all failure modes | PARTIALLY SATISFIED | 400/502/504/429/generic covered. Zero-dep returns 200 (clean state), not an error — per plan decision. |
| FE-05 | 48-02 | Plausible events: supply_chain_scan_started, supply_chain_scan_completed, infected_found, vulnerable_found, share_clicked | PARTIALLY SATISFIED | 4/5 events fire with exact names. share_clicked fires as 'Share Clicked' via pre-existing ShareButton. |
| RES-01 | 48-02 | Results page displays tiered summary cards (5 tiers with counts) | SATISFIED | All 5 tiers in supply-chain-summary.tsx |
| RES-02 | 48-02 | Each finding shows package name, version, OSV advisory ID, description, fix action | SATISFIED | FindingRow in supply-chain-findings.tsx has all 5 fields |

**Orphaned requirements mapped to Phase 48 in REQUIREMENTS.md but not claimed by any plan:** None.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `supply-chain-summary.tsx` | 74 | `rounded-(card)` CSS class | INFO | Valid Tailwind v4 CSS variable syntax — `--card-radius` defined in globals.css. Used throughout codebase. Not a bug. |
| `inline-results-loader.tsx` | 46, 51, 64 | `rounded-(card)` CSS class | INFO | Same as above — valid project-wide pattern. |
| `supply-chain-form.tsx` | 136 | "Scanning dependencies..." (no N count) | WARNING | Deviates from FE-02 and roadmap SC-2 specification "Scanning N dependencies...". Intentional per plan decision — N only available post-response. |
| `share-button.tsx` | 10 | `'Share Clicked'` event name | WARNING | FE-05 specifies `share_clicked`. Pre-existing component fires `'Share Clicked'` (different naming convention). Supply chain results page reuses this component without overriding. |

No placeholder returns, no empty implementations, no TODO/FIXME in any supply chain files. All form handlers are wired to real fetch calls.

### Human Verification Required

#### 1. Loading Spinner Message — "Scanning N dependencies..." vs "Scanning dependencies..."

**Test:** Submit a scan from /supply-chain and observe the loading spinner message.
**Expected per roadmap:** Spinner displays "Scanning N dependencies..." with a dep count.
**Actual:** Spinner displays "Scanning dependencies..." (no count).
**Why human:** This was a plan-time decision: the backend's POST endpoint is synchronous and only returns dep count after the scan completes. The N count appears on the results page instead. Confirm whether this UX tradeoff is acceptable or if a pre-flight dep count call is needed.

#### 2. Plausible Event Name: share_clicked vs Share Clicked

**Test:** On /supply-chain/results/[token], click "Copy Link" (ShareButton) and check Plausible event fired.
**Expected per FE-05:** Event name `share_clicked` fires.
**Actual:** Event name `Share Clicked` fires (pre-existing ShareButton naming).
**Why human:** Analytics naming convention must match Plausible dashboard setup. If the Plausible property has `Share Clicked` registered as the event name for the existing web app scanner share button, reusing it may be intentional. If supply chain needs its own `share_clicked` event, the supply chain results page needs a custom wrapper. Product owner must decide.

### Gaps Summary

No hard blockers found. All artifacts exist, are substantive, and are properly wired with real data flowing through. The two partial truths are both intentional plan-time decisions documented explicitly:

1. **Loading message** — "Scanning dependencies..." instead of "Scanning N dependencies..." is intentional (N only available post-response for a synchronous endpoint).
2. **share_clicked event name** — The pre-existing ShareButton fires `'Share Clicked'`. FE-05 spec says `share_clicked`. These are naming convention differences in an analytics event.

Both items require product owner confirmation rather than code fixes. No gaps structured in frontmatter as the issues require human judgment, not code remediation.

---

_Verified: 2026-04-07T15:30:00Z_
_Verifier: Claude (gsd-verifier)_
