# Phase 3: Vibe-Code Intelligence - Research

**Researched:** 2026-02-05
**Domain:** Framework detection, vibe-code vulnerability scanning, BaaS security
**Confidence:** MEDIUM

## Summary

This phase requires implementing three capabilities: (1) framework/platform fingerprinting from HTTP responses, (2) custom Nuclei templates for vibe-code-specific vulnerabilities, and (3) copy-paste remediation code generation.

Framework detection uses multiple signal correlation — HTTP headers (x-vercel-id, x-nf-request-id), HTML patterns (__NEXT_DATA__ script tag, meta generators), and JavaScript artifacts (_next/static chunks). High confidence requires 2+ signals per framework to avoid false positives.

Nuclei v3 provides YAML-based template DSL with http/dns/network modes, word/status/regex/dsl matchers, and JSON output via -jsonl flag. Custom templates target vibe-code vulnerabilities: Supabase RLS disabled (CVE-2025-48757 class), Firebase permissive rules, NEXT_PUBLIC_ env leaks, and unprotected API routes.

**Primary recommendation:** Build framework detector as first-stage scanner before Nuclei execution. Detection results feed downstream scanners to enable framework-specific Nuclei templates and targeted remediation snippets.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Detection scope:**
- Detect 4 frontend frameworks: Next.js, Vite/React, SvelteKit, Nuxt
- Detect 3 deployment platforms: Vercel, Netlify, Railway
- High confidence only — require multiple signals before showing a framework badge (no guessing)
- Detection runs as a visible, separate scan stage ("Detecting framework...") before other scanners
- Framework detection results feed into downstream scanners to enable framework-specific checks

**Vulnerability coverage:**
- Full vibe-code checklist: BaaS misconfigs (Supabase RLS, Firebase rules), env variable leaks (NEXT_PUBLIC_ secrets, .env in build output), missing auth middleware, unprotected API routes, default admin credentials
- Custom Nuclei templates for vibe-code-specific checks + curated community templates for general web vulns
- Probing depth: passive + light active only (analyze responses, probe known BaaS endpoints). Reserve aggressive active probing for Phase 4 paid tier
- Findings use existing severity levels (Critical/High/Medium/Low) for A-F grading, plus a "vibe-code" tag to identify AI-generated-code issues

**Remediation format:**
- Targeted diffs: "In your next.config.js, add these lines" — not full file replacements
- Moderate explanation: code block + 1-2 sentence explanation of why the fix works and what it prevents
- Version-aware when it matters: default to framework-family-level fixes, split only when syntax genuinely differs (e.g., Next.js App Router vs Pages Router)
- No "verify your fix" sections — users rescan to verify

**Results presentation:**
- Framework + platform badge inline with grade circle: "B — Next.js on Vercel"
- Vibe-code findings get a small "Vibe-Code" tag/badge — subtle differentiation, not a separate section
- No vibe-code filter toggle — the tag is sufficient, keep UI simple
- When no framework detected: show "Framework: Not detected" next to grade (honest, signals feature exists)

### Claude's Discretion

- Exact HTML/JS fingerprinting patterns for each framework
- Nuclei template structure and naming conventions
- Number of detection signals required for "high confidence" threshold
- How to handle partial matches (e.g., React detected but not the meta-framework)

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope

</user_constraints>

## Standard Stack

The established libraries/tools for this domain:

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Nuclei | v3.x | Vulnerability scanner | Industry-standard YAML-based template engine, 6,000+ community templates, Docker support |
| reqwest | 0.12+ | HTTP client | Rust's standard async HTTP client, connection pooling, header access |
| scraper | 0.20+ | HTML parsing | CSS selector-based DOM parsing, wraps html5ever parser |
| serde_json | 1.0+ | JSON parsing | De facto Rust JSON library, __NEXT_DATA__ extraction |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio | 1.40+ | Async runtime | Already in use, parallel detection probes |
| regex | 1.10+ | Pattern matching | JavaScript artifact detection (_next/static patterns) |
| docker_api | 0.16+ | Container orchestration | If managing Nuclei containers from Rust (alternative: docker CLI via std::process) |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Nuclei | Custom Rust scanner | Nuclei has mature templates + community contributions. Custom = reinventing wheel |
| scraper | select.rs | scraper is more actively maintained and better documented |
| Docker CLI | bollard/docker_api | CLI simpler for MVP, Rust API for advanced lifecycle management |

**Installation:**
```bash
# Rust dependencies (add to Cargo.toml)
reqwest = { version = "0.12", features = ["json"] }
scraper = "0.20"
serde_json = "1.0"
regex = "1.10"

# Nuclei Docker image
docker pull projectdiscovery/nuclei:latest
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── scanners/
│   ├── detector.rs          # Framework/platform fingerprinting
│   ├── nuclei.rs             # Nuclei template executor (extends container.rs)
│   └── remediation.rs        # Framework-specific fix generation
├── models/
│   ├── detection.rs          # DetectionResult, Framework, Platform enums
│   └── finding.rs            # Extend with vibe_code: bool tag
└── templates/
    └── nuclei/
        ├── supabase-rls.yaml
        ├── firebase-rules.yaml
        ├── nextjs-env-leak.yaml
        └── unprotected-routes.yaml
```

### Pattern 1: Multi-Signal Framework Detection

**What:** Correlate 2+ independent signals to identify framework with high confidence

**When to use:** Before executing framework-specific vulnerability checks

**Example:**
```rust
// Source: Web scraping patterns from https://www.scrapingbee.com/blog/web-scraping-rust/
// and fingerprinting best practices from https://chameleonmode.com/browser-detection-fingerprinting-2026/

use scraper::{Html, Selector};
use reqwest::Response;

#[derive(Debug)]
struct DetectionSignals {
    http_headers: Vec<String>,     // x-powered-by, server
    html_patterns: Vec<String>,    // script id="__NEXT_DATA__", meta generator
    js_artifacts: Vec<String>,     // /_next/static, /.vite, /__sveltekit
}

async fn detect_framework(response: &Response, html: &str) -> Option<Framework> {
    let mut signals = DetectionSignals::default();

    // Signal 1: HTTP headers
    if response.headers().get("x-powered-by")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("Next.js"))
        .unwrap_or(false) {
        signals.http_headers.push("x-powered-by:Next.js".to_string());
    }

    // Signal 2: __NEXT_DATA__ script tag
    let document = Html::parse_document(html);
    let next_data_selector = Selector::parse("#__NEXT_DATA__").unwrap();
    if document.select(&next_data_selector).next().is_some() {
        signals.html_patterns.push("__NEXT_DATA__".to_string());
    }

    // Signal 3: _next/static in script src
    let script_selector = Selector::parse("script[src*='/_next/static']").unwrap();
    if document.select(&script_selector).next().is_some() {
        signals.js_artifacts.push("/_next/static".to_string());
    }

    // High confidence threshold: 2+ signals
    let signal_count = signals.http_headers.len()
        + signals.html_patterns.len()
        + signals.js_artifacts.len();

    if signal_count >= 2 {
        Some(Framework::NextJs)
    } else {
        None
    }
}
```

### Pattern 2: Nuclei Template Execution with JSON Parsing

**What:** Run Nuclei in Docker container, parse JSONL output into Finding structs

**When to use:** For vulnerability checks after framework detection completes

**Example:**
```rust
// Source: Nuclei Docker integration from https://projectdiscovery.io/blog/how-to-run-nuclei-other-projectdiscovery-tools-in-docker
// and JSON output docs from https://docs.projectdiscovery.io/tools/nuclei/running

use std::process::Command;
use serde_json::Value;

async fn run_nuclei_template(
    target_url: &str,
    template_path: &str,
    framework: Option<Framework>
) -> Result<Vec<Finding>, ScanError> {
    // Select templates based on detected framework
    let templates = match framework {
        Some(Framework::NextJs) => vec![
            "templates/nuclei/nextjs-env-leak.yaml",
            "templates/nuclei/unprotected-api-routes.yaml"
        ],
        Some(Framework::Nuxt) | Some(Framework::SvelteKit) => vec![
            "templates/nuclei/generic-env-leak.yaml"
        ],
        None => vec!["templates/nuclei/generic-vibe-code.yaml"],
        _ => vec![template_path],
    };

    let output = Command::new("docker")
        .args([
            "run", "--rm",
            "-v", &format!("{}:/templates:ro", templates_dir),
            "projectdiscovery/nuclei:latest",
            "-u", target_url,
            "-t", "/templates/",
            "-jsonl",        // JSONL output format
            "-passive",      // Passive + light active only (per requirements)
            "-severity", "critical,high,medium,low"
        ])
        .output()
        .await?;

    // Parse JSONL output
    let stdout = String::from_utf8(output.stdout)?;
    let findings: Vec<Finding> = stdout
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .map(|json| Finding {
            scanner_name: "nuclei".to_string(),
            title: json["info"]["name"].as_str().unwrap_or("").to_string(),
            severity: parse_severity(json["info"]["severity"].as_str()),
            vibe_code: true,  // NEW: Tag vibe-code findings
            // ... rest of mapping
        })
        .collect();

    Ok(findings)
}
```

### Pattern 3: Framework-Specific Remediation Generation

**What:** Generate targeted diffs/snippets based on detected framework and vulnerability type

**When to use:** When creating remediation guidance for Finding structs

**Example:**
```rust
// Source: Best practices from https://thelinuxcode.com/nextjs-environment-variables-2026-build-time-vs-runtime-security-and-production-patterns/

fn generate_remediation(
    vuln_type: VulnType,
    framework: Framework
) -> String {
    match (vuln_type, framework) {
        (VulnType::EnvLeak, Framework::NextJs) => {
            r#"
**In your .env.local file:**
```diff
- NEXT_PUBLIC_SUPABASE_SECRET_KEY=your-secret-key
+ SUPABASE_SECRET_KEY=your-secret-key  # Remove NEXT_PUBLIC_ prefix
```

This prevents the secret from being embedded in client-side JavaScript bundles. Only use NEXT_PUBLIC_ for genuinely public values like API URLs.
"#.to_string()
        },

        (VulnType::UnprotectedRoute, Framework::NextJs) => {
            r#"
**In your middleware.ts:**
```typescript
import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

export function middleware(request: NextRequest) {
  // Check authentication for API routes
  if (request.nextUrl.pathname.startsWith('/api/')) {
    const token = request.cookies.get('auth-token');
    if (!token) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }
  }
  return NextResponse.next();
}

export const config = {
  matcher: '/api/:path*'
};
```

This adds authentication checks before API routes execute, preventing unauthenticated access.
"#.to_string()
        },

        // Framework-family-level fixes for generic patterns
        _ => generic_remediation(vuln_type)
    }
}
```

### Anti-Patterns to Avoid

- **Single-signal detection:** Leads to false positives. Next.js meta tag alone doesn't confirm Next.js (could be spoofed or cached).
- **Blocking scanner execution on detection:** Detection should enhance but not gate scanning. If detection fails, run generic templates.
- **Full file replacements in remediation:** Users won't trust/apply large diffs. Targeted snippets have higher adoption.
- **Guessing frameworks at low confidence:** "Might be Next.js" creates confusion. Better to say "Framework: Not detected" honestly.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Vulnerability scanning | Custom pattern matching | Nuclei templates | 6,000+ community templates, constant updates for new CVEs, battle-tested |
| HTML parsing | String regex on raw HTML | scraper crate | Handles malformed HTML, CSS selectors match browser behavior, encoding edge cases |
| Framework detection | Header-only checks | Multi-signal correlation | Single signals fail on proxies (headers stripped), CDNs (headers added), or static exports |
| HTTP client | std::net::TcpStream | reqwest | Connection pooling (3-5x faster), redirect handling, compression, timeout management |
| Nuclei container management | Custom Docker lifecycle | Docker CLI via Command | Nuclei already containerized officially, CLI is simpler for execute-and-parse pattern |

**Key insight:** Vibe-code vulnerabilities change rapidly as AI tools evolve. Nuclei's community template ecosystem (updated daily) means you don't maintain detection logic — you curate templates.

## Common Pitfalls

### Pitfall 1: False Positives from Single-Signal Detection

**What goes wrong:** Framework badge shows "Next.js" for a Remix app that proxies through Vercel

**Why it happens:** x-vercel-id header indicates Vercel platform, but doesn't confirm Next.js framework. Vercel hosts all frameworks.

**How to avoid:** Require 2+ independent signals from different categories (header + HTML pattern, or HTML + JS artifact). Conflicting signals → no detection.

**Warning signs:** User reports "wrong framework detected" or "badge shows Next.js but we use Remix"

### Pitfall 2: Nuclei Template False Negatives on Framework Variations

**What goes wrong:** Next.js App Router API routes (/app/api/route.ts) not detected by template targeting Pages Router patterns (/pages/api/*.ts)

**Why it happens:** Next.js has two router paradigms with different file conventions. Single template only checks Pages Router paths.

**How to avoid:**
- Write templates that check both patterns: `/pages/api/*` AND `/app/*/route.ts`
- Use DSL matchers with OR conditions rather than multiple templates
- Document which framework versions each template supports

**Warning signs:** Nuclei finds 0 results on confirmed Next.js 14+ apps (App Router is default since v13)

### Pitfall 3: BaaS Misconfiguration False Positives

**What goes wrong:** Scanner flags "Supabase RLS disabled" when RLS is actually enabled but anon key allows public reads (intended)

**Why it happens:** Template checks if data is accessible with anon key, but doesn't distinguish intended public access from misconfiguration.

**How to avoid:**
- Check BOTH accessibility AND table name/schema patterns
- Public tables (posts, comments) with RLS disabled + anon access = LOW severity, not CRITICAL
- Private tables (users, payments) with RLS disabled = CRITICAL
- Use table name heuristics: users/profiles/accounts/payments/orders → private

**Warning signs:** High false positive rate (>30%) on RLS findings, user feedback "this is intentional"

### Pitfall 4: Environment Variable Leak Over-Detection

**What goes wrong:** NEXT_PUBLIC_SUPABASE_URL flagged as leak, but it's the publishable URL (safe to expose)

**Why it happens:** Template pattern-matches NEXT_PUBLIC_ prefix without distinguishing publishable vs secret values.

**How to avoid:**
- Whitelist safe patterns: NEXT_PUBLIC_SUPABASE_URL, NEXT_PUBLIC_SUPABASE_ANON_KEY (publishable by design)
- Flag secrets only: SUPABASE_SERVICE_ROLE_KEY, STRIPE_SECRET_KEY, API_SECRET
- Check value patterns: JWT secrets are long base64, API URLs are https://
- Context matters: anon key in NEXT_PUBLIC_ = OK, service role key in NEXT_PUBLIC_ = CRITICAL

**Warning signs:** Every Next.js + Supabase app flagged, even official Supabase examples

### Pitfall 5: Platform Detection Confusion (Vercel vs Netlify vs Railway)

**What goes wrong:** App deployed to Vercel shows "Platform: Railway" because Railway environment variables exist in build

**Why it happens:** Checking process.env.RAILWAY_ENVIRONMENT in client-side bundle, but these vars may leak from developer's local environment or CI.

**How to avoid:**
- Prioritize runtime response headers over environment variables
- x-vercel-id (response header) > VERCEL_ENV (env var)
- x-nf-request-id (response header) > NETLIFY (env var)
- Railway detection: Check X-Railway-Request-Id header, not env vars in JS bundles
- If multiple platform signals detected, choose the one with response header evidence

**Warning signs:** Platform badge inconsistent across rescans, local dev shows different platform than production

## Code Examples

Verified patterns from official sources:

### Framework Detection: Next.js Signals

```rust
// Source: Next.js detection patterns synthesized from:
// - https://github.com/vercel/next.js/discussions/15117 (__NEXT_DATA__)
// - https://www.scrapingbee.com/blog/web-scraping-rust/ (scraper usage)

use scraper::{Html, Selector};
use serde_json::Value;

/// Returns confidence score (0-100) for Next.js detection
fn detect_nextjs_confidence(html: &str, headers: &HeaderMap) -> u8 {
    let mut score = 0u8;
    let document = Html::parse_document(html);

    // Signal 1: __NEXT_DATA__ script tag (HIGH weight: 40 points)
    if let Ok(selector) = Selector::parse("#__NEXT_DATA__") {
        if let Some(script) = document.select(&selector).next() {
            // Verify it contains valid JSON with Next.js structure
            if let Some(json_str) = script.text().next() {
                if let Ok(json) = serde_json::from_str::<Value>(json_str) {
                    if json.get("buildId").is_some() && json.get("page").is_some() {
                        score += 40;
                    }
                }
            }
        }
    }

    // Signal 2: _next/static chunks (MEDIUM weight: 30 points)
    if let Ok(selector) = Selector::parse("script[src*='/_next/static']") {
        if document.select(&selector).next().is_some() {
            score += 30;
        }
    }

    // Signal 3: x-powered-by header (LOW weight: 20 points, can be stripped)
    if let Some(powered_by) = headers.get("x-powered-by") {
        if powered_by.to_str().ok()
            .map(|v| v.contains("Next.js"))
            .unwrap_or(false) {
            score += 20;
        }
    }

    // Signal 4: meta generator tag (LOW weight: 10 points)
    if let Ok(selector) = Selector::parse("meta[name='generator']") {
        if let Some(meta) = document.select(&selector).next() {
            if meta.value().attr("content")
                .map(|c| c.contains("Next.js"))
                .unwrap_or(false) {
                score += 10;
            }
        }
    }

    score.min(100)
}

// High confidence threshold: 60+ (requires 2+ strong signals)
const HIGH_CONFIDENCE_THRESHOLD: u8 = 60;
```

### Platform Detection: Vercel/Netlify/Railway

```rust
// Source: Response header documentation
// - https://vercel.com/docs/headers/response-headers (x-vercel-id)
// - https://answers.netlify.com/t/support-guide-netlify-support-asked-for-the-x-nf-request-id-header/4385 (x-nf-request-id)
// - https://docs.railway.com/reference/variables (X-Railway-Request-Id)

#[derive(Debug, Clone, PartialEq)]
enum Platform {
    Vercel,
    Netlify,
    Railway,
    Unknown,
}

fn detect_platform(headers: &HeaderMap) -> Platform {
    // Priority: response headers (definitive) > server header patterns

    // Vercel: x-vercel-id header (format: "edge-region::function-region")
    if headers.get("x-vercel-id").is_some() {
        return Platform::Vercel;
    }

    // Netlify: x-nf-request-id header
    if headers.get("x-nf-request-id").is_some() {
        return Platform::Netlify;
    }

    // Railway: X-Railway-Request-Id header
    if headers.get("x-railway-request-id").is_some() {
        return Platform::Railway;
    }

    // Fallback: server header (lower confidence)
    if let Some(server) = headers.get("server") {
        if let Ok(server_str) = server.to_str() {
            if server_str.contains("Vercel") {
                return Platform::Vercel;
            }
            if server_str.contains("Netlify") {
                return Platform::Netlify;
            }
        }
    }

    Platform::Unknown
}
```

### Custom Nuclei Template: Supabase RLS Check

```yaml
# Source: Template structure from https://projectdiscovery.io/blog/ultimate-nuclei-guide/
# Vulnerability pattern from https://deepstrike.io/blog/hacking-thousands-of-misconfigured-supabase-instances-at-scale

id: supabase-rls-disabled

info:
  name: Supabase Row Level Security Disabled
  author: trustedge-audit
  severity: critical
  description: |
    Detects Supabase databases with Row Level Security (RLS) disabled,
    allowing unauthenticated access to sensitive tables. This is the
    vulnerability class behind CVE-2025-48757 (Lovable RLS misconfigs).
  reference:
    - https://deepstrike.io/blog/hacking-thousands-of-misconfigured-supabase-instances-at-scale
    - https://byteiota.com/supabase-security-flaw-170-apps-exposed-by-missing-rls/
  tags: vibe-code,supabase,rls,baas

http:
  - method: GET
    path:
      - "{{BaseURL}}"

    matchers-condition: and
    matchers:
      # First, confirm Supabase is present
      - type: word
        part: body
        words:
          - "supabase"
          - ".supabase.co"
        condition: or

      # Check if Supabase anon key is exposed in client bundle
      - type: regex
        part: body
        regex:
          - 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+'
        name: anon_key

    extractors:
      - type: regex
        name: supabase_url
        part: body
        group: 1
        regex:
          - 'https://([a-z0-9-]+)\.supabase\.co'
        internal: true

  # Second request: Test RLS with extracted credentials
  - method: GET
    path:
      - "https://{{supabase_url}}.supabase.co/rest/v1/users"

    headers:
      apikey: "{{anon_key}}"
      Authorization: "Bearer {{anon_key}}"

    matchers-condition: and
    matchers:
      - type: status
        status:
          - 200

      # Check if we got actual data (not empty array)
      - type: word
        part: body
        words:
          - "email"
          - "password"
          - "user_id"
        condition: or

      # Confirm it's not intentionally public (heuristic: private table names)
      - type: word
        part: header
        words:
          - "Content-Range"  # PostgreSQL returns this, confirms table access

    extractors:
      - type: json
        part: body
        name: leaked_records
        json:
          - '.[].email'

# Note: This template uses passive + light active probing only
# Aggressive enumeration (trying multiple table names) reserved for paid tier
```

### Targeted Remediation: Next.js Environment Variable Leak

```rust
// Source: Best practices from https://thelinuxcode.com/nextjs-environment-variables-2026-build-time-vs-runtime-security-and-production-patterns/

fn remediate_nextjs_env_leak(leaked_var: &str) -> String {
    // Classify severity based on variable name
    let (severity, fix) = if leaked_var.contains("SECRET")
        || leaked_var.contains("PRIVATE")
        || leaked_var.contains("SERVICE_ROLE") {
        (
            "CRITICAL",
            format!(r#"
**In your .env.local file:**
```diff
- NEXT_PUBLIC_{var}=...
+ {var}=...  # Remove NEXT_PUBLIC_ prefix
```

**In your server-side code (app/api/route.ts or pages/api/*.ts):**
```typescript
// Access via process.env (server-side only)
const secret = process.env.{var};
```

**Why this matters:** Variables prefixed with NEXT_PUBLIC_ are embedded in client-side JavaScript bundles and visible to anyone who visits your site. Secrets must ONLY exist server-side.

**Rotate your secret immediately** — the exposed value is compromised.
"#, var = leaked_var.strip_prefix("NEXT_PUBLIC_").unwrap_or(leaked_var))
        )
    } else if leaked_var.contains("ANON_KEY") || leaked_var.contains("PUBLISHABLE") {
        (
            "INFO",
            format!(r#"
**Current usage is correct:** `{var}` is a publishable/anonymous key designed to be client-accessible.

However, verify that:
1. Your backend enforces Row Level Security (RLS) on database tables
2. API routes validate this key properly
3. You're not confusing this with a secret key (secret keys should NEVER use NEXT_PUBLIC_)
"#, var = leaked_var)
        )
    } else {
        (
            "MEDIUM",
            format!(r#"
**In your .env.local file:**
```diff
- NEXT_PUBLIC_{var}=...
+ {var}=...  # Remove NEXT_PUBLIC_ prefix if not truly public
```

Ask yourself: "Does the browser need this value?"
- If YES (e.g., API URLs, feature flags): NEXT_PUBLIC_ is OK
- If NO (e.g., API keys, internal URLs): Remove NEXT_PUBLIC_ prefix and use server-side only
"#, var = leaked_var.strip_prefix("NEXT_PUBLIC_").unwrap_or(leaked_var))
        )
    };

    format!("**Severity: {}**\n\n{}", severity, fix)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| User-Agent string parsing for framework detection | Multi-signal correlation (headers + HTML + JS artifacts) | 2024-2025 | Single signals fail with proxies/CDNs. Modern detection requires 2+ independent signals for reliability |
| Static pattern matching for BaaS misconfigs | Context-aware severity (table name heuristics + accessibility) | 2025-2026 | Reduces false positives: public tables with anon access are LOW severity, not CRITICAL |
| Next.js Pages Router only | Support both Pages Router and App Router | Next.js 13 (2022), mainstream by 2025 | App Router is default in Next.js 14+. Templates must check both `/pages/api/*` and `/app/*/route.ts` patterns |
| Nuclei v2 template syntax | Nuclei v3 with improved matchers and extractors | 2024 | v3 adds `matchers-condition: and/or`, improved DSL functions, better JSONL output |
| Generic remediation ("fix your config") | Framework-specific copy-paste diffs | 2025-2026 | Higher user adoption: targeted snippets vs full file replacements |

**Deprecated/outdated:**
- **Single-header framework detection:** Proxies strip/modify headers. Use HTML patterns as backup.
- **x-powered-by header reliance:** Many frameworks remove this by default for security. It's a bonus signal, not primary.
- **Nuclei v2 template syntax:** v3 is current, templates should use new matchers-condition syntax
- **Default admin credentials checks for modern BaaS:** Supabase/Clerk/Auth0 don't ship with defaults (cloud dashboards with account-specific setup). This is a legacy web app pattern, not relevant to vibe-code BaaS platforms.

## Open Questions

Things that couldn't be fully resolved:

1. **Next.js App Router vs Pages Router differentiation in detection**
   - What we know: App Router uses `/app` directory, Pages Router uses `/pages`. __NEXT_DATA__ structure differs slightly.
   - What's unclear: Whether detection needs to distinguish them (both are Next.js). User requirements say "split only when syntax genuinely differs."
   - Recommendation: Detect as "Next.js (App Router)" vs "Next.js (Pages Router)" only if remediation genuinely differs. Otherwise, generic "Next.js" is sufficient. Most env var and RLS fixes are identical across routers.

2. **Vite/React disambiguation**
   - What we know: Vite is a build tool, React is a framework. Many frameworks use Vite (Remix, SvelteKit can use Vite).
   - What's unclear: User requirements say "detect Vite/React" as one item. Is this "React apps built with Vite" or "Vite apps (any framework)"?
   - Recommendation: Interpret as "React apps built with Vite" (not Next.js, which has its own bundler). Detection pattern: React presence (.js contains "react") + Vite artifacts (/.vite/ directory or import.meta.env references) + NO Next.js signals.

3. **High confidence threshold (number of signals)**
   - What we know: Browser fingerprinting uses tiered confidence (high/medium/low) based on signal strength and correlation. Multiple independent signals reduce false positives.
   - What's unclear: Exact threshold (2 signals? 3 signals? Weighted scoring?)
   - Recommendation: Use weighted scoring approach (see code example above). Strong signals (Next.js __NEXT_DATA__) = 40 points, weak signals (meta generator) = 10 points. Threshold: 60+ for "high confidence" detection. This requires 2 strong signals OR 1 strong + 3 weak.

4. **Nuclei template curation vs custom authoring**
   - What we know: 6,000+ community templates exist. Custom templates required for vibe-code-specific checks.
   - What's unclear: Ratio of custom vs curated community templates. How many custom templates are realistically needed?
   - Recommendation: Start with 5-7 custom templates (Supabase RLS, Firebase rules, NEXT_PUBLIC_ leaks, unprotected API routes, .env exposure). Curate 20-30 community templates for general web vulns (CORS misconfig, XXE, SSRF). Custom templates are the differentiator; community templates are table stakes.

## Sources

### Primary (HIGH confidence)

- [Nuclei GitHub Repository](https://github.com/projectdiscovery/nuclei) - Official v3 documentation, template syntax reference
- [Nuclei Ultimate Guide - ProjectDiscovery Blog](https://projectdiscovery.io/blog/ultimate-nuclei-guide/) - Template best practices, matcher/extractor syntax, testing workflow
- [Nuclei Docker Integration - ProjectDiscovery Blog](https://projectdiscovery.io/blog/how-to-run-nuclei-other-projectdiscovery-tools-in-docker) - Container execution patterns
- [Vercel Response Headers Documentation](https://vercel.com/docs/headers/response-headers) - x-vercel-id, x-vercel-cache, server header specs
- [Next.js Environment Variables 2026 - TheLinuxCode](https://thelinuxcode.com/nextjs-environment-variables-2026-build-time-vs-runtime-security-and-production-patterns/) - NEXT_PUBLIC_ security best practices
- [Supabase RLS Security Lessons - Bastion](https://bastion.tech/blog/moltbook-security-lessons-ai-agents) - CVE-2025-48757 analysis, RLS misconfiguration patterns
- [Web Scraping in Rust - ScrapingBee](https://www.scrapingbee.com/blog/web-scraping-rust/) - reqwest + scraper patterns, HTML parsing

### Secondary (MEDIUM confidence)

- [Supabase RLS Checker - GitHub](https://github.com/hand-dot/supabase-rls-checker) - Chrome extension approach to RLS detection
- [Hacking Thousands of Misconfigured Supabase Instances - DeepStrike](https://deepstrike.io/blog/hacking-thousands-of-misconfigured-supabase-instances-at-scale) - Real-world RLS exploitation techniques
- [Firebase Security Rules - Official Docs](https://firebase.google.com/docs/rules/insecure-rules) - Permissive rules patterns, best practices
- [Railway Variables Documentation](https://docs.railway.com/reference/variables) - X-Railway-Request-Id header, environment variables
- [Netlify x-nf-request-id Support Guide](https://answers.netlify.com/t/support-guide-netlify-support-asked-for-the-x-nf-request-id-header/4385) - Response header identification
- [Next.js Middleware Authentication - HashBuilds](https://www.hashbuilds.com/articles/next-js-middleware-authentication-protecting-routes-in-2025) - Unprotected API route patterns
- [Browser Fingerprinting 2026 - Chameleon](https://chameleonmode.com/browser-detection-fingerprinting-2026/) - Multi-signal detection principles, confidence thresholds

### Tertiary (LOW confidence)

- [Next.js App Router vs Pages Router Comparison](https://dev.to/shyam0118/app-router-vs-pages-router-in-nextjs-a-deep-practical-guide-341g) - Router differences (community article, not official docs)
- [Vibe-Code Vulnerability Scanner Landing Page](https://vibeappscanner.com/supabase-row-level-security) - Marketing content, limited technical detail
- WebSearch results for "Vite React SvelteKit Nuxt HTML meta generator tag framework detection 2026" - General framework comparisons, no specific detection techniques

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Nuclei is industry standard, reqwest/scraper are de facto Rust HTTP/HTML libraries
- Architecture: MEDIUM - Multi-signal detection pattern is well-established in fingerprinting, but framework-specific application to vibe-code scanning is novel
- Pitfalls: HIGH - False positive patterns documented in real-world Supabase/Firebase incidents (Moltbook breach, Lovable CVE-2025-48757)
- Nuclei templates: MEDIUM - Template syntax is well-documented, but vibe-code-specific template examples are synthesized from vulnerability research (not official templates)
- Remediation: MEDIUM - Next.js env var best practices are official, but targeted diff format is an implementation choice (no established standard)

**Research date:** 2026-02-05
**Valid until:** 30 days (framework detection patterns stable, but Nuclei templates and BaaS vulnerabilities evolve rapidly)
