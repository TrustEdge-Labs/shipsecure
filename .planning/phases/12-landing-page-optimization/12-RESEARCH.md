# Phase 12: Landing Page Optimization - Research

**Researched:** 2026-02-09
**Domain:** Developer-focused copywriting, landing page optimization, technical marketing, open-source attribution
**Confidence:** HIGH

## Summary

Phase 12 optimizes ShipSecure's landing page for a Hacker News launch by implementing developer-focused copy, methodology transparency, and open-source attribution. The phase addresses three core requirements: LAND-01 (developer-focused, technically honest copy), LAND-02 (clear "how it works" methodology section), and LAND-03 (open-source tool attribution in footer).

The research reveals that successful developer-focused landing pages use **technical honesty over marketing jargon**, with examples like Tailscale using "spec document" style copywriting and straightforward naming. HN-friendly messaging avoids "10 secret ways to..." formats in favor of direct technical descriptions. The landing page should balance benefits (for credibility) with features (for developer trust), using straightforward language without dumbing down technical concepts.

For methodology transparency, developer tools almost always explain "how it works" on the homepage, often using step-by-step formats with code snippets, GIFs, or interactive elements. Security scanner landing pages specifically should document scan methodology (passive vs active checks), provide proof of findings (screenshots, payloads), and map vulnerabilities to recognized frameworks (CWE, OWASP Top 10).

Open-source attribution requires listing **Title, Author, Source, and License** with clickable links. For ShipSecure, this means crediting Nuclei (MIT License, ProjectDiscovery), testssl.sh (GPLv2, with encouraged attribution for services), and other scanners. Best practice places attribution in the footer where it's discoverable but non-intrusive.

**Primary recommendation:** Rewrite landing page headline/tagline for technical clarity ("Security scanning for AI-generated web apps" vs marketing fluff). Add "How It Works" section with scan methodology breakdown (what each scanner checks, passive vs active techniques). Add open-source attribution section to existing footer with proper licensing links.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Next.js | 16.1.6 | App Router, Server Components | Existing stack, server-side rendering for copy |
| Tailwind CSS | Latest | Utility-first styling | Existing stack, matches current design patterns |
| Markdown/MDX | N/A | Content formatting for methodology | Standard for technical documentation |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| next/image | Built-in | Optimized images for "how it works" section | If adding diagrams or visual explanations |
| lucide-react | Latest | Icons for methodology steps | If adding visual indicators to scanners |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Plain HTML | CMS (Contentful, etc.) | Overkill for 3 pages, adds complexity |
| Inline copy | External copy file | No i18n needed yet, inline is simpler |
| Footer attribution | Separate credits page | Less discoverable, footer is standard practice |

**Installation:**
No additional packages needed - all work is copy updates and component modifications using existing stack.

## Architecture Patterns

### Recommended Content Structure
```
frontend/app/page.tsx
├── Hero Section (existing)
│   ├── Headline: developer-focused, target audience explicit
│   ├── Subhead: technical value prop (no superlatives)
│   └── CTA: scan form
├── "What We Check" Section (existing, may enhance)
│   └── Four scanners with technical detail
├── "How It Works" Section (NEW)
│   ├── Methodology overview (3-5 steps)
│   ├── Scanner breakdown (passive vs active)
│   └── Evidence/transparency note
├── Social Proof (existing scan counter)
└── Footer (existing, add OSS attribution)
    ├── Legal links (Privacy, Terms)
    ├── Copyright
    └── OSS Attribution (NEW)
        ├── "Powered by open source"
        ├── Nuclei by ProjectDiscovery (MIT)
        ├── testssl.sh (GPLv2)
        └── Link to full credits/licenses
```

### Pattern 1: Developer-Focused Headline Structure
**What:** Clear statement of product purpose and target audience without marketing jargon
**When to use:** Landing page hero, H1 tag, page title
**Example:**
```tsx
// BAD (marketing jargon)
<h1>Revolutionary AI-Powered Security Platform for Modern Teams</h1>
<p>Ship 10x faster with enterprise-grade protection</p>

// GOOD (developer-focused)
<h1>Security scanning for AI-generated web apps</h1>
<p>Free vulnerability detection for vibe-coded projects. No signup required.</p>
```
**Source:** [HN marketing learnings from Tailscale](https://www.markepear.dev/blog/developer-marketing-hacker-news) - use technical jargon-heavy conversational copywriting, avoid "10 secret ways to" style

### Pattern 2: "How It Works" Methodology Section
**What:** Step-by-step explanation of scan process with technical transparency
**When to use:** Below hero/CTA, before social proof
**Example:**
```tsx
// Structure from Evil Martians study of 100 dev tools
<section>
  <h2>How it works</h2>

  <ol>
    <li>
      <h3>Submit URL</h3>
      <p>No signup required. Scans start immediately.</p>
    </li>
    <li>
      <h3>Passive & Active Checks</h3>
      <p>Security headers (passive), TLS config (SSL Labs API),
         exposed files (HTTP probes), JS secrets (static analysis)</p>
    </li>
    <li>
      <h3>Results in 60 seconds</h3>
      <p>A-F grade with severity-prioritized findings.
         Paid tier adds Nuclei-based framework checks.</p>
    </li>
  </ol>

  <details>
    <summary>Technical methodology</summary>
    <ul>
      <li>Security headers: HTTP response header analysis (passive)</li>
      <li>TLS: Qualys SSL Labs API (external service, cached)</li>
      <li>Exposed files: Probe common paths (.env, .git, /debug, etc.)</li>
      <li>JS secrets: Regex pattern matching on bundled JavaScript</li>
      <li>Vibe-code (paid): Nuclei with custom templates for framework-specific issues</li>
    </ul>
  </details>
</section>
```
**Sources:**
- [Evil Martians: 100 dev tool landing pages study 2025](https://evilmartians.com/chronicles/we-studied-100-devtool-landing-pages-here-is-what-actually-works-in-2025)
- [Security scanner methodology](https://www.hackerone.com/knowledge-center/website-security-scans-process-and-tips-effective-scanning)

### Pattern 3: Open-Source Attribution in Footer
**What:** Credits for open-source tools used, with links to projects and licenses
**When to use:** Footer component, visible on all pages
**Example:**
```tsx
// Best practice: Title, Author, Source, License
<footer>
  <nav>{/* Legal links */}</nav>
  <p>&copy; {year} ShipSecure</p>

  <div className="text-xs text-gray-500">
    <p>Powered by open source:</p>
    <ul>
      <li>
        <a href="https://github.com/projectdiscovery/nuclei">Nuclei</a> by{' '}
        <a href="https://projectdiscovery.io">ProjectDiscovery</a> (
        <a href="https://github.com/projectdiscovery/nuclei/blob/main/LICENSE.md">
          MIT License
        </a>
        )
      </li>
      <li>
        <a href="https://testssl.sh">testssl.sh</a> (
        <a href="https://github.com/testssl/testssl.sh/blob/3.3dev/LICENSE">
          GPLv2
        </a>
        )
      </li>
    </ul>
  </div>
</footer>
```
**Sources:**
- [Best practices for OSS attribution](https://aboutcode.org/2015/oss-attribution-best-practices/)
- [Nuclei MIT License requirements](https://github.com/projectdiscovery/nuclei/blob/main/LICENSE.md)
- [testssl.sh attribution encouragement](https://testssl.sh/)

### Pattern 4: Technically Honest Value Props
**What:** Use factual, measurable claims instead of superlatives
**When to use:** Subheadings, feature descriptions, CTA copy
**Example:**
```tsx
// BAD (vague, marketing)
<p>Industry-leading protection for modern applications</p>
<p>Eliminate security risks instantly with AI-powered scanning</p>

// GOOD (specific, honest)
<p>Detects 14 vulnerability types including exposed .env files,
   weak TLS ciphers, and hardcoded API keys</p>
<p>Results in ~60 seconds. Free tier requires email only, no signup.</p>
```
**Source:** [Developer landing page copywriting](https://www.markepear.dev/blog/developer-marketing-hacker-news) - developers trust other developers, speak in technical language they use

### Anti-Patterns to Avoid
- **Marketing superlatives:** "Revolutionary", "game-changing", "10x faster", "enterprise-grade" without evidence
- **Vague benefits:** "Improve security" → be specific: "Detect exposed .env files and weak CSP headers"
- **Hidden methodology:** Black-box scanning erodes trust. Explain what you check and how.
- **Long-winded copy:** Developers skim. Shorter landing pages convert better ([Gartner copywriting research](https://www.gartner.com/en/digital-markets/insights/landing-page-copywriting))
- **Jargon mismatch:** Avoid MBA-speak ("synergy", "leverage") when targeting IC developers

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Open-source license compliance | Custom attribution parser | Manual footer credits with links | Small number of dependencies, manual is clearer |
| Copy A/B testing | Custom analytics split testing | Plausible + qualitative HN feedback | MVP launch, qualitative > quantitative initially |
| Content management | Headless CMS for 3 pages | Inline copy in Next.js components | No i18n, no marketing team, no content velocity |
| "How it works" diagrams | Custom SVG animations | Static text with `<details>` for depth | Faster to ship, accessibility better, mobile-friendly |

**Key insight:** Landing page optimization is copywriting, not engineering. Don't over-engineer. The value is in **clear communication**, not fancy components.

## Common Pitfalls

### Pitfall 1: Marketing Jargon Alienates Developer Audience
**What goes wrong:** Using phrases like "revolutionary AI-powered security platform" or "ship 10x faster" on a landing page targeting HN developers
**Why it happens:** Copywriters trained on B2B SaaS conventions bring MBA-speak to dev tools
**How to avoid:** Read successful HN launches. Use technical terminology. Avoid superlatives. Test copy by asking: "Would I upvote this on HN or dismiss it as marketing fluff?"
**Warning signs:** Words like "revolutionary", "game-changing", "enterprise-grade", "synergy", "leverage", "10x", "unlock", "empower"
**Source:** [How to market on Hacker News](https://www.markepear.dev/blog/developer-marketing-hacker-news)

### Pitfall 2: Vague "How It Works" Loses Credibility
**What goes wrong:** Security scanner claims to "check for vulnerabilities" without explaining methodology, what it scans, or how it scans
**Why it happens:** Fear of exposing limitations or competitors copying approach
**How to avoid:** Transparency builds trust. Document scan types (passive headers, active probes, external API calls). List specific checks (.env, .git, CSP headers). Admit what you DON'T check.
**Warning signs:** No mention of scan methodology, no list of specific vulnerability types, claims to "check everything"
**Source:** [Website security scanning methodology](https://www.hackerone.com/knowledge-center/website-security-scans-process-and-tips-effective-scanning)

### Pitfall 3: Missing or Vague OSS Attribution
**What goes wrong:** Using Nuclei or testssl.sh without crediting them, or burying credits in /about page
**Why it happens:** Founders fear users will bypass product and use open-source tools directly
**How to avoid:** Add footer attribution with Title, Author, Source, License links. For services using testssl.sh, author "strongly encourages" public acknowledgement. MIT (Nuclei) requires license notice.
**Warning signs:** No mention of underlying tools, vague "powered by industry-standard scanners"
**Source:** [OSS attribution best practices](https://aboutcode.org/2015/oss-attribution-best-practices/), [testssl.sh license](https://testssl.sh/)

### Pitfall 4: Headline Doesn't Communicate Product or Audience
**What goes wrong:** Headlines like "Ship fast, stay safe" don't tell you what the product does or who it's for
**Why it happens:** Copywriting advice to "focus on benefits not features" taken too literally
**How to avoid:** Headline must answer: What does this do? Who is it for? Example: "Security scanning for AI-generated web apps" (what + who). Subhead can add benefit: "Free vulnerability detection for vibe-coded projects"
**Warning signs:** Headline is a vague tagline, no mention of product category, unclear target audience
**Source:** [Landing page copywriting principles](https://www.getresponse.com/blog/copywriting-landing-page-conversions)

### Pitfall 5: Over-Engineering "How It Works" Section
**What goes wrong:** Building interactive diagrams, animated SVGs, or video explainers before testing if copy alone converts
**Why it happens:** Engineers default to building features instead of writing clear copy
**How to avoid:** Start with text. Use `<details>` tags for progressive disclosure. Add visuals only if user feedback requests them. Ship faster with simpler implementation.
**Warning signs:** Scope includes "animated flow diagram", "interactive scanner visualization", multi-week estimate for copy changes

## Code Examples

Verified patterns from current codebase and research sources:

### Current Landing Page (Before Optimization)
```tsx
// frontend/app/page.tsx - current state
<h1>Ship fast, stay safe.</h1>
<p>Free security scanning for vibe-coded web apps.</p>
<p className="text-sm">Catch security flaws before they become breaches.</p>
```
**Issues:**
- Headline doesn't communicate what product does ("ship fast, stay safe" is vague)
- No mention of target audience (developers using AI code generation)
- Missing "how it works" methodology transparency
- No open-source attribution in footer

### Recommended Landing Page Hero (Developer-Focused)
```tsx
// Improved version - developer-focused, technically honest
<h1>Security scanning for AI-generated web apps</h1>
<p>
  Free vulnerability detection for vibe-coded projects.
  Detects exposed .env files, weak TLS ciphers, hardcoded API keys,
  and framework-specific misconfigurations.
</p>
<p className="text-sm text-gray-500">
  No signup required. Results in ~60 seconds.
</p>
```
**Improvements:**
- Headline states product category and target audience
- Subhead lists specific vulnerability types (technical detail)
- Small print emphasizes friction-free access (developer-friendly)

### "How It Works" Section Implementation
```tsx
// Add to frontend/app/page.tsx below scan form
<section className="mb-12">
  <h2 className="text-2xl font-semibold mb-6 text-center">How it works</h2>

  <div className="grid md:grid-cols-3 gap-6 mb-6">
    <div>
      <div className="text-blue-600 text-3xl mb-2">1</div>
      <h3 className="font-semibold mb-2">Submit URL</h3>
      <p className="text-sm text-gray-600">
        No signup required. Scans start immediately.
      </p>
    </div>

    <div>
      <div className="text-blue-600 text-3xl mb-2">2</div>
      <h3 className="font-semibold mb-2">Multi-Scanner Analysis</h3>
      <p className="text-sm text-gray-600">
        Security headers, TLS config (SSL Labs API), exposed files,
        JS secrets. Paid tier adds Nuclei-based framework checks.
      </p>
    </div>

    <div>
      <div className="text-blue-600 text-3xl mb-2">3</div>
      <h3 className="font-semibold mb-2">Results in ~60s</h3>
      <p className="text-sm text-gray-600">
        A-F grade with severity-prioritized findings and
        copy-paste remediation steps.
      </p>
    </div>
  </div>

  <details className="text-sm text-gray-600">
    <summary className="cursor-pointer hover:text-blue-600">
      Technical methodology
    </summary>
    <ul className="mt-4 space-y-2 ml-6 list-disc">
      <li><strong>Security headers:</strong> HTTP response header analysis (passive check for CSP, HSTS, X-Frame-Options, etc.)</li>
      <li><strong>TLS configuration:</strong> Qualys SSL Labs API - certificate validity, cipher strength, protocol versions</li>
      <li><strong>Exposed files:</strong> HTTP probes for common sensitive paths (.env, .git, /debug, /admin, config files)</li>
      <li><strong>JavaScript secrets:</strong> Regex pattern matching on bundled JS for API keys, tokens, credentials</li>
      <li><strong>Vibe-code scanning (paid):</strong> Nuclei with custom templates for Supabase RLS, Firebase rules, framework-specific issues</li>
    </ul>
  </details>
</section>
```
**Pattern:** Step-by-step visual flow + progressive disclosure for technical depth

### Open-Source Attribution in Footer
```tsx
// frontend/components/footer.tsx - add OSS attribution
export function Footer() {
  const currentYear = new Date().getFullYear()

  return (
    <footer className="border-t border-gray-200 dark:border-gray-800 py-8 mt-auto">
      <div className="container mx-auto px-4">
        {/* Existing legal links nav */}
        <nav className="flex flex-col sm:flex-row items-center justify-center gap-2 sm:gap-4 mb-3">
          <Link href="/privacy">Privacy Policy</Link>
          <span>•</span>
          <Link href="/terms">Terms of Service</Link>
        </nav>

        {/* Existing copyright */}
        <p className="text-center text-sm text-gray-500 mb-4">
          &copy; {currentYear} ShipSecure. All rights reserved.
        </p>

        {/* NEW: Open-source attribution */}
        <div className="text-center text-xs text-gray-500 dark:text-gray-400">
          <p className="mb-2">Powered by open source:</p>
          <div className="flex flex-wrap justify-center gap-x-4 gap-y-1">
            <span>
              <a
                href="https://github.com/projectdiscovery/nuclei"
                className="hover:text-blue-600 underline"
                target="_blank"
                rel="noopener noreferrer"
              >
                Nuclei
              </a>
              {' '}by{' '}
              <a
                href="https://projectdiscovery.io"
                className="hover:text-blue-600"
                target="_blank"
                rel="noopener noreferrer"
              >
                ProjectDiscovery
              </a>
              {' '}(
              <a
                href="https://github.com/projectdiscovery/nuclei/blob/main/LICENSE.md"
                className="hover:text-blue-600"
                target="_blank"
                rel="noopener noreferrer"
              >
                MIT
              </a>
              )
            </span>
            <span>•</span>
            <span>
              <a
                href="https://testssl.sh"
                className="hover:text-blue-600 underline"
                target="_blank"
                rel="noopener noreferrer"
              >
                testssl.sh
              </a>
              {' '}(
              <a
                href="https://github.com/testssl/testssl.sh/blob/3.3dev/LICENSE"
                className="hover:text-blue-600"
                target="_blank"
                rel="noopener noreferrer"
              >
                GPLv2
              </a>
              )
            </span>
          </div>
        </div>
      </div>
    </footer>
  )
}
```
**Pattern:** Title + Author + License links, footer placement, subtle styling

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Benefits-only landing pages | Technical honesty for dev tools | ~2020-2023 | Developer tools adopt plain language, avoid marketing jargon ([Stripe, Vercel examples](https://everydeveloper.com/developer-tool-homepages/)) |
| Black-box security scanners | Methodology transparency | ~2022-2025 | Users demand to know what/how you scan. "AI-powered" no longer sufficient ([HackerOne guidelines](https://www.hackerone.com/knowledge-center/website-security-scans-process-and-tips-effective-scanning)) |
| Separate credits page | Footer attribution | Ongoing | OSS communities expect visible attribution, not buried /credits page ([AboutCode best practices](https://aboutcode.org/2015/oss-attribution-best-practices/)) |
| Generic vulnerability scanners | AI/vibe-code specific tools | 2025-2026 | 92% of devs use AI tools, 41% of code AI-generated → new security niche ([vibe coding statistics](https://www.secondtalent.com/resources/vibe-coding-statistics/)) |

**Deprecated/outdated:**
- **Third-party SEO libs (next-seo):** Next.js 13+ App Router has built-in Metadata API, no library needed (Phase 9 research)
- **Marketing jargon for dev tools:** HN 2024-2026 launches show technical honesty outperforms benefit-focused copy
- **Hiding scan methodology:** Post-2023 trend toward transparency in security tools

## Vibe Coding Market Context (2026)

**Market size & adoption:**
- Vibe coding market: $4.7B globally (2026), projected $12.3B by 2027 (38% CAGR)
- 92% of US developers use AI coding tools daily
- 41% of all code written globally is AI-generated
- Primary tools: Cursor, Bolt, Lovable, v0, GitHub Copilot

**Security concerns (ShipSecure's value prop):**
- 24.7% of AI-generated code has security flaws
- AI co-authored code: 1.7x more major issues vs human-written
- Logic errors 75% more common, security vulnerabilities 2.74x higher
- CVE-2025-48757: 170+ Lovable apps exposed PII/API keys via RLS misconfiguration

**Developer sentiment:**
- Developers value **understanding and verifying** AI suggestions
- Best practices: mandatory code review, security scanning protocols
- Vibe coding "accelerates development but doesn't replace technical understanding"

**Implications for landing page copy:**
1. **Target audience clarity:** Developers using Cursor, Bolt, Lovable (vibe-coded apps)
2. **Value prop:** Catch the 24.7% of AI code with security flaws
3. **Messaging tone:** Respect developer intelligence. Don't imply AI code is "bad", position as validation step for fast shipping
4. **Social proof angle:** Reference CVE-2025-48757 Lovable incident as real-world example

**Sources:**
- [Vibe coding statistics 2026](https://www.secondtalent.com/resources/vibe-coding-statistics/)
- [Security risks in vibe-coded apps](https://dev.to/devin-rosario/how-to-secure-vibe-coded-applications-in-2026-208d)
- [What is vibe coding - complete guide 2026](https://natively.dev/articles/what-is-vibe-coding)

## Open Questions

1. **Should we mention specific CVEs (CVE-2025-48757 Lovable RLS bug)?**
   - What we know: Real incident, 170+ apps exposed, public information
   - What's unclear: Does name-dropping competitors on landing page alienate vs build credibility?
   - Recommendation: Consider for blog post / HN launch post, avoid on landing page (focus on problem not blame)

2. **How detailed should "technical methodology" disclosure be?**
   - What we know: Transparency builds trust, HN values technical depth
   - What's unclear: Balance between "enough to trust" vs "TMI for landing page"
   - Recommendation: Use `<details>` progressive disclosure. Basic methodology visible, deep technical details click-to-expand

3. **Should OSS attribution link to GitHub repos or official websites?**
   - What we know: Both are valid, GitHub shows activity/stars, official site shows project polish
   - What's unclear: User behavior - do they want source code or project info?
   - Recommendation: Link tool name to official site, license to GitHub (best of both)

4. **Do we need to attribute SSL Labs / Qualys?**
   - What we know: TLS scanner uses Qualys SSL Labs API
   - What's unclear: API usage vs embedding software - does usage require attribution?
   - Recommendation: Mention in methodology ("via SSL Labs API"), not footer (not OSS, it's a service)

## Sources

### Primary (HIGH confidence)
- **Next.js 16.1.6 documentation** - App Router, Metadata API (existing stack from Phase 9 research)
- [Nuclei MIT License](https://github.com/projectdiscovery/nuclei/blob/main/LICENSE.md) - MIT license requirements, copyright 2025 ProjectDiscovery
- [testssl.sh](https://testssl.sh/) and [testssl.sh LICENSE](https://github.com/testssl/testssl.sh/blob/3.3dev/LICENSE) - GPLv2, attribution encouraged for services
- [Best Practices for OSS Attribution - AboutCode](https://aboutcode.org/2015/oss-attribution-best-practices/) - Title, Author, Source, License standard

### Secondary (MEDIUM confidence)
- [How to market on Hacker News - Markepear](https://www.markepear.dev/blog/developer-marketing-hacker-news) - Tailscale case study, technical jargon over marketing
- [We studied 100 dev tool landing pages - Evil Martians](https://evilmartians.com/chronicles/we-studied-100-devtool-landing-pages-here-is-what-actually-works-in-2025) - "How it works" almost mandatory, avoid salesy language
- [Website Security Scans - HackerOne](https://www.hackerone.com/knowledge-center/website-security-scans-process-and-tips-effective-scanning) - Methodology transparency, passive vs active checks
- [9 Landing Page Copywriting Principles - GetResponse](https://www.getresponse.com/blog/copywriting-landing-page-conversions) - Clarity over jargon, benefits + features balance
- [Vibe Coding Statistics 2026 - Second Talent](https://www.secondtalent.com/resources/vibe-coding-statistics/) - Market size, adoption rates, security concerns
- [How to Secure Vibe Coded Applications - DEV Community](https://dev.to/devin-rosario/how-to-secure-vibe-coded-applications-in-2026-208d) - Security flaw statistics, CVE-2025-48757 details
- [What is Vibe Coding - Natively](https://natively.dev/articles/what-is-vibe-coding) - Definition, developer workflow, market context

### Tertiary (LOW confidence)
- [Landing page examples - various sources](https://www.lapa.ninja/category/development-tools/) - Visual inspiration only, not prescriptive guidance
- Search results on "developer tool homepages" - Generic best practices, not ShipSecure-specific

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** - No new dependencies, existing Next.js + Tailwind
- Architecture: **HIGH** - Research-backed patterns from Evil Martians study, HN marketing analysis, OSS attribution standards
- Pitfalls: **MEDIUM-HIGH** - Based on multiple credible sources (Markepear, Evil Martians, AboutCode) but some subjective (HN cultural norms)
- Vibe coding context: **MEDIUM** - Statistics from aggregator sites (Second Talent, Natively), not primary research
- OSS license requirements: **HIGH** - Direct from official GitHub repos and license files

**Research date:** 2026-02-09
**Valid until:** ~60 days (stable domain - copywriting best practices, OSS licensing unchanging)

**Key uncertainties:**
- Optimal headline phrasing (needs A/B testing or HN feedback)
- Depth of methodology disclosure (progressive disclosure mitigates)
- CVE name-dropping appropriateness (recommend blog post not landing page)
