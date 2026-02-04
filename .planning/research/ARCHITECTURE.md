# Architecture Patterns for Security Scanning SaaS

**Domain:** Security scanning SaaS platform
**Project:** TrustEdge Audit
**Researched:** 2026-02-04
**Confidence:** MEDIUM (based on training knowledge of similar systems, not verified with current sources due to tool restrictions)

## Executive Summary

Security scanning SaaS systems follow a producer-consumer pattern with clear separation between web API, job orchestration, scanner execution, and result processing. The architecture must handle concurrent scans, long-running jobs (30s-5min), containerized tool isolation, and multi-stage result aggregation. For TrustEdge Audit, a Rust backend with PostgreSQL, background worker pool, and containerized scanners provides the optimal balance of performance, safety, and operational simplicity on Render.

## Recommended Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Layer                              │
│  ┌──────────────────┐              ┌──────────────────┐        │
│  │   Landing Page   │              │  Results Dashboard│        │
│  │   (Next.js SSG)  │              │   (Next.js SSR)   │        │
│  └──────────────────┘              └──────────────────┘        │
└─────────────────────────────────────────────────────────────────┘
                         │                       │
                         ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                      API Gateway Layer                          │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │   Rust Backend (Axum)                                    │  │
│  │   - POST /api/scans (create scan)                        │  │
│  │   - GET  /api/scans/:id (poll status)                    │  │
│  │   - GET  /api/scans/:id/results (findings)               │  │
│  │   - GET  /api/scans/:id/pdf (download report)            │  │
│  │   - POST /api/payments/webhook (Stripe)                  │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Data Layer                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │   PostgreSQL                                             │  │
│  │   - scans (id, url, status, tier, created_at)           │  │
│  │   - scan_jobs (scan_id, scanner_type, status, output)   │  │
│  │   - findings (scan_id, severity, title, description)    │  │
│  │   - users (email, stripe_customer_id)                   │  │
│  │   - payments (user_id, amount, status)                  │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Job Orchestration Layer                       │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │   Scan Orchestrator (in-process Rust worker pool)       │  │
│  │   - Polling loop: SELECT jobs WHERE status='pending'    │  │
│  │   - Spawn scanner tasks (tokio tasks)                   │  │
│  │   - Timeout enforcement (tokio::time::timeout)          │  │
│  │   - Concurrency limit (semaphore, max 5 concurrent)     │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Scanner Execution Layer                      │
│  ┌──────────────┬──────────────┬──────────────┬──────────────┐ │
│  │  Headers     │  TLS         │  Files       │  Secrets     │ │
│  │  Scanner     │  Scanner     │  Scanner     │  Scanner     │ │
│  │              │              │              │              │ │
│  │  (HTTP lib)  │  (testssl)   │  (Nuclei)    │  (regex)     │ │
│  │  In-process  │  Container   │  Container   │  In-process  │ │
│  └──────────────┴──────────────┴──────────────┴──────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Findings Processing Layer                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │   Findings Aggregator                                    │  │
│  │   - Parse scanner output (JSON/text)                     │  │
│  │   - Normalize to common schema                          │  │
│  │   - Deduplicate findings (hash title+url+type)          │  │
│  │   - Apply severity scoring (critical/high/med/low/info) │  │
│  │   - Map to remediation playbooks                        │  │
│  │   - Write to findings table                             │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Delivery Layer                             │
│  ┌──────────────────┐              ┌──────────────────┐        │
│  │  Email Service   │              │  PDF Generator   │        │
│  │  (Resend/SES)    │              │  (typst/weasypr) │        │
│  └──────────────────┘              └──────────────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

## Component Boundaries

### 1. Web API (Rust Backend)

**Responsibility:** HTTP request handling, authentication, business logic
**Technology:** Axum (recommended over Actix-web for simpler async and better ecosystem fit)
**Communicates with:** PostgreSQL, Scanner Orchestrator (in-process), Email Service, Stripe API

**Key endpoints:**
```rust
// Scan lifecycle
POST   /api/scans              // Create scan, return scan_id
GET    /api/scans/:id          // Poll status (pending/running/completed/failed)
GET    /api/scans/:id/results  // Retrieve findings (JSON)
GET    /api/scans/:id/pdf      // Download PDF report

// Payment flow
POST   /api/checkout           // Create Stripe checkout session
POST   /api/payments/webhook   // Stripe webhook handler

// Health and status
GET    /health                 // Kubernetes-style healthcheck
GET    /api/scanners/status    // Worker pool status
```

**Responsibilities:**
- Request validation (URL format, tier permissions)
- Scan creation (INSERT INTO scans, spawn jobs)
- Status polling (SELECT status FROM scans WHERE id=?)
- Results retrieval (JOIN scans and findings)
- Authentication (JWT or session for paid tier dashboard)
- Rate limiting (per-IP for free tier, per-user for paid)

### 2. Database (PostgreSQL)

**Responsibility:** Persistent storage, transactional integrity, queuing primitive
**Technology:** PostgreSQL 15+ with SQLx (compile-time checked queries)
**Communicates with:** Web API, Scanner Orchestrator

**Schema:**

```sql
-- Scans table (one row per scan request)
CREATE TABLE scans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    url TEXT NOT NULL,
    email TEXT NOT NULL,
    tier TEXT NOT NULL CHECK (tier IN ('free', 'paid')),
    status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    error_message TEXT
);

-- Scan jobs table (one row per scanner invocation)
CREATE TABLE scan_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_id UUID NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
    scanner_type TEXT NOT NULL CHECK (scanner_type IN ('headers', 'tls', 'files', 'secrets', 'nuclei')),
    status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    raw_output JSONB,  -- Store scanner output
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    UNIQUE(scan_id, scanner_type)  -- One job per scanner per scan
);

-- Findings table (normalized security issues)
CREATE TABLE findings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_id UUID NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
    severity TEXT NOT NULL CHECK (severity IN ('critical', 'high', 'medium', 'low', 'info')),
    category TEXT NOT NULL,  -- 'headers', 'tls', 'secrets', 'exposure', 'vuln'
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    remediation_id TEXT,  -- Maps to remediation playbook
    metadata JSONB,  -- Scanner-specific data (e.g., CVE, CVSS score)
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Users table (for paid tier)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    stripe_customer_id TEXT,
    tier TEXT NOT NULL DEFAULT 'free' CHECK (tier IN ('free', 'paid_once', 'pro', 'agency')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Payments table
CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    stripe_payment_id TEXT NOT NULL,
    amount INTEGER NOT NULL,  -- Cents
    currency TEXT NOT NULL DEFAULT 'usd',
    status TEXT NOT NULL CHECK (status IN ('pending', 'succeeded', 'failed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes for performance
CREATE INDEX idx_scans_status ON scans(status) WHERE status IN ('pending', 'running');
CREATE INDEX idx_scan_jobs_pending ON scan_jobs(status, created_at) WHERE status = 'pending';
CREATE INDEX idx_findings_scan ON findings(scan_id);
CREATE INDEX idx_scans_email ON scans(email);
```

**Why PostgreSQL:**
- JSONB for flexible scanner output storage
- Strong typing (ENUMs via CHECK constraints)
- ACID transactions for scan creation (insert scan + insert jobs atomically)
- Native UUID support
- Listen/Notify for real-time updates (future: websocket status)
- Mature Rust ecosystem (SQLx, diesel)

### 3. Scanner Orchestrator (Worker Pool)

**Responsibility:** Job queue processing, concurrency control, timeout enforcement
**Technology:** In-process Rust worker (tokio tasks, no external queue)
**Communicates with:** PostgreSQL, Scanner Execution Layer

**Architecture choice: Database as queue vs. Redis/RabbitMQ**

For this system, **database-as-queue** is recommended:
- Simpler operations (one fewer service)
- Transactional scan creation + job enqueue
- Sufficient performance for <1000 scans/day
- Render deployment simplicity

**Implementation pattern:**

```rust
// Worker pool (runs in background tokio task)
async fn scanner_orchestrator(pool: PgPool) {
    let semaphore = Arc::new(Semaphore::new(5)); // Max 5 concurrent scans

    loop {
        // Poll for pending jobs
        let jobs = sqlx::query!(
            "SELECT id, scan_id, scanner_type FROM scan_jobs
             WHERE status = 'pending'
             ORDER BY created_at ASC
             LIMIT 10"
        )
        .fetch_all(&pool)
        .await?;

        for job in jobs {
            let permit = semaphore.clone().acquire_owned().await?;
            let pool = pool.clone();

            tokio::spawn(async move {
                let _permit = permit; // Hold semaphore
                execute_scan_job(pool, job).await;
            });
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

async fn execute_scan_job(pool: PgPool, job: ScanJob) {
    // Mark as running
    sqlx::query!("UPDATE scan_jobs SET status = 'running', started_at = now() WHERE id = $1", job.id)
        .execute(&pool).await?;

    // Execute scanner with timeout
    let result = tokio::time::timeout(
        Duration::from_secs(300), // 5 min max
        run_scanner(job.scanner_type, job.scan_id)
    ).await;

    match result {
        Ok(Ok(output)) => {
            // Success: store output, extract findings
            sqlx::query!(
                "UPDATE scan_jobs SET status = 'completed', raw_output = $1, completed_at = now() WHERE id = $2",
                output, job.id
            ).execute(&pool).await?;

            process_findings(pool, job.scan_id, job.scanner_type, output).await;
        }
        Ok(Err(e)) => {
            // Scanner error
            sqlx::query!(
                "UPDATE scan_jobs SET status = 'failed', error_message = $1, completed_at = now() WHERE id = $2",
                e.to_string(), job.id
            ).execute(&pool).await?;
        }
        Err(_) => {
            // Timeout
            sqlx::query!(
                "UPDATE scan_jobs SET status = 'failed', error_message = 'Timeout', completed_at = now() WHERE id = $1",
                job.id
            ).execute(&pool).await?;
        }
    }

    // Check if all jobs complete, update scan status
    update_scan_status(pool, job.scan_id).await;
}
```

**Concurrency strategy:**
- Semaphore limits concurrent scanner executions (prevents resource exhaustion)
- Each scanner runs in isolated tokio task
- Database polling every 2 seconds (sufficient latency for user experience)
- Future optimization: PostgreSQL LISTEN/NOTIFY for immediate job dispatch

**Timeout handling:**
- Hard timeout: 5 minutes per scanner (tokio::time::timeout)
- Soft timeout: 2 minutes warning (log slow scanner)
- Container timeout: Docker --timeout flag as backstop

### 4. Scanner Execution Layer

**Responsibility:** Invoke security scanning tools, capture output
**Technology:** Mix of in-process (Rust) and containerized (Docker) scanners
**Communicates with:** Scanner Orchestrator

**Scanner types and execution models:**

| Scanner | Execution | Reason |
|---------|-----------|--------|
| Headers | In-process (reqwest) | Simple HTTP fetch, no external deps |
| Secrets | In-process (regex) | JS bundle fetch + pattern matching |
| TLS | Container (testssl.sh) | Complex bash script, shellshock risk |
| Files | Container (Nuclei) | Go binary, template isolation |
| Nuclei | Container (Nuclei) | Active scanning, network isolation |

**Container execution pattern:**

```rust
async fn run_nuclei_scan(url: &str) -> Result<ScanOutput> {
    let output = Command::new("docker")
        .args([
            "run", "--rm",
            "--network", "bridge",  // Isolated network
            "--cpus", "1.0",         // CPU limit
            "--memory", "512m",      // Memory limit
            "--timeout", "240s",     // Container timeout
            "projectdiscovery/nuclei:latest",
            "-u", url,
            "-t", "/app/templates/",  // Custom templates
            "-json",                   // JSON output
            "-silent"                  // No banner
        ])
        .output()
        .await?;

    if !output.status.success() {
        return Err(ScanError::ToolFailed(String::from_utf8_lossy(&output.stderr).to_string()));
    }

    Ok(serde_json::from_slice(&output.stdout)?)
}
```

**Container resource limits:**
- CPU: 1 core max (prevents runaway processes)
- Memory: 512MB (sufficient for Nuclei, testssl.sh)
- Network: Bridge mode (outbound only, no inter-container)
- Timeout: 4 minutes (shorter than orchestrator timeout)

**In-process scanner pattern:**

```rust
async fn scan_headers(url: &str) -> Result<HeaderFindings> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let resp = client.get(url).send().await?;

    let mut findings = Vec::new();

    // Check for missing security headers
    if resp.headers().get("content-security-policy").is_none() {
        findings.push(Finding {
            severity: Severity::Medium,
            title: "Missing Content-Security-Policy header".into(),
            description: "CSP prevents XSS attacks by controlling resource loading".into(),
            remediation_id: Some("missing_csp".into()),
        });
    }

    // Check HSTS
    if let Some(hsts) = resp.headers().get("strict-transport-security") {
        let value = hsts.to_str()?;
        if !value.contains("max-age") {
            findings.push(Finding {
                severity: Severity::High,
                title: "Invalid HSTS header".into(),
                description: format!("HSTS value '{}' missing max-age directive", value),
                remediation_id: Some("invalid_hsts".into()),
            });
        }
    }

    Ok(HeaderFindings { findings })
}
```

### 5. Findings Processing Layer

**Responsibility:** Parse scanner output, normalize findings, deduplicate, score severity, map to remediation
**Technology:** Rust (part of backend service)
**Communicates with:** PostgreSQL

**Data flow:**

```
Raw scanner output (JSON/text)
    ↓
Parse to scanner-specific structs
    ↓
Normalize to common Finding schema
    ↓
Deduplicate (hash-based: title + URL + category)
    ↓
Apply severity scoring rules
    ↓
Map to remediation playbook ID
    ↓
INSERT INTO findings table
```

**Normalization pattern:**

```rust
struct Finding {
    severity: Severity,      // critical, high, medium, low, info
    category: Category,      // headers, tls, secrets, exposure, vuln
    title: String,
    description: String,
    remediation_id: Option<String>,
    metadata: serde_json::Value,
}

impl From<NucleiOutput> for Vec<Finding> {
    fn from(nuclei: NucleiOutput) -> Vec<Finding> {
        nuclei.results.into_iter().map(|r| {
            Finding {
                severity: match r.info.severity.as_str() {
                    "critical" => Severity::Critical,
                    "high" => Severity::High,
                    "medium" => Severity::Medium,
                    "low" => Severity::Low,
                    _ => Severity::Info,
                },
                category: Category::Vuln,
                title: r.info.name,
                description: r.info.description.unwrap_or_default(),
                remediation_id: r.template_id.map(|id| format!("nuclei_{}", id)),
                metadata: json!({
                    "matched_at": r.matched_at,
                    "matcher_name": r.matcher_name,
                }),
            }
        }).collect()
    }
}
```

**Deduplication strategy:**
```rust
fn dedup_findings(findings: Vec<Finding>) -> Vec<Finding> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    findings.into_iter().filter(|f| {
        let key = format!("{}-{}-{}", f.title, f.category, f.severity);
        seen.insert(key)
    }).collect()
}
```

**Severity scoring rules:**

| Finding Type | Default Severity | Upgrade Conditions |
|-------------|------------------|-------------------|
| Missing CSP | Medium | → High if no XSS protection headers |
| Missing HSTS | Medium | → High if site handles auth |
| Hardcoded secret | High | → Critical if production API key pattern |
| TLS < 1.2 | High | Always critical |
| Exposed .env | Critical | Always critical |
| Missing header | Low | → Medium if security-relevant |

### 6. Frontend (Next.js)

**Responsibility:** User interface, form submission, results display
**Technology:** Next.js 14+ (App Router), Tailwind CSS, shadcn/ui
**Communicates with:** Rust backend API

**Page structure:**

```
/app
├── page.tsx                    // Landing page (SSG)
├── scan/page.tsx               // Scan submission form
├── scan/[id]/page.tsx          // Results page (SSR)
├── scan/[id]/pdf/route.ts      // PDF download proxy
├── dashboard/page.tsx          // Paid tier dashboard
├── api/
│   └── checkout/route.ts       // Stripe checkout session
└── components/
    ├── ScanForm.tsx
    ├── ResultsTable.tsx
    ├── FindingCard.tsx
    └── SeverityBadge.tsx
```

**Scan submission flow:**

```typescript
// app/scan/page.tsx
'use client';

export default function ScanPage() {
  const [scanId, setScanId] = useState<string | null>(null);
  const [status, setStatus] = useState<'idle' | 'submitting' | 'polling' | 'complete'>('idle');

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    setStatus('submitting');

    // Create scan
    const res = await fetch('/api/scans', {
      method: 'POST',
      body: JSON.stringify({ url, email, tier: 'free' }),
    });
    const { scan_id } = await res.json();
    setScanId(scan_id);
    setStatus('polling');

    // Poll for completion
    const interval = setInterval(async () => {
      const statusRes = await fetch(`/api/scans/${scan_id}`);
      const { status } = await statusRes.json();

      if (status === 'completed') {
        clearInterval(interval);
        setStatus('complete');
        router.push(`/scan/${scan_id}`);
      } else if (status === 'failed') {
        clearInterval(interval);
        // Handle error
      }
    }, 2000); // Poll every 2 seconds
  }

  return (
    <form onSubmit={handleSubmit}>
      <input type="url" name="url" required />
      <input type="email" name="email" required />
      <button type="submit">Scan Now</button>

      {status === 'polling' && <PollingIndicator />}
    </form>
  );
}
```

**Results page (SSR for SEO):**

```typescript
// app/scan/[id]/page.tsx
export default async function ResultsPage({ params }: { params: { id: string } }) {
  // Server-side fetch
  const findings = await fetch(`${process.env.BACKEND_URL}/api/scans/${params.id}/results`);
  const data = await findings.json();

  return (
    <div>
      <h1>Scan Results</h1>
      <SeverityChart findings={data.findings} />
      <FindingsTable findings={data.findings} />

      {data.tier === 'free' && <UpgradePrompt scanId={params.id} />}
      {data.tier === 'paid' && <PDFDownloadButton scanId={params.id} />}
    </div>
  );
}
```

### 7. Email Service

**Responsibility:** Transactional emails (scan complete, PDF delivery)
**Technology:** Resend (recommended) or AWS SES
**Communicates with:** Rust backend (outbound only)

**Why Resend over SES:**
- Better DX (simpler API, webhook handling)
- Built-in template system
- Free tier: 3,000 emails/month (sufficient for MVP)
- Render-friendly (no AWS credentials complexity)

**Email triggers:**

| Event | Template | Content |
|-------|----------|---------|
| Free scan complete | scan_complete_free | Summary + link to results + upgrade CTA |
| Paid scan complete | scan_complete_paid | Summary + PDF attachment + dashboard link |
| Scan failed | scan_failed | Error message + support email |
| Payment success | payment_success | Receipt + scan limit increase notice |

**Implementation:**

```rust
async fn send_scan_complete_email(scan: &Scan, findings_summary: &FindingsSummary) -> Result<()> {
    let client = reqwest::Client::new();

    let email = if scan.tier == "free" {
        json!({
            "from": "TrustEdge Audit <scans@trustedge.audit>",
            "to": [scan.email],
            "subject": format!("Security Scan Complete: {} findings", findings_summary.total),
            "html": render_template("scan_complete_free", json!({
                "url": scan.url,
                "critical_count": findings_summary.critical,
                "high_count": findings_summary.high,
                "results_link": format!("https://trustedge.audit/scan/{}", scan.id),
            })),
        })
    } else {
        // Paid tier: attach PDF
        let pdf_bytes = generate_pdf_report(scan.id).await?;
        let pdf_base64 = base64::encode(pdf_bytes);

        json!({
            "from": "TrustEdge Audit <scans@trustedge.audit>",
            "to": [scan.email],
            "subject": format!("Security Audit Complete: {}", scan.url),
            "html": render_template("scan_complete_paid", /* ... */),
            "attachments": [{
                "filename": format!("security-audit-{}.pdf", scan.id),
                "content": pdf_base64,
            }],
        })
    };

    client.post("https://api.resend.com/emails")
        .header("Authorization", format!("Bearer {}", env!("RESEND_API_KEY")))
        .json(&email)
        .send()
        .await?;

    Ok(())
}
```

### 8. PDF Generation

**Responsibility:** Professional security audit reports
**Technology:** typst (recommended) or WeasyPrint (HTML-to-PDF)
**Communicates with:** Rust backend (in-process or container)

**Why typst over WeasyPrint:**
- Native binary (easier to embed in Rust)
- Faster rendering (compiled, not Python)
- Better typography (designed for technical documents)
- Smaller attack surface (no HTML/CSS parser)

**Alternative: WeasyPrint** if you need HTML templates for easier iteration.

**Report structure:**

```
┌─────────────────────────────────────┐
│  TRUSTEDGE AUDIT                    │
│  Security Scan Report               │
│  Date: 2026-02-04                   │
│  URL: https://example.com           │
└─────────────────────────────────────┘

Executive Summary
─────────────────
✗ 3 Critical findings
✗ 5 High findings
⚠ 12 Medium findings
ℹ 8 Low/Info findings

Overall Grade: C

Findings by Severity
────────────────────

CRITICAL
────────
1. Hardcoded API key exposed in JavaScript
   Location: /static/js/bundle.min.js
   Risk: Attacker can access your database

   Remediation:
   1. Rotate exposed key immediately
   2. Move API calls to backend
   [Code example]

2. Missing Row Level Security on users table
   [...]

Recommendations
───────────────
1. Enable RLS on all Supabase tables
2. Implement CSP header
3. Rotate all exposed secrets

Appendix
────────
- Scan metadata
- Methodology
- Contact info
```

**Implementation options:**

```rust
// Option 1: typst (recommended)
async fn generate_pdf_typst(scan_id: Uuid) -> Result<Vec<u8>> {
    let findings = fetch_findings(scan_id).await?;

    // Render typst template
    let template = include_str!("../templates/report.typ");
    let rendered = render_typst_template(template, findings)?;

    // Compile to PDF
    let pdf = std::process::Command::new("typst")
        .args(["compile", "-", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    Ok(pdf.stdout)
}

// Option 2: WeasyPrint (HTML-based)
async fn generate_pdf_weasyprint(scan_id: Uuid) -> Result<Vec<u8>> {
    let findings = fetch_findings(scan_id).await?;

    // Render HTML template (use askama or tera)
    let html = render_html_template("report.html", findings)?;

    // Convert to PDF via container
    let output = Command::new("docker")
        .args([
            "run", "--rm", "-i",
            "weasyprint/weasyprint:latest",
            "-", "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    output.stdin.unwrap().write_all(html.as_bytes())?;
    let pdf = output.wait_with_output()?.stdout;

    Ok(pdf)
}
```

## Data Flow

### Scan Creation Flow

```
User submits URL + email
    ↓
Next.js frontend → POST /api/scans
    ↓
Rust backend validates input
    ↓
BEGIN TRANSACTION
  INSERT INTO scans (url, email, tier, status='pending')
  INSERT INTO scan_jobs (scan_id, scanner_type='headers', status='pending')
  INSERT INTO scan_jobs (scan_id, scanner_type='tls', status='pending')
  INSERT INTO scan_jobs (scan_id, scanner_type='files', status='pending')
  INSERT INTO scan_jobs (scan_id, scanner_type='secrets', status='pending')
COMMIT
    ↓
Return scan_id to frontend
    ↓
Frontend polls GET /api/scans/:id every 2 seconds
```

### Scanner Execution Flow

```
Orchestrator polls: SELECT * FROM scan_jobs WHERE status='pending'
    ↓
For each job (up to 5 concurrent):
    ↓
UPDATE scan_jobs SET status='running', started_at=now()
    ↓
Execute scanner (in-process or container)
    ↓
Capture output (stdout/stderr)
    ↓
UPDATE scan_jobs SET status='completed', raw_output=[JSON], completed_at=now()
    ↓
Parse output → normalize findings
    ↓
Deduplicate findings
    ↓
INSERT INTO findings (scan_id, severity, title, description, remediation_id)
    ↓
Check if all jobs complete:
  IF all jobs complete:
    UPDATE scans SET status='completed', completed_at=now()
    Trigger email delivery
```

### Results Retrieval Flow

```
User opens /scan/:id
    ↓
Next.js SSR → GET /api/scans/:id/results
    ↓
Rust backend queries:
  SELECT * FROM findings WHERE scan_id = :id ORDER BY severity DESC
    ↓
Group by severity, attach remediation text
    ↓
Return JSON to frontend
    ↓
Frontend renders findings table
    ↓
If paid tier: Show PDF download button
    ↓
User clicks PDF download
    ↓
GET /api/scans/:id/pdf
    ↓
Generate PDF (typst/weasyprint)
    ↓
Cache PDF in object storage (future optimization)
    ↓
Stream PDF to browser
```

### Payment Flow

```
User clicks "Upgrade to Paid Scan"
    ↓
Frontend → POST /api/checkout
    ↓
Rust backend creates Stripe checkout session
    ↓
Redirect to Stripe Checkout
    ↓
User completes payment
    ↓
Stripe webhook → POST /api/payments/webhook
    ↓
Verify webhook signature
    ↓
UPDATE scans SET tier='paid' WHERE id=:scan_id
INSERT INTO payments (user_id, amount, status='succeeded')
    ↓
Trigger deeper scan (nuclei, additional checks)
    ↓
Completion triggers PDF generation + email
```

## Deployment Topology on Render

Render deployment uses three services:

```
┌──────────────────────────────────────────────────────────────┐
│  Render Region (Oregon)                                      │
│                                                              │
│  ┌────────────────────┐                                     │
│  │  Web Service       │                                     │
│  │  (Rust Backend)    │                                     │
│  │  - Axum HTTP       │                                     │
│  │  - Worker pool     │                                     │
│  │  - Docker access   │                                     │
│  │  Instance: 512MB   │                                     │
│  └────────────────────┘                                     │
│           │                                                  │
│           ↓                                                  │
│  ┌────────────────────┐       ┌──────────────────────────┐ │
│  │  PostgreSQL        │       │  Static Site (Next.js)   │ │
│  │  (Managed)         │       │  - Landing page (SSG)    │ │
│  │  - 1GB storage     │       │  - Results (SSR)         │ │
│  │  - Backups daily   │       │  - API proxy to backend  │ │
│  └────────────────────┘       └──────────────────────────┘ │
│                                                              │
│  Environment Variables:                                     │
│  - DATABASE_URL (Render managed)                            │
│  - RESEND_API_KEY                                           │
│  - STRIPE_SECRET_KEY                                        │
│  - STRIPE_WEBHOOK_SECRET                                    │
│  - BACKEND_URL (for Next.js SSR)                            │
└──────────────────────────────────────────────────────────────┘
```

**Service configuration:**

1. **Rust Backend (Web Service)**
   - Build command: `cargo build --release`
   - Start command: `./target/release/trustedge-backend`
   - Health check: `GET /health`
   - Docker enabled: Yes (for scanner containers)
   - Instance size: 512MB (free tier) → 1GB (production)
   - Auto-deploy: Yes (on main branch push)

2. **PostgreSQL (Managed Database)**
   - Version: 15
   - Storage: 1GB (free tier) → 10GB (production)
   - Backups: Daily (7-day retention)
   - Connection pooling: Render managed

3. **Next.js Frontend (Static Site)**
   - Build command: `npm run build`
   - Publish directory: `.next`
   - Environment: `NEXT_PUBLIC_API_URL=$BACKEND_URL`
   - CDN: Render global edge

**Container execution on Render:**

Render Web Services have Docker daemon access, allowing containerized scanner execution:

```rust
// Executes on Render compute instance
let output = Command::new("docker")
    .args(["run", "--rm", "projectdiscovery/nuclei:latest", /* ... */])
    .output()
    .await?;
```

**Limitations and workarounds:**

| Limitation | Workaround |
|-----------|------------|
| No background workers service in free tier | Use in-process worker pool (polling pattern) |
| Docker image pull latency on first run | Pre-pull images in Dockerfile, cache in instance |
| 512MB RAM limit (free tier) | Limit concurrent scans to 3, use memory-efficient scanners |
| No object storage in free tier | Store PDFs in PostgreSQL BYTEA (up to 1MB) or defer to paid tier |

## API Design

### REST Endpoints

**Scan lifecycle:**

```
POST /api/scans
Request:
{
  "url": "https://example.com",
  "email": "user@example.com",
  "tier": "free" | "paid",
  "payment_intent_id": "pi_xxx" (if tier=paid)
}
Response:
{
  "scan_id": "uuid",
  "status": "pending",
  "created_at": "2026-02-04T12:00:00Z"
}

GET /api/scans/:id
Response:
{
  "scan_id": "uuid",
  "url": "https://example.com",
  "status": "pending" | "running" | "completed" | "failed",
  "created_at": "2026-02-04T12:00:00Z",
  "completed_at": "2026-02-04T12:05:23Z",
  "tier": "free"
}

GET /api/scans/:id/results
Response:
{
  "scan_id": "uuid",
  "url": "https://example.com",
  "summary": {
    "critical": 2,
    "high": 5,
    "medium": 12,
    "low": 8,
    "info": 3
  },
  "findings": [
    {
      "id": "uuid",
      "severity": "critical",
      "category": "secrets",
      "title": "Hardcoded API key in JavaScript bundle",
      "description": "...",
      "remediation": {
        "id": "hardcoded_secret_js",
        "title": "How to fix hardcoded secrets",
        "steps": ["1. Rotate key", "2. Move to backend", "3. Use env vars"],
        "code_example": "..."
      }
    }
  ]
}

GET /api/scans/:id/pdf
Response: application/pdf (stream)
```

**Payment endpoints:**

```
POST /api/checkout
Request:
{
  "scan_id": "uuid",
  "tier": "paid_once" | "pro" | "agency"
}
Response:
{
  "checkout_url": "https://checkout.stripe.com/xxx"
}

POST /api/payments/webhook
Headers: stripe-signature
Request: Stripe webhook payload
Response: 200 OK
```

**Rate limiting:**

```
Free tier: 5 scans per email per day
Paid tier (one-time): 1 scan per payment
Pro tier: Unlimited scans
```

Implementation:
```rust
// Rate limiting middleware
async fn rate_limit_middleware(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, StatusCode> {
    let email = extract_email(&req)?;
    let tier = lookup_tier(email).await?;

    if tier == "free" {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM scans WHERE email = $1 AND created_at > now() - interval '24 hours'",
            email
        ).fetch_one(&pool).await?;

        if count >= 5 {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
    }

    Ok(next.run(req).await)
}
```

## Patterns to Follow

### Pattern 1: Scan State Machine

**What:** Explicit state transitions for scan lifecycle
**When:** Every scan status change
**Why:** Prevents invalid states (e.g., completed → running), enables audit trail

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum ScanStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl ScanStatus {
    fn can_transition_to(&self, new: ScanStatus) -> bool {
        match (self, new) {
            (Pending, Running) => true,
            (Running, Completed) => true,
            (Running, Failed) => true,
            _ => false,
        }
    }
}

async fn update_scan_status(pool: &PgPool, scan_id: Uuid, new_status: ScanStatus) -> Result<()> {
    let current = sqlx::query_scalar!("SELECT status FROM scans WHERE id = $1", scan_id)
        .fetch_one(pool).await?;

    let current_status = ScanStatus::from_str(&current)?;

    if !current_status.can_transition_to(new_status) {
        return Err(Error::InvalidStateTransition(current_status, new_status));
    }

    sqlx::query!(
        "UPDATE scans SET status = $1, updated_at = now() WHERE id = $2",
        new_status.to_string(), scan_id
    ).execute(pool).await?;

    Ok(())
}
```

### Pattern 2: Structured Scanner Output

**What:** Each scanner returns a typed struct, converted to common Finding schema
**When:** All scanner integrations
**Why:** Type safety, easier testing, consistent normalization

```rust
// Scanner-specific output
#[derive(Deserialize)]
struct NucleiResult {
    template_id: String,
    info: NucleiInfo,
    matched_at: String,
}

// Common finding schema
struct Finding {
    severity: Severity,
    category: Category,
    title: String,
    description: String,
    remediation_id: Option<String>,
}

// Conversion trait
trait ToFindings {
    fn to_findings(&self) -> Vec<Finding>;
}

impl ToFindings for Vec<NucleiResult> {
    fn to_findings(&self) -> Vec<Finding> {
        self.iter().map(|r| Finding {
            severity: map_severity(&r.info.severity),
            category: Category::Vuln,
            title: r.info.name.clone(),
            description: r.info.description.clone().unwrap_or_default(),
            remediation_id: Some(format!("nuclei_{}", r.template_id)),
        }).collect()
    }
}
```

### Pattern 3: Remediation Playbook Mapping

**What:** Map finding types to remediation content via ID
**When:** Findings aggregation
**Why:** Consistent advice, easy to update remediation content

```rust
// Remediation playbook (loaded from YAML/JSON)
struct RemediationPlaybook {
    id: String,
    title: String,
    steps: Vec<String>,
    code_example: Option<String>,
    references: Vec<String>,
}

// Load at startup
lazy_static! {
    static ref REMEDIATIONS: HashMap<String, RemediationPlaybook> =
        load_remediations("remediations.yaml");
}

// Attach during findings retrieval
async fn get_findings_with_remediation(scan_id: Uuid) -> Result<Vec<FindingWithRemediation>> {
    let findings = sqlx::query_as!(Finding, "SELECT * FROM findings WHERE scan_id = $1", scan_id)
        .fetch_all(&pool).await?;

    findings.into_iter().map(|f| {
        let remediation = f.remediation_id
            .and_then(|id| REMEDIATIONS.get(&id))
            .cloned();

        FindingWithRemediation { finding: f, remediation }
    }).collect()
}
```

### Pattern 4: Graceful Scanner Failure

**What:** Scan continues even if one scanner fails
**When:** Scanner execution
**Why:** Partial results better than no results

```rust
async fn execute_all_scanners(scan_id: Uuid, url: &str) -> Result<()> {
    let scanners = vec![
        ("headers", scan_headers(url)),
        ("tls", scan_tls(url)),
        ("files", scan_files(url)),
        ("secrets", scan_secrets(url)),
    ];

    for (scanner_name, scanner_future) in scanners {
        match scanner_future.await {
            Ok(findings) => {
                insert_findings(scan_id, findings).await?;
                mark_job_complete(scan_id, scanner_name).await?;
            }
            Err(e) => {
                log::error!("Scanner {} failed: {}", scanner_name, e);
                mark_job_failed(scan_id, scanner_name, e.to_string()).await?;
                // Continue to next scanner
            }
        }
    }

    // Mark scan complete even if some scanners failed
    if all_jobs_finished(scan_id).await? {
        update_scan_status(scan_id, ScanStatus::Completed).await?;
    }

    Ok(())
}
```

### Pattern 5: Idempotent Job Processing

**What:** Reprocessing same job produces same result
**When:** Job queue polling
**Why:** Prevents duplicate findings on retry

```rust
async fn process_scan_job(pool: &PgPool, job_id: Uuid) -> Result<()> {
    // Atomic claim: only one worker processes this job
    let claimed = sqlx::query_scalar!(
        "UPDATE scan_jobs SET status = 'running', started_at = now()
         WHERE id = $1 AND status = 'pending'
         RETURNING id",
        job_id
    ).fetch_optional(pool).await?;

    if claimed.is_none() {
        // Another worker claimed it
        return Ok(());
    }

    // Execute scanner
    let output = run_scanner(job_id).await?;

    // Idempotent insert: use ON CONFLICT DO NOTHING for findings
    for finding in output.findings {
        sqlx::query!(
            "INSERT INTO findings (scan_id, severity, title, description, category)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (scan_id, title, category) DO NOTHING",
            finding.scan_id, finding.severity, finding.title, finding.description, finding.category
        ).execute(pool).await?;
    }

    sqlx::query!(
        "UPDATE scan_jobs SET status = 'completed', completed_at = now() WHERE id = $1",
        job_id
    ).execute(pool).await?;

    Ok(())
}
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Synchronous Scan Execution in API Handler

**What goes wrong:** User submits scan → API handler blocks for 2-5 minutes running scanners → timeout
**Why bad:** Poor user experience, server resource exhaustion, no visibility into progress
**Instead:** Async job queue pattern (create scan, return immediately, poll for status)

```rust
// WRONG
async fn create_scan_sync(url: String) -> Result<ScanResults> {
    let findings = run_all_scanners(&url).await?; // Blocks for minutes
    Ok(ScanResults { findings })
}

// CORRECT
async fn create_scan_async(url: String) -> Result<ScanId> {
    let scan_id = insert_scan(&url).await?;
    insert_scan_jobs(scan_id).await?;
    // Worker pool picks up jobs asynchronously
    Ok(scan_id)
}
```

### Anti-Pattern 2: Storing Large Scanner Output in Memory

**What goes wrong:** Nuclei scan produces 50MB JSON → loaded into memory → OOM on concurrent scans
**Why bad:** Memory exhaustion, crashes on Render 512MB instances
**Instead:** Stream scanner output to database, process incrementally

```rust
// WRONG
async fn process_nuclei_output(scan_id: Uuid) -> Result<()> {
    let output = run_nuclei(scan_id).await?; // 50MB string
    let findings: Vec<Finding> = parse_json(&output)?; // All in memory
    insert_findings(findings).await?;
    Ok(())
}

// CORRECT
async fn process_nuclei_output(scan_id: Uuid) -> Result<()> {
    let output_stream = run_nuclei_streaming(scan_id).await?;
    let mut reader = BufReader::new(output_stream);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        if let Ok(finding) = serde_json::from_str::<NucleiFinding>(&line) {
            insert_finding(scan_id, finding.to_finding()).await?;
        }
        line.clear();
    }
    Ok(())
}
```

### Anti-Pattern 3: No Timeout on Scanner Execution

**What goes wrong:** testssl.sh hangs on unresponsive server → worker stuck forever → all workers blocked
**Why bad:** Resource exhaustion, no capacity for new scans
**Instead:** Hard timeout on all scanner executions

```rust
// WRONG
async fn run_scanner(url: &str) -> Result<Output> {
    Command::new("docker")
        .args(["run", "scanner:latest", url])
        .output()
        .await // May never complete
}

// CORRECT
async fn run_scanner(url: &str) -> Result<Output> {
    tokio::time::timeout(
        Duration::from_secs(300), // 5 min hard limit
        Command::new("docker")
            .args(["run", "--timeout", "240s", "scanner:latest", url])
            .output()
    ).await
    .map_err(|_| Error::ScannerTimeout)?
}
```

### Anti-Pattern 4: Exposing Database IDs in URLs

**What goes wrong:** User sees `/scan/123` → tries `/scan/122` → sees other user's results
**Why bad:** Privacy violation, potential data leak
**Instead:** Use UUIDs, check ownership

```rust
// WRONG
GET /api/scans/:id  (where id is integer)

// CORRECT
GET /api/scans/:uuid  (UUID, cryptographically random)

// Plus authorization check
async fn get_scan_results(scan_uuid: Uuid, requester_email: &str) -> Result<ScanResults> {
    let scan = sqlx::query_as!(Scan, "SELECT * FROM scans WHERE id = $1", scan_uuid)
        .fetch_one(&pool).await?;

    // For free tier, only the email owner can see results
    if scan.tier == "free" && scan.email != requester_email {
        return Err(Error::Unauthorized);
    }

    Ok(get_findings(scan_uuid).await?)
}
```

### Anti-Pattern 5: Generating PDF on Every Request

**What goes wrong:** User refreshes results page → regenerates PDF → CPU spike → slow response
**Why bad:** Wasteful computation, poor performance, higher costs
**Instead:** Generate once, cache in database or object storage

```rust
// WRONG
async fn download_pdf(scan_id: Uuid) -> Result<Vec<u8>> {
    let findings = get_findings(scan_id).await?;
    generate_pdf(findings).await // Regenerates every time
}

// CORRECT
async fn download_pdf(scan_id: Uuid) -> Result<Vec<u8>> {
    // Check cache
    if let Some(cached) = get_cached_pdf(scan_id).await? {
        return Ok(cached);
    }

    // Generate once
    let findings = get_findings(scan_id).await?;
    let pdf = generate_pdf(findings).await?;

    // Cache for future requests
    cache_pdf(scan_id, &pdf).await?;

    Ok(pdf)
}

// Implementation with PostgreSQL
async fn cache_pdf(scan_id: Uuid, pdf: &[u8]) -> Result<()> {
    sqlx::query!(
        "UPDATE scans SET pdf_report = $1 WHERE id = $2",
        pdf, scan_id
    ).execute(&pool).await?;
    Ok(())
}
```

### Anti-Pattern 6: Running Scanners as Root in Containers

**What goes wrong:** Nuclei container runs as root → vulnerability in Nuclei → host compromise
**Why bad:** Security risk, violates principle of least privilege
**Instead:** Run containers with non-root user, read-only filesystem

```dockerfile
# WRONG
FROM projectdiscovery/nuclei:latest
# Runs as root by default

# CORRECT
FROM projectdiscovery/nuclei:latest
USER nobody
```

```rust
// CORRECT: Docker run with security constraints
Command::new("docker")
    .args([
        "run", "--rm",
        "--user", "nobody",           // Non-root user
        "--read-only",                // Read-only filesystem
        "--tmpfs", "/tmp:rw,size=10m", // Temp dir for writes
        "--network", "bridge",        // Network isolation
        "--cap-drop", "ALL",          // Drop all capabilities
        "projectdiscovery/nuclei:latest",
        /* ... */
    ])
    .output()
    .await
```

## Scalability Considerations

| Concern | At 100 scans/day | At 1,000 scans/day | At 10,000 scans/day |
|---------|------------------|-------------------|---------------------|
| **Job Queue** | PostgreSQL polling (2s interval) | Same, increase polling workers to 10 | Migrate to Redis/RabbitMQ for lower latency |
| **Database** | 1GB PostgreSQL (Render free) | 10GB PostgreSQL (Render $15/mo) | Read replicas for analytics, pgBouncer for connection pooling |
| **Scanner Concurrency** | 5 concurrent scans | 20 concurrent (2GB instance) | Dedicated worker service, autoscaling (10-50 workers) |
| **PDF Storage** | PostgreSQL BYTEA (1MB/PDF) | Object storage (S3/R2) with DB reference | Same, add CDN caching |
| **Email Delivery** | Resend free tier (3K/mo) | Resend paid ($20/mo, 50K/mo) | AWS SES (cheaper at scale) |
| **Frontend** | Render static site (SSG) | Same, add ISR for results pages | Edge caching (Cloudflare), separate CDN |
| **Rate Limiting** | Database-backed (query count) | Redis-backed sliding window | Cloudflare WAF + Redis |

### Migration Path: 100 → 10,000 scans/day

**Phase 1 (100/day):** Single Render service, in-process workers, PostgreSQL queue
**Phase 2 (1,000/day):** Separate worker service, Redis queue, object storage for PDFs
**Phase 3 (10,000/day):** Autoscaling workers, read replicas, edge caching, Cloudflare

## Build Order Recommendations

Suggested development sequence based on dependencies:

### Phase 1: Core Infrastructure (Week 1)
1. PostgreSQL schema + migrations
2. Rust backend skeleton (Axum, SQLx)
3. Basic health check endpoint
4. Deploy to Render

**Dependencies:** None
**Deliverable:** Backend responds to GET /health

### Phase 2: Simple Scanner (Week 1)
1. Headers scanner (in-process, reqwest)
2. POST /api/scans endpoint (creates scan + jobs)
3. GET /api/scans/:id endpoint (returns status)
4. Worker loop (poll pending jobs, execute headers scanner)
5. Findings table + insertion

**Dependencies:** Phase 1
**Deliverable:** Can scan headers, store findings

### Phase 3: Frontend MVP (Week 2)
1. Next.js landing page (form to submit URL + email)
2. Polling mechanism (GET /api/scans/:id every 2s)
3. Results page (display findings)
4. Deploy Next.js to Render

**Dependencies:** Phase 2
**Deliverable:** End-to-end free tier flow

### Phase 4: Email Delivery (Week 2)
1. Resend integration
2. Email template rendering
3. Trigger email on scan completion
4. Free tier email (summary + link)

**Dependencies:** Phase 3
**Deliverable:** Users receive email when scan completes

### Phase 5: Additional Scanners (Week 3)
1. TLS scanner (testssl.sh in container)
2. File scanner (Nuclei in container)
3. Secrets scanner (regex-based)
4. Findings normalization for each scanner
5. Deduplication logic

**Dependencies:** Phase 2 (scanner infrastructure)
**Deliverable:** Comprehensive scan results

### Phase 6: Payments (Week 4)
1. Stripe integration (checkout session creation)
2. Webhook handler (payment success)
3. Tier-based scan logic (free vs. paid)
4. Paid tier email template

**Dependencies:** Phase 3 (frontend), Phase 4 (email)
**Deliverable:** Users can purchase paid scans

### Phase 7: PDF Reports (Week 5)
1. typst template design
2. PDF generation function
3. GET /api/scans/:id/pdf endpoint
4. Attach PDF to paid tier emails
5. Cache PDFs in database

**Dependencies:** Phase 5 (all scanners), Phase 6 (tier detection)
**Deliverable:** Paid users receive PDF reports

### Phase 8: Dashboard (Week 6)
1. User authentication (JWT or session)
2. Dashboard page (list user's scans)
3. Scan history
4. Re-scan functionality

**Dependencies:** Phase 6 (user accounts)
**Deliverable:** Logged-in users see scan history

### Phase 9: Continuous Monitoring (Week 6)
1. GitHub App setup
2. Webhook receiver
3. Automated re-scan on push
4. Email alerts for new findings

**Dependencies:** Phase 8 (user accounts), Phase 5 (scanners)
**Deliverable:** Pro tier continuous monitoring

## Technology Decisions Summary

| Component | Recommended | Alternative | Reason |
|-----------|-------------|-------------|--------|
| **Backend framework** | Axum | Actix-web | Simpler async, better ecosystem, less boilerplate |
| **Database client** | SQLx | Diesel | Compile-time query checking, async-first |
| **Job queue** | PostgreSQL-backed | Redis, RabbitMQ | Simpler ops, sufficient for <1K scans/day |
| **Worker pattern** | In-process tokio tasks | Separate service | Fewer moving parts, easier deployment |
| **Container runtime** | Docker CLI | podman | Universal availability, Render compatibility |
| **Email service** | Resend | AWS SES | Better DX, simpler API, generous free tier |
| **PDF generator** | typst | WeasyPrint | Faster, safer, native binary |
| **Frontend framework** | Next.js 14 App Router | SvelteKit | Better Rust backend integration, larger ecosystem |
| **Styling** | Tailwind + shadcn/ui | Plain CSS | Faster iteration, professional components |
| **Payment processing** | Stripe | Paddle | Standard, well-documented, Rust SDK available |
| **Hosting** | Render | Fly.io, Railway | Docker support, managed Postgres, simple pricing |

## Confidence Assessment

| Area | Confidence | Reason |
|------|------------|--------|
| Overall architecture | HIGH | Standard producer-consumer pattern for scanning SaaS |
| Rust backend + Axum | HIGH | Well-established ecosystem, good async support |
| PostgreSQL schema | HIGH | Relational model fits scan/findings hierarchy well |
| Job queue pattern | MEDIUM | Database-as-queue works for MVP, may need Redis later |
| Container execution | MEDIUM | Docker on Render works, resource limits need tuning |
| Render deployment | LOW | Unable to verify current Render Docker support (tool restrictions) |
| PDF generation | MEDIUM | typst is newer, WeasyPrint more proven but heavier |
| Email delivery | HIGH | Resend is standard choice for transactional emails |

## Sources

Unable to verify with external sources due to tool restrictions. This document is based on:
- Training knowledge of security scanning system architectures (as of January 2025)
- Established patterns for SaaS job processing systems
- Rust ecosystem best practices (Axum, SQLx, tokio)
- PRD context provided

**Recommended validation steps:**
1. Verify Render Docker support for containerized scanners
2. Check latest Axum version and async patterns
3. Verify typst PDF generation capabilities
4. Confirm Resend API limits and pricing

## Open Questions for Implementation

1. **Container image caching:** Does Render cache Docker images between runs, or does each scanner invocation pull fresh?
2. **PostgreSQL connection pooling:** What's the optimal pool size for Render managed PostgreSQL?
3. **PDF size limits:** What's the realistic upper bound for PDF report size (affects database BYTEA storage)?
4. **Scanner timeout tuning:** What's the 95th percentile runtime for each scanner type?
5. **Concurrency sweet spot:** At what point does in-process worker pool become bottleneck vs. cost of separate worker service?
