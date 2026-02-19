# Phase 08: Analytics & Tracking - Research

**Researched:** 2026-02-08
**Domain:** Privacy-friendly web analytics for Next.js applications
**Confidence:** HIGH

## Summary

Phase 8 implements privacy-friendly analytics to track pageviews and conversion events (scan submission, paid audit purchase) without compromising user privacy or performance. Two main options exist: Plausible Analytics (managed SaaS) and Umami Analytics (self-hosted). Both are GDPR-compliant, cookie-free, and designed to avoid consent banner requirements, though legal interpretation varies by jurisdiction.

**Primary recommendation:** Start with Plausible Cloud ($9/month) for immediate deployment with zero maintenance overhead. Plausible offers a managed proxy on Enterprise plans to bypass ad blockers (6-26% of traffic blocked), but next-plausible library with withPlausibleProxy() provides self-service proxying for standard plans. Umami self-hosted is viable on existing DigitalOcean infrastructure ($0 additional cost beyond current droplet resources) but requires ongoing maintenance and won't scale as efficiently as Plausible's ClickHouse backend.

For ShipSecure's MVP stage, Plausible Cloud optimizes for speed-to-market and reliability. Migrate to Umami self-hosted only if monthly costs exceed $19/month (20K pageviews) or data sovereignty becomes a hard requirement.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| next-plausible | 3.x | Next.js integration for Plausible | Official recommendation in Plausible docs, provides PlausibleProvider and usePlausible hook |
| Plausible Analytics | Cloud SaaS | Privacy-friendly analytics platform | 16,000+ paying subscribers, EU-hosted, GDPR-compliant out-of-box, no cookies |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @plausible/tracker | Latest | Standalone JS tracker | If not using React/Next.js or need manual pageview control |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Plausible Cloud ($9/mo) | Umami self-hosted (free) | Umami: no monthly cost, full data control, but requires maintenance, PostgreSQL storage (less efficient at scale than ClickHouse), limited support |
| Plausible Cloud | Plausible self-hosted | Self-hosted: free (AGPL license), full control, but requires 2GB+ RAM for ClickHouse, ongoing maintenance, no managed proxy, feature lag |
| next-plausible | Manual Script tag | Manual: more control over loading, but no type-safe hooks, no automatic proxy setup, more error-prone |

**Installation:**
```bash
npm install next-plausible
```

## Architecture Patterns

### Recommended Project Structure
Integration is minimal - analytics wraps existing application structure:

```
frontend/
├── app/
│   └── layout.tsx              # PlausibleProvider wraps app (App Router)
├── components/
│   └── analytics/
│       └── ConversionEvents.tsx  # Reusable event tracking hooks
└── lib/
    └── analytics.ts            # Event tracking utilities
```

For Pages Router:
```
frontend/
├── pages/
│   └── _app.tsx               # PlausibleProvider wraps app
```

### Pattern 1: Provider-Based Integration (Recommended)
**What:** Wrap the entire Next.js application with PlausibleProvider at the root layout/app level
**When to use:** All implementations - this is the standard approach
**Example:**
```typescript
// Source: https://github.com/4lejandrito/next-plausible
// app/layout.tsx (App Router)
import PlausibleProvider from 'next-plausible'

export default function RootLayout({ children }) {
  return (
    <html>
      <head>
        <PlausibleProvider domain="shipsecure.ai" />
      </head>
      <body>{children}</body>
    </html>
  )
}
```

### Pattern 2: Custom Event Tracking with usePlausible Hook
**What:** Use type-safe hook to trigger custom events with properties
**When to use:** Conversion tracking (scan submission, purchase completion)
**Example:**
```typescript
// Source: https://plausible.io/docs/custom-event-goals
import { usePlausible } from 'next-plausible'

export function ScanForm() {
  const plausible = usePlausible()

  const handleSubmit = async (url: string) => {
    // Track scan submission
    plausible('Scan Submitted', { props: { scanType: 'URL' } })

    // Submit scan...
  }

  return <form onSubmit={handleSubmit}>...</form>
}
```

### Pattern 3: Ad Blocker Bypass with Proxy
**What:** Configure Next.js to proxy analytics requests through your domain
**When to use:** When ad blocker blocking rate is significant (>10%)
**Example:**
```javascript
// Source: https://github.com/4lejandrito/next-plausible
// next.config.js
const { withPlausibleProxy } = require('next-plausible')

module.exports = withPlausibleProxy()({
  // Your Next.js config
})
```

### Pattern 4: Performance-Optimized Loading
**What:** Use Next.js Script component's default strategy (afterInteractive)
**When to use:** Always - next-plausible handles this automatically
**Note:** PlausibleProvider automatically uses afterInteractive strategy, loading script after page becomes interactive but before full hydration. Script is <1KB and won't block rendering.

### Anti-Patterns to Avoid
- **Hardcoding script tags in <head>:** Use PlausibleProvider instead - it handles deferred loading, proxy setup, and TypeScript types
- **Tracking PII in custom properties:** NEVER pass email, IP addresses, or user IDs - breaks privacy guarantee
- **Pre-ticked consent boxes:** If you add a consent banner (not required for Plausible/Umami), all toggles must default to OFF per GDPR 2026 standards
- **Inconsistent event naming:** Use lowercase, underscores, max 40 chars. Define naming convention upfront (e.g., `scan_submitted`, `audit_purchased`)
- **Duplicate tracking scripts:** Ensure only ONE analytics script loads - common mistake when migrating or using multiple tools

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Pageview tracking | Custom pageview counter in backend | Plausible/Umami automatic tracking | Handles SPAs, back/forward navigation, deduplication, referrer parsing, device detection |
| Event tracking | Custom event logging to database | Plausible custom events | Pre-built dashboard, funnel analysis, conversion tracking, no schema design |
| Ad blocker detection | Custom detection scripts | Proxying with withPlausibleProxy() | Maintains privacy compliance while bypassing blockers |
| GDPR compliance | Custom anonymization logic | Plausible/Umami built-in privacy | Legal review, daily-rotating hashes, no PII storage, compliant by design |
| Analytics dashboard | Admin panel with charts | Plausible/Umami web interface | Real-time updates, filtering, export, team sharing |

**Key insight:** Analytics is a solved problem with privacy-first solutions. Building custom analytics seems simple but requires handling session management, bot filtering, privacy compliance, data retention policies, and dashboard UX. Privacy-friendly tools eliminate this complexity while maintaining legal compliance.

## Common Pitfalls

### Pitfall 1: Not Setting Up Goals in Dashboard
**What goes wrong:** Custom events fire from your code but don't appear in analytics
**Why it happens:** Plausible/Umami require explicit goal creation in dashboard with exact name match
**How to avoid:** After adding event tracking code, immediately create matching goals in dashboard Settings > Goals
**Warning signs:** Events logged in browser console but not showing in dashboard; "No data" for conversion funnels

### Pitfall 2: Blocking Page Rendering with Analytics
**What goes wrong:** Analytics script loaded with `beforeInteractive` strategy causes delayed LCP and lower Lighthouse scores
**Why it happens:** Misunderstanding Next.js Script loading strategies; manually adding script to <head> without defer
**How to avoid:** Use PlausibleProvider which automatically uses `afterInteractive` strategy; never use `beforeInteractive` for analytics
**Warning signs:** Lighthouse Performance score drops when analytics added; Total Blocking Time (TBT) increases

### Pitfall 3: Inconsistent Event Naming Across Team
**What goes wrong:** Dashboard shows `Scan_Submitted`, `scanSubmitted`, `scan-submitted`, `SCAN_SUBMITTED` as separate events
**Why it happens:** No naming convention documented; multiple developers implementing tracking
**How to avoid:** Define naming standard upfront (recommend: lowercase with underscores, verb_past_tense format)
**Warning signs:** Duplicate-looking events in dashboard; conversion rates don't match expected values

### Pitfall 4: Assuming No Consent Banner Needed Without Legal Review
**What goes wrong:** GDPR/ePrivacy violation fines despite using "cookie-free" analytics
**Why it happens:** Plausible/Umami don't use cookies, but ePrivacy Directive has nuances; interpretation varies by jurisdiction
**How to avoid:** While Plausible/Umami are designed to avoid consent requirements, consult data protection lawyer for EU traffic; document legal basis
**Warning signs:** Privacy Policy doesn't mention analytics; no legal review of tracking implementation

### Pitfall 5: Not Accounting for Ad Blocker Traffic Loss
**What goes wrong:** Analytics show 30% less traffic than expected; decision-making based on incomplete data
**Why it happens:** 6-26% of visitors block analytics scripts (varies by audience technical sophistication)
**How to avoid:** Implement proxy from day one using withPlausibleProxy(); monitor discrepancy between server logs and analytics
**Warning signs:** Analytics traffic significantly lower than Nginx access logs; tech-heavy audiences show extreme drop-off

### Pitfall 6: Tracking Too Many Events (Over-Tracking)
**What goes wrong:** Dashboard becomes unusable; impossible to find meaningful metrics in noise
**Why it happens:** Tracking every button click, hover, scroll; lack of strategy about what matters
**How to avoid:** Limit to conversion-critical events: `scan_submitted`, `audit_purchased`, optionally `scan_completed`, `result_viewed`
**Warning signs:** Goals list has 20+ items; team asks "which event tracks X?"; funnel analysis shows 10+ steps

### Pitfall 7: Not Testing Events in Development
**What goes wrong:** Events fire in production with wrong names or properties; can't fix without code deploy
**Why it happens:** Plausible/Umami scripts don't load on localhost by default
**How to avoid:** Enable localhost tracking with `<PlausibleProvider trackLocalhost={true}>` during development; verify in browser Network tab
**Warning signs:** Events working in production but team never saw them fire during testing

## Code Examples

Verified patterns from official sources:

### App Router Integration
```typescript
// Source: https://github.com/4lejandrito/next-plausible
// app/layout.tsx
import PlausibleProvider from 'next-plausible'

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <head>
        <PlausibleProvider
          domain="shipsecure.ai"
          trackOutboundLinks
        />
      </head>
      <body>{children}</body>
    </html>
  )
}
```

### Pages Router Integration
```typescript
// Source: https://github.com/4lejandrito/next-plausible
// pages/_app.tsx
import PlausibleProvider from 'next-plausible'
import type { AppProps } from 'next/app'

export default function MyApp({ Component, pageProps }: AppProps) {
  return (
    <PlausibleProvider domain="shipsecure.ai">
      <Component {...pageProps} />
    </PlausibleProvider>
  )
}
```

### Custom Event Tracking
```typescript
// Source: https://plausible.io/docs/custom-event-goals
'use client'

import { usePlausible } from 'next-plausible'

export function ScanSubmitButton({ url }: { url: string }) {
  const plausible = usePlausible()

  const handleScan = async () => {
    // Track conversion event
    plausible('Scan Submitted', {
      props: {
        scanType: 'URL',
        urlLength: url.length.toString()
      }
    })

    // Submit scan request...
  }

  return (
    <button onClick={handleScan}>
      Scan Now
    </button>
  )
}
```

### Purchase Conversion Tracking
```typescript
// Source: https://plausible.io/docs/custom-event-goals
'use client'

import { usePlausible } from 'next-plausible'

export function CheckoutSuccess({ amount }: { amount: number }) {
  const plausible = usePlausible()

  useEffect(() => {
    // Track purchase conversion with revenue
    plausible('Audit Purchased', {
      props: {
        value: amount.toString(),
        currency: 'USD'
      }
    })
  }, [])

  return <div>Thank you for your purchase!</div>
}
```

### Ad Blocker Bypass Configuration
```javascript
// Source: https://github.com/4lejandrito/next-plausible
// next.config.mjs
import { withPlausibleProxy } from 'next-plausible'

export default withPlausibleProxy()({
  // Proxies requests to /api/_plausible/* to plausible.io
  // Your other Next.js config
})
```

### Localhost Development Testing
```typescript
// Source: https://github.com/4lejandrito/next-plausible
// app/layout.tsx (development)
import PlausibleProvider from 'next-plausible'

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <head>
        <PlausibleProvider
          domain="shipsecure.ai"
          trackLocalhost={process.env.NODE_ENV === 'development'}
        />
      </head>
      <body>{children}</body>
    </html>
  )
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Google Analytics with cookies | Privacy-first, cookie-free analytics (Plausible/Umami) | 2020-2024 | No consent banners needed, GDPR-compliant by default, simpler UX |
| Manual script tags in <head> | next-plausible provider + hooks | 2021-present | Type-safe events, automatic proxy setup, cleaner code |
| beforeInteractive script loading | afterInteractive (default in next-plausible) | Next.js 11+ (2021) | Zero impact on Lighthouse Performance scores |
| Accept/Reject consent buttons with visual bias | Equal visual weight required | Austria 2025 ruling | Colored Accept + gray Reject now violates GDPR parity |
| Pre-selected consent checkboxes | All toggles OFF by default | GDPR enforcement 2026 | Active opt-in required, not passive acceptance |

**Deprecated/outdated:**
- **Google Analytics Universal (UA):** Sunset July 2023, replaced by GA4. Privacy-first alternatives like Plausible/Umami preferred for new projects
- **Cookie consent "legitimate interest" for analytics:** GDPR interpretation tightened - consent now required for most analytics unless truly anonymous
- **Manual pageview tracking in SPAs:** Modern tools auto-detect client-side navigation; manual tracking causes double-counting

## Open Questions

1. **Self-hosted Umami vs Plausible Cloud: TCO at scale**
   - What we know: Umami free on current DigitalOcean droplet (2GB RAM available), Plausible $9/mo (10K views) → $19/mo (20K views)
   - What's unclear: At what traffic level does ClickHouse RAM requirement make Plausible self-hosted more expensive than cloud?
   - Recommendation: Start with Plausible Cloud for MVP. Re-evaluate if hitting $29/mo (50K pageviews) or data sovereignty requirement emerges

2. **Legal requirement for consent banner in practice**
   - What we know: Plausible/Umami designed to avoid consent requirements (no cookies, no PII); legal assessment says GDPR-compliant but ePrivacy nuanced
   - What's unclear: ShipSecure legal structure (US company? EU entity?), jurisdiction of majority traffic, risk tolerance for EU fines
   - Recommendation: Document in Privacy Policy that analytics are anonymous and cookie-free. Consult lawyer before EU expansion

3. **Ad blocker proxy necessity for MVP**
   - What we know: 6-26% traffic blocked without proxy; withPlausibleProxy() available for free; Managed Proxy requires Enterprise plan
   - What's unclear: ShipSecure audience technical sophistication (security-focused users likely higher block rate)
   - Recommendation: Implement withPlausibleProxy() from day one - zero cost, minimal config, prevents data loss

4. **Custom event property constraints**
   - What we know: Plausible custom events support props; docs show examples with method, currency, value
   - What's unclear: Limits on property count, key length, value length, cardinality impact on dashboard performance
   - Recommendation: Keep properties minimal (2-3 per event), use predefined values (e.g., scanType: 'URL'), test in dashboard before production

## Sources

### Primary (HIGH confidence)
- Plausible Next.js integration: https://plausible.io/docs/nextjs-integration
- Plausible custom events: https://plausible.io/docs/custom-event-goals
- next-plausible library: https://github.com/4lejandrito/next-plausible
- Plausible GDPR compliance: https://plausible.io/data-policy
- Plausible legal assessment: https://plausible.io/blog/legal-assessment-gdpr-eprivacy
- Plausible pricing: https://plausible.io/docs/subscription-plans

### Secondary (MEDIUM confidence)
- Umami documentation: https://umami.is/docs
- Umami Next.js integration: https://github.com/kdcokenny/next-umami
- DigitalOcean Umami setup: https://www.digitalocean.com/community/tutorials/how-to-install-umami-web-analytics-software-on-ubuntu-20-04
- Self-hosting Umami with Docker Compose: https://www.paulsblog.dev/self-host-umami-analytics-with-docker-compose
- Next.js Script optimization: https://developer.chrome.com/blog/script-component
- Plausible proxy setup: https://plausible.io/docs/proxy/introduction
- GDPR cookie consent 2026 requirements: https://secureprivacy.ai/blog/cookie-consent-implementation

### Tertiary (LOW confidence - WebSearch only)
- Plausible vs Umami comparison: https://vemetric.com/blog/plausible-vs-umami
- Analytics performance impact: https://dev.to/lovestaco/how-i-fixed-lighthouse-score-drops-caused-by-google-tag-manager-analytics-2anm
- Conversion funnel best practices: https://improvado.io/blog/conversion-funnel

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official documentation, active maintenance, 16K+ Plausible users, clear Next.js integration path
- Architecture: HIGH - next-plausible is official recommendation, patterns verified from source code and docs
- Pitfalls: MEDIUM-HIGH - Ad blocker blocking rates from official Plausible docs (HIGH), GDPR nuances from legal assessment (MEDIUM), event tracking mistakes from industry best practices (MEDIUM)
- Performance: HIGH - Next.js Script component behavior documented, next-plausible uses afterInteractive by default
- Legal compliance: MEDIUM - Plausible has independent legal assessment for GDPR/ePrivacy, but interpretation varies by jurisdiction and requires lawyer consultation

**Research date:** 2026-02-08
**Valid until:** 2026-03-08 (30 days - analytics space is stable, GDPR requirements evolving slowly)
