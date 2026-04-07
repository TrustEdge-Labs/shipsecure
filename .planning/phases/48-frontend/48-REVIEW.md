---
phase: 48-frontend
status: issues_found
files_reviewed: 14
findings:
  critical: 1
  warning: 6
  info: 4
  total: 11
---

## Code Review: Phase 48 (Frontend)

### Findings

---

### CR-01: Unvalidated redirect via `data.share_url` (critical)
**File:** `frontend/components/supply-chain-form.tsx`
**Line:** 111
**Issue:** The backend response field `share_url` is passed directly to `router.push()` without any validation. If the backend returns a malicious or unexpected URL (e.g., `javascript:alert(1)`, or an open redirect to an external domain), the browser will follow it. While the backend is trusted here, the frontend should not assume trust on a value that came over the wire and will be used for navigation.
**Fix:** Validate `share_url` before navigating. At minimum, assert it starts with `/supply-chain/results/` (a relative path) or is a known absolute URL matching the app's origin. Example:
```ts
const url = new URL(data.share_url, window.location.origin)
if (url.origin === window.location.origin) {
  router.push(url.pathname)
} else {
  // fallback to inline
}
```

---

### CR-02: `NEXT_PUBLIC_BACKEND_URL` used as server-side base URL in actions (warning)
**File:** `frontend/app/actions/supply-chain-scan.ts`
**Line:** 48
**Issue:** `submitSupplyChainScan` is a plain async function called from a Client Component — it runs in the browser. The `process.env.NEXT_PUBLIC_BACKEND_URL` is baked in at build time. The dev `docker-compose.yml` bakes in `http://localhost:3000` as the `NEXT_PUBLIC_BACKEND_URL` build arg. In production, if `NEXT_PUBLIC_BACKEND_URL` is empty or wrong, the fetch falls back to a relative URL (`/api/v1/scans/supply-chain`), which is actually correct since Nginx proxies `/api/` on the same origin. However, the fallback (`?? ''`) is silent and produces a relative URL only accidentally. The intent is unclear and the build arg is fragile.
**Fix:** Either commit to always using relative URLs (remove `NEXT_PUBLIC_BACKEND_URL` usage from client-side code) or add a clear comment explaining the empty-string fallback is intentional for same-origin proxying. The CLAUDE.md already documents that client-side code should use relative URLs.

---

### CR-03: No file size validation on upload before sending (warning)
**File:** `frontend/components/supply-chain-form.tsx`
**Line:** 49–55
**Issue:** The drop zone UI displays "max 5 MB" but the `handleDrop` and `handleFileChange` handlers do not enforce any size limit before calling `submitSupplyChainScan`. A user can upload a multi-hundred-MB file, which will be sent to the backend and waste bandwidth/time. The backend may or may not reject it.
**Fix:** Add a client-side size check in both `handleDrop` and `handleFileChange`:
```ts
if (file.size > 5 * 1024 * 1024) {
  setValidationError('File must be under 5 MB')
  return
}
```

---

### CR-04: `sessionStorage` data parsed without runtime type validation (warning)
**File:** `frontend/app/supply-chain/results/[token]/inline-results-loader.tsx`
**Line:** 25
**Issue:** `JSON.parse(raw)` is cast directly to `SupplyChainScanResponse` with no runtime shape validation. If sessionStorage contains stale data from a previous schema version (or corrupted data), the app will silently render with wrong/missing fields — potentially throwing at render time on a property access (e.g., `results.infected.length`). A JSON parse error is caught and redirects, but a structurally invalid object that parses successfully is not caught.
**Fix:** Add a lightweight guard checking that the required top-level fields exist before accepting the parsed value, or use a validation library like `zod`. At minimum: check `parsed.results && Array.isArray(parsed.results.infected)`.

---

### CR-05: `setLoading(false)` not called on parse failure in `InlineResultsLoader` (warning)
**File:** `frontend/app/supply-chain/results/[token]/inline-results-loader.tsx`
**Line:** 27–30
**Issue:** When `JSON.parse` throws and the catch block runs, the function does an early `return` without calling `setLoading(false)`. The component will remain in the spinner state briefly before `router.replace` resolves. While the redirect masks this, if the redirect is slow the spinner is shown indefinitely. More importantly, `setScanResponse` is also never called, so `loading` stays `true` forever if the redirect doesn't happen.
**Fix:** Call `setLoading(false)` before returning in the catch block, or restructure to set loading only after data is confirmed valid.

---

### CR-06: Markdown report injects unsanitized user-controlled content (warning)
**File:** `src/api/results.rs`
**Line:** 304, 341–355
**Issue:** The markdown download (`download_results_markdown`) formats `scan.target_url`, `finding.title`, `finding.description`, and `finding.remediation` directly into a markdown string via `format!()`. A crafted target URL or finding title with markdown control characters (backticks, `###`, table pipe `|`, etc.) can corrupt the report structure. While there is no code execution risk from markdown injection, the report's table could be broken and the output is misleading for downstream consumers who render it.
**Fix:** Escape markdown special characters in user-controlled fields (at minimum `|`, `#`, `*`, `` ` ``) before inserting them into the template, or wrap them in code spans.

---

### CR-07: `Content-Disposition` filename not sanitized for header injection (warning)
**File:** `src/api/results.rs`
**Line:** 383–385
**Issue:** The `Content-Disposition` header value is built from `token_prefix` (the first 8 characters of the token). While tokens are presumably URL-safe alphanumerics, there is no explicit validation. If a token ever contains `"` or a newline (e.g., from a DB inconsistency or future format change), this could produce a malformed or injected header.
**Fix:** Sanitize `token_prefix` to only alphanumerics before interpolating into the header value: `token_prefix.chars().filter(|c| c.is_alphanumeric() || *c == '-').collect::<String>()`.

---

### CR-08: CSP uses `unsafe-inline` and `unsafe-eval` for scripts (info)
**File:** `frontend/next.config.ts`
**Line:** 8
**Issue:** The Content-Security-Policy allows `'unsafe-inline'` and `'unsafe-eval'` in `script-src`. These directives significantly weaken the XSS protection CSP is meant to provide. `unsafe-eval` is particularly dangerous as it permits `eval()`, `Function()`, `setTimeout(string)`, etc.
**Fix:** This is often unavoidable with Next.js in development, but in production it may be possible to use nonce-based CSP (`'nonce-{random}'`) or hash-based allowlisting. If Clerk or Plausible require `unsafe-inline`, document the constraint explicitly. At minimum, consider removing `unsafe-eval` if it isn't strictly required.

---

### CR-09: `NEXT_PUBLIC_BACKEND_URL` is used to construct CSP `connect-src` at build time (info)
**File:** `frontend/next.config.ts`
**Line:** 3–4, 12
**Issue:** The CSP `connect-src` directive is constructed at Next.js config evaluation time using `process.env.NEXT_PUBLIC_BACKEND_URL`. In the production Docker build, this env var is passed as a build ARG. If it's empty (e.g., when deploying to a new environment without the build arg), the `connect-src` directive omits the backend origin. This is silently permissive for same-origin fetches but could block cross-origin API calls without clear error messages.
**Fix:** Log a warning during build if `NEXT_PUBLIC_BACKEND_URL` is not set. Add a comment explaining when an empty value is expected vs. a misconfiguration.

---

### CR-10: `nav` breadcrumb missing `aria-label` in supply chain page (info)
**File:** `frontend/app/supply-chain/page.tsx`
**Line:** 27
**Issue:** The `<nav>` element used for the breadcrumb has no `aria-label` attribute. When multiple `<nav>` landmarks exist on a page (header nav + breadcrumb), screen readers cannot distinguish them without labels.
**Fix:** Add `aria-label="Breadcrumb"` to the breadcrumb `<nav>` element.

---

### CR-11: `formatDate` uses `toLocaleString` with no timezone — output varies by server locale (info)
**File:** `frontend/app/supply-chain/results/[token]/page.tsx`
**Line:** 83–89
**Issue:** `formatDate` is called in a Server Component. `new Date(dateStr).toLocaleString("en-US", {...})` will render using the server's system timezone (UTC on Docker, but this is implicit). Users in other timezones will see dates that appear to be in an unexpected timezone without any indicator. The `hour: "2-digit"` format also doesn't specify `hour12`, so the AM/PM vs 24h behavior depends on the locale's default.
**Fix:** Add `timeZone: 'UTC'` and `hour12: false` (or `true` with explicit notation) to the `toLocaleString` options, and append a `UTC` suffix to the displayed string so users understand the timezone context.
