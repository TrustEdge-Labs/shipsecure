---
phase: 31-results-gating
verified: 2026-02-18T12:00:00Z
status: passed
score: 9/9 must-haves verified
re_verification: false
gaps: []
human_verification:
  - test: "Lock overlay visual appearance for high/critical findings"
    expected: "User sees a blurred overlay with a lock icon, severity label, scanner category, and 'Sign up free to view' button when expanding a gated finding"
    why_human: "Cannot verify CSS blur/overlay rendering or visual layout programmatically"
  - test: "Clerk SignUp modal opens on CTA click"
    expected: "Clicking 'Sign up free to view' opens the Clerk SignUp modal inline without navigating away from the results page"
    why_human: "openSignUp({}) invocation is wired but modal behavior requires browser execution with a live Clerk instance"
  - test: "Authenticated owner sees full details with no lock overlays"
    expected: "When signed in as the scan owner, all finding accordions expand to show description and remediation with no lock overlay"
    why_human: "Requires a live Clerk session token and a scan with matching clerk_user_id in the database"
---

# Phase 31: Results Gating Verification Report

**Phase Goal:** Anonymous users see teaser cards for high/critical findings that drive signup — and cannot bypass gating by calling the API directly

**Verified:** 2026-02-18T12:00:00Z
**Status:** PASSED
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #  | Truth                                                                                                       | Status     | Evidence                                                                                                          |
|----|-------------------------------------------------------------------------------------------------------------|------------|-------------------------------------------------------------------------------------------------------------------|
| 1  | `curl GET /api/v1/results/:token` without Authorization returns high/critical findings with null description and null remediation | VERIFIED | `results.rs:100-101` sets `"description": null, "remediation": null` when `is_gated=true`; `is_gated` triggers when `!owner_verified && matches!(severity, High | Critical)` |
| 2  | Each high/critical finding in the anonymous response has `gated: true`; medium/low have `gated: false`       | VERIFIED | `results.rs:105` sets `"gated": true` in the gated branch; `results.rs:116` sets `"gated": false` in the open branch; only High/Critical enter the gated branch |
| 3  | API response includes `owner_verified: false` when no Authorization header is present                        | VERIFIED | `results.rs:54-57`: `(Some, Some)` match only — `None` on either side yields `false`; `results.rs:140` places `"owner_verified": owner_verified` at top level |
| 4  | API response includes `owner_verified: true` when Authorization header contains a valid JWT matching `scan.clerk_user_id` | VERIFIED | `extract_optional_clerk_user` (results.rs:19-30) decodes Bearer JWT via `state.jwt_decoder.decode(token)`; match at `results.rs:55` returns `true` when IDs equal |
| 5  | Authenticated owner sees full description and remediation for all severity levels                             | VERIFIED | When `owner_verified=true`, `is_gated` is always `false` regardless of severity; all findings take the else branch at `results.rs:107-119` returning full fields |
| 6  | `download_results_markdown` also applies gating (prevents `curl /download` bypass)                           | VERIFIED | `results.rs:155` calls `extract_optional_clerk_user`; `results.rs:170-173` computes `owner_verified`; `results.rs:248-249` gates High/Critical in markdown output with sign-up CTA strings |
| 7  | Unauthenticated users see a lock overlay on high/critical findings with severity and category visible         | VERIFIED | `auth-gate.tsx:31` renders `{severity.charAt(0).toUpperCase() + severity.slice(1)} severity finding`; line 34 renders `{scannerName}`; both are inside the overlay div, not in the hidden children |
| 8  | Lock overlay has a "Sign up free" CTA button that calls Clerk `openSignUp({})`                               | VERIFIED | `auth-gate.tsx:13` destructures `openSignUp` from `useClerk()`; `auth-gate.tsx:37` wires `onClick={() => openSignUp({})}` to the button labeled "Sign up free to view" |
| 9  | Medium and low findings are always fully visible regardless of auth state                                    | VERIFIED | `results.rs:94-95`: `is_gated` requires `matches!(f.severity, Severity::High | Severity::Critical)` — medium/low always evaluate `is_gated=false` and return full JSON; frontend: `finding.gated` will be `false` for these severities |

**Score:** 9/9 truths verified

### Required Artifacts

| Artifact                                          | Expected                                              | Status   | Details                                                                                                          |
|---------------------------------------------------|-------------------------------------------------------|----------|------------------------------------------------------------------------------------------------------------------|
| `src/models/scan.rs`                              | Scan struct with `clerk_user_id` field                | VERIFIED | Line 39: `pub clerk_user_id: Option<String>` present in Scan struct after `created_at`                          |
| `src/db/scans.rs`                                 | All four `query_as::<_, Scan>` queries include `clerk_user_id` | VERIFIED | Lines 22, 42, 70, 160 — all four queries end with `clerk_user_id`; confirmed with grep (4 occurrences, 4 queries) |
| `src/api/results.rs`                              | Optional JWT extraction, owner_verified computation, gating logic | VERIFIED | `extract_optional_clerk_user` at lines 19-30; `owner_verified` at lines 54-57 and 170-173; gating at lines 94-119 and 247-258; `"owner_verified"` in response at line 140 |
| `frontend/lib/types.ts`                           | Finding with `gated?: boolean`, `description: string \| null`, `remediation: string \| null`; ScanResponse with `owner_verified: boolean` | VERIFIED | Lines 26/28: `string \| null`; line 31: `gated?: boolean`; line 61: `owner_verified: boolean` |
| `frontend/components/auth-gate.tsx`               | Lock overlay component with Clerk SignUp CTA, severity/category visible | VERIFIED | File exists (55 lines); `'use client'` directive; `useClerk` hook; `openSignUp({})`; severity and scannerName rendered in overlay; not stub |
| `frontend/components/finding-accordion.tsx`       | Wraps body in `<AuthGate>` with `finding.gated === true` | VERIFIED | Line 5: `import { AuthGate } from './auth-gate'`; lines 76-90: `<AuthGate gated={finding.gated === true} severity={finding.severity} scannerName={...}>` wraps description/remediation |
| `frontend/app/results/[token]/page.tsx`           | Server Component forwarding Clerk session JWT; conditional download button | VERIFIED | Lines 18-19, 67-68: `auth().getToken()` called in both `generateMetadata` and `ResultsPage`; lines 21-23, 70-72: conditional `Authorization` header; line 198: `{data.owner_verified && (...)}` hides download for non-owners |

### Key Link Verification

| From                                       | To                                                 | Via                                                             | Status   | Details                                                                                                                         |
|--------------------------------------------|----------------------------------------------------|-----------------------------------------------------------------|----------|---------------------------------------------------------------------------------------------------------------------------------|
| `src/api/results.rs`                       | `state.jwt_decoder` (AppState)                     | `extract_optional_clerk_user` calls `state.jwt_decoder.decode(token).await` | VERIFIED | `results.rs:28`: `state.jwt_decoder.decode(token).await.ok()?` — identical pattern to auth.rs; JWT decode is live, not stubbed |
| `src/api/results.rs`                       | `src/db/scans.rs` `get_scan_by_token`              | `scan.clerk_user_id` used in owner_verified computation         | VERIFIED | `results.rs:42-51`: `get_scan_by_token` called; `results.rs:54-57`: `&scan.clerk_user_id` used in match; column confirmed in query at db/scans.rs:160 |
| `src/api/results.rs`                       | JSON response                                      | `"gated"` field on each finding + `"owner_verified"` at top level | VERIFIED | `results.rs:105, 116`: `"gated": true/false`; `results.rs:140`: `"owner_verified": owner_verified` |
| `frontend/app/results/[token]/page.tsx`    | `GET /api/v1/results/:token`                       | `fetch` with `Authorization: Bearer <sessionToken>` header     | VERIFIED | `results/[token]/page.tsx:71`: `requestHeaders['Authorization'] = \`Bearer ${sessionToken}\``; `line 81`: passed to `fetch(..., { cache: 'no-store', headers: requestHeaders })` |
| `frontend/components/finding-accordion.tsx` | `frontend/components/auth-gate.tsx`               | `<AuthGate gated={finding.gated === true} ...>` wraps body     | VERIFIED | `finding-accordion.tsx:5`: import; `lines 76-90`: AuthGate wraps description+remediation block |
| `frontend/components/auth-gate.tsx`        | `@clerk/nextjs useClerk`                           | `openSignUp()` called on button click                           | VERIFIED | `auth-gate.tsx:3`: `import { useClerk } from '@clerk/nextjs'`; `line 13`: `const { openSignUp } = useClerk()`; `line 37`: `onClick={() => openSignUp({})}` |

### Requirements Coverage

| Requirement | Source Plan | Description                                                                                 | Status    | Evidence                                                                                      |
|-------------|-------------|---------------------------------------------------------------------------------------------|-----------|-----------------------------------------------------------------------------------------------|
| GATE-01     | 31-01       | API strips description/remediation from high/critical findings for anonymous scan tokens    | SATISFIED | `results.rs:94-106`: `is_gated` logic + `"description": null, "remediation": null` for anonymous high/critical |
| GATE-02     | 31-01       | API returns `gated: true` flag and `owner_verified` field on results                        | SATISFIED | `results.rs:105, 116`: `"gated": true/false` per finding; `results.rs:140`: `"owner_verified"` at top level |
| GATE-03     | 31-02       | Frontend renders teaser cards with lock overlay for gated findings                          | SATISFIED | `auth-gate.tsx`: full lock overlay with blur, icon, severity, category; `finding-accordion.tsx:76-90`: AuthGate wraps body |
| GATE-04     | 31-02       | Teaser cards show severity and category but not details, with "Sign up free" CTA            | SATISFIED | `auth-gate.tsx:31`: severity rendered; `line 34`: scannerName (category) rendered; `lines 36-44`: "Sign up free to view" button with `openSignUp({})` |

No orphaned requirements — all four GATE-xx IDs appear in plan frontmatter and are accounted for.

### Anti-Patterns Found

| File                               | Line | Pattern                          | Severity | Impact                                                                                         |
|------------------------------------|------|----------------------------------|----------|------------------------------------------------------------------------------------------------|
| `frontend/components/auth-gate.tsx` | 49   | Text "Placeholder for description height" | INFO | This is the invisible spacer div content (CSS class `invisible`) — intentional design decision to maintain accordion height; not a stub; never visible to users |
| `frontend/app/results/[token]/page.tsx` | 130 | `return null` in `getExpiryWarning` | INFO | Guard clause returning null when no expiry date — idiomatic React; not a stub |

No blocker or warning anti-patterns found. The "Placeholder" text in auth-gate.tsx is inside `<div className="invisible">` — it is never rendered visibly and is an intentional layout technique documented in the plan.

### Human Verification Required

#### 1. Lock overlay visual appearance

**Test:** Open a results page for an anonymous scan that has high or critical findings. Expand one of those findings.
**Expected:** The accordion body shows a blurred overlay with a lock icon, the severity label (e.g. "High severity finding"), the scanner category (e.g. "Headers"), and a "Sign up free to view" button — no description or remediation text is visible.
**Why human:** CSS `backdrop-blur-sm`, absolute positioning, and `bg-surface-elevated/90` opacity cannot be visually verified by grep; requires browser rendering.

#### 2. Clerk SignUp modal opens on CTA click

**Test:** On the same results page as above, click the "Sign up free to view" button inside a gated finding overlay.
**Expected:** The Clerk SignUp modal opens inline without a page navigation; the results page remains in the background.
**Why human:** `openSignUp({})` is wired correctly in code, but the modal behavior requires a live Clerk publishable key and browser execution.

#### 3. Authenticated owner sees full details with no lock overlays

**Test:** Sign in as the user who submitted the scan, navigate to that scan's results URL.
**Expected:** All finding accordions expand to show full description and remediation text; no lock overlays appear on any severity level; the "Download Markdown Report" button is visible.
**Why human:** Requires a live Clerk session whose `sub` claim matches `scan.clerk_user_id` in the database.

### Gaps Summary

No gaps found. All nine observable truths pass all three verification levels (exists, substantive, wired). The phase goal is achieved:

- **OWASP A01 bypass prevention:** The server strips description and remediation server-side before the response leaves. A `curl` call without a valid owner JWT receives `null` for those fields — the frontend cannot be bypassed.
- **`gated` and `owner_verified` fields:** Present in the JSON response at the correct levels. Frontend consumes them directly from server-rendered data.
- **Frontend lock overlay:** `AuthGate` is a real, substantive component — not a placeholder. It is imported and wired into `FindingAccordion`. Severity and category are rendered inside the overlay. The CTA calls `openSignUp({})`.
- **Auth token forwarding:** The Next.js Server Component correctly retrieves the Clerk session token and attaches it as `Authorization: Bearer` on both the main page fetch and the `generateMetadata` fetch.
- **Download gating:** Both `get_results_by_token` and `download_results_markdown` apply identical gating logic. The download button is also hidden from non-owners in the UI.
- **`cargo check`:** Passes with zero errors (3 pre-existing unrelated warnings in a different module).
- **All four task commits verified:** `44441d8`, `33c2b82`, `3c0ac6e`, `4eed227` exist in git history.

---

_Verified: 2026-02-18T12:00:00Z_
_Verifier: Claude (gsd-verifier)_
