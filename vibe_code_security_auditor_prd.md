# Vibe Code Security Auditor - Product Requirements Document

## Executive Summary

A SaaS security scanning platform targeting developers using AI code generation tools (Cursor, Bolt, Lovable, etc.) who ship fast but lack security expertise. The platform orchestrates existing open-source security tools, adds vibe-code-specific detection rules, and delivers actionable remediation guidance written for non-security professionals.

**Market Validation:**
- 45% of AI-generated code contains security flaws
- 86% of AI tools fail XSS defenses; 88% fail log injection tests
- Lovable's built-in scanner catches vulnerabilities only 66% of the time; Bolt's fails entirely
- CVE-2025-48757 exposed 170+ Lovable apps with RLS misconfigurations leaking PII and API keys

**Differentiator:** Vibe-code-specific rules, remediation playbooks for non-security users, and 40+ years cybersecurity credibility.

---

## Product Tiers & Pricing

| Tier | Price | Features |
|------|-------|----------|
| Free | $0 | URL scan only, basic findings, email report, upsell funnel |
| One-Time Audit | $49-99 | Full URL + repo scan, detailed PDF report, remediation steps |
| Pro | $149/month | Continuous monitoring, GitHub webhook triggers, cert expiration alerts |
| Agency | $299-499/month | White-label reports, multiple repos, priority support |

---

## Audit Surfaces & Checks

### Phase 1: Passive Recon (URL Only - Free Tier)

#### Security Headers
- Content-Security-Policy (CSP)
- Strict-Transport-Security (HSTS)
- X-Frame-Options
- X-Content-Type-Options
- Referrer-Policy
- Permissions-Policy

**Tool:** SecurityHeaders.io API or custom header fetch

#### TLS/Certificate Analysis
- Protocol support (fail SSLv2/v3, warn TLSv1.0/1.1)
- Certificate expiration (critical if < 30 days)
- Self-signed detection
- Weak signatures (SHA-1, MD5)
- Cipher suite analysis (weak ciphers, forward secrecy)
- Known vulnerabilities (Heartbleed, POODLE, BEAST, ROBOT)
- HSTS preload eligibility
- OCSP stapling
- CAA DNS records

**Tools:** 
- Free tier: SSL Labs API v4 (requires registration, rate limited)
- Paid tier: testssl.sh containerized (no limits, faster)

#### Exposed Files & Directories
- /.env
- /.git/config
- /debug
- /admin
- /.source-map files
- /api-docs, /swagger
- /phpinfo.php, /server-status
- robots.txt enumeration
- sitemap.xml enumeration

**Tool:** Custom HTTP probes + Nuclei templates

#### Client-Side Secret Detection
- Fetch JavaScript bundles
- Regex scan for API keys, tokens, credentials
- Common patterns: AWS keys, Stripe keys, Supabase anon keys, Firebase configs

**Tool:** Custom JS fetcher + Gitleaks regex patterns

### Phase 2: Active Scanning (URL - Paid Tier)

#### Endpoint Discovery & Testing
- API endpoint enumeration
- Authentication bypass attempts
- CORS misconfiguration (`Access-Control-Allow-Origin: *`)
- Missing rate limiting
- Verbose error messages
- Debug endpoints enabled

**Tool:** Nuclei with custom vibe-code templates, OWASP ZAP baseline

#### Vibe-Code-Specific Checks
- Supabase RLS misconfigurations
- Firebase security rules (permissive read/write)
- Vercel environment variable leaks
- Netlify function exposure
- Railway/Render debug endpoints

**Tool:** Custom Nuclei templates

### Phase 3: Code Audit (GitHub Repo - Paid Tier)

#### Static Analysis
- Hardcoded secrets (API keys, passwords, tokens)
- SQL injection patterns
- XSS vulnerabilities
- Command injection
- Path traversal
- Insecure deserialization
- Authentication/authorization gaps

**Tool:** Semgrep with custom rulesets

#### Secret Detection
- API keys in code
- .env files committed
- Private keys
- Database connection strings
- JWT secrets

**Tool:** Gitleaks, TruffleHog

#### Dependency Analysis
- Known vulnerable packages
- Outdated dependencies
- License compliance (optional)

**Tool:** npm audit, pip-audit, OSV-Scanner

#### Vibe-Code Patterns
- Supabase anon key in frontend code
- Firebase config with permissive rules
- Client-side authentication logic
- Direct database queries from frontend
- Missing input validation

**Tool:** Custom Semgrep rules

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Web Interface                           │
│                    (Simple form: URL + repo)                    │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Scan Orchestrator                        │
│                  (Queue management, job dispatch)               │
└─────────────────────────────────────────────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        ▼                       ▼                       ▼
┌───────────────┐      ┌───────────────┐      ┌───────────────┐
│  URL Scanner  │      │  TLS Scanner  │      │ Code Scanner  │
│               │      │               │      │               │
│ • Headers     │      │ • SSL Labs    │      │ • Semgrep     │
│ • Nuclei      │      │ • testssl.sh  │      │ • Gitleaks    │
│ • ZAP         │      │               │      │ • npm audit   │
│ • File probes │      │               │      │               │
└───────────────┘      └───────────────┘      └───────────────┘
        │                       │                       │
        └───────────────────────┼───────────────────────┘
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Findings Aggregator                        │
│            (Dedupe, prioritize, map to remediation)             │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Report Generator                           │
│              (PDF, email summary, dashboard view)               │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                         Delivery                                │
│     • Email with PDF attachment                                 │
│     • Dashboard results page                                    │
│     • GitHub Issues export (markdown file for user to import)   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Tech Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Backend | Python (FastAPI) or Rust | Fast async, good ecosystem |
| Queue | Redis + RQ or Celery | Simple job management |
| Scanning Tools | Nuclei, Semgrep, Gitleaks, testssl.sh, ZAP | Best-in-class OSS |
| TLS (Free) | SSL Labs API v4 | Recognizable grades |
| TLS (Paid) | testssl.sh in Docker | No rate limits |
| Report Gen | WeasyPrint or Playwright PDF | Clean PDF output |
| Frontend | Simple HTML/HTMX or Next.js | Start simple, iterate |
| Payments | Stripe | Standard |
| Hosting | Railway, Fly.io, or VPS | Container-friendly |
| GitHub Integration | GitHub App | Webhook triggers, repo access |

---

## MVP Scope (6-Week Build)

### Week 1-2: Core URL Scanner
- [ ] Landing page with URL input form
- [ ] Security headers check (custom fetch)
- [ ] SSL Labs API integration (grade + top findings)
- [ ] Exposed file/directory probes (/.env, /.git, etc.)
- [ ] Client-side JS secret scanning
- [ ] Basic results page with findings
- [ ] Email delivery of results

### Week 3: Nuclei Integration
- [ ] Containerize Nuclei
- [ ] Create vibe-code-specific templates (Supabase, Firebase, Vercel)
- [ ] Integrate into scan pipeline
- [ ] Add findings to report

### Week 4: Code Scanning
- [ ] GitHub App setup for repo access
- [ ] Semgrep integration with custom rules
- [ ] Gitleaks integration
- [ ] Dependency scanning (npm audit, pip-audit)
- [ ] Merge findings into unified report

### Week 5: Payments & PDF Reports
- [ ] Stripe integration for one-time audits ($49-99)
- [ ] Stripe subscriptions for Pro tier ($149/month)
- [ ] PDF report generation with:
  - Executive summary
  - Findings by severity (Critical, High, Medium, Low, Info)
  - Remediation steps (copy-paste friendly)
  - Re-scan instructions

### Week 6: Continuous Monitoring
- [ ] GitHub webhook receiver for push events
- [ ] Automated re-scan on push
- [ ] Certificate expiration monitoring
- [ ] Email alerts for new findings
- [ ] testssl.sh integration for paid tier TLS scans

---

## Remediation Playbook Examples

### Supabase RLS Misconfiguration
**Finding:** Row Level Security disabled on `users` table
**Severity:** Critical
**Remediation:**
```sql
-- Run in Supabase SQL Editor
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

-- Add policy for authenticated users to read own data
CREATE POLICY "Users can view own data" ON users
  FOR SELECT USING (auth.uid() = id);

-- Add policy for authenticated users to update own data  
CREATE POLICY "Users can update own data" ON users
  FOR UPDATE USING (auth.uid() = id);
```

### Hardcoded API Key in Frontend
**Finding:** Stripe secret key found in `/static/js/main.js`
**Severity:** Critical
**Remediation:**
1. Immediately rotate the exposed key in Stripe Dashboard
2. Move API calls to backend/serverless function
3. Use environment variables, never commit secrets
```javascript
// BEFORE (vulnerable)
const stripe = Stripe('sk_live_xxxxx');

// AFTER (secure)
// Call your backend instead
const response = await fetch('/api/create-payment', {
  method: 'POST',
  body: JSON.stringify({ amount: 1000 })
});
```

### Missing Security Headers
**Finding:** Content-Security-Policy header not set
**Severity:** Medium
**Remediation (Vercel):**
```json
// vercel.json
{
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        { "key": "Content-Security-Policy", "value": "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';" },
        { "key": "X-Frame-Options", "value": "DENY" },
        { "key": "X-Content-Type-Options", "value": "nosniff" },
        { "key": "Referrer-Policy", "value": "strict-origin-when-cross-origin" },
        { "key": "Strict-Transport-Security", "value": "max-age=31536000; includeSubDomains" }
      ]
    }
  ]
}
```

---

## Revenue Projections

| Milestone | Subscribers | MRR | Notes |
|-----------|-------------|-----|-------|
| Month 3 | 25 Pro | $3,750 | Early adopters, content marketing |
| Month 6 | 75 Pro + 5 Agency | $12,750 | Referrals, partnerships |
| Month 12 | 150 Pro + 15 Agency | $27,000 | Established brand |

**Additional revenue streams:**
- One-time audits: $49-99 × estimated 50/month = $2,500-5,000/month
- Consulting upsells: $150-300/hour for remediation help
- Agency white-label partnerships

---

## Go-to-Market Strategy

### Content Marketing
- Blog: "Top 10 Vulnerabilities in Cursor-Built Apps This Month"
- Twitter/X: Build in public, share anonymized findings
- YouTube: "I Scanned 50 Vibe-Coded Apps - Here's What I Found"
- LinkedIn: Thought leadership on AI code security

### Community Presence
- r/vibecoding, r/cursor, r/lovable - helpful comments, not spammy
- Discord servers for Cursor, Bolt, Lovable
- Indie Hackers community

### Partnerships
- Dev agencies using AI tools (white-label offering)
- Code review services
- Hosting platforms (Railway, Vercel) - potential integration

### Credibility Leverage
- 40+ years in security engineering
- Enterprise background (Bose, Ford)
- TrustEdge Labs connection for hardware security angle

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| False positives damage reputation | Conservative severity ratings, manual review option for critical findings |
| Legal liability from disclosures | Never auto-create public GitHub issues; provide export file for user to import |
| Rate limiting from SSL Labs | testssl.sh fallback for paid tier |
| Competition from built-in platform security | Focus on cross-platform, deeper analysis, better remediation |
| Scope creep | Strict MVP scope, ship fast, iterate based on user feedback |

---

## Success Metrics

- **Week 2:** 100 free scans completed
- **Week 4:** First paying customer
- **Month 1:** 500 free scans, 10 paying customers
- **Month 3:** 1,000 free scans/month, $3,000+ MRR
- **Month 6:** 2,500 free scans/month, $10,000+ MRR

---

## Open Questions for Build Phase

1. **Domain/Brand:** "Vibe Code Security Auditor" vs something catchier?
2. **Hosting:** Start with Railway for simplicity or VPS for cost control?
3. **Report format:** PDF primary or web-first with PDF export?
4. **GitHub Issues:** Include at all in MVP or defer?
5. **Nuclei templates:** Build custom first or start with community templates?

---

## Next Steps

1. Set up GitHub repo with basic project structure
2. Build URL scanner CLI tool (headers + file probes + JS secrets)
3. Integrate SSL Labs API
4. Create landing page with email capture
5. Ship free tier, collect feedback
6. Iterate to paid tier
