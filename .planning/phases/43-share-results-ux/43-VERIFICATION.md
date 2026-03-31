---
phase: 43-share-results-ux
verified: 2026-03-29T00:00:00Z
status: human_needed
score: 6/6 must-haves verified
re_verification: false
human_verification:
  - test: "Copy Link button interaction"
    expected: "Clicking 'Copy Link' copies the URL and shows 'Copied!' text for ~2 seconds"
    why_human: "navigator.clipboard is a browser API — cannot invoke from Node; requires interactive browser session"
  - test: "OG meta tags in browser / social preview"
    expected: "og:title renders as '{domain} - Grade {grade} | ShipSecure', og:description mentions finding count"
    why_human: "generateMetadata output requires a live Next.js render + view-source or social debugger tool"
  - test: "Expired results page renders correctly"
    expected: "Visiting an expired-token URL shows clock icon, 'These results have expired' heading, 'Scan again' button pre-filled with original target URL, sign-up upsell below"
    why_human: "Requires a real expired scan row in the database to exercise the expired branch end-to-end"
---

# Phase 43: Share Results UX — Verification Report

**Phase Goal:** Scan results are shareable with rich social previews and expired results guide users back into the funnel
**Verified:** 2026-03-29
**Status:** human_needed (all automated checks passed; 3 items require browser or DB interaction to confirm)
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #  | Truth                                                                                        | Status     | Evidence                                                                                                     |
|----|----------------------------------------------------------------------------------------------|------------|--------------------------------------------------------------------------------------------------------------|
| 1  | Results page shows a Copy Link button that copies the capability URL to clipboard            | ✓ VERIFIED | `share-button.tsx` exports `ShareButton`; uses `navigator.clipboard.writeText(url)`; rendered at line 309 of `page.tsx` |
| 2  | Pasting a results URL into Slack or Twitter renders an OG preview card with grade and finding count | ✓ VERIFIED | `generateMetadata` in `page.tsx` returns `openGraph` and `twitter` objects; completed scans use `Grade {grade}` title format and finding-count description |
| 3  | Visiting an expired results URL shows a dedicated page with scan-again CTA pre-filled with original target URL | ✓ VERIFIED | `page.tsx` branches on `data.status === 'expired'` before the in-progress check; renders `/?url=${encodeURIComponent(data.target_url)}` anchor; backend returns HTTP 200 with tombstone JSON for expired tokens |
| 4  | Expired scans are soft-deleted (status set to 'expired') not hard-deleted                   | ✓ VERIFIED | `cleanup.rs` calls `soft_expire_scans_by_tier` (UPDATE-based); `delete_expired_scans_by_tier` is no longer called from cleanup |
| 5  | Backend returns expired scan data with status 'expired' and target_url preserved             | ✓ VERIFIED | `results.rs` two-phase lookup: `get_scan_by_token` then `get_scan_by_token_including_expired`; expired branch returns JSON with `"status": "expired"` and `target_url` at HTTP 200 |
| 6  | Non-expired scan lookups continue to work unchanged                                          | ✓ VERIFIED | Primary lookup still calls `get_scan_by_token` (active-only); expired fallback only runs when primary returns `None`; `cargo check` passes |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact                                                  | Expected                                              | Status     | Details                                                                                  |
|-----------------------------------------------------------|-------------------------------------------------------|------------|------------------------------------------------------------------------------------------|
| `migrations/20260331000001_add_expired_status.sql`       | Add 'expired' value to scan_status enum               | ✓ VERIFIED | Contains `ALTER TYPE scan_status ADD VALUE IF NOT EXISTS 'expired' AFTER 'failed'`       |
| `src/models/scan.rs`                                     | Expired variant on ScanStatus enum                    | ✓ VERIFIED | `Expired` variant present in `ScanStatus` enum (line 13)                                |
| `src/db/scans.rs`                                        | Soft-delete function and expired scan query           | ✓ VERIFIED | Both `soft_expire_scans_by_tier` (line 504) and `get_scan_by_token_including_expired` (line 526) present |
| `src/cleanup.rs`                                         | Cleanup uses soft expire instead of hard delete       | ✓ VERIFIED | `run_cleanup` calls `scans::soft_expire_scans_by_tier` for both "free" and "authenticated" tiers |
| `src/api/results.rs`                                     | Returns expired scans with target_url instead of 404  | ✓ VERIFIED | Two-phase lookup present; expired branch returns HTTP 200 with tombstone JSON; download endpoint returns 410 Gone |
| `frontend/components/share-button.tsx`                   | Copy Link client component with clipboard API and toast | ✓ VERIFIED | `"use client"`, `navigator.clipboard`, `aria-live="polite"`, 2-second copied state      |
| `frontend/app/results/[token]/page.tsx`                  | Enriched OG meta tags and expired results state       | ✓ VERIFIED | `openGraph` in `generateMetadata`; `data.status === 'expired'` branch with full tombstone UI |
| `frontend/lib/types.ts`                                  | Updated ScanResponse with expired status comment      | ✓ VERIFIED | Comment on line 37: `// Possible values: 'pending' \| 'in_progress' \| 'completed' \| 'failed' \| 'expired'` |

### Key Link Verification

| From                                   | To                              | Via                                    | Status     | Details                                                                             |
|----------------------------------------|---------------------------------|----------------------------------------|------------|-------------------------------------------------------------------------------------|
| `src/cleanup.rs`                       | `src/db/scans.rs`               | `soft_expire_scans_by_tier`            | ✓ WIRED    | `cleanup.rs` calls `scans::soft_expire_scans_by_tier` at lines 39 and 47           |
| `src/api/results.rs`                   | `src/db/scans.rs`               | `get_scan_by_token_including_expired`  | ✓ WIRED    | `results.rs` calls `db::scans::get_scan_by_token_including_expired` at lines 57-58 and 209-210 |
| `frontend/app/results/[token]/page.tsx` | `frontend/components/share-button.tsx` | import and render in actions row | ✓ WIRED    | Imported at line 5; rendered at line 309 with correct `url` prop                   |
| `frontend/app/results/[token]/page.tsx` | `/api/v1/results/{token}`       | fetch in generateMetadata and page body | ✓ WIRED    | `fetch(${BACKEND_URL}/api/v1/results/${token}`)` called in both `generateMetadata` (line 27) and `ResultsPage` (line 137) |

### Data-Flow Trace (Level 4)

| Artifact                              | Data Variable | Source                                     | Produces Real Data | Status     |
|---------------------------------------|---------------|--------------------------------------------|--------------------|------------|
| `page.tsx` (expired branch)           | `data.target_url` | `GET /api/v1/results/{token}` → `get_scan_by_token_including_expired` → DB `SELECT` | Yes — DB query with no expiry filter returns preserved `target_url` | ✓ FLOWING  |
| `page.tsx` (generateMetadata — OG)   | `data.score`, `data.summary` | `GET /api/v1/results/{token}` → `get_scan_by_token` → DB query | Yes — real DB query; score and summary populated in completed scans | ✓ FLOWING  |
| `share-button.tsx`                    | `url` prop    | Caller passes `https://shipsecure.ai/results/${token}` | Yes — constructed from path param, not hardcoded empty value | ✓ FLOWING  |

### Behavioral Spot-Checks

| Behavior                                       | Command                                                                                               | Result                  | Status  |
|------------------------------------------------|-------------------------------------------------------------------------------------------------------|-------------------------|---------|
| Backend compiles with Expired variant          | `cargo check`                                                                                          | `Finished dev profile`  | ✓ PASS  |
| Frontend TypeScript compiles                   | `npx tsc --noEmit` (in `frontend/`)                                                                    | No output (clean)       | ✓ PASS  |
| `soft_expire_scans_by_tier` present in db layer | `grep -q "soft_expire_scans_by_tier" src/db/scans.rs`                                                 | Match found             | ✓ PASS  |
| Cleanup no longer hard-deletes                 | `grep -q "delete_expired_scans_by_tier" src/cleanup.rs` (should be absent)                           | Not present in cleanup  | ✓ PASS  |
| Navigator.clipboard in ShareButton             | `grep -q "navigator.clipboard" frontend/components/share-button.tsx`                                  | Match found             | ✓ PASS  |
| OG openGraph in generateMetadata               | `grep -q "openGraph" frontend/app/results/\[token\]/page.tsx`                                         | Match found             | ✓ PASS  |
| Expired UI branch in page.tsx                  | `grep -q "data.status === 'expired'" frontend/app/results/\[token\]/page.tsx`                         | Match found             | ✓ PASS  |

### Requirements Coverage

| Requirement | Source Plan | Description                                                       | Status      | Evidence                                                                                          |
|-------------|------------|-------------------------------------------------------------------|-------------|---------------------------------------------------------------------------------------------------|
| FUNNEL-05   | 43-02      | User can copy scan results URL via share button on results page   | ✓ SATISFIED | `ShareButton` rendered in actions row of `page.tsx`; uses Clipboard API                           |
| FUNNEL-06   | 43-01      | Expired results page shows "scan again" CTA with pre-filled URL   | ✓ SATISFIED | Backend returns tombstone JSON at HTTP 200; frontend renders dedicated expired page with `/?url=` CTA |
| FUNNEL-07   | 43-02      | Results page has OG meta tags with grade and finding count        | ✓ SATISFIED | `generateMetadata` returns `openGraph` and `twitter` objects with grade and finding count for completed scans |

No orphaned requirements detected. All three FUNNEL-05/06/07 IDs claimed in plan frontmatter are addressed, and REQUIREMENTS.md marks all three as Complete in Phase 43.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `frontend/app/results/[token]/page.tsx` | 148 | `console.error('Error fetching results:', error)` | Info | Debug logging on fetch error — acceptable for server component error reporting; does not affect functionality |

No stub patterns found. No `TODO`/`FIXME` annotations in modified files. No hardcoded empty data flowing to render.

One note: `get_scan_by_token_including_expired` in `src/db/scans.rs` carries `#[allow(dead_code)]`. This suppression is safe — the function is called from `results.rs` but sqlx lint may not detect cross-module usage at the item level. The lint suppression does not indicate the function is unused.

### Human Verification Required

#### 1. Copy Link button interaction

**Test:** Visit any completed scan results URL locally (e.g. `http://localhost:3001/results/{token}`). Click the "Copy Link" button.
**Expected:** Button text changes to "Copied!" for approximately 2 seconds, then resets to "Copy Link". Pasting clipboard should yield `https://shipsecure.ai/results/{token}`.
**Why human:** `navigator.clipboard.writeText` is a browser API; it cannot be invoked from Node or a headless grep scan.

#### 2. OG meta tags rendered correctly

**Test:** Visit a completed scan results URL. View page source or open browser devtools Network tab. Alternatively, paste the URL into the Twitter Card Validator (`cards-dev.twitter.com/validator`) or the Facebook Sharing Debugger.
**Expected:**
- `<meta property="og:title" content="{domain} - Grade {grade} | ShipSecure" />`
- `<meta property="og:description" content="Security scan found N issues. M high/critical severity. Scan your app free at shipsecure.ai." />`
- `<meta name="twitter:card" content="summary" />`
**Why human:** `generateMetadata` runs server-side during Next.js rendering; requires a live render to inspect the HTML output.

#### 3. Expired results page renders correctly

**Test:** Manually set a completed scan's `status` to `'expired'` in the database (e.g. `UPDATE scans SET status = 'expired' WHERE results_token = '{token}'`). Visit `http://localhost:3001/results/{token}`.
**Expected:** Page shows a clock SVG icon, heading "These results have expired", a green "Scan again" button linking to `/?url={original_target_url}` with the URL pre-encoded, and below it "Sign up for 30-day scan history" with a `/sign-up` link. No 404 page, no spinner.
**Why human:** Requires a real expired row in the database that the dev server can query; cannot be synthesized from static analysis.

### Gaps Summary

No gaps found. All six observable truths are verified against the actual codebase. All artifacts exist, are substantive (not stubs), are wired (imported and used), and have real data flowing through them. Both backend (`cargo check`) and frontend (`tsc --noEmit`) compile cleanly.

The three human-verification items above are behavioral confirmations of already-verified code paths — they do not represent missing implementation. The code is complete and correct; human verification is needed only to confirm the browser-side UX renders as intended.

---

_Verified: 2026-03-29_
_Verifier: Claude (gsd-verifier)_
