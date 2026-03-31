---
phase: 42-funnel-unlock
verified: 2026-03-30T23:00:00Z
status: passed
score: 6/6 must-haves verified
re_verification: false
---

# Phase 42: Funnel Unlock Verification Report

**Phase Goal:** Any visitor can scan any URL with no demo lockdown, and authenticated users can scan without domain verification
**Verified:** 2026-03-30T23:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| #  | Truth                                                                                     | Status     | Evidence                                                                                    |
|----|------------------------------------------------------------------------------------------|------------|---------------------------------------------------------------------------------------------|
| 1  | Anonymous scans are blocked after 3 from the same IP per day                             | ✓ VERIFIED | `ANONYMOUS_IP_DAILY_HARD_CAP: i64 = 3` in middleware.rs; error message confirmed in code   |
| 2  | A domain scanned 5 times in the past hour returns cached results instead of re-scanning  | ✓ VERIFIED | `PER_TARGET_HOURLY_CAP: i64 = 5`; returns `Ok(Some(cached_scan.id))`; wired in scans.rs   |
| 3  | Authenticated user can scan any URL without domain verification                           | ✓ VERIFIED | Domain verification gate deleted from scans.rs; no `is_domain_verified` references remain  |
| 4  | Anonymous user can paste any public URL into the scan form                                | ✓ VERIFIED | Single always-enabled `<input id="url" name="url">` in scan-form.tsx; no disabled/hidden   |
| 5  | No demo-only messaging or Juice Shop lockdown visible to anonymous users                  | ✓ VERIFIED | `DEMO_TARGET_URL`, hidden input, and demo paragraph fully removed from scan-form.tsx        |
| 6  | E2E tests validate anonymous users can enter custom URLs                                  | ✓ VERIFIED | error-flows.spec.ts has "anonymous user can enter custom URL" test; free-scan.spec.ts fills URL explicitly |

**Score:** 6/6 truths verified

---

### Required Artifacts

| Artifact                               | Provides                                              | Exists | Substantive | Wired  | Status     |
|---------------------------------------|-------------------------------------------------------|--------|-------------|--------|------------|
| `src/rate_limit/middleware.rs`        | Two-layer anonymous rate limiting                     | ✓      | ✓           | ✓      | ✓ VERIFIED |
| `src/db/scans.rs`                     | Per-target hourly count + cached scan lookup          | ✓      | ✓           | ✓      | ✓ VERIFIED |
| `src/api/scans.rs`                    | Scan creation without domain verification gate        | ✓      | ✓           | ✓      | ✓ VERIFIED |
| `frontend/app/actions/scan.ts`        | Server action without domain verification check       | ✓      | ✓           | ✓      | ✓ VERIFIED |
| `frontend/components/scan-form.tsx`   | Editable URL input for all users                      | ✓      | ✓           | ✓      | ✓ VERIFIED |
| `frontend/e2e/error-flows.spec.ts`    | Updated error tests without Juice Shop lockdown       | ✓      | ✓           | ✓      | ✓ VERIFIED |
| `frontend/e2e/free-scan.spec.ts`      | Free scan flow with user-entered URL fill             | ✓      | ✓           | ✓      | ✓ VERIFIED |

---

### Key Link Verification

| From                            | To                              | Via                                     | Status     | Details                                                                   |
|---------------------------------|---------------------------------|-----------------------------------------|------------|---------------------------------------------------------------------------|
| `src/api/scans.rs`              | `src/rate_limit/middleware.rs`  | `rate_limit::check_rate_limits` call    | ✓ WIRED    | Line 135: `rate_limit::check_rate_limits(...)` with full args, returns `Option<Uuid>` |
| `src/rate_limit/middleware.rs`  | `src/db/scans.rs`               | `scans::count_scans_by_domain_last_hour` call | ✓ WIRED | Line 51: per-target count called before IP cap; cached scan returned on hit |
| `src/api/scans.rs`              | cached response branch          | `if let Some(scan_id) = cached_scan_id` | ✓ WIRED    | Lines 145-153: returns 200 with `"cached": true` JSON body                |
| `frontend/components/scan-form.tsx` | `frontend/app/actions/scan.ts` | `name="url"` form field submission     | ✓ WIRED    | `formAction` from `useActionState(submitScan,...)`; URL input carries `name="url"` for all users |

---

### Data-Flow Trace (Level 4)

| Artifact                        | Data Variable   | Source                                           | Produces Real Data | Status     |
|---------------------------------|-----------------|--------------------------------------------------|--------------------|------------|
| `src/api/scans.rs`              | `cached_scan_id`| `count_scans_by_domain_last_hour` + `get_recent_completed_scan_for_domain` DB queries | Yes — live SQL COUNT + SELECT | ✓ FLOWING |
| `frontend/components/scan-form.tsx` | `state.scanId` | `submitScan` server action → backend `/api/v1/scans` | Yes — backend returns real scan ID | ✓ FLOWING |

---

### Behavioral Spot-Checks

| Behavior                                       | Check                                                              | Result                | Status  |
|------------------------------------------------|--------------------------------------------------------------------|-----------------------|---------|
| `cargo check` compiles without errors          | `cargo check 2>&1 \| tail -5`                                     | `Finished dev profile` | ✓ PASS  |
| TypeScript compiles without errors             | `npx tsc --noEmit`                                                 | No output (clean)     | ✓ PASS  |
| IP daily cap is 3 (not 10)                     | `grep "ANONYMOUS_IP_DAILY_HARD_CAP: i64 = 3" middleware.rs`       | 1 match               | ✓ PASS  |
| Per-target cap is 5                            | `grep "PER_TARGET_HOURLY_CAP: i64 = 5" middleware.rs`             | 1 match               | ✓ PASS  |
| No domain verification in scans.rs             | `grep -c "is_domain_verified" src/api/scans.rs`                   | 0                     | ✓ PASS  |
| No domain verification in scan.ts              | `grep -c "DOMAIN_VERIFICATION_REQUIRED" frontend/app/actions/scan.ts` | 0                 | ✓ PASS  |
| No Juice Shop lockdown in scan-form.tsx        | `grep -c "DEMO_TARGET_URL" frontend/components/scan-form.tsx`     | 0                     | ✓ PASS  |
| URL input is always enabled (no `disabled=` on URL field) | `grep "disabled=" scan-form.tsx` | Submit button only (line 118) | ✓ PASS |
| Rate limit returns `Option<Uuid>`              | `grep "Result<Option<Uuid>, ApiError>" middleware.rs`             | 1 match               | ✓ PASS  |
| Quota text updated to 3/day                    | `grep "3 free scans per day" scan-form.tsx`                       | 1 match               | ✓ PASS  |

---

### Requirements Coverage

| Requirement | Source Plan | Description                                                       | Status      | Evidence                                                           |
|-------------|------------|-------------------------------------------------------------------|-------------|--------------------------------------------------------------------|
| FUNNEL-01   | 42-02      | Anonymous user can scan any URL (not locked to demo target)       | ✓ SATISFIED | scan-form.tsx: single editable input, no hidden/disabled URL field |
| FUNNEL-02   | 42-01      | Anonymous scans rate-limited to 3 per IP per day                  | ✓ SATISFIED | `ANONYMOUS_IP_DAILY_HARD_CAP = 3` in middleware.rs; enforced in anonymous branch |
| FUNNEL-03   | 42-01      | Per-target rate limit 5/domain/hour returning cached results      | ✓ SATISFIED | `PER_TARGET_HOURLY_CAP = 5`; `count_scans_by_domain_last_hour` + `get_recent_completed_scan_for_domain` wired; returns `Ok(Some(id))` |
| FUNNEL-04   | 42-01      | Authenticated user can scan any URL without domain verification   | ✓ SATISFIED | Domain verification gate removed from scans.rs; no `is_domain_verified` references anywhere in modified files |

All 4 requirements declared across plans are satisfied. No orphaned requirements: FUNNEL-05/06/07 are mapped to Phase 43 in REQUIREMENTS.md and were never claimed by this phase.

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/rate_limit/middleware.rs` | 117 | Test comment: "Leaving as a placeholder for integration tests" | ℹ️ Info | Inside `#[ignore]` test — not user-visible code. Pre-existing, not introduced by this phase. |
| `frontend/components/scan-form.tsx` | 69, 86 | HTML `placeholder=` attributes | ℹ️ Info | Legitimate HTML input placeholder text for UX — not stub code. |

No blocker or warning anti-patterns found. The `disabled={pending}` on the submit button (line 118) is correct — disabling during async submission prevents double-submit, and it does not affect the URL input.

---

### Human Verification Required

#### 1. Anonymous scan form renders correctly in browser

**Test:** Open https://shipsecure.ai in a private/incognito browser tab (not logged in). Observe the scan form.
**Expected:** URL input field is editable and focused, no "Demo only" or Juice Shop messaging visible, bottom text reads "3 free scans per day. Sign in for 5 scans/month and scan history."
**Why human:** Visual rendering, CSS class application, and real-browser behavior cannot be verified from source alone.

#### 2. Rate limit error message shown after 3 anonymous scans

**Test:** Submit 3 scans from the same IP address without logging in. Submit a 4th.
**Expected:** 4th scan returns an error in the form: "You've used your 3 free scans today. Sign up for 5 scans/month and scan history." with a sign-in link visible.
**Why human:** Requires live DB state and real HTTP requests to test IP counting.

#### 3. Per-target caching returns existing scan ID

**Test:** As an anonymous user, submit a scan for the same URL 6 times within one hour (or coordinate with someone else on a different IP to hit the same domain 5 times, then submit one more).
**Expected:** The 6th request returns an existing scan ID with `cached: true` in the response instead of creating a new scan.
**Why human:** Requires real DB state with 5+ completed scans for the same domain within the test window.

#### 4. Authenticated user can scan any arbitrary URL

**Test:** Log in to shipsecure.ai, navigate to the home page, enter a URL other than the OWASP Juice Shop (e.g., https://vercel.app or your own app), submit.
**Expected:** Scan starts normally — no "domain not verified" error, no domain ownership gate.
**Why human:** Requires active Clerk session and live backend to verify no domain gate fires.

---

### Gaps Summary

No gaps. All 6 observable truths are verified, all artifacts are substantive and wired, all key links confirmed, both compilers pass cleanly, and all 4 FUNNEL requirements (FUNNEL-01 through FUNNEL-04) are satisfied by real implementation evidence in the codebase.

The phase goal — "Any visitor can scan any URL with no demo lockdown, and authenticated users can scan without domain verification" — is fully achieved.

---

_Verified: 2026-03-30T23:00:00Z_
_Verifier: Claude (gsd-verifier)_
