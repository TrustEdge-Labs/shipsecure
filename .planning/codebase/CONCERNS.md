# Codebase Concerns

**Analysis Date:** 2026-02-21

## Tech Debt

**Unwrap/Panic Points in Production Code:**
- Issue: Regex patterns in `SecretPattern` initialization use `.unwrap()` at definition time, causing panic if any regex pattern is invalid
- Files: `src/scanners/js_secrets.rs:40`
- Impact: Application crashes on startup if any regex pattern has syntax errors, no graceful handling
- Fix approach: Validate all regex patterns at startup before creating SecretPattern, or compile them as compile-time constants using `lazy_static`

**Hardcoded Port and Configuration in SSRF Validator:**
- Issue: DNS resolution in SSRF validator assumes hardcoded fallback ports (443 for HTTPS, 80 for HTTP)
- Files: `src/ssrf/validator.rs:54`
- Impact: Custom port configuration not tested; may silently resolve wrong target IPs if non-standard ports are used
- Fix approach: Pass port through from original URL validation context, test non-standard port scenarios

**Cascading Optional/Unwrap Patterns in Container Scanner:**
- Issue: Multiple `.unwrap_or()` chains with fallback strings make it hard to distinguish between intended defaults and parsing failures
- Files: `src/scanners/container.rs` (multiple lines with `.unwrap_or("")`, `.unwrap_or("medium")`)
- Impact: Parsing errors silently fail with empty/default values; hard to debug malformed Nuclei output
- Fix approach: Use explicit error variants instead of defaults, log when fallbacks are triggered

**Database Connection Pool Size is Small:**
- Issue: Fixed max_connections = 10 in database pool
- Files: `src/main.rs:185`
- Impact: At max_concurrent_scans = 5 with each scan taking DB operations, connections may get exhausted if queries are long-running; no connection pool monitoring
- Fix approach: Make pool size configurable via env var, add metrics for pool usage, monitor connection wait times

**Hard-coded Regex.new().unwrap() Pattern:**
- Issue: Line 40 of `js_secrets.rs` creates Regex with unwrap, will crash if pattern is malformed
- Files: `src/scanners/js_secrets.rs:40`
- Impact: Runtime panic on invalid regex, affects startup reliability
- Fix approach: Use compile-time regex macro or validate all patterns during initialization

## Known Bugs

**Domain Verification Blocklist Missing Platforms:**
- Symptoms: Users of other shared hosting platforms (e.g., Heroku, Railway custom domains if applicable) may try to verify domains that should be blocked
- Files: `src/api/domains.rs:24`, `frontend/app/verify-domain/page.tsx:15`
- Trigger: Attempt to verify domain on a shared hosting platform not in BLOCKED_ROOT_TLDS list
- Workaround: List is maintained in both backend and frontend; manual deployment required to add new blocklisted platforms
- Fix: Centralize blocklist in database with automatic sync or make it a dynamic config file

**Frontend Error Handling in createScan:**
- Symptoms: Generic "Failed to start scan" error thrown from API error response without preserving rate limit details
- Files: `frontend/lib/api.ts:13`
- Trigger: Rate limit error from backend hits generic error handler
- Workaround: User sees generic error but backend responds with resets_at timestamp
- Fix: Frontend should detect rate limit errors and pass resets_at through to UI for user feedback

**Results Gating May Not Trigger on Slow Networks:**
- Symptoms: High/critical findings briefly visible before gating overlay renders
- Files: `frontend/app/results/[token]/page.tsx`
- Trigger: Network lag between initial render and auth state population
- Workaround: None currently
- Fix: Fetch findings only after auth state confirmed, or start with empty state and hydrate post-auth

**Cleanup Task Errors Are Logged But Not Retried:**
- Symptoms: If retention cleanup query fails, scan data may persist beyond expiry with no immediate retry
- Files: `src/cleanup.rs:37-42`
- Trigger: Database becomes temporarily unavailable or times out during cleanup window
- Workaround: Next hourly cleanup will attempt again; manual intervention needed if pattern repeats
- Fix: Implement exponential backoff for cleanup failures or switch to per-record cleanup on next access

## Security Considerations

**SSRF Validator Has Incomplete DNS Rebinding Protection:**
- Risk: Time-of-check/time-of-use (TOCTOU) vulnerability — DNS resolution checked, but connection made seconds later by remote scanner
- Files: `src/ssrf/validator.rs:34-65`, `src/scanners/container.rs`
- Current mitigation: Validates hostname resolution synchronously before returning to orchestrator
- Recommendations:
  - Document that scanner components (Nuclei, testssl.sh) are trusted and run in isolated sandbox
  - Consider DNS pinning if connecting directly from Rust (currently delegated to binaries)
  - Add logging of resolved IPs for debugging if TOCTOU incidents occur

**Bearer Token Authorization in Domain Verification Not Rate-Limited:**
- Risk: Verified domains endpoint accepts Clerk JWT; no rate limit on verification endpoint itself
- Files: `src/api/domains.rs` (verify-start, verify-confirm, verify-check)
- Current mitigation: Requires valid Clerk JWT; signup flow has rate limits on account creation
- Recommendations:
  - Add per-user rate limit on domain verification attempts (e.g., 10/hour) to prevent enumeration/brute force
  - Log verification failures by user ID for anomaly detection

**Frontend Authorization Token Passed in Bearer Header Over HTTP in Dev:**
- Risk: In development mode, Clerk secret key exposed if NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY is logged
- Files: `frontend/lib/api.ts:41-42` (correct, uses Bearer), `frontend/proxy.ts` (middleware)
- Current mitigation: CLERK_SECRET_KEY is server-only, not sent to client; frontend only uses public key
- Recommendations:
  - Ensure .env.example never includes SECRET_KEY values, even as placeholders
  - Document that CLERK_SECRET_KEY should never be committed even if rotated

**JavaScript Bundle Analysis May Leak Build Secrets:**
- Risk: If CI builds with API keys in environment, they may end up in source maps uploaded to scanner
- Files: `src/scanners/js_secrets.rs` (detects these), but frontend build process
- Current mitigation: Next.js build correctly separates `NEXT_PUBLIC_*` from server-only vars
- Recommendations:
  - Audit CI build process to ensure no server env vars are included in bundles
  - Document that NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY is the only public credential

**Rate Limit Reset Times Not Validated Against Clock Skew:**
- Risk: Client clock ahead of server may see `resets_at` in the past
- Files: `src/api/scans.rs:42` (first_of_next_month_utc), `src/api/errors.rs:46`
- Current mitigation: Sent as RFC3339 timestamp; client just displays it
- Recommendations:
  - Document that reset times are UTC server time, not client time
  - Consider returning `reset_seconds_remaining` as alternative for better client-side UX

## Performance Bottlenecks

**JavaScript Scanner (js_secrets.rs) May Perform Redundant Regex Matching:**
- Problem: Each finding may iterate through all SECRET_PATTERNS regex list, even after match
- Files: `src/scanners/js_secrets.rs`
- Cause: Pattern matching is sequential; no early-exit optimization between patterns
- Improvement path:
  - Profile JS file scanning with large bundles (>10MB source)
  - Consider combining patterns with alternation if matching multiple patterns per file
  - Cache compiled regexes more efficiently (currently done via lazy_static)

**Orchestrator Scanner Timeout is Fixed at 60 Seconds:**
- Problem: All scanners (headers, TLS, secrets, vibecode, detection, exposed files) share 60s timeout
- Files: `src/orchestrator/worker_pool.rs:68`
- Cause: Single `max_scanner_timeout` doesn't account for scanner-specific complexity
- Improvement path:
  - Profile slowest scanner (likely js_secrets on large bundles)
  - Make timeout configurable per scanner via env var
  - Add timeout telemetry to identify which scanners regularly hit limits

**Database Query N+1 on Scan History:**
- Problem: Fetching scan history may issue individual queries for finding counts if not optimized
- Files: `src/db/scans.rs` (ScanHistoryRow query), `src/api/users.rs`
- Cause: If finding count aggregation is done in application code per scan
- Improvement path:
  - Verify single query uses GROUP BY to aggregate findings by scan_id and severity
  - Add index on scans(clerk_user_id, created_at) for history ordering
  - Benchmark with 100+ scans per user

**TLS Scanner Uses External testssl.sh Binary Synchronously:**
- Problem: Spawning subprocess and reading output blocks async runtime
- Files: `src/scanners/tls.rs` (uses container.rs subprocess)
- Cause: testssl.sh is external binary, no async/streaming output available
- Improvement path:
  - Ensure testssl.sh runs with strict timeout (configurable, currently 60s global)
  - Consider pre-warming testssl.sh or caching SSL Lab results for popular domains
  - Monitor subprocess spawn time on busy servers

**Memory Usage of Large JavaScript Bundles in Scanning:**
- Problem: Loading entire JS bundle source into memory before analysis
- Files: `src/scanners/js_secrets.rs` (fetches all JS files)
- Cause: No streaming/chunked processing; scopes limited by tier (20 vs 30 files)
- Improvement path:
  - Profile memory usage with 30 large bundles (>1MB each)
  - Implement early termination if secrets found in first N patterns
  - Stream-process if possible or split processing across workers

## Fragile Areas

**Domain Verification Meta Tag Extraction Using Simple Scraper:**
- Files: `src/api/domains.rs` (verify_check function)
- Why fragile: Uses CSS selector matching on unvalidated HTML; malformed HTML, comments with meta tags, or obfuscation can break verification
- Safe modification:
  - Always test with edge cases (minified HTML, conditionally rendered meta tags via JS)
  - Add integration tests with real domain verification flows
  - Log parsed HTML snippet on verification failure for debugging
- Test coverage: Verify works with both static and dynamically-inserted meta tags (currently may fail if meta tag is JS-injected)

**Scanner Container Orchestration with Semaphore-Based Concurrency:**
- Files: `src/orchestrator/worker_pool.rs` (semaphore logic)
- Why fragile: No backpressure when semaphore is exhausted; new requests just wait indefinitely
- Safe modification:
  - Change concurrency limits only via env var restart, never at runtime
  - Monitor semaphore available_permits() in health check
  - Add maximum wait time for semaphore acquisition (timeout if queue grows)
- Test coverage: Test behavior under overload (> MAX_CONCURRENT_SCANS requests arriving simultaneously)

**Frontend Clerk Integration via Proxy Middleware:**
- Files: `frontend/proxy.ts` (middleware for auth routing)
- Why fragile: Clerk session cookies + JWT are handled by middleware; misconfiguration blocks all protected routes
- Safe modification:
  - Do not move or rename Clerk environment variables without updating CI secrets
  - Test sign-in and sign-up flows on deploy (currently tested in E2E via Playwright)
  - Verify protected routes (/dashboard, /verify-domain) behind auth gate
- Test coverage: E2E tests exist but may not catch timing issues (e.g., session expired during test)

**Remediation Playbook Text in Database:**
- Files: `src/scanners/remediation.rs` (static remediation logic)
- Why fragile: Remediation steps are hardcoded strings; grammar/clarity issues can't be hot-fixed
- Safe modification:
  - Changes require rebuild + redeploy (no versioning per finding)
  - Coordinate updates with product/docs team
  - Test remediation rendering in UI with various finding types
- Test coverage: No unit tests for remediation text; only verified in E2E results page

**Exposure of Findings by Capability Token Without Rate Limiting:**
- Files: `src/api/results.rs` (GET /api/v1/results/{token})
- Why fragile: Results can be shared via unguessable token; no rate limit on result fetches (attacker could attempt token prediction)
- Safe modification:
  - Tokens are 256-bit random (cryptographically sound)
  - But no throttling on failed token attempts
  - Add rate limiting per IP on result endpoints (429 after 100 failures/hour)
- Test coverage: Test invalid token behavior and error messages

## Scaling Limits

**Semaphore-Based Scan Concurrency Has Hard Limit:**
- Current capacity: MAX_CONCURRENT_SCANS (default 5)
- Limit: Machine CPU cores and I/O bandwidth; Nuclei + testssl.sh are CPU-heavy
- Scaling path:
  - Horizontally: Spin up multiple backend instances with load balancer; use shared database queue (already architecture supports this)
  - Increase MAX_CONCURRENT_SCANS carefully; profile CPU/memory on single instance first
  - Monitor active scan count via GET /health endpoint metrics

**Database Pool (10 connections) Insufficient for High Scan Load:**
- Current capacity: 10 PgPool connections shared across HTTP server + background jobs
- Limit: With 5 concurrent scans + health checks + API requests = connection exhaustion
- Scaling path:
  - Increase pool size to 20-30 and monitor connection wait times with metrics
  - Consider separate pools for OLTP (API) vs background cleanup tasks
  - Add query timeout at database level (statement_timeout in PostgreSQL)

**JavaScript Bundle Analysis Limited by Network I/O:**
- Current capacity: 20 files (free tier) / 30 files (authenticated tier) per scan
- Limit: Each file fetched sequentially; large bundles (>1MB each) cause timeout
- Scaling path:
  - Parallel fetching up to 5-10 concurrent requests with circuit breaker
  - Implement smart file size filtering (skip files > 2MB)
  - Cache popular JS files (bootstrap, jQuery) to reduce network roundtrips

**Email Sending Through Resend API Has Rate Limits:**
- Current capacity: Not checked; assumes Resend quota sufficient
- Limit: Resend free tier may have limits; no internal rate limiting on email sends
- Scaling path:
  - Add optional queue (Redis/Bull) for email delivery if Resend rate limits are hit
  - Log failed email sends and implement retry with exponential backoff
  - Monitor Resend API health via GET /api/v1/domains/verify-start (sends verification email)

## Scaling Limits (Continued)

**Single Database Instance Not Horizontally Scalable:**
- Current capacity: PostgreSQL 16 with typical 10-100GB storage depending on retention policy
- Limit: Write bottleneck for scan creation/updates; no replication or sharding
- Scaling path:
  - Set up PostgreSQL primary-replica for read scaling (replicas can serve health checks, scan history queries)
  - Use read replicas for analytics/reporting only, writes still go to primary
  - Consider connection pooling layer (PgBouncer) if connection count becomes issue

## Dependencies at Risk

**Deprecated Rust Edition in Cargo.toml:**
- Risk: Edition 2024 is experimental; may break in future Rust versions
- Files: `Cargo.toml:4`
- Impact: Undefined behavior or compilation errors with future rustc versions; no stable release date announced
- Migration plan: Switch to edition 2021 (stable) as soon as feasible; file issue to track

**Next.js 16 Rapid Release Cycle:**
- Risk: Major versions every 6-12 months; breaking changes in minor versions possible
- Files: `frontend/package.json:19` (next: 16.1.6)
- Impact: Security patches in older minors may be backported inconsistently; new breaking API every release
- Migration plan: Pin to latest LTS when available; test upgrades in CI before adopting breaking changes

**Playwright Test Dependency Version Mismatch Risk:**
- Risk: Playwright browser versions must match test runner; mismatches cause flaky E2E tests
- Files: `frontend/package.json:26` (@playwright/test: 1.58.2)
- Impact: CI may fail if browser cache is stale or version skew between runner and chromium
- Migration plan: Lock chromium version in playwright.config.ts; update both test runner and browsers together

**External Security Scanner Binaries (Nuclei, testssl.sh) Unverified:**
- Risk: No signature verification of downloaded binaries; potential for supply chain attack
- Files: `src/scanners/container.rs` (spawns external processes)
- Impact: Compromised Nuclei/testssl.sh binary could run arbitrary code in scan sandbox
- Migration plan:
  - Document how to verify binary checksums before deployment
  - Consider pinning exact versions in Dockerfile instead of latest
  - Add cryptographic signature verification if binaries support it

## Missing Critical Features

**No Audit Trail or Scan History for Unauthenticated Users:**
- Problem: Free tier users cannot view past scans; email is only recovery method
- Blocks: Users cannot track vulnerability fixes over time without signup
- Fix approach: Store scan results in browser localStorage with token, or offer temporary access links that expire after 7 days

**No Email Notifications on Scan Completion:**
- Problem: Async scan design means user must poll; no email notification when ready
- Blocks: User experience is poor; users check results page multiple times
- Fix approach: Send email with results link on completion (requires integration with Resend backend)

**No API Key or Integration Support:**
- Problem: No way for developers to automate scanning in CI/CD pipeline
- Blocks: Integration with GitHub Actions, GitLab CI, or other workflows
- Fix approach: Add API key management to authenticated tier; document rate limits per key

**No Custom Rules or Exclusion List:**
- Problem: All findings are reported; no way to suppress known false positives
- Blocks: Organizations cannot tailor scanner output
- Fix approach: Add exclusion list per domain (e.g., "ignore CSP warnings on subdomains")

## Test Coverage Gaps

**Backend (Rust) Has No Unit Tests:**
- What's not tested: Scanner logic (headers, TLS, secrets, vibecode, exposed files), database operations, rate limiting
- Files: `src/` directory (all files)
- Risk: Refactoring scanners could inadvertently break detection without regression test
- Priority: High — scanner correctness is critical to product value

**Frontend Component Tests Missing Edge Cases:**
- What's not tested: Error states, loading states, network timeouts, auth state race conditions
- Files: `frontend/__tests__/components/` — tests exist but coverage gaps
- Risk: UI may show loading spinner forever if API fails, or show auth-gated content to anonymous users
- Priority: Medium — most covered by E2E tests but component-level isolation helps debugging

**Domain Verification Has No Integration Tests:**
- What's not tested: Real meta tag injection, verification with various HTML structures, verification expiry
- Files: `src/api/domains.rs` (verify_check function)
- Risk: Verification may fail silently for certain HTML structures (e.g., meta tag in <head> vs <body>)
- Priority: High — domain verification is critical for authenticated tier

**Rate Limiting Logic Not Unit Tested:**
- What's not tested: Edge cases around month boundaries, timezone handling, concurrent requests hitting limit simultaneously
- Files: `src/rate_limit/` module
- Risk: Users may bypass rate limits during month rollover or receive inconsistent error messages
- Priority: High — rate limiting protects API from abuse

**SSRF Validator Missing IPv6 Edge Cases:**
- What's not tested: IPv6 loopback (::1), IPv6 link-local (fe80::/10), IPv6 private ranges
- Files: `src/ssrf/validator.rs` — only partial IPv6 coverage in check_ipv6_blocked
- Risk: SSRF protection incomplete for IPv6; could scan internal services via IPv6 localhost
- Priority: High — security-critical

**Graceful Shutdown Not Tested in CI:**
- What's not tested: SIGTERM signal handling, in-flight scan completion during shutdown, connection draining
- Files: `src/main.rs` (shutdown_signal, TaskTracker usage)
- Risk: Data loss or incomplete scans during rolling deploys if shutdown hangs
- Priority: Medium — operational reliability

---

*Concerns audit: 2026-02-21*
