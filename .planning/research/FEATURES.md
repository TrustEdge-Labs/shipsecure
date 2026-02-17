# Feature Landscape: Auth, Domain Verification & Tiered Access

**Domain:** Auth/tier gating for security scanning SaaS targeting vibe-coders
**Researched:** 2026-02-17
**Milestone:** v1.6 Auth & Tiered Access
**Overall confidence:** HIGH (Clerk docs official, SaaS gating patterns widely documented, domain verification is a well-established industry pattern)

---

## Table Stakes

Features users expect. Missing = product feels incomplete or untrustworthy.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Clerk SignIn/SignUp embedded components** | Industry-standard auth flow. Redirecting to external hosted page creates friction and erodes trust for security tool. | Low | Embed `<SignIn />` / `<SignUp />` in dedicated `/sign-in` and `/sign-up` routes. Use modal mode for CTA from results page to preserve context. |
| **Google/GitHub OAuth** | Solo devs expect social login. Email+password alone feels dated. Signup abandonment increases without SSO options. | Low | Clerk handles OAuth automatically — configure providers in Clerk Dashboard, zero backend code. |
| **Persistent sessions across browser restarts** | Users expect to stay logged in. Expiring sessions = perceived brokenness. | Low | Clerk manages session cookies automatically. Short-lived JWT (__session cookie) + long-lived cookie on Clerk domain. |
| **UserButton in header** | Standard SaaS pattern — avatar/initials dropdown with "Sign out" and "Manage account". Users expect this. | Low | `<UserButton />` component renders the Google-style user menu. Drop into sticky header alongside existing nav. |
| **Protected dashboard routes** | `/dashboard`, `/settings` must redirect unauthenticated users to sign-in. | Low | `clerkMiddleware()` in `middleware.ts` with `auth().protect()` on route groups. |
| **Email address as user identity** | Users expect their account tied to email, not just OAuth UID. | Low | Clerk captures email during signup. Expose via `currentUser().emailAddresses[0].emailAddress` in server components. |
| **Domain ownership verification** | Cannot let users run authenticated scans against sites they don't own. Verification is a table-stakes security requirement for the product. | Medium | Two methods: meta tag (add `<meta name="shipsecure-verification" content="[token]">`) or file upload (`/.well-known/shipsecure-verification.txt`). Backend polls on demand. |
| **Scan quota display** | Users need to see how many scans remain. No visibility = frustration and confusion at the paywall. | Low | Show "3 of 5 scans used this month" in header or dashboard. Resets at month boundary. |
| **Results gating for anonymous scans** | The core conversion mechanic. Low/med findings shown in full; high/critical shown as teaser cards with "Sign up to see full details." | Medium | Severity-based conditional rendering in results page. Server-side: never return finding details for high/critical on anonymous scans — return count + severity only. |
| **Scan history list** | Authenticated users expect to see their past scans. Without this, the Developer tier offers no persistence advantage over anonymous. | Medium | Simple list: domain, scan date, severity summary (N critical, N high, N med, N low), link to full results. |
| **Data retention enforcement** | Anonymous scans deleted after 24hr. Developer scans after 7-14 days. Users must see expiry. | Medium | Backend cron/cleanup job. Show "expires in 2 days" timestamp in scan history. |
| **Rate limit feedback** | When a user hits their quota, show a clear, non-hostile message. Silent 429s are a conversion killer. | Low | Display "You've used all 5 scans this month. Resets February 28." with upgrade CTA. |

---

## Differentiators

Features that set ShipSecure apart. Not universally expected, but high value for conversion and retention.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **FOMO teaser cards for high/critical findings** | Anonymous users see locked cards with finding category (e.g., "Critical: API Key Exposed in JavaScript Bundle") but not the file path or value. Creates real urgency because the finding is real, not fabricated. Overlay pattern shows 17% average conversion, up to 40% for engaged users. | Medium | Client component: render teaser card with blurred/redacted content and lock icon overlay. "Sign up free to see 3 critical findings" CTA inside the card. Server enforces — finding details never in the API response for anonymous tier. |
| **Inline upgrade prompts at quota limit** | Warn at 80% quota (4/5 scans used), not just at 100%. Proactive warning increases conversion 31.4% vs. silent limit. | Low | Track scan count in user metadata or DB. Frontend banner: "1 scan remaining this month — upgrade for unlimited." |
| **Post-scan signup flow** | After anonymous scan completes, if high/critical findings exist, show modal: "We found 3 critical vulnerabilities. Sign up free to see the full details and remediation steps." | Medium | Modal triggered by scan results page when `anonymousTier && criticalCount > 0`. Clerk SignUp in modal mode preserves current page context. After signup, redirect back to results and reveal findings. |
| **Domain verification status in dashboard** | Show verified badge on domain. "Verified" green check creates trust and communicates security posture. | Low | Domain verification status row in dashboard. States: unverified, pending (user added tag, awaiting check), verified, failed. |
| **Scan comparison across time** | "Last scan vs. this scan" delta. Vibe-coders ship fast — seeing regressions since last scan is high value. | High | Requires structured findings schema that enables diff. Defer to a phase after scan history is solid. |
| **One-click re-scan** | From scan history, "Re-scan" button re-runs the same URL with current scan config. | Low | POST to existing scan endpoint with pre-filled URL. Deduct quota. |
| **Scan share link for verified sites** | Authenticated users can generate a shareable results URL for their verified domain. | Low | Extend existing capability URL pattern. Restrict generation to verified domain owners. |
| **Onboarding checklist for new signups** | After signup: "1. Verify your site 2. Run your first full scan 3. Fix a finding." Turns empty state into a guided journey. | Low | Simple client-side progress tracker. Not a heavy onboarding tool — just 3 steps shown in dashboard empty state. |

---

## Anti-Features

Features to explicitly NOT build in v1.6.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **DNS TXT record verification** | DNS management is opaque and intimidating for vibe-coders (the target user). They often use Vercel/Netlify managed DNS and may not have direct DNS access. High abandonment risk. | Use meta tag (requires access to HTML/head) or file upload (requires access to deploy pipeline). Both are natural for developers. |
| **Email verification before scan** | Adds a round-trip before the user sees value. Anonymous scan already captures email for follow-up. Don't gate the scan behind email confirmation. | Let anonymous scan complete immediately. Email verification happens naturally as part of Clerk signup flow for Developer tier. |
| **Custom auth from scratch (JWT, sessions, passwords)** | Clerk solves this completely. Building auth is weeks of work, creates security liability, and produces an inferior UX. | Use Clerk. This is not a differentiator to build in-house. |
| **Storing Clerk user data locally in PostgreSQL** | Duplicating Clerk's user table creates sync complexity and double-source-of-truth bugs. | Use Clerk user ID as foreign key in the local `users` table. Store only: `clerk_user_id`, `tier`, `scan_count_this_month`, `quota_reset_at`, `created_at`. Fetch email/name from Clerk on demand. |
| **Scan gating on email verification** | Requiring verified email before any authenticated scan is overkill for a security scanner. | Require domain verification before *authenticated full scans*, not before account creation. Clerk handles email verification for password signups automatically. |
| **Multi-user organizations/teams** | Significant complexity. Target user is solo vibe-coder. Org management is a Pro+ feature. | One user = one account. Pro tier (future) can add team seats. |
| **Custom role/permission system beyond tier** | Over-engineering. Three tiers (anonymous, developer, pro) map cleanly to a tier field on the user record. | Store `tier: "anonymous" | "developer" | "pro"` in DB. Check tier in Rust API handlers. |
| **Stripe subscription for Developer tier** | Developer tier is free. Adding Stripe complexity now for a free tier is waste. | Stripe only when Pro tier is built. Developer signup is just Clerk auth + DB record. |
| **Real-time quota countdown** | WebSocket for quota tracking is overengineered. Users don't need millisecond quota updates. | Poll quota on page load. Update optimistically on scan completion. |

---

## Feature Dependencies

```
Clerk SDK installed & configured
  → clerkMiddleware() in Next.js middleware
    → Protected routes (/dashboard, /settings, /verify)
    → auth() helper in server components
    → currentUser() for user identity
      → User record created in PostgreSQL on signup (via Clerk webhook user.created)
        → Tier field (default: "developer")
        → Scan quota fields (scan_count_this_month, quota_reset_at)
        → Domain verification table (user_id FK, domain, verification_token, status, verified_at)

Domain Verification Flow
  → Requires: Clerk auth (user must be signed in)
  → Requires: PostgreSQL domain table
  → Generates: unique verification token (stored in DB)
  → Backend check: HTTP GET to domain for meta tag OR file
    → On success: update domain.status = "verified", domain.verified_at = now()
  → Unlocks: authenticated full scans against verified domain

Scan Tier Logic (Backend, Rust)
  → Requires: Clerk user ID extracted from JWT (via Authorization header or session cookie)
  → Requires: DB lookup of user tier and quota
  → Anonymous tier: rate limit by IP + email, return full low/med, return teaser for high/critical
  → Developer tier: rate limit by user ID (3-5/month), require verified domain, return full results

Results Gating (Frontend, Next.js)
  → Anonymous: server returns FindingSummary{severity, category, count} for high/critical (NO details)
  → Developer: server returns FullFinding{severity, category, location, remediation} for all
  → Teaser card component: renders locked state when finding.details is null
    → Lock icon overlay + "Sign up to see 3 critical findings" CTA
    → Clerk SignUp in modal mode (preserves results page)
    → After auth redirect: results page re-fetches with auth header, findings unlock

Scan History Dashboard
  → Requires: Clerk auth
  → Requires: scans table with clerk_user_id column (add in migration)
  → Lists scans by user, sorted by created_at desc
  → Shows: domain, scan date, severity counts, expiry date, re-scan button
  → Empty state: "No scans yet. Verify your site to run your first full scan." + CTA

Rate Limiting (Backend)
  → Anonymous: IP-based sliding window (existing) + email-based (1 scan total)
  → Developer: per clerk_user_id in PostgreSQL (scan_count_this_month column)
  → Quota check on POST /api/v1/scans
  → Return 429 with JSON: {error: "quota_exceeded", resets_at: "2026-03-01", tier: "developer"}
  → Frontend: display friendly quota exhausted UI on 429
```

---

## User Journey: Each Tier

### Journey 1 — Instant Audit (Anonymous)

```
1. Land on homepage (unchanged)
2. Paste URL → accept CFAA checkbox → click "Scan Now"
   [No email capture upfront — or optional email for report delivery]
3. Scan runs (existing progress UI)
4. Results page loads:
   - Full details for low/medium findings (existing behavior)
   - Teaser cards for high/critical:
       [!] Critical Finding (3)
       ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
       🔒 Sign up to see full details
       [Sign up free — takes 30 seconds]
5. If critical/high findings exist:
   - Modal appears: "We found X critical issues.
     Sign up free to see full remediation steps."
   - [Sign up with GitHub] [Sign up with Google] [Sign up with email]
6. If user ignores/closes:
   - Results page persists for 24hr (existing behavior)
   - Email sent with summary (if email captured)
7. On return after 24hr: results expired, prompt to re-scan
```

### Journey 2 — Developer (Free Signup)

```
1. Signup triggered from:
   a. Teaser card on results page (modal context — stays on results)
   b. Header "Sign up free" CTA (redirect to /sign-up)
   c. Direct navigation to /sign-up

2. Signup with Clerk (Google OAuth takes 1 click, email takes 3 fields)
   - Clerk sends verification email for password signups (automatic)
   - On success: Clerk fires user.created webhook

3. Webhook received (POST /api/webhooks/clerk):
   - Create users row: {clerk_user_id, tier: "developer", scan_count_this_month: 0, quota_reset_at: end_of_month}
   - Log: user onboarded

4. Redirect to /dashboard after signup
   - Dashboard empty state:
     ┌────────────────────────────────┐
     │  Welcome to ShipSecure         │
     │                                │
     │  To run full scans, verify     │
     │  ownership of your site first. │
     │                                │
     │  [Verify your site →]          │
     └────────────────────────────────┘

5. Domain Verification (/dashboard/verify):
   a. User enters their domain (e.g., myapp.vercel.app)
   b. System generates unique token: shipsecure-abc123
   c. User sees two options:
      Option A — Meta Tag:
        Add this to your <head>:
        <meta name="shipsecure-verification" content="shipsecure-abc123">
        Then click: [Verify now]
      Option B — File Upload:
        Create file at: /.well-known/shipsecure-verification.txt
        With contents: shipsecure-abc123
        Then click: [Verify now]
   d. User clicks "Verify now"
   e. Backend fetches domain, checks for token
      - Success: domain.status = "verified", show green check
      - Failure: show error "Token not found. Did you publish your changes?"
        with retry button and instructions
   f. Verified domain appears in dashboard with green badge

6. Run authenticated scan (/dashboard or homepage):
   a. URL pre-filled with verified domain (or user pastes any URL — server validates against verified domains)
   b. Scan runs with full scanner config
   c. Results: ALL severity levels shown with full details + remediation
   d. Scan stored in history, associated with clerk_user_id
   e. scan_count_this_month incremented

7. Quota tracking:
   - Header/dashboard shows: "3 of 5 scans used this month (resets Mar 1)"
   - At 4/5: yellow banner "1 scan remaining this month"
   - At 5/5: scan button disabled, friendly message: "Quota reached. Resets March 1."
   - POST /api/v1/scans returns 429 with resets_at timestamp

8. Scan history (/dashboard/history):
   - List of past scans with severity summary, expiry countdown
   - Re-scan button (consumes quota)
   - After 7-14 days: scan marked "expired", results unavailable
```

### Journey 3 — Pro (Future, not v1.6)

```
[Placeholder — build after Developer tier proves conversion]
- Unlimited sites via verified domain list
- Unlimited scans
- Permanent history
- Deep scan mode (extended Nuclei templates, full TLS analysis)
- PDF/CSV export
- Stripe subscription ($X/month)
- API access
```

---

## MVP Recommendation

### Must Build (v1.6)

1. **Clerk SDK integration** — `@clerk/nextjs`, `clerkMiddleware()`, `<ClerkProvider>`, SignIn/SignUp pages at `/sign-in` and `/sign-up`
2. **Clerk webhook handler** — POST `/api/webhooks/clerk`, verify with svix, create user row in DB on `user.created`
3. **Database schema additions** — `users` table (clerk_user_id, tier, scan_count_this_month, quota_reset_at), `verified_domains` table (user_id, domain, token, status, verified_at)
4. **Results gating** — Backend: never return finding details for high/critical on anonymous requests. Frontend: teaser card component with locked overlay and signup CTA
5. **Post-scan signup modal** — Triggered when anonymous scan has high/critical findings. Clerk SignUp in modal mode
6. **Domain verification flow** — `/dashboard/verify` page with meta tag and file upload options, backend verification endpoint
7. **Scan history dashboard** — `/dashboard` with scan list, severity summaries, expiry dates, re-scan button
8. **Per-user rate limiting** — Developer tier quota tracked in DB (scan_count_this_month), 429 with resets_at on exhaustion
9. **Quota display** — In header (when authenticated) and dashboard. Warning at 80% used
10. **Remove $49 Stripe audit** — Remove Stripe checkout flow, paid audit route, audit-specific scan config

### Defer (After v1.6)

| Feature | Reason to Defer |
|---------|-----------------|
| Scan result comparison / delta view | Requires stable findings schema across multiple scans; adds significant schema complexity |
| Scan share links for verified sites | Low priority; existing capability URL already shareable |
| Email drip campaign after anonymous scan | Requires Resend sequence integration; separate feature |
| Pro tier (Stripe subscription) | Build after Developer tier conversion proves out |
| Scheduled/automated re-scans | Pro tier feature |

---

## Complexity Notes

| Feature | Effort | Complexity Driver |
|---------|--------|------------------|
| Clerk SDK + middleware | 2-3 hours | Straightforward — excellent Next.js docs |
| Clerk webhook handler | 2 hours | svix signature verification, DB insert |
| Results gating (backend) | 3-4 hours | Modify scan response serialization by tier; test anonymous vs authenticated |
| Results gating (frontend teaser card) | 3 hours | New component, conditional rendering, modal integration |
| Domain verification flow | 4-6 hours | UI + token generation + backend HTTP check + status polling |
| Scan history dashboard | 4-5 hours | New page, DB query, empty state, expiry logic |
| Per-user rate limiting | 3-4 hours | DB-backed counter, quota reset logic, 429 response format |
| Remove Stripe audit | 1-2 hours | Delete routes, components, references; clean DB migration |
| Clerk webhook + DB user creation | 2 hours | One-time setup, well-documented pattern |

**Total estimated effort: 24-30 hours (3-4 development days)**

---

## Sources

**Clerk documentation (HIGH confidence):**
- [Clerk Next.js App Router Quickstart](https://clerk.com/docs/nextjs/getting-started/quickstart)
- [clerkMiddleware() reference](https://clerk.com/docs/reference/nextjs/clerk-middleware)
- [auth() App Router reference](https://clerk.com/docs/reference/nextjs/app-router/auth)
- [User metadata](https://clerk.com/docs/users/metadata)
- [Sync Clerk data via webhooks](https://clerk.com/docs/guides/development/webhooks/syncing)
- [Basic RBAC with metadata](https://clerk.com/docs/guides/secure/basic-rbac)
- [SignIn component](https://clerk.com/docs/nextjs/reference/components/authentication/sign-in)
- [SignUp component](https://clerk.com/docs/nextjs/reference/components/authentication/sign-up)
- [UserButton component](https://clerk.com/docs/nextjs/reference/components/user/user-button)
- [Complete Auth Guide for Next.js App Router 2025](https://clerk.com/articles/complete-authentication-guide-for-nextjs-app-router)

**Domain verification (MEDIUM-HIGH confidence — based on Google Search Console pattern, widely adopted):**
- [Google Search Console verification methods](https://support.google.com/webmasters/answer/9008080?hl=en)
- [WorkOS developer guide to domain verification](https://workos.com/blog/the-developers-guide-to-domain-verification)
- [DomainDetails KB — verification methods 2025](https://domaindetails.com/kb/domain-management/how-to-verify-domain-ownership)

**Tiered access and feature gating (MEDIUM confidence — general SaaS patterns, not security-tool-specific):**
- [Feature gating strategies for SaaS freemium](https://demogo.com/2025/06/25/feature-gating-strategies-for-your-saas-freemium-model-to-boost-conversions/)
- [Freemium conversion rate benchmarks 2026](https://firstpagesage.com/seo-blog/saas-freemium-conversion-rates/)
- [Feature gating guide — Orb](https://www.withorb.com/blog/feature-gating)
- [Gated content conversion statistics 2025](https://www.amraandelma.com/gated-content-conversion-statistics/)

**Rate limiting (HIGH confidence — well-established patterns):**
- [Rate limiting in PostgreSQL — Neon](https://neon.com/guides/rate-limiting)
- [10 best practices for API rate limiting 2025 — Zuplo](https://zuplo.com/learning-center/10-best-practices-for-api-rate-limiting-in-2025)
- [token-bucket-postgres on GitHub](https://github.com/fafl/token-bucket-postgres)

**Empty states and dashboard UX (MEDIUM confidence — general SaaS patterns):**
- [Empty state in SaaS applications — Userpilot](https://userpilot.com/blog/empty-state-saas/)
- [Dashboard design UX patterns — Pencil & Paper](https://www.pencilandpaper.io/articles/ux-pattern-analysis-data-dashboards)
