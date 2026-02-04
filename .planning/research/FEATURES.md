# Feature Landscape: Security Scanning SaaS

**Domain:** URL-based security scanning for web applications
**Target:** Vibe-code apps (Cursor, Bolt, Lovable)
**Researched:** 2026-02-04
**Confidence:** MEDIUM (based on training data about security scanning platforms through 2025)

## Research Note

This research is based on pre-training knowledge of security scanning platforms (Snyk, Detectify, Intruder, Mozilla Observatory, SecurityHeaders.io, SSL Labs) through early 2025. Features should be verified against current platform documentation for 2026 accuracy.

---

## Table Stakes

Features users expect. Missing = product feels incomplete or untrustworthy.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **SSL/TLS Configuration Scan** | Basic security hygiene check, users expect this first | Low | Certificate validity, protocol versions, cipher suites |
| **Security Headers Check** | Standard OWASP recommendation, easy to verify | Low | CSP, X-Frame-Options, HSTS, X-Content-Type-Options |
| **Known Vulnerability Detection** | Core value prop of any security scanner | Medium | CVE databases, outdated libraries, framework versions |
| **Severity Scoring** | Users need to prioritize fixes | Low | Critical/High/Medium/Low or numeric (CVSS) |
| **Scan History** | Track progress over time, prove value | Medium | Store past scan results, show trends |
| **Email Delivery** | Free tier needs results delivery without signup friction | Low | Send results to email address provided |
| **Public URL Scanning** | Core capability - scan any accessible URL | Low | HTTP/HTTPS endpoint scanning |
| **Basic XSS/Injection Checks** | Expected in any web security scanner | Medium | Common OWASP Top 10 patterns |
| **Response Time < 5 min** | Free tier users won't wait longer | Medium | Balance thoroughness with speed expectations |
| **Mobile-Friendly Results** | Devs check results on phone while deploying | Low | Responsive design for results page |
| **Clear Pass/Fail Status** | Instant understanding of security posture | Low | Visual indicators (red/yellow/green) |
| **Remediation Links** | Point to how to fix issues | Low | Link to docs/guides per issue type |

## Differentiators

Features that set TrustEdge apart. Not expected, but highly valued for vibe-code context.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Vibe-Code-Specific Rules** | Detects AI-generated code patterns that miss security | High | Requires pattern recognition for Cursor/Bolt/Lovable output |
| **No-Jargon Explanations** | Non-security devs understand issues | Medium | Translate CVE/security-speak to plain English |
| **Copy-Paste Fixes** | Actual code snippets, not "configure HSTS" | Medium | Framework-specific fixes (Next.js, React, etc.) |
| **Framework Detection** | Auto-identify what stack was used | Medium | Next.js, Vite, CRA, Vercel, Netlify patterns |
| **AI Tool Detection** | Identify if app looks Cursor/Bolt/Lovable generated | High | Code structure patterns, common AI scaffolding |
| **Deployment Platform Scan** | Check Vercel/Netlify/Railway config, not just app | Medium | Platform-level security (env vars exposed, etc.) |
| **One-Click Fix PRs** (future) | Generate PR with fixes for GitHub repos | High | Requires GitHub integration, code generation |
| **Risk Context for Non-Security Users** | "This is exploitable if..." with real scenarios | Medium | Explain impact in terms devs understand |
| **Progress Tracking Dashboard** | Visual progress as issues get fixed | Medium | Rescan + comparison with previous results |
| **Paid Audit: Manual Review** | Human expert reviews vibe-code specifics | High | Combines automated scan + human analysis |
| **Paid Audit: Video Walkthrough** | Security expert explains findings in video | Medium | Loom-style recording explaining each issue |
| **Compliance-Speak Translation** | "This meets SOC 2 requirement X" for selling to enterprise | Medium | Map findings to compliance frameworks |
| **Shareable Security Badge** | "Scanned by TrustEdge" badge for marketing | Low | Embed widget showing scan date/status |
| **Comparison to Similar Apps** | "92% of Next.js apps we scan have HSTS" | Medium | Aggregate anonymized data for context |
| **Free Rescan Reminders** | Email after 30 days: "Want to rescan?" | Low | Engagement mechanism for free tier |

## Anti-Features

Features to explicitly NOT build. Common mistakes in security scanning domain.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Aggressive Penetration Testing** | Could crash/damage user's production site; legal liability | Passive checks + safe active scanning only |
| **Login/Auth Required for Free Scan** | Friction kills free tier viral growth | Email-only for results delivery |
| **Comprehensive CVE Database Scanning** | Duplicates Snyk/Dependabot, not our differentiator | Focus on vibe-code-specific issues, reference others for CVEs |
| **Real-Time Monitoring/Alerts** | Scope creep, expensive infrastructure, moves to different market | One-time scans, rescan on demand |
| **Complex Scanning Configuration** | Users won't configure; they want "just scan this URL" | Smart defaults, zero config for free tier |
| **Detailed Exploit Proof-of-Concepts** | Could enable attackers, ethical issues | Show it's vulnerable without providing exploit code |
| **White-Label/Multi-Tenant** | Enterprise feature, wrong market for MVP | Focus on individual devs, small teams |
| **API-First Architecture** (initially) | YAGNI for MVP; adds complexity | Build for web UI first, API later if needed |
| **Browser Extension** | Maintenance burden, limited value vs. URL scan | Stay focused on URL-based scanning |
| **Automated Fix Deployment** | Too risky; users need control over changes | Provide fixes, let users apply them |
| **Scan Scheduling/CRON** | Adds complexity, moves toward monitoring product | On-demand scanning only |
| **Team/Organization Features** (MVP) | Premature for validating individual dev PMF | Single-user focused, add team features post-PMF |

## Feature Dependencies

```
Core Scanning Flow:
URL Input → Framework Detection → Security Checks → Results Presentation

Framework Detection enables:
  → Copy-Paste Fixes (need to know framework)
  → Deployment Platform Scan (infer from framework)
  → Comparison Data (segment by framework)

Scan History requires:
  → Email collection (for lookup key)
  → Database (store results)
  → Results Presentation (comparison view)

Paid Audit Flow:
Payment → Deep Scan → Manual Review → PDF Report Generation
                   ↓
              Video Walkthrough (optional add-on)

Vibe-Code Detection enables:
  → AI Tool Detection
  → Vibe-Code-Specific Rules
  → Custom Remediation Playbooks
```

## MVP Feature Set Recommendation

### Free Tier (No Signup)

**Must Have (Table Stakes):**
1. SSL/TLS configuration scan
2. Security headers check
3. Basic OWASP Top 10 checks (XSS, injection points)
4. Severity scoring (Critical/High/Medium/Low)
5. Email results delivery
6. Clear pass/fail visual status
7. Response time < 5 minutes

**Differentiators to Include:**
8. Framework detection (Next.js, Vite, etc.)
9. No-jargon explanations
10. Basic vibe-code pattern detection (common AI scaffolding issues)
11. Copy-paste fixes for detected framework
12. Free rescan reminder (30 days)

**Defer to Post-MVP:**
- Scan history dashboard (requires user accounts)
- Progress tracking
- Shareable security badge
- Comparison to similar apps

### Paid Tier ($49-99 One-Time Audit)

**Must Have:**
1. All free tier checks
2. Deeper scanning (more comprehensive OWASP coverage)
3. Manual expert review
4. PDF report (professional, shareable with stakeholders)
5. Vibe-code-specific deep analysis
6. Priority email support

**Differentiators to Include:**
7. Video walkthrough of findings (5-10 min recording)
8. Compliance framework mapping (SOC 2, GDPR basics)
9. Deployment platform configuration review
10. Remediation priority roadmap
11. 30-day follow-up rescan included

**Defer to Post-MVP:**
- One-click fix PRs (requires GitHub integration)
- Team sharing features
- Custom rule configuration

## Free-to-Paid Conversion Mechanisms

Based on security SaaS patterns:

| Mechanism | How It Works | When to Trigger | Expected Conversion |
|-----------|--------------|-----------------|-------------------|
| **Limited Depth** | Free shows surface issues; paid shows deeper analysis | End of free scan results | 2-5% |
| **"X Critical Issues Found"** | Show count of critical issues, require payment to see details | Free scan results page | 5-10% |
| **"Manual Review Available"** | CTA that human expert can review (paid) | High severity findings | 3-7% |
| **Time-Sensitive Discount** | "Get audit report for $49 (reg $99) within 24 hours" | Email delivery of free results | 10-15% |
| **Comparison Teaser** | "Sites like yours average 8.2 issues; you have 12" | Free scan results | 2-4% |
| **PDF Report Preview** | Show what paid PDF looks like (blurred/sample) | Free scan results | 5-8% |
| **Stakeholder Mode** | "Need to share with boss/client? Get PDF report" | Free scan results | 3-5% |
| **Fix Verification** | "Fix issues and get verified clean report (paid)" | After user has rescanned | 8-12% |
| **Compliance Gate** | "Need this for SOC 2? Get compliance-mapped report" | Free scan results | 5-10% |

### Recommended Conversion Flow

1. **Free scan completes** → Show summary with clear severity breakdown
2. **Surface-level issues shown** → "Manual expert review available for deeper analysis"
3. **Email results** → Include 24-hour discount offer ($49 vs $99)
4. **Rescan reminder (30 days)** → "Fixed issues? Get verified clean report for stakeholders"

### What Makes Conversion Work

**DO:**
- Show real value in free tier (users trust you)
- Make paid tier about depth/expertise, not gatekeeping
- Price point that's expense-able ($49-99, not $499)
- Clear deliverable (PDF report + video)
- Time-sensitive offers (creates urgency)

**DON'T:**
- Hide critical security issues behind paywall (unethical)
- Make free tier feel crippled
- Unclear what paid tier includes
- Subscription for one-time need (wrong model)

## Results Presentation: What Users Expect

### Free Scan Results Page

**Above the Fold:**
- Overall security score (A-F or 0-100)
- Critical issue count (if any)
- Visual status indicator (red/yellow/green)
- CTA: "Get Detailed Audit Report" (paid tier)

**Results Layout:**
- Issues grouped by severity (Critical → High → Medium → Low)
- Each issue shows:
  - What it is (plain English, not CVE-speak)
  - Why it matters (risk context)
  - How to fix it (copy-paste code if possible)
  - Learn more link (external docs)
- Expandable details (collapsed by default)
- Share results button (generates shareable link)

**Below Results:**
- "What we checked" (transparency builds trust)
- "What we didn't check" (sets expectations)
- "Rescan after fixing" CTA
- "Get deeper analysis" CTA (paid)

### Paid Audit PDF Report

**Expected Sections:**
1. **Executive Summary** (1 page)
   - Overall assessment
   - Critical findings count
   - Recommended priority actions
   - Risk level (Low/Medium/High)

2. **Detailed Findings** (5-10 pages)
   - Each issue with:
     - Technical description
     - Risk impact (with scenario)
     - Remediation steps (numbered, specific)
     - Code examples
   - Screenshots of issues
   - CVSS scores (if applicable)

3. **Vibe-Code Analysis** (differentiator)
   - AI-generated code patterns detected
   - Common vibe-code pitfalls found
   - Framework-specific recommendations

4. **Remediation Roadmap** (1-2 pages)
   - Priority order for fixes
   - Time estimates
   - Quick wins vs. longer work

5. **Compliance Mapping** (1 page)
   - How findings map to SOC 2, GDPR, etc.
   - What's needed for compliance

6. **Appendix**
   - Full technical details
   - Tool versions used
   - Scan metadata (date, duration, scope)

**Report Must-Haves:**
- Professional design (not just HTML-to-PDF)
- Branded (TrustEdge logo, colors)
- Page numbers, table of contents
- Shareable (can forward to boss/client)
- Dated with validity period
- Includes video walkthrough link

## Remediation Guides: Useful vs. Ignored

### What Makes Guides IGNORED

- Generic advice: "Configure HSTS properly"
- Links to 50-page documentation
- No code examples
- Framework-agnostic (user has to figure out their stack)
- Security jargon without translation
- No explanation of impact
- Just description of problem, no solution

### What Makes Guides USEFUL

**Framework-Specific:**
```
Bad:  "Enable HSTS"
Good: "Add this to your next.config.js:
       headers: [{ key: 'Strict-Transport-Security', value: 'max-age=63072000' }]"
```

**Copy-Paste Ready:**
```
Not this: "Configure CSP header with appropriate directives"
But this: "Copy this middleware into middleware.ts:
          [complete, working code snippet]"
```

**Risk Context for Non-Security Devs:**
```
Not this: "Missing X-Frame-Options allows clickjacking"
But this: "Without X-Frame-Options, attackers can embed your site in invisible iframes
          and trick users into clicking malicious links. This is how [famous breach] happened.
          Fix in 30 seconds: add this header..."
```

**Prioritized with Effort Estimates:**
```
Critical (5 mins):  Add security headers
High (30 mins):     Fix XSS in form handler
Medium (2 hours):   Update authentication library
Low (4 hours):      Implement rate limiting
```

**Verification Steps:**
```
After fixing:
1. Add the code snippet above
2. Redeploy your site
3. Rescan at trusteddgeaudit.com
4. Look for green checkmark on "Security Headers"
```

**Visual Examples:**
- Before/After screenshots
- What the attack looks like
- What the fix prevents

### Remediation Guide Structure (Per Issue)

```markdown
## [Issue Name in Plain English]

RISK: [One sentence: what bad thing can happen]
TIME TO FIX: [5 mins / 30 mins / 2 hours]
PRIORITY: [Critical / High / Medium / Low]

### What's Wrong

[2-3 sentences explaining the issue without jargon]

### Why It Matters

[Real-world scenario: "If an attacker...then they could..."]

### How to Fix

[Framework detected: Next.js]

1. [Step 1 with specific file name]
2. [Step 2 with code snippet]
   ```typescript
   // Complete, working code
   ```
3. [Step 3: deploy/test]

### Verify It's Fixed

- [ ] Deploy changes
- [ ] Rescan at [URL]
- [ ] Check for green checkmark

### Learn More

- [Why this matters] (link to our blog)
- [Official Next.js docs] (external)
```

## Competitive Feature Comparison

Based on training data through early 2025:

| Feature Category | Snyk | Detectify | Intruder | Mozilla Obs | TrustEdge (Proposed) |
|-----------------|------|-----------|----------|-------------|----------------------|
| **Scan Type** | Code + Container | Web App | Infra + Web | Web Headers | Web App (URL-based) |
| **Target Users** | Dev teams | Security teams | Security teams | Anyone | Vibe-code devs |
| **Pricing Model** | Freemium SaaS | Subscription | Subscription | Free | Free + One-time audit |
| **Auth Required** | Yes | Yes | Yes | No | No (free tier) |
| **Continuous Monitoring** | Yes | Yes | Yes | No | No |
| **Manual Review** | Enterprise | No | Optional | No | Yes (paid) |
| **Framework Detection** | Yes (code) | Limited | No | No | Yes (web) |
| **Remediation Guides** | Good | Basic | Good | Basic | Excellent (vibe-code) |
| **PDF Reports** | Yes | Yes | Yes | No | Yes (paid) |
| **Vibe-Code Focus** | No | No | No | No | **YES** |

### TrustEdge Positioning

**We are NOT competing on:**
- Continuous monitoring (Snyk, Detectify, Intruder)
- Enterprise team features
- Infrastructure scanning
- Comprehensive CVE database

**We ARE competing on:**
- Vibe-code-specific detection (unique)
- No-auth free tier (like Mozilla Observatory)
- One-time audit model (different from subscription)
- Remediation quality for non-security devs
- Price point for individual devs ($49-99 vs $hundreds/month)

**Our lane:** Security scanning specifically for AI-generated web apps, targeting non-security developers who need to ship safely but don't have security expertise.

## Feature Complexity Assessment

| Feature | Complexity | Why | Dependencies |
|---------|------------|-----|--------------|
| SSL/TLS scan | Low | Use existing libraries (ssllabs-scan) | None |
| Security headers | Low | HTTP request + header parsing | None |
| Framework detection | Medium | Pattern matching on HTML/JS | None |
| XSS/Injection checks | Medium | Requires safe test payload generation | Framework detection |
| Vibe-code detection | High | ML/pattern recognition, requires research | Framework detection |
| Copy-paste fixes | Medium | Template generation per framework | Framework detection |
| Email delivery | Low | SMTP/SendGrid integration | None |
| PDF generation | Medium | Report templating + PDF library | All scan results |
| Video walkthrough | Medium | Screen recording + hosting | Paid scan completion |
| Manual review | High | Human expert time, scheduling | Paid tier purchase |
| Scan history | Medium | Database + user identification | Email as lookup key |
| Comparison data | Medium | Aggregation + privacy considerations | Database with scan history |
| One-click fix PRs | High | GitHub API + code generation + testing | GitHub OAuth, framework detection |

## Sources

**Note:** This research is based on training data about security scanning platforms through early 2025. Current platform capabilities should be verified for 2026 accuracy by visiting:

- Snyk.io product pages
- Detectify.com features
- Intruder.io documentation
- Mozilla Observatory (observatory.mozilla.org)
- SecurityHeaders.io
- SSL Labs (ssllabs.com)

**Confidence Level: MEDIUM** - Features are based on established security scanning patterns and OWASP standards, but specific competitor capabilities as of 2026 should be verified before finalizing roadmap.
