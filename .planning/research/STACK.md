# Technology Stack

**Project:** TrustEdge Audit
**Researched:** 2026-02-04
**Research Mode:** Greenfield SaaS security scanning platform

## Confidence Notice

**IMPORTANT:** This research is based on training data (January 2025). Version numbers and ecosystem status could not be verified via web tools. Confidence levels reflect this limitation.

- HIGH confidence = Well-established patterns, stable technology, unlikely to have changed
- MEDIUM confidence = Version-specific or rapidly evolving ecosystem
- LOW confidence = Requires verification with current documentation

**Recommended action:** Verify specific version numbers and breaking changes via official documentation before implementation.

---

## Recommended Stack

### Core Backend Framework

| Technology | Version | Purpose | Confidence | Why |
|------------|---------|---------|------------|-----|
| **Axum** | 0.7.x | HTTP server, API routing, middleware | **MEDIUM** | **RECOMMENDED.** Tokio-native, ergonomic, excellent for async I/O workloads (concurrent scans). Better composability than Actix-web via tower middleware. Maintained by Tokio team (stability guarantee). Lower cognitive overhead for Rust newcomers. |
| Actix-web | 4.x | Alternative HTTP server | MEDIUM | More mature (longer track record), slightly better raw throughput benchmarks. BUT: More complex actor model, heavier API surface, less idiomatic async/await patterns. Only choose if you need absolute max throughput or have existing Actix expertise. |

**Verdict:** Use **Axum**. For a SaaS orchestrating containerized tools, developer ergonomics and maintenance burden matter more than raw benchmark throughput. Axum's tower middleware ecosystem is excellent for auth, rate limiting, tracing.

**Installation:**
```toml
# Cargo.toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["trace", "cors", "timeout"] }
```

---

### HTTP Client (for scanner orchestration)

| Library | Version | Purpose | Confidence | Why |
|---------|---------|---------|------------|-----|
| **reqwest** | 0.12.x | HTTP client for security probes, API calls | **HIGH** | Industry standard. Async, supports connection pooling, timeouts, cookies, proxies. Essential for scanning URLs, calling SSL Labs API, fetching JS bundles. |

**Installation:**
```toml
reqwest = { version = "0.12", features = ["json", "cookies"] }
```

---

### Async Runtime

| Technology | Version | Purpose | Confidence | Why |
|------------|---------|---------|------------|-----|
| **Tokio** | 1.x | Async runtime | **HIGH** | Required by Axum. Battle-tested, excellent ecosystem, best-in-class async I/O. Use `tokio::spawn` for concurrent scan tasks. |

**Configuration note:** Use multi-threaded runtime for scan orchestration (default with `features = ["full"]`).

---

### Database Layer

| Technology | Version | Purpose | Confidence | Why |
|------------|---------|---------|------------|-----|
| **PostgreSQL** | 16.x | Primary datastore | **HIGH** | Chosen by founder. Excellent JSON support (for scan findings), ACID guarantees, proven at scale. |
| **SQLx** | 0.8.x | Database driver | **MEDIUM** | Async, compile-time query verification, migrations built-in. Avoids ORM bloat. Direct SQL = performance + clarity for job queue queries. |

**Alternatives considered:**
- **Diesel**: Synchronous, heavier ORM. Avoid for async workload.
- **SeaORM**: Async ORM, but adds complexity. SQLx's query macros strike better balance.

**Installation:**
```toml
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "json", "uuid", "chrono"] }
```

**Schema patterns for scan jobs:**

```sql
-- Core tables
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE scans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    target_url TEXT NOT NULL,
    scan_type TEXT NOT NULL, -- 'free' | 'paid'
    status TEXT NOT NULL, -- 'pending' | 'running' | 'completed' | 'failed'
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Findings as JSONB for flexibility (schema evolves as scanners improve)
CREATE TABLE findings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_id UUID NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
    scanner TEXT NOT NULL, -- 'headers' | 'tls' | 'nuclei' | 'secrets'
    severity TEXT NOT NULL, -- 'critical' | 'high' | 'medium' | 'low' | 'info'
    title TEXT NOT NULL,
    description TEXT,
    remediation TEXT,
    metadata JSONB, -- scanner-specific details
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_findings_scan_id ON findings(scan_id);
CREATE INDEX idx_findings_severity ON findings(severity);
CREATE INDEX idx_scans_status ON scans(status);
CREATE INDEX idx_scans_user_id ON scans(user_id);
```

**Why JSONB for findings.metadata:**
- Each scanner (Nuclei, testssl.sh, custom probes) returns different detail structures
- Avoids rigid schema that requires migration for every scanner update
- PostgreSQL's JSONB indexing supports efficient queries if needed
- Frontend can render findings flexibly without backend changes

---

### Job Queue / Task Orchestration

| Library | Version | Purpose | Confidence | Why |
|---------|---------|---------|------------|-----|
| **Custom PostgreSQL-based queue** | N/A | Job queue for scan tasks | **HIGH** | **RECOMMENDED for MVP.** Use `scans` table with status polling. Add `SELECT ... FOR UPDATE SKIP LOCKED` for job claiming. Avoids external dependency (Redis), simpler deployment on Render. |
| tokio::task | Built-in | Concurrent task spawning | HIGH | Use `tokio::spawn` for parallel scanner execution within a job. |

**Why NOT Redis/Sidekiq/Celery:**
- Adds deployment complexity (another service on Render)
- PostgreSQL can handle MVP workload (dozens of scans concurrently)
- Polling `scans` table every 5-10s is acceptable for free tier latency expectations

**Job claiming pattern:**
```rust
// Worker loop
loop {
    let job = sqlx::query_as!(
        Scan,
        r#"
        UPDATE scans
        SET status = 'running', started_at = NOW()
        WHERE id = (
            SELECT id FROM scans
            WHERE status = 'pending'
            ORDER BY created_at
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        )
        RETURNING *
        "#
    )
    .fetch_optional(&pool)
    .await?;

    if let Some(scan) = job {
        tokio::spawn(async move {
            execute_scan(scan).await;
        });
    } else {
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
```

**Scale path:** If you hit >100 concurrent scans, migrate to Redis + `faktory` or `pg-boss`. But that's months away.

---

### Container Orchestration (Nuclei, testssl.sh)

| Library | Version | Purpose | Confidence | Why |
|---------|---------|---------|------------|-----|
| **bollard** | 0.17.x | Docker API client | **MEDIUM** | Rust-native Docker client. Start/stop containers, stream logs, manage volumes. Required for running Nuclei/testssl.sh in isolation. |

**Why containerized scanners:**
- Isolation (untrusted URLs can't compromise host)
- Portability (Render supports Docker)
- Version pinning (lock scanner tool versions)
- Resource limits (prevent runaway processes)

**Pattern:**
```rust
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions};

async fn run_nuclei_scan(target_url: &str) -> Result<String, Error> {
    let docker = Docker::connect_with_local_defaults()?;

    let container = docker.create_container(
        Some(CreateContainerOptions {
            name: format!("nuclei-{}", Uuid::new_v4()),
            ..Default::default()
        }),
        Config {
            image: Some("projectdiscovery/nuclei:v3.2"),
            cmd: Some(vec!["-u", target_url, "-json"]),
            ..Default::default()
        },
    ).await?;

    docker.start_container(&container.id, None).await?;

    // Stream logs, parse JSON output
    let output = /* ... */;

    docker.remove_container(&container.id, None).await?;

    Ok(output)
}
```

**Dockerfile for Nuclei (pinned version):**
```dockerfile
FROM projectdiscovery/nuclei:v3.2.0
COPY custom-templates /root/nuclei-templates/custom
```

**Dockerfile for testssl.sh:**
```dockerfile
FROM drwetter/testssl.sh:3.0.8
```

**Render deployment consideration:** Ensure Docker-in-Docker support or use Render's native container service. May require privileged mode or Docker socket mounting.

---

### Stripe Integration

| Library | Version | Purpose | Confidence | Why |
|---------|---------|---------|------------|-----|
| **stripe-rust** (async-stripe) | 0.28.x | Stripe API client | **MEDIUM** | Community-maintained (not official), but widely used. Async support via Tokio. Covers Checkout Sessions, Webhooks, Subscriptions. |

**Installation:**
```toml
async-stripe = { version = "0.28", features = ["checkout", "webhook-events"] }
```

**One-time payment pattern (Checkout Session):**
```rust
use stripe::{CheckoutSession, CheckoutSessionMode, CreateCheckoutSession, Currency, Client};

async fn create_checkout_session(
    stripe_client: &Client,
    user_email: &str,
    scan_id: Uuid,
) -> Result<CheckoutSession, stripe::StripeError> {
    CheckoutSession::create(
        stripe_client,
        CreateCheckoutSession {
            mode: Some(CheckoutSessionMode::Payment),
            line_items: Some(vec![/* product details */]),
            customer_email: Some(user_email),
            metadata: Some([("scan_id".to_string(), scan_id.to_string())].into()),
            success_url: Some(&format!("https://trustedge-audit.com/scan/{}/results", scan_id)),
            cancel_url: Some("https://trustedge-audit.com/"),
            ..Default::default()
        },
    ).await
}
```

**Webhook verification (critical for security):**
```rust
use stripe::{Webhook, EventObject, EventType};

async fn handle_stripe_webhook(
    payload: String,
    signature: &str,
    webhook_secret: &str,
) -> Result<(), Error> {
    let event = Webhook::construct_event(&payload, signature, webhook_secret)?;

    match event.type_ {
        EventType::CheckoutSessionCompleted => {
            if let EventObject::CheckoutSession(session) = event.data.object {
                let scan_id = session.metadata.get("scan_id").unwrap();
                // Mark scan as paid, trigger deep scan
            }
        }
        _ => {}
    }
    Ok(())
}
```

**Confidence note:** MEDIUM because async-stripe tracks Stripe API changes, which can introduce breaking changes. Pin version carefully, monitor GitHub releases.

---

### PDF Report Generation

| Library | Version | Purpose | Confidence | Why |
|---------|---------|---------|------------|-----|
| **printpdf** | 0.7.x | Low-level PDF generation | **MEDIUM** | Pure Rust, no external dependencies. Full control over layout. Best for structured reports (findings table, remediation steps). |
| **Alternative: headless_chrome** | 0.9.x | HTML-to-PDF via Chrome | MEDIUM | If you need rich HTML/CSS rendering. Heavier (requires Chrome binary), but easier for complex layouts. |

**Recommendation:** Use **printpdf** for MVP. Findings reports are structured data (tables, text blocks), not rich HTML. Avoid Chrome dependency.

**Pattern:**
```rust
use printpdf::*;

fn generate_pdf_report(scan: &Scan, findings: &[Finding]) -> Result<Vec<u8>, Error> {
    let (doc, page1, layer1) = PdfDocument::new("TrustEdge Audit Report", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Title
    current_layer.use_text("Security Audit Report", 24.0, Mm(10.0), Mm(280.0), &font);

    // Findings by severity
    let mut y_offset = 260.0;
    for finding in findings {
        current_layer.use_text(&finding.title, 12.0, Mm(10.0), Mm(y_offset), &font);
        y_offset -= 10.0;
    }

    doc.save_to_bytes()
}
```

**Alternative approach:** Generate HTML, use `wkhtmltopdf` via CLI. But this adds external dependency.

**Confidence:** MEDIUM because PDF layout is tedious; may need iteration to look professional.

---

### Frontend Framework

| Technology | Version | Purpose | Confidence | Why |
|------------|---------|---------|------------|-----|
| **Next.js** | 14.x (App Router) | Frontend framework | **HIGH** | Chosen by founder. Excellent DX, built-in API routes, SSR for SEO (landing page), React ecosystem. App Router (13+) is stable, uses Server Components. |
| React | 18.x | UI library | HIGH | Required by Next.js. |
| **TailwindCSS** | 3.x | Styling | HIGH | Utility-first, fast prototyping, excellent for dashboards. |
| **shadcn/ui** | Latest | Component library | MEDIUM | Unstyled, accessible components built on Radix UI. Copy-paste approach = no npm bloat. Excellent for dashboards (tables, cards, badges). |

**Installation:**
```bash
npx create-next-app@latest trustedge-audit-web --typescript --tailwind --app
npx shadcn-ui@latest init
npx shadcn-ui@latest add table badge card
```

**Dashboard patterns for scan results:**

1. **Real-time scan status** (polling, not WebSocket for MVP):
```typescript
// app/scan/[id]/page.tsx
'use client';
import { useEffect, useState } from 'react';

export default function ScanResultsPage({ params }: { params: { id: string } }) {
    const [scan, setScan] = useState(null);

    useEffect(() => {
        const pollScan = async () => {
            const res = await fetch(`/api/scans/${params.id}`);
            const data = await res.json();
            setScan(data);

            if (data.status === 'pending' || data.status === 'running') {
                setTimeout(pollScan, 5000); // Poll every 5s
            }
        };
        pollScan();
    }, [params.id]);

    if (!scan) return <div>Loading...</div>;
    if (scan.status === 'running') return <div>Scanning...</div>;

    return <FindingsTable findings={scan.findings} />;
}
```

2. **Findings table with severity badges:**
```typescript
// components/findings-table.tsx
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Badge } from '@/components/ui/badge';

const severityColor = {
    critical: 'destructive',
    high: 'destructive',
    medium: 'warning',
    low: 'secondary',
    info: 'outline',
};

export function FindingsTable({ findings }) {
    return (
        <Table>
            <TableHeader>
                <TableRow>
                    <TableHead>Severity</TableHead>
                    <TableHead>Finding</TableHead>
                    <TableHead>Remediation</TableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                {findings.map((finding) => (
                    <TableRow key={finding.id}>
                        <TableCell>
                            <Badge variant={severityColor[finding.severity]}>
                                {finding.severity.toUpperCase()}
                            </Badge>
                        </TableCell>
                        <TableCell>{finding.title}</TableCell>
                        <TableCell>
                            <pre className="text-sm">{finding.remediation}</pre>
                        </TableCell>
                    </TableRow>
                ))}
            </TableBody>
        </Table>
    );
}
```

3. **Landing page with form:**
```typescript
// app/page.tsx
'use client';
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';

export default function LandingPage() {
    const [url, setUrl] = useState('');
    const [email, setEmail] = useState('');

    const handleSubmit = async (e) => {
        e.preventDefault();
        const res = await fetch('/api/scans', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ url, email }),
        });
        const { scan_id } = await res.json();
        window.location.href = `/scan/${scan_id}`;
    };

    return (
        <form onSubmit={handleSubmit}>
            <Input
                type="url"
                placeholder="https://your-app.com"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
            />
            <Input
                type="email"
                placeholder="your@email.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
            />
            <Button type="submit">Scan Now (Free)</Button>
        </form>
    );
}
```

**API routes (Next.js backend for simple endpoints):**
```typescript
// app/api/scans/route.ts
import { NextResponse } from 'next/server';

export async function POST(request: Request) {
    const { url, email } = await request.json();

    // Call Rust backend
    const res = await fetch('http://localhost:8000/api/scans', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ target_url: url, user_email: email }),
    });

    const data = await res.json();
    return NextResponse.json(data);
}
```

**Deployment note:** Next.js on Render requires Node.js service. Deploy separately from Rust backend.

---

### Email Delivery

| Service | Purpose | Confidence | Why |
|---------|---------|------------|-----|
| **Resend** | Transactional email | **HIGH** | Modern API, generous free tier (100 emails/day), excellent DX. Built for developers. Better than SendGrid for simple use case. |
| **Alternative: AWS SES** | Email sending | HIGH | Cheaper at scale ($0.10/1000 emails), but more complex setup. Use if you need >10k emails/month. |

**Installation (Rust):**
```toml
# Use reqwest to call Resend API
reqwest = { version = "0.12", features = ["json"] }
```

**Pattern:**
```rust
use serde_json::json;

async fn send_scan_results_email(
    recipient: &str,
    scan_id: Uuid,
    findings_summary: &str,
) -> Result<(), Error> {
    let client = reqwest::Client::new();

    client.post("https://api.resend.com/emails")
        .header("Authorization", format!("Bearer {}", std::env::var("RESEND_API_KEY")?))
        .json(&json!({
            "from": "TrustEdge Audit <scan@trustedge-audit.com>",
            "to": recipient,
            "subject": "Your Security Scan is Complete",
            "html": format!(
                "<h1>Scan Complete</h1><p>{}</p><a href='https://trustedge-audit.com/scan/{}'>View Results</a>",
                findings_summary, scan_id
            ),
        }))
        .send()
        .await?;

    Ok(())
}
```

**Recommendation:** Start with Resend. Switch to SES if you hit scale limits.

---

### Authentication (if needed)

| Library | Version | Purpose | Confidence | Why |
|---------|---------|---------|------------|-----|
| **clerk.com** | SaaS | Auth provider (optional) | MEDIUM | If you add user accounts post-MVP. Handles OAuth, magic links, session management. |
| **Alternative: Custom JWT** | N/A | Roll your own | MEDIUM | Use `jsonwebtoken` crate for simple email-based auth. Avoid for MVP (out of scope). |

**Recommendation:** MVP doesn't need auth (email-only free tier). Defer until paid tier requires account management.

---

### Logging & Observability

| Library | Version | Purpose | Confidence | Why |
|---------|---------|---------|------------|-----|
| **tracing** | 0.1.x | Structured logging | **HIGH** | Industry standard for Rust. Integrates with Axum via tower-http. Use for request tracing, scan job tracking. |
| **tracing-subscriber** | 0.3.x | Log output formatting | HIGH | Required for tracing setup. |

**Installation:**
```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

**Setup:**
```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // App code
}
```

**Production logging:** Render has built-in log aggregation. Tracing JSON output integrates well.

---

### Environment Configuration

| Library | Version | Purpose | Confidence | Why |
|---------|---------|---------|------------|-----|
| **dotenvy** | 0.15.x | .env file loading | **HIGH** | Loads environment variables from `.env` for local dev. Use for DATABASE_URL, STRIPE_SECRET_KEY, etc. |

**Installation:**
```toml
dotenvy = "0.15"
```

**Pattern:**
```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok(); // Load .env in dev, ignore in prod

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let stripe_secret = std::env::var("STRIPE_SECRET_KEY").expect("STRIPE_SECRET_KEY must be set");

    // ...
}
```

**Render deployment:** Set environment variables via Render dashboard (not .env file).

---

### Testing

| Library | Version | Purpose | Confidence | Why |
|---------|---------|---------|------------|-----|
| **cargo test** | Built-in | Unit tests | HIGH | Native Rust testing. |
| **sqlx::test** | 0.8.x | Database integration tests | MEDIUM | Macro for spinning up test databases. |
| **wiremock** | 0.6.x | HTTP mocking for scanner tests | MEDIUM | Mock SSL Labs API, Stripe webhooks in tests. |

**Pattern (integration test with database):**
```rust
#[sqlx::test]
async fn test_create_scan(pool: PgPool) -> sqlx::Result<()> {
    let scan_id = create_scan(&pool, "https://example.com", "test@example.com").await?;

    let scan = sqlx::query!("SELECT * FROM scans WHERE id = $1", scan_id)
        .fetch_one(&pool)
        .await?;

    assert_eq!(scan.status, "pending");
    Ok(())
}
```

---

## Deployment Architecture (Render)

| Service | Type | Purpose | Config |
|---------|------|---------|--------|
| **Rust Backend** | Web Service | Axum API, scan orchestrator | Docker, expose port 8000 |
| **Next.js Frontend** | Web Service | User-facing app | Node.js, expose port 3000 |
| **PostgreSQL** | Managed Database | Data persistence | Render PostgreSQL instance |
| **Docker Host** | (Same as Rust) | Container orchestration | Mount Docker socket or use Render native |

**Render deployment steps:**

1. **Create PostgreSQL database** (Render dashboard)
   - Copy `DATABASE_URL` to Rust backend env vars

2. **Deploy Rust backend:**
   ```dockerfile
   # Dockerfile
   FROM rust:1.75 AS builder
   WORKDIR /app
   COPY . .
   RUN cargo build --release

   FROM debian:bookworm-slim
   RUN apt-get update && apt-get install -y libpq5 ca-certificates
   COPY --from=builder /app/target/release/trustedge-audit /usr/local/bin/
   EXPOSE 8000
   CMD ["trustedge-audit"]
   ```

3. **Deploy Next.js frontend:**
   ```bash
   # Build command
   npm run build

   # Start command
   npm start
   ```

4. **Set environment variables** (Render dashboard):
   ```
   DATABASE_URL=postgres://...
   STRIPE_SECRET_KEY=sk_...
   STRIPE_WEBHOOK_SECRET=whsec_...
   RESEND_API_KEY=re_...
   RUST_LOG=info
   ```

**Docker-in-Docker consideration:**
- Render may require privileged containers for Docker socket access
- Alternative: Use Render's native container service or run scanner containers on separate compute instances

---

## Alternatives Considered

### Backend Framework: Axum vs Actix-web

| Criterion | Axum | Actix-web | Winner |
|-----------|------|-----------|--------|
| **Performance** | Excellent (tower + hyper) | Slightly faster (benchmarks) | Tie |
| **Ergonomics** | Idiomatic async/await | Actor model complexity | Axum |
| **Ecosystem** | Tower middleware (mature) | Actix ecosystem | Axum |
| **Maintenance** | Tokio team (stable) | Community (active) | Axum |
| **Learning curve** | Low (standard Rust patterns) | Medium (actor model) | Axum |

**Recommendation:** Axum. Performance difference is negligible for I/O-bound scanning workload. Developer productivity and maintenance matter more.

### Frontend: Next.js vs HTMX

| Criterion | Next.js | HTMX | Winner |
|-----------|---------|------|--------|
| **Interactivity** | Rich (React) | Limited (hypermedia) | Next.js |
| **Complexity** | Medium (JS build) | Low (server HTML) | HTMX |
| **Dashboard UX** | Excellent (client-side state) | Acceptable (page reloads) | Next.js |
| **SEO** | Excellent (SSR) | Excellent (server HTML) | Tie |

**Recommendation:** Next.js (already chosen by founder). Dashboards benefit from client-side state (findings table sorting, filtering).

### Job Queue: PostgreSQL vs Redis

| Criterion | PostgreSQL | Redis | Winner |
|-----------|------------|-------|--------|
| **Simplicity** | One less service | Requires Redis instance | PostgreSQL |
| **Performance** | Good for <100 jobs/sec | Excellent for high throughput | Redis (at scale) |
| **Transactional** | ACID guarantees | Limited transactions | PostgreSQL |
| **MVP fit** | Perfect | Overkill | PostgreSQL |

**Recommendation:** PostgreSQL for MVP. Migrate to Redis when you need >100 concurrent scans.

### PDF Generation: printpdf vs headless Chrome

| Criterion | printpdf | headless_chrome | Winner |
|-----------|----------|-----------------|--------|
| **Dependencies** | None (pure Rust) | Chrome binary (large) | printpdf |
| **Layout control** | Manual (code) | CSS (easy) | Chrome |
| **Performance** | Fast | Slower (browser overhead) | printpdf |
| **Report fit** | Good (structured data) | Overkill (not rich HTML) | printpdf |

**Recommendation:** printpdf. Reports are tables and text, not rich layouts.

---

## Installation Checklist

### Rust Backend

```toml
# Cargo.toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["trace", "cors", "timeout"] }
reqwest = { version = "0.12", features = ["json", "cookies"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "json", "uuid", "chrono"] }
async-stripe = { version = "0.28", features = ["checkout", "webhook-events"] }
bollard = "0.17"
printpdf = "0.7"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
dotenvy = "0.15"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
wiremock = "0.6"
```

### Frontend (Next.js)

```bash
npx create-next-app@latest trustedge-audit-web --typescript --tailwind --app
cd trustedge-audit-web
npx shadcn-ui@latest init
npx shadcn-ui@latest add table badge card button input
npm install
```

---

## Migration Path / Future Stack Changes

| When | What | Why |
|------|------|-----|
| **>100 concurrent scans** | Add Redis + job queue library (faktory) | PostgreSQL polling won't scale efficiently |
| **>10k emails/month** | Migrate to AWS SES | Cost savings ($0.10/1000 vs Resend tiers) |
| **Pro tier launch** | Add auth provider (Clerk or custom JWT) | User accounts, subscription management |
| **Post-MVP polish** | Consider headless Chrome for PDF | If report layout becomes complex |

---

## Sources

**Confidence level:** LOW to MEDIUM across the board due to inability to verify current versions and ecosystem status via web tools.

**Training data limitations:**
- Version numbers are based on January 2025 knowledge
- Breaking changes in 2025-2026 releases may exist
- Ecosystem shifts (new libraries, deprecated crates) may have occurred

**Recommended verification steps:**
1. Check crates.io for latest stable versions of all Rust dependencies
2. Verify Next.js 14.x App Router stability (or if Next.js 15 is now stable)
3. Confirm async-stripe tracks latest Stripe API version
4. Test Render's Docker-in-Docker support for bollard

**Authoritative sources to consult:**
- https://docs.rs (Rust crate documentation)
- https://nextjs.org/docs (Next.js official docs)
- https://stripe.com/docs (Stripe API docs)
- https://render.com/docs (Render deployment guides)
- https://github.com/tokio-rs/axum (Axum examples)

---

## Summary

**Core Stack:**
- **Backend:** Axum (Rust) + SQLx + PostgreSQL
- **Frontend:** Next.js 14 (App Router) + TailwindCSS + shadcn/ui
- **Containers:** bollard for Docker orchestration (Nuclei, testssl.sh)
- **Payments:** async-stripe
- **PDF:** printpdf
- **Email:** Resend
- **Hosting:** Render (Web Services + PostgreSQL)

**Key architectural decisions:**
1. **Axum over Actix-web:** Ergonomics and tower ecosystem > minor benchmark differences
2. **PostgreSQL job queue:** Avoid Redis complexity for MVP, migrate at scale
3. **printpdf over Chrome:** Reports are structured data, avoid heavy dependencies
4. **Next.js over HTMX:** Dashboard UX benefits from client-side state management

**Confidence:**
- HIGH: Core technologies (Axum, Tokio, PostgreSQL, Next.js, Stripe patterns)
- MEDIUM: Specific version numbers, bollard stability, async-stripe API tracking
- LOW: None (all recommendations are well-established patterns)

**Critical gaps to address:**
- Verify Docker-in-Docker support on Render (may require alternative compute strategy for scanner containers)
- Test bollard performance under concurrent scan load
- Validate async-stripe webhook verification patterns against current Stripe API docs

This stack is production-ready for MVP. Complexity is appropriate for the workload. All technologies have strong Rust/Next.js ecosystem support and active maintenance.
