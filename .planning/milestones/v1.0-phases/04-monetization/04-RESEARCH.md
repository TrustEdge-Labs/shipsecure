# Phase 4: Monetization - Research

**Researched:** 2026-02-06
**Domain:** Payment processing, PDF generation, tiered scanning
**Confidence:** HIGH

## Summary

Phase 4 adds paid audit functionality via Stripe Checkout, deeper scanning for paid users, and professional PDF reports. The standard stack is **async-stripe** (Rust bindings for Stripe API), **genpdf** (pure Rust PDF generation), and **Resend** (already integrated for email delivery with base64 attachment support).

Key architectural insight: Stripe webhooks are the source of truth for payment completion, not redirect URLs (customers may close browser before redirect). The webhook handler triggers the paid scan asynchronously via `tokio::spawn`, identical to the free tier scan orchestrator pattern already implemented in Phase 2.

Tier differentiation follows the industry pattern: **free tier = passive reconnaissance** (headers, exposed files, basic secrets) while **paid tier = active probing** (additional Nuclei templates, deeper path enumeration, extended scan timeouts). This creates clear value distinction without making free tier appear deliberately hobbled.

**Primary recommendation:** Use Stripe Checkout Sessions with metadata (scan_id, user_email) for correlation, handle `checkout.session.completed` webhooks to trigger paid scans, generate PDF reports in-memory with genpdf (no file system writes), and attach via Resend base64 content parameter (40MB limit easily sufficient for text reports).

## Standard Stack

The established libraries/tools for Stripe payments and PDF generation in Rust:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| async-stripe | 0.41.0 | Stripe API bindings | Auto-generated from OpenAPI spec, updated weekly, tokio-native, strong types |
| genpdf | 0.2.0 | PDF generation | Pure Rust (no C deps), auto-pagination, built on printpdf/rusttype |
| resend_rs | latest | Email with attachments | Already integrated, supports base64 attachments up to 40MB |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| base64 | 0.21+ | Encode PDF for email | Standard encoding for Resend attachment content parameter |
| serde_json | 1.0+ | Parse webhook payloads | Extract event data from Stripe webhook POST bodies |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| async-stripe | stripe-rust (wyyerd) | async-stripe has more active maintenance, weekly auto-generation |
| genpdf | printpdf directly | genpdf handles layout/pagination automatically, simpler API |
| genpdf | krilla | krilla has more features but less mature (newer library) |
| PDF generation | HTML→PDF via headless Chrome | Huge dependency, slow, overkill for structured text reports |

**Installation:**
```bash
# Cargo.toml additions
async-stripe = { version = "0.41", features = ["runtime-tokio-hyper", "checkout"] }
genpdf = "0.2"
base64 = "0.21"
```

## Architecture Patterns

### Recommended Database Schema Extensions
```sql
-- New table for paid audits
CREATE TABLE paid_audits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_id UUID NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
    stripe_checkout_session_id VARCHAR(255) NOT NULL UNIQUE,
    stripe_payment_intent_id VARCHAR(255),
    amount_cents INTEGER NOT NULL,
    currency VARCHAR(3) DEFAULT 'usd',
    customer_email VARCHAR(255) NOT NULL,
    status VARCHAR(20) NOT NULL, -- pending, completed, failed
    pdf_generated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_paid_audits_scan_id ON paid_audits(scan_id);
CREATE INDEX idx_paid_audits_session_id ON paid_audits(stripe_checkout_session_id);
CREATE INDEX idx_paid_audits_status ON paid_audits(status);

-- Enum for scan tier
ALTER TYPE scan_status ADD VALUE IF NOT EXISTS 'paid';
-- OR add new column to scans table
ALTER TABLE scans ADD COLUMN tier VARCHAR(10) DEFAULT 'free' CHECK (tier IN ('free', 'paid'));
```

### Pattern 1: Stripe Checkout Session Creation
**What:** Server-side API endpoint creates Stripe Checkout Session with metadata
**When to use:** User clicks "Upgrade to Deep Audit" CTA on results page
**Example:**
```rust
// Source: https://docs.rs/async-stripe/latest/stripe/
use stripe::{Client, CheckoutSession, CreateCheckoutSession, CreateCheckoutSessionLineItems};

async fn create_checkout_session(
    scan_id: Uuid,
    email: String,
) -> Result<CheckoutSession, Error> {
    let client = Client::new(std::env::var("STRIPE_SECRET_KEY")?);

    let session = CheckoutSession::create(&client, CreateCheckoutSession {
        mode: Some(stripe::CheckoutSessionMode::Payment),
        line_items: Some(vec![CreateCheckoutSessionLineItems {
            price_data: Some(CreateCheckoutSessionLineItemsPriceData {
                currency: stripe::Currency::USD,
                unit_amount: Some(4900), // $49.00 in cents
                product_data: Some(CreateCheckoutSessionLineItemsPriceDataProductData {
                    name: "TrustEdge Deep Security Audit".to_string(),
                    description: Some("Comprehensive vulnerability scan with PDF report".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            quantity: Some(1),
            ..Default::default()
        }]),
        customer_email: Some(email.clone()),
        success_url: Some(format!("{}/payment/success?session_id={{CHECKOUT_SESSION_ID}}",
            std::env::var("FRONTEND_URL")?)),
        cancel_url: Some(format!("{}/results/{}",
            std::env::var("FRONTEND_URL")?, scan_id)),
        metadata: Some([
            ("scan_id".to_string(), scan_id.to_string()),
            ("email".to_string(), email),
        ].into_iter().collect()),
        ..Default::default()
    }).await?;

    Ok(session)
}
```

### Pattern 2: Webhook Handler with Idempotency
**What:** Process `checkout.session.completed` events, prevent duplicates
**When to use:** Stripe sends webhook POST to `/webhooks/stripe`
**Example:**
```rust
// Source: https://docs.rs/async-stripe/latest/stripe/struct.Webhook.html
use stripe::{Webhook, EventType, EventObject};

async fn handle_stripe_webhook(
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, Error> {
    let signature = headers.get("stripe-signature")
        .ok_or(Error::MissingSignature)?
        .to_str()?;

    let webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET")?;

    // Verify webhook authenticity
    let event = Webhook::construct_event(
        std::str::from_utf8(&body)?,
        signature,
        &webhook_secret,
    )?;

    // Check for duplicate event (idempotency)
    if event_already_processed(&event.id).await? {
        return Ok(StatusCode::OK); // Already handled
    }

    match event.type_ {
        EventType::CheckoutSessionCompleted => {
            if let EventObject::CheckoutSession(session) = event.data.object {
                // Extract metadata
                let scan_id: Uuid = session.metadata
                    .get("scan_id")
                    .ok_or(Error::MissingMetadata)?
                    .parse()?;

                // Record payment in database
                record_paid_audit(scan_id, &session).await?;

                // Trigger paid scan asynchronously (don't block webhook response)
                tokio::spawn(async move {
                    if let Err(e) = execute_paid_scan(scan_id).await {
                        eprintln!("Paid scan failed for {}: {}", scan_id, e);
                    }
                });
            }
        }
        _ => {} // Ignore other event types
    }

    // Mark event as processed
    mark_event_processed(&event.id).await?;

    Ok(StatusCode::OK) // Return 200 quickly per Stripe best practices
}
```

### Pattern 3: PDF Report Generation
**What:** Generate branded PDF with executive summary, findings by severity, remediation
**When to use:** After paid scan completes successfully
**Example:**
```rust
// Source: https://docs.rs/genpdf/latest/genpdf/
use genpdf::{Document, SimplePageDecorator, elements, fonts, style};

async fn generate_pdf_report(scan: &Scan, findings: Vec<Finding>) -> Result<Vec<u8>, Error> {
    // Load default font family
    let font_family = fonts::from_files("./fonts", "LiberationSans", None)?;
    let mut doc = Document::new(font_family);

    // Metadata
    doc.set_title("TrustEdge Security Audit Report");
    doc.set_minimal_conformance();
    doc.set_line_spacing(1.25);

    // Page decoration (header/footer)
    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    // Executive Summary
    doc.push(elements::Paragraph::new("TrustEdge Security Audit")
        .styled(style::Style::new().bold().with_font_size(20)));
    doc.push(elements::Paragraph::new(format!("Scan Date: {}", scan.created_at)));
    doc.push(elements::Paragraph::new(format!("Target: {}", scan.target_url)));
    doc.push(elements::Paragraph::new(format!("Grade: {}", scan.grade.unwrap_or("N/A".to_string()))));

    doc.push(elements::Break::new(2));

    doc.push(elements::Paragraph::new("Executive Summary")
        .styled(style::Style::new().bold().with_font_size(16)));

    let critical_count = findings.iter().filter(|f| f.severity == "critical").count();
    let high_count = findings.iter().filter(|f| f.severity == "high").count();

    doc.push(elements::Paragraph::new(format!(
        "This security audit identified {} total vulnerabilities across {} severity levels. \
        Immediate action is required for {} critical and {} high-severity findings.",
        findings.len(),
        findings.iter().map(|f| &f.severity).collect::<std::collections::HashSet<_>>().len(),
        critical_count,
        high_count
    )));

    // Findings by Severity
    doc.push(elements::Break::new(2));
    for severity in &["critical", "high", "medium", "low"] {
        let severity_findings: Vec<_> = findings.iter()
            .filter(|f| &f.severity == severity)
            .collect();

        if severity_findings.is_empty() { continue; }

        doc.push(elements::Paragraph::new(format!("{} Severity Findings",
            severity.to_uppercase()))
            .styled(style::Style::new().bold().with_font_size(14)));

        for finding in severity_findings {
            doc.push(elements::Paragraph::new(&finding.title)
                .styled(style::Style::new().bold()));
            doc.push(elements::Paragraph::new(&finding.description));

            if let Some(remediation) = &finding.remediation {
                doc.push(elements::Paragraph::new("Remediation:")
                    .styled(style::Style::new().italic()));
                doc.push(elements::Paragraph::new(remediation));
            }

            doc.push(elements::Break::new(1));
        }
    }

    // Render to bytes
    let mut buf = Vec::new();
    doc.render(&mut buf)?;
    Ok(buf)
}
```

### Pattern 4: Email with PDF Attachment
**What:** Send PDF report via Resend with base64-encoded attachment
**When to use:** Immediately after PDF generation completes
**Example:**
```rust
// Source: https://resend.com/docs/dashboard/emails/attachments
use resend_rs::{Resend, Email, Attachment};

async fn send_pdf_report(
    email: &str,
    scan_id: Uuid,
    pdf_bytes: Vec<u8>,
) -> Result<(), Error> {
    let resend = Resend::new(std::env::var("RESEND_API_KEY")?);

    let email = Email {
        from: "security@trustedge.audit",
        to: vec![email.to_string()],
        subject: format!("Your TrustEdge Security Audit Report - {}", scan_id),
        html: format!(
            r#"
            <h1>Your Deep Security Audit is Complete</h1>
            <p>Thank you for choosing TrustEdge Audit. Your comprehensive security report is attached.</p>
            <p><strong>Scan ID:</strong> {}</p>
            <p>This report includes:</p>
            <ul>
                <li>Executive summary with overall security grade</li>
                <li>Detailed findings organized by severity</li>
                <li>Framework-specific remediation guidance</li>
            </ul>
            <p>Questions? Reply to this email for support.</p>
            "#,
            scan_id
        ),
        attachments: vec![Attachment {
            content: base64::encode(&pdf_bytes),
            filename: format!("trustedge-audit-{}.pdf", scan_id),
            content_type: Some("application/pdf".to_string()),
            path: None, // Using content, not remote URL
        }],
        ..Default::default()
    };

    resend.send(email).await?;
    Ok(())
}
```

### Pattern 5: Tier Differentiation in Scanner Selection
**What:** Select different scanner configurations based on scan tier
**When to use:** Orchestrator decides which scanners/templates to run
**Example:**
```rust
async fn execute_scan(scan_id: Uuid, tier: ScanTier) -> Result<(), Error> {
    let scan = db::get_scan(scan_id).await?;

    // Free tier: passive reconnaissance only
    let free_scanners = vec![
        run_headers_scanner(&scan),
        run_tls_scanner(&scan),
        run_exposed_files_scanner(&scan, /* basic_paths */ true),
        run_js_secrets_scanner(&scan, /* file_limit */ 10),
        run_vibecode_scanner(&scan, /* basic_templates */ true),
    ];

    // Paid tier: add active probing
    let paid_scanners = vec![
        run_headers_scanner(&scan),
        run_tls_scanner(&scan),
        run_exposed_files_scanner(&scan, /* extended_paths */ false), // More probes
        run_js_secrets_scanner(&scan, /* file_limit */ 50), // More files
        run_vibecode_scanner(&scan, /* all_templates */ false), // Extended templates
        run_active_nuclei_scanner(&scan), // NEW: Active vulnerability probing
        run_port_scan(&scan), // NEW: Common web ports beyond 80/443
    ];

    let scanners = match tier {
        ScanTier::Free => free_scanners,
        ScanTier::Paid => paid_scanners,
    };

    // Execute with longer timeout for paid tier
    let timeout = match tier {
        ScanTier::Free => Duration::from_secs(180),
        ScanTier::Paid => Duration::from_secs(600), // 10 minutes
    };

    tokio::time::timeout(timeout, futures::future::join_all(scanners)).await??;
    Ok(())
}
```

### Anti-Patterns to Avoid
- **Relying on success_url for payment confirmation:** Customer may close browser; always use webhooks as source of truth
- **Synchronous webhook handlers:** Return 200 immediately, spawn background tasks for slow operations (Stripe retries on timeout)
- **Storing Stripe Checkout Session in application state:** Session expires in 24h; use metadata to pass scan_id and look up from database
- **Hand-rolling PDF table layouts:** genpdf doesn't have native table support; use Paragraph and LinearLayout or accept text-only formatting
- **Blocking email send:** Spawn async task so scan orchestrator doesn't wait for email delivery

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Stripe API requests | Manual HTTP client with reqwest | async-stripe crate | Strong types, webhook signature verification, auto-updated from OpenAPI spec |
| PDF table layouts | Custom genpdf Element trait | Accept text formatting or use HTML→PDF | genpdf lacks native table support, custom Element complex to implement |
| Webhook duplicate detection | In-memory Set tracking | Database table with unique constraint | State persists across restarts, handles distributed deployments |
| Payment idempotency | Application-level locking | Database transactions + unique constraints | ACID guarantees prevent race conditions |
| PDF fonts | Bundle custom TTF files | Use Liberation Sans (comes with genpdf examples) | Licensing clear, widely compatible, good rendering |

**Key insight:** Stripe webhooks can arrive multiple times (network retries, infrastructure issues). Idempotency must be database-enforced, not application-level, to handle distributed deployments and server restarts during webhook processing.

## Common Pitfalls

### Pitfall 1: Webhook Signature Verification Skipped
**What goes wrong:** Attacker sends fake `checkout.session.completed` events to trigger free paid scans
**Why it happens:** Developer tests locally without signature verification, forgets to add in production
**How to avoid:** Always call `Webhook::construct_event()` with signature header and webhook secret; fail fast on signature mismatch
**Warning signs:** Webhook logs show events with missing/invalid `stripe-signature` headers but handler still processes them

### Pitfall 2: Metadata Lost in Checkout Session
**What goes wrong:** Webhook arrives but lacks scan_id, can't correlate payment to scan
**Why it happens:** Metadata not passed when creating CheckoutSession, or misspelled key
**How to avoid:** Always include `metadata: { scan_id, email }` in CreateCheckoutSession; validate metadata exists before returning session URL to client
**Warning signs:** Checkout completes successfully but webhook handler errors with "missing scan_id in metadata"

### Pitfall 3: PDF Generation Blocks Webhook Response
**What goes wrong:** Stripe retries webhook because handler times out during PDF generation
**Why it happens:** PDF generation (especially with many findings) takes 5-10 seconds, exceeds Stripe's timeout
**How to avoid:** Return 200 immediately after recording payment in database, spawn `tokio::spawn` for PDF generation + email delivery
**Warning signs:** Stripe dashboard shows webhook retries, same event processed multiple times, duplicate emails sent

### Pitfall 4: Resend Attachment Size Miscalculation
**What goes wrong:** Email send fails with "payload too large" error
**Why it happens:** Base64 encoding increases size by ~33%; 30MB PDF becomes 40MB encoded
**How to avoid:** Limit PDF to ~25MB raw (far larger than text reports need); validate encoded size before calling Resend API
**Warning signs:** Email delivery errors in logs showing 413 or payload size errors from Resend

### Pitfall 5: Tier Differentiation Too Subtle
**What goes wrong:** Users don't see value in paid tier, low conversion
**Why it happens:** Paid tier only adds 2-3 findings beyond free tier, doesn't feel worth $49
**How to avoid:** Free tier should be deliberately limited (10 JS files, basic Nuclei templates, no active probing); paid tier should consistently find 5-10x more issues
**Warning signs:** Conversion rate <2% from free results page to checkout; user feedback asks "what's the difference?"

### Pitfall 6: Success URL Trusted for Scan Triggering
**What goes wrong:** User never gets paid scan because they closed tab before redirect
**Why it happens:** Scan triggered on success_url page load instead of webhook
**How to avoid:** Success URL only shows "processing" message; webhook handler triggers actual scan
**Warning signs:** Stripe shows successful payments but corresponding scans stuck in "pending" status

### Pitfall 7: Nuclei Template Paths Hardcoded
**What goes wrong:** Paid tier can't find extended templates, falls back to free tier scanning
**Why it happens:** Template selection logic references wrong paths for tier-specific templates
**How to avoid:** Organize templates in `templates/free/` and `templates/paid/` directories; use tier enum to select directory
**Warning signs:** Paid scans complete quickly (same duration as free), findings count nearly identical

## Code Examples

Verified patterns from official sources:

### Stripe Checkout URL Endpoint (Axum)
```rust
// Source: Stripe Checkout Sessions API + async-stripe examples
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateCheckoutRequest {
    scan_id: Uuid,
}

#[derive(Serialize)]
struct CreateCheckoutResponse {
    checkout_url: String,
}

async fn create_checkout(
    State(app_state): State<AppState>,
    Json(req): Json<CreateCheckoutRequest>,
) -> Result<Json<CreateCheckoutResponse>, ApiError> {
    // Validate scan exists and is completed
    let scan = db::get_scan(&req.scan_id).await?;
    if scan.status != "completed" {
        return Err(ApiError::BadRequest("Scan not completed".into()));
    }

    // Create Stripe session
    let client = Client::new(app_state.stripe_secret_key);
    let session = CheckoutSession::create(&client, CreateCheckoutSession {
        mode: Some(stripe::CheckoutSessionMode::Payment),
        line_items: Some(vec![CreateCheckoutSessionLineItems {
            price_data: Some(CreateCheckoutSessionLineItemsPriceData {
                currency: stripe::Currency::USD,
                unit_amount: Some(4900),
                product_data: Some(CreateCheckoutSessionLineItemsPriceDataProductData {
                    name: "TrustEdge Deep Audit".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            quantity: Some(1),
            ..Default::default()
        }]),
        customer_email: Some(scan.email.clone()),
        success_url: Some(format!("{}/payment/success?session_id={{CHECKOUT_SESSION_ID}}",
            app_state.frontend_url)),
        cancel_url: Some(format!("{}/results/{}", app_state.frontend_url, req.scan_id)),
        metadata: Some([
            ("scan_id".to_string(), req.scan_id.to_string()),
            ("email".to_string(), scan.email.clone()),
        ].into_iter().collect()),
        ..Default::default()
    }).await?;

    // Record pending audit in database
    db::create_paid_audit(PaidAudit {
        scan_id: req.scan_id,
        stripe_checkout_session_id: session.id.to_string(),
        amount_cents: 4900,
        customer_email: scan.email,
        status: "pending".to_string(),
        ..Default::default()
    }).await?;

    Ok(Json(CreateCheckoutResponse {
        checkout_url: session.url.ok_or(ApiError::StripeError)?,
    }))
}
```

### Webhook Event Idempotency Check
```rust
// Source: Stripe webhook best practices + PostgreSQL unique constraints
async fn event_already_processed(event_id: &str) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO stripe_events (event_id, processed_at)
        VALUES ($1, NOW())
        ON CONFLICT (event_id) DO NOTHING
        RETURNING event_id
        "#,
        event_id
    )
    .fetch_optional(&pool)
    .await?;

    // If no row returned, event_id already existed (duplicate)
    Ok(result.is_none())
}

// Migration:
// CREATE TABLE stripe_events (
//     event_id VARCHAR(255) PRIMARY KEY,
//     processed_at TIMESTAMPTZ DEFAULT NOW()
// );
```

### Upgrade CTA Component (React/Next.js)
```tsx
// Source: SaaS CTA best practices 2026
export function UpgradeCTA({ scanId }: { scanId: string }) {
  const [loading, setLoading] = useState(false);

  const handleUpgrade = async () => {
    setLoading(true);
    try {
      const res = await fetch('/api/create-checkout', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ scan_id: scanId }),
      });
      const { checkout_url } = await res.json();
      window.location.href = checkout_url;
    } catch (err) {
      console.error('Checkout failed:', err);
      setLoading(false);
    }
  };

  return (
    <div className="border-2 border-blue-500 rounded-lg p-6 my-8 bg-blue-50">
      <h3 className="text-xl font-bold mb-2">Upgrade to Deep Audit</h3>
      <p className="text-gray-700 mb-4">
        Get a comprehensive security analysis with:
      </p>
      <ul className="list-disc list-inside mb-4 space-y-1">
        <li>10x more vulnerability checks (extended Nuclei templates)</li>
        <li>Active probing for hidden vulnerabilities</li>
        <li>Professional PDF report with remediation roadmap</li>
        <li>Extended scan coverage (50+ JS files, deep path enumeration)</li>
      </ul>
      <button
        onClick={handleUpgrade}
        disabled={loading}
        className="bg-blue-600 text-white px-6 py-3 rounded-lg font-semibold hover:bg-blue-700 disabled:opacity-50"
      >
        {loading ? 'Redirecting...' : 'Upgrade for $49'}
      </button>
      <p className="text-sm text-gray-500 mt-2">One-time payment • Instant report delivery</p>
    </div>
  );
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual Stripe HTTP requests | async-stripe crate | 2024+ | Weekly auto-generation keeps pace with Stripe API changes |
| HTML→PDF via headless Chrome | Pure Rust PDF libs (genpdf, krilla) | 2023-2024 | 10x faster, no heavy dependencies, better memory efficiency |
| File-based PDF delivery | Base64 email attachments | 2020+ | No filesystem cleanup, simpler architecture, 40MB limit sufficient |
| success_url triggers payment actions | Webhooks as source of truth | Always (Stripe best practice) | Handles browser close, network issues, ensures delivery |
| Fixed pricing ($49 or $99) | Dynamic pricing based on target complexity | Future trend | Some scanners charge more for large sites/apps |

**Deprecated/outdated:**
- **stripe-rust (wyyerd/stripe-rs):** Less active than async-stripe, not auto-generated from OpenAPI spec
- **PDF libraries with C dependencies:** printpdf alone (underlying genpdf) requires managing external font libraries
- **Synchronous Stripe client:** Blocking operations incompatible with async Rust (Axum, Tokio)

## Open Questions

Things that couldn't be fully resolved:

1. **Optimal price point ($49 vs $99)**
   - What we know: Stripe fees are 2.9% + $0.30; $49 = $1.72 fee, $99 = $3.17 fee
   - What's unclear: Market research needed to test price sensitivity; competitor pricing ranges $30-$150
   - Recommendation: Start at $49 (lower barrier), A/B test $79 after initial sales data

2. **Table formatting in PDF reports**
   - What we know: genpdf lacks native table support; custom Element trait is complex
   - What's unclear: Whether text-based formatting (whitespace-aligned columns) is acceptable for professional reports
   - Recommendation: Use bullet lists and bold headers; defer rich tables to v2 unless user feedback demands it

3. **Paid tier scan timeout limits**
   - What we know: Free tier uses 180s timeout; paid tier needs more time for extended scanning
   - What's unclear: Optimal timeout to balance completion rate vs. user wait time
   - Recommendation: Start with 600s (10 minutes), monitor completion rates, adjust if >5% hit timeout

4. **Nuclei template licensing for commercial use**
   - What we know: Nuclei is MIT licensed (safe for commercial use); community templates are also MIT
   - What's unclear: Whether custom templates need attribution if derived from community templates
   - Recommendation: Create fully custom templates for paid tier to avoid any ambiguity; cite ProjectDiscovery in docs

5. **Stripe webhook retry handling on persistent errors**
   - What we know: Stripe retries for 3 days with exponential backoff
   - What's unclear: What to do if scan orchestrator fails repeatedly (infrastructure issue)
   - Recommendation: Monitor webhook failure rate; if scan fails >3 times, send refund + apology email with manual review

## Sources

### Primary (HIGH confidence)
- [async-stripe v0.41.0 documentation](https://docs.rs/async-stripe) - API patterns, webhook handling
- [genpdf v0.2.0 documentation](https://docs.rs/genpdf) - PDF generation, auto-pagination
- [Resend attachments documentation](https://resend.com/docs/dashboard/emails/attachments) - Base64 attachments, size limits
- [Stripe API Reference: Checkout Sessions](https://docs.stripe.com/api/checkout/sessions/create) - Metadata, success_url patterns
- [Stripe webhooks documentation](https://docs.stripe.com/webhooks) - Idempotency, retry behavior
- [Stripe metadata documentation](https://docs.stripe.com/metadata) - Key-value limits, use cases

### Secondary (MEDIUM confidence)
- [Shuttle blog: Stripe Payments with Rust](https://www.shuttle.dev/blog/2024/03/07/stripe-payments-rust) - Integration patterns
- [Stripe webhook best practices (Hookdeck)](https://hookdeck.com/webhooks/platforms/guide-to-stripe-webhooks-features-and-best-practices) - Idempotency, signatures
- [SaaS pricing page best practices 2026 (MADX)](https://www.madx.digital/learn/saas-pricing-pages) - CTA placement, transparency
- [Security scanner tier differentiation (Comparitech)](https://www.comparitech.com/net-admin/free-network-vulnerability-scanners/) - Feature patterns
- [Tokio spawning tutorial](https://tokio.rs/tokio/tutorial/spawning) - Background task patterns

### Tertiary (LOW confidence - needs verification)
- [Rust PDF library comparison (DocRaptor)](https://docraptor.com/rust-html-to-pdf) - General overview, no specific version info
- [Payment database schema patterns (Indie Hackers)](https://www.indiehackers.com/post/suggested-database-architecture-for-my-first-saas-with-stripe-7b6ff9927f) - Community discussion, not authoritative

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - async-stripe and genpdf are actively maintained with official docs
- Architecture: HIGH - Patterns verified against Stripe official docs and async-stripe examples
- Pitfalls: MEDIUM - Based on Stripe best practices docs and community experiences, not project-specific

**Research date:** 2026-02-06
**Valid until:** 2026-03-08 (30 days - stable domain, Stripe API changes infrequent)
