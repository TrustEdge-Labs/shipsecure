# Phase 2: Free Tier MVP - Research

**Researched:** 2026-02-05
**Domain:** Next.js frontend with transactional email, containerized security scanners, and token-based result access
**Confidence:** MEDIUM-HIGH

## Summary

Phase 2 builds the public-facing MVP: Next.js landing page, real-time progress tracking, results dashboard with token-based access, and email delivery. The standard stack is Next.js 15 App Router with React Server Actions for forms, Zod for validation, and Server-Sent Events or polling for progress updates. Transactional email via Resend (developer experience) or Postmark (deliverability focus). Containerized scanners (Nuclei, testssl.sh) run in Docker with security hardening. SSL Labs API integration requires careful rate limit management (concurrent assessment limits, cool-off periods).

Key architectural decisions: Server Actions for internal form submission (not API routes for external clients), database-backed polling for progress (SSE reserved for higher scale), cryptographically secure tokens for result URLs (256-bit minimum), and markdown generation for downloadable reports. Critical pitfalls include SSL Labs 429 errors without proper throttling, SSRF protection for user-submitted URLs (already implemented in Phase 1), and false positive management (confidence ratings, contextual validation).

**Primary recommendation:** Use Next.js 15 with App Router, Zod + useActionState for forms, Resend for transactional email (better DX, sufficient deliverability for MVP), Docker for Nuclei/testssl.sh with read-only filesystems and non-root users, and Node.js crypto.randomBytes for secure token generation. Implement SSL Labs client with header-based rate limit tracking and exponential backoff.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Next.js | 15+ | Frontend framework | App Router with React Server Components, built-in form handling, official Vercel support |
| React | 19 RC | UI library | Server Components stable in React 19, integrated with Next.js 15 App Router |
| Zod | 3+ | Schema validation | Type-safe validation, integrates with Server Actions, standard for Next.js forms 2026 |
| Resend | Latest | Transactional email | Best developer experience, React Email templates, modern SDKs, SOC 2 compliant |
| Nuclei | Latest | Vulnerability scanner | Community-driven, YAML-based templates, official Docker images, CI/CD ready |
| testssl.sh | 3.2+ | TLS scanner | Comprehensive SSL/TLS testing, official Docker support, non-root container |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| useActionState | React 19 | Form state management | Server Action error handling, pending states, progressive enhancement |
| crypto (Node.js) | Built-in | Token generation | Cryptographically secure random tokens for result URLs (256-bit minimum) |
| marked | 14+ | Markdown generation | Convert scan results to downloadable markdown reports |
| Postmark | Latest | Alternative email service | If deliverability is more critical than DX (better inbox placement) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Resend | Postmark | Postmark has better deliverability focus (message streams, strict policies) but Resend has superior DX (React Email, better docs) |
| Server Actions | API Routes | API Routes for external clients/webhooks; Server Actions for internal mutations (simpler, type-safe) |
| Polling | Server-Sent Events | SSE better for high-frequency updates but polling simpler for MVP (2-second intervals adequate) |
| Nuclei | Custom scanner | Nuclei has 1000+ community templates, custom scanner requires maintaining vulnerability database |

**Installation:**
```bash
# Next.js frontend (separate repo or monorepo)
npx create-next-app@latest frontend --typescript --app --tailwind
cd frontend
npm install zod marked

# Transactional email
npm install resend

# Docker images (pull from registries)
docker pull projectdiscovery/nuclei:latest
docker pull drwetter/testssl.sh:latest
```

## Architecture Patterns

### Recommended Project Structure
```
frontend/
├── app/
│   ├── page.tsx                 # Landing page with form
│   ├── scan/[id]/page.tsx       # Progress page (polls backend)
│   ├── results/[token]/page.tsx # Results dashboard (token-protected)
│   ├── actions/
│   │   └── scan.ts              # Server Actions for form submission
│   └── api/
│       └── results/[token]/     # API route for markdown download
│           └── download/route.ts
├── components/
│   ├── scan-form.tsx            # Landing page form with validation
│   ├── progress-checklist.tsx   # Stage-by-stage progress UI
│   ├── results-dashboard.tsx    # Findings display with accordions
│   └── grade-summary.tsx        # A-F grade + finding counts
└── lib/
    ├── email.ts                 # Resend client wrapper
    ├── tokens.ts                # Secure token generation
    └── markdown.ts              # Report generation
```

### Pattern 1: Server Actions for Form Submission
**What:** React Server Actions with Zod validation and useActionState for error handling
**When to use:** Internal form submissions (not external API clients)
**Example:**
```typescript
// Source: https://nextjs.org/docs/app/guides/forms
// app/actions/scan.ts
'use server'

import { z } from 'zod'

const schema = z.object({
  url: z.string().url('Invalid URL format'),
  email: z.string().email('Invalid email address'),
})

export async function submitScan(prevState: any, formData: FormData) {
  const validatedFields = schema.safeParse({
    url: formData.get('url'),
    email: formData.get('email'),
  })

  if (!validatedFields.success) {
    return {
      errors: validatedFields.error.flatten().fieldErrors,
    }
  }

  // Call Rust backend API
  const response = await fetch('http://backend/api/v1/scans', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(validatedFields.data),
  })

  if (!response.ok) {
    return { errors: { _form: 'Failed to start scan' } }
  }

  const { scan_id } = await response.json()
  redirect(`/scan/${scan_id}`)
}
```

```typescript
// app/page.tsx
'use client'

import { useActionState } from 'react'
import { submitScan } from './actions/scan'

export default function LandingPage() {
  const [state, formAction, pending] = useActionState(submitScan, {})

  return (
    <form action={formAction}>
      <input name="url" type="text" required />
      {state?.errors?.url && <p>{state.errors.url[0]}</p>}

      <input name="email" type="email" required />
      {state?.errors?.email && <p>{state.errors.email[0]}</p>}

      <button disabled={pending}>
        {pending ? 'Starting scan...' : 'Scan Now'}
      </button>
    </form>
  )
}
```

### Pattern 2: Database-Backed Polling for Progress
**What:** Client polls backend API every 2 seconds for scan status updates
**When to use:** MVP scale, simple implementation, adequate for scan durations (60-180s)
**Example:**
```typescript
// app/scan/[id]/page.tsx
'use client'

import { useEffect, useState } from 'react'

interface ScanStatus {
  status: 'pending' | 'in_progress' | 'completed' | 'failed'
  stages: {
    headers: boolean
    tls: boolean
    files: boolean
    secrets: boolean
  }
}

export default function ScanProgress({ params }: { params: { id: string } }) {
  const [status, setStatus] = useState<ScanStatus | null>(null)

  useEffect(() => {
    const interval = setInterval(async () => {
      const res = await fetch(`/api/v1/scans/${params.id}/status`)
      const data = await res.json()
      setStatus(data)

      if (data.status === 'completed') {
        clearInterval(interval)
        // Redirect to results page with token
        window.location.href = `/results/${data.results_token}`
      }
    }, 2000)

    return () => clearInterval(interval)
  }, [params.id])

  return (
    <div>
      <h1>Scanning in progress...</h1>
      <ul>
        <li>{status?.stages.headers ? '✓' : '○'} Security Headers</li>
        <li>{status?.stages.tls ? '✓' : '○'} TLS Configuration</li>
        <li>{status?.stages.files ? '✓' : '○'} Exposed Files</li>
        <li>{status?.stages.secrets ? '✓' : '○'} JavaScript Secrets</li>
      </ul>
    </div>
  )
}
```

### Pattern 3: Secure Token-Based Result Access
**What:** Cryptographically random 256-bit tokens for non-guessable result URLs
**When to use:** Public result access without authentication
**Example:**
```typescript
// lib/tokens.ts
// Source: https://goteleport.com/learn/authentication-and-authorization/simple-random-tokens-secure-authentication/
import { randomBytes } from 'crypto'

export function generateResultsToken(): string {
  // 32 bytes = 256 bits (recommended minimum for production)
  return randomBytes(32).toString('base64url')
}
```

```rust
// Rust backend: store token with scan results
use rand::Rng;

pub async fn complete_scan(pool: &PgPool, scan_id: Uuid) -> Result<String> {
    let token = generate_secure_token();

    sqlx::query!(
        "UPDATE scans SET results_token = $1, expires_at = NOW() + INTERVAL '3 days' WHERE id = $2",
        token,
        scan_id
    )
    .execute(pool)
    .await?;

    Ok(token)
}

fn generate_secure_token() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    base64::encode_config(&bytes, base64::URL_SAFE_NO_PAD)
}
```

### Pattern 4: SSL Labs API Client with Rate Limit Awareness
**What:** Client that tracks concurrent assessments via response headers and respects cool-off periods
**When to use:** All SSL Labs API integrations
**Example:**
```rust
// Source: https://github.com/ssllabs/ssllabs-scan/blob/master/ssllabs-api-docs-v4.md
use std::sync::Arc;
use tokio::sync::Semaphore;
use reqwest::Client;

pub struct SslLabsClient {
    client: Client,
    // Track concurrency via API headers
    max_assessments: Arc<Semaphore>,
    cool_off_ms: u64,
}

impl SslLabsClient {
    pub async fn new() -> Result<Self> {
        // Get initial limits from /api/v4/info
        let info = self.get_info().await?;

        Ok(Self {
            client: Client::new(),
            max_assessments: Arc::new(Semaphore::new(info.max_assessments)),
            cool_off_ms: info.new_assessment_cool_off,
        })
    }

    pub async fn analyze(&self, host: &str) -> Result<Assessment> {
        // Acquire permit (blocks if at max concurrency)
        let permit = self.max_assessments.acquire().await?;

        // Respect cool-off period
        tokio::time::sleep(Duration::from_millis(self.cool_off_ms)).await;

        let response = self.client
            .get(format!("https://api.ssllabs.com/api/v4/analyze?host={}", host))
            .send()
            .await?;

        // Update limits from response headers
        if let Some(max) = response.headers().get("X-Max-Assessments") {
            // Update semaphore capacity (requires atomic operation)
        }

        // Poll with variable intervals: 5s until IN_PROGRESS, then 10s
        let mut assessment = response.json::<Assessment>().await?;

        while assessment.status != "READY" {
            let interval = if assessment.status == "DNS" { 5000 } else { 10000 };
            tokio::time::sleep(Duration::from_millis(interval)).await;

            assessment = self.client
                .get(format!("https://api.ssllabs.com/api/v4/analyze?host={}", host))
                .send()
                .await?
                .json()
                .await?;
        }

        drop(permit);
        Ok(assessment)
    }
}
```

### Pattern 5: Containerized Scanner Execution
**What:** Docker containers with security hardening (non-root, read-only, resource limits)
**When to use:** All external scanner tools (Nuclei, testssl.sh)
**Example:**
```rust
// Source: https://deepwiki.com/testssl/testssl.sh/6-docker-containerization
use tokio::process::Command;

pub async fn run_testssl(target: &str) -> Result<String> {
    let output = Command::new("docker")
        .args([
            "run",
            "-i",
            "--rm",
            "--read-only",              // CIS Docker Security Benchmark
            "--cap-drop", "all",        // Drop all capabilities
            "--memory", "100M",         // Memory limit
            "--pids-limit", "1000",     // Process limit
            "--cpu-shares", "512",      // CPU limit
            "--user", "1000:1000",      // Non-root user
            "drwetter/testssl.sh:latest",
            "--jsonfile-pretty", "/dev/stdout",
            target,
        ])
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow!("testssl.sh failed"));
    }

    Ok(String::from_utf8(output.stdout)?)
}

pub async fn run_nuclei(target: &str) -> Result<String> {
    let output = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--read-only",
            "--cap-drop", "all",
            "--memory", "512M",
            "--user", "1000:1000",
            "projectdiscovery/nuclei:latest",
            "-u", target,
            "-jsonl",                   // JSON lines output
            "-silent",
        ])
        .output()
        .await?;

    Ok(String::from_utf8(output.stdout)?)
}
```

### Pattern 6: Markdown Report Generation
**What:** Convert scan results to downloadable markdown with all findings and remediation
**When to use:** Free tier download option (PDF reserved for paid)
**Example:**
```typescript
// lib/markdown.ts
import { marked } from 'marked'

interface Finding {
  title: string
  severity: 'Critical' | 'High' | 'Medium' | 'Low' | 'Info'
  description: string
  remediation: string
}

export function generateMarkdownReport(
  scan: { url: string; grade: string; created_at: string },
  findings: Finding[]
): string {
  const grouped = findings.reduce((acc, f) => {
    acc[f.severity] = acc[f.severity] || []
    acc[f.severity].push(f)
    return acc
  }, {} as Record<string, Finding[]>)

  let md = `# Security Scan Report

**Target:** ${scan.url}
**Grade:** ${scan.grade}
**Scanned:** ${new Date(scan.created_at).toLocaleString()}

---

## Summary

`

  Object.entries(grouped).forEach(([severity, items]) => {
    md += `- **${severity}:** ${items.length} finding(s)\n`
  })

  md += '\n---\n\n'

  // Findings by severity
  for (const severity of ['Critical', 'High', 'Medium', 'Low', 'Info']) {
    if (!grouped[severity]) continue

    md += `## ${severity} Findings\n\n`

    grouped[severity].forEach((finding, idx) => {
      md += `### ${idx + 1}. ${finding.title}\n\n`
      md += `**Description:** ${finding.description}\n\n`
      md += `**Remediation:**\n${finding.remediation}\n\n`
      md += '---\n\n'
    })
  }

  md += `\n*Generated by TrustEdge Audit*`
  return md
}
```

### Anti-Patterns to Avoid

- **Unbounded concurrent scans:** Phase 1 already implements semaphore-based limiting (5 workers default); do not remove this for Phase 2
- **API Routes for internal forms:** Use Server Actions for form submission; reserve API Routes for external clients or webhook endpoints
- **Guessable result URLs:** Never use sequential IDs or UUIDs as public result URLs; always use cryptographically secure random tokens (256-bit minimum)
- **Inline polling in Server Components:** Polling must happen in Client Components ('use client'); Server Components are rendered once, not continuously
- **Ignoring SSL Labs rate limits:** Always track X-Max-Assessments and X-Current-Assessments headers; implement exponential backoff for 429 responses

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Form validation | Custom validation functions | Zod + useActionState | Type-safe, automatic error handling, field-level errors, framework integration |
| Email delivery | SMTP server | Resend or Postmark | Deliverability monitoring, bounce handling, templates, compliance (DKIM/SPF/DMARC) |
| TLS scanning | OpenSSL wrapper | testssl.sh Docker image | Comprehensive cipher checks, protocol version detection, heartbleed/POODLE checks |
| Vulnerability scanning | Custom regex patterns | Nuclei with community templates | 1000+ maintained templates, low false positives, active development |
| Secure random tokens | Math.random() or UUID | Node.js crypto.randomBytes or Rust rand crate | Cryptographically secure, non-guessable, meets security standards |
| SSL Labs integration | Direct API calls | Rate-limit-aware client with header tracking | Concurrent assessment limits, cool-off periods, exponential backoff |
| Markdown to HTML | String concatenation | marked library | Secure HTML escaping, CommonMark compliant, extensible |

**Key insight:** Security scanner accuracy (false positives/negatives) requires domain expertise and continuous template updates. Community-driven tools (Nuclei, testssl.sh) have dedicated maintainers and thousands of user contributions. Custom scanners lag behind threat landscape unless you staff dedicated security researchers.

## Common Pitfalls

### Pitfall 1: SSL Labs 429 Errors Without Proper Throttling
**What goes wrong:** Exceeding concurrent assessment limits or submitting new assessments too quickly causes 429 (Too Many Requests) errors
**Why it happens:** API enforces maxAssessments (typically 2-5 concurrent) and newAssessmentCoolOff (wait between new submissions)
**How to avoid:**
- Track X-Max-Assessments and X-Current-Assessments response headers
- Use semaphore to cap concurrent assessments
- Implement cool-off period between new assessment requests
- Use exponential backoff for 429 responses (15-30 minute wait)
**Warning signs:** Repeated 429 errors, assessments failing to start, sudden API access loss

### Pitfall 2: False Positives Undermining Credibility
**What goes wrong:** Scanner reports non-existent vulnerabilities (e.g., flagging intentional public endpoints as "exposed admin panels")
**Why it happens:** Pattern-based detection without contextual validation; low-confidence findings treated as certain
**How to avoid:**
- Use confidence ratings to prioritize (Certain > Firm > Tentative)
- Cross-check findings against application architecture
- Implement multi-layered detection (pattern + entropy + context)
- Document common false positives and create ignore rules
- Review findings before user delivery
**Warning signs:** User complaints about incorrect findings, high finding counts with low actionability

### Pitfall 3: Render Free Tier Idle Timeout Breaking Scans
**What goes wrong:** Render free tier services go idle after 15 minutes inactivity; cold starts delay scan processing
**Why it happens:** Render free tier designed for low-traffic personal projects, not continuous background workers
**How to avoid:**
- Upgrade to paid Render plan for production (persistent workers)
- Implement wake-up endpoint and keep-alive pings during development
- Set clear expectations: "Scans may take 2-5 minutes" (accounts for cold starts)
- Monitor cold start metrics and upgrade trigger
**Warning signs:** Scan start delays >30 seconds, timeout errors, inconsistent processing times

### Pitfall 4: Result URL Token Expiry Without User Notification
**What goes wrong:** User receives email with result link, waits 4 days, link returns 404 (3-day expiry passed)
**Why it happens:** Expiry logic implemented but not communicated to user
**How to avoid:**
- Display expiry date on results page ("Available until Feb 10, 2026")
- Include expiry in email ("View your results within 3 days")
- Implement soft-delete (keep data, hide from UI) for potential recovery
- Log expired access attempts for upgrade funnel metrics
**Warning signs:** Support requests about "broken links", expired link access attempts in logs

### Pitfall 5: JavaScript Secret False Positives from Test Data
**What goes wrong:** Scanner flags test API keys, placeholder values, or intentionally public keys as secrets
**Why it happens:** Entropy-based detection without validation or context awareness
**How to avoid:**
- Verify API key format against known patterns (AWS starts with AKIA, Stripe with sk_live_)
- Check for common test patterns ("test", "example", "placeholder", "YOUR_KEY_HERE")
- Validate against service APIs when possible (rate-limited)
- Use confidence scoring (HIGH for validated, MEDIUM for pattern-only, LOW for entropy-only)
- Filter out keys in test files or documentation directories
**Warning signs:** High volume of "secret" findings, user reports of false positives

### Pitfall 6: SSRF Bypass via URL Parser Discrepancies
**What goes wrong:** Attacker crafts URLs that pass validation but resolve to internal IPs (e.g., http://127.0.0.1.example.com)
**Why it happens:** Different URL parsers and DNS resolution logic between validation and execution
**How to avoid:**
- Validate AFTER DNS resolution, not before (already implemented in Phase 1 ssrf::validator)
- Block cloud metadata IPs explicitly (169.254.169.254, fd00:ec2::254)
- Use Rust std::net IP classification (is_private, is_loopback, is_link_local)
- Test with bypass payloads: http://127.0.0.1.example.com, http://[::1], http://2130706433 (decimal IP)
**Warning signs:** Internal service requests in logs, metadata API access attempts

### Pitfall 7: Docker Container Breakout via Privileged Mode
**What goes wrong:** Running scanners with --privileged or excessive capabilities allows container escape
**Why it happens:** Copying Docker examples without understanding security implications
**How to avoid:**
- Always use --cap-drop all to remove all capabilities
- Run as non-root user (--user 1000:1000)
- Mount filesystems read-only (--read-only)
- Set resource limits (--memory, --pids-limit, --cpu-shares)
- Never use --privileged unless absolutely necessary (it's not for scanners)
**Warning signs:** Security audit failures, container accessing host filesystem

## Code Examples

Verified patterns from official sources:

### Email Delivery with Resend
```typescript
// lib/email.ts
import { Resend } from 'resend'

const resend = new Resend(process.env.RESEND_API_KEY)

export async function sendScanCompleteEmail(
  to: string,
  scan: {
    url: string
    grade: string
    findings: { Critical: number; High: number; Medium: number; Low: number }
    results_token: string
  }
) {
  await resend.emails.send({
    from: 'TrustEdge Audit <scans@trustedgeaudit.com>',
    to,
    subject: `Scan Complete: ${scan.grade} Grade for ${scan.url}`,
    html: `
      <h1>Your Security Scan is Complete</h1>
      <p><strong>Target:</strong> ${scan.url}</p>
      <p><strong>Grade:</strong> ${scan.grade}</p>

      <h2>Findings Summary</h2>
      <ul>
        <li>Critical: ${scan.findings.Critical}</li>
        <li>High: ${scan.findings.High}</li>
        <li>Medium: ${scan.findings.Medium}</li>
        <li>Low: ${scan.findings.Low}</li>
      </ul>

      <p>
        <a href="https://trustedgeaudit.com/results/${scan.results_token}">
          View Full Results
        </a>
      </p>

      <p><strong>Fixed some issues?</strong> Scan again to see your new score.</p>

      <p style="color: #666; font-size: 12px;">
        This link expires in 3 days. Results available until ${new Date(Date.now() + 3 * 24 * 60 * 60 * 1000).toLocaleDateString()}.
      </p>
    `,
  })
}
```

### Results Dashboard with Severity Toggle
```typescript
// components/results-dashboard.tsx
'use client'

import { useState } from 'react'

type GroupBy = 'severity' | 'category'

interface Finding {
  title: string
  severity: 'Critical' | 'High' | 'Medium' | 'Low' | 'Info'
  category: 'Headers' | 'TLS' | 'Files' | 'Secrets'
  description: string
  remediation: string
}

export function ResultsDashboard({ findings }: { findings: Finding[] }) {
  const [groupBy, setGroupBy] = useState<GroupBy>('severity')
  const [expanded, setExpanded] = useState<Record<string, boolean>>({})

  const grouped = findings.reduce((acc, f) => {
    const key = groupBy === 'severity' ? f.severity : f.category
    acc[key] = acc[key] || []
    acc[key].push(f)
    return acc
  }, {} as Record<string, Finding[]>)

  return (
    <div>
      <div className="toggle">
        <button onClick={() => setGroupBy('severity')}>By Severity</button>
        <button onClick={() => setGroupBy('category')}>By Category</button>
      </div>

      {Object.entries(grouped).map(([group, items]) => (
        <div key={group}>
          <h2>{group} ({items.length})</h2>
          {items.map((finding, idx) => (
            <div key={idx} className="accordion">
              <button
                onClick={() => setExpanded({ ...expanded, [`${group}-${idx}`]: !expanded[`${group}-${idx}`] })}
              >
                {finding.title}
              </button>
              {expanded[`${group}-${idx}`] && (
                <div>
                  <p>{finding.description}</p>
                  <h4>How to Fix:</h4>
                  <p>{finding.remediation}</p>
                </div>
              )}
            </div>
          ))}
        </div>
      ))}
    </div>
  )
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Pages Router with getServerSideProps | App Router with Server Components | Next.js 13+ (stable in 15) | Simpler data fetching, automatic loading states, streaming |
| API Routes for all forms | Server Actions for internal mutations | React 19 + Next.js 15 | Type-safe by default, no API route boilerplate, progressive enhancement |
| Client-side polling with useEffect | Server-Sent Events with EventSource | Web standard (SSE) | Lower latency, fewer requests, but polling still valid for MVP |
| Raw SMTP for emails | Transactional email services (Resend/Postmark) | 2020s | Better deliverability, bounce handling, analytics, compliance |
| GET requests cached by default | GET requests NOT cached by default | Next.js 15 | Explicit caching via fetch options, prevents stale data bugs |
| Custom validation logic | Zod schema validation | 2023-2024 standard | Type-safety, automatic error messages, reusable schemas |
| UUID for public URLs | Cryptographically secure random tokens | Security best practice 2020s | Non-guessable, prevents enumeration attacks |

**Deprecated/outdated:**
- **Pages Router:** Still supported but App Router is recommended for new projects (Server Components, streaming)
- **API Routes for internal forms:** Use Server Actions; API Routes only for external clients
- **Math.random() for tokens:** Use crypto.randomBytes (Node.js) or rand crate (Rust) for security-critical randomness

## Open Questions

Things that couldn't be fully resolved:

1. **Render Docker support for free tier**
   - What we know: Free tier supports Docker deployments but services go idle after 15 minutes inactivity
   - What's unclear: Exact cold start times, impact on scan processing, whether background workers stay active
   - Recommendation: Test cold start behavior; upgrade to paid plan if idle timeout causes >30s delays

2. **SSL Labs API exact rate limits**
   - What we know: Enforces maxAssessments (concurrent) and newAssessmentCoolOff (between submissions); limits communicated via headers
   - What's unclear: Exact default values (likely 2-5 concurrent), whether limits increase with API key, cooldown duration after 429
   - Recommendation: Fetch limits from /api/v4/info endpoint; implement header-based tracking; monitor 429 responses

3. **JavaScript secret scanning accuracy**
   - What we know: Pattern + entropy detection; AI-assisted validation emerging; GitGuardian recognizes 350+ secret types
   - What's unclear: Best open-source tool for MVP (SecretFinder vs custom Nuclei templates); validation API limits; false positive rate
   - Recommendation: Start with Nuclei built-in secrets templates; add Trufflehog or GitGuardian if false positives high

4. **Markdown vs PDF for free tier**
   - What we know: User decision is markdown for free, PDF for paid tier
   - What's unclear: User expectations (do users expect PDF even on free tier?); conversion effort for Phase 4
   - Recommendation: Stick to markdown for Phase 2; validate user feedback; prepare PDF library research for Phase 4

5. **Progress stage granularity**
   - What we know: User wants checklist of stages (Headers, TLS, Files, Secrets)
   - What's unclear: How to track stage completion with containerized scanners (Docker exit codes? Database updates?)
   - Recommendation: Update scan record with stage timestamps; poll for status changes; display "In Progress" until container exits

## Sources

### Primary (HIGH confidence)
- [Next.js Forms Official Guide](https://nextjs.org/docs/app/guides/forms) - Server Actions, validation, error handling
- [SSL Labs API v4 Documentation](https://github.com/ssllabs/ssllabs-scan/blob/master/ssllabs-api-docs-v4.md) - Rate limits, polling strategy, headers
- [Render Docker Documentation](https://render.com/docs/docker) - Docker deployment, requirements, limitations
- [testssl.sh Docker Documentation](https://deepwiki.com/testssl/testssl.sh/6-docker-containerization) - Container security, CIS benchmarks
- [Nuclei Docker Image](https://hub.docker.com/r/projectdiscovery/nuclei) - Official Docker image, usage patterns

### Secondary (MEDIUM confidence)
- [Next.js 15 App Router Best Practices (2026)](https://nextjs.org/docs/app) - Verified with official docs
- [Resend vs Postmark Comparison (2026)](https://forwardemail.net/en/blog/postmark-vs-resend-email-service-comparison) - Independent comparison
- [SecurityScorecard Methodology](https://support.securityscorecard.com/hc/en-us/articles/8366223642651-How-SecurityScorecard-calculates-your-scores) - A-F grading methodology
- [OWASP Docker Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Docker_Security_Cheat_Sheet.html) - Security hardening patterns
- [OWASP SSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html) - SSRF mitigation strategies

### Tertiary (LOW confidence - requires validation)
- [Render Free Tier Idle Behavior](https://www.freetiers.com/directory/render) - Community-reported, needs testing
- [JavaScript Secret Scanning Tools Comparison (2026)](https://blog.gitguardian.com/secret-scanning-tools/) - Marketing content, not neutral comparison
- [Secrets in JavaScript Bundles Research](https://www.intruder.io/research/secrets-detection-javascript) - Single source, needs verification

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Next.js, Zod, Nuclei, testssl.sh verified with official docs
- Architecture: HIGH - Server Actions, polling, token patterns verified with official Next.js docs
- Pitfalls: MEDIUM - SSL Labs rate limits need testing; false positive patterns based on community reports
- Email services: MEDIUM - Resend/Postmark comparison from secondary sources; deliverability claims need validation

**Research date:** 2026-02-05
**Valid until:** 2026-03-07 (30 days - relatively stable stack)

**Note:** Version numbers for Rust crates (Phase 1 research) not re-verified; use existing Phase 1 research for backend libraries.
