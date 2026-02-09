# Architecture Research: Launch Readiness Features

**Domain:** SaaS Security Scanner - Launch Readiness Integration
**Researched:** 2026-02-08
**Confidence:** HIGH

## Standard Architecture for Launch Features

### System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Client Browser                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
│  │ Analytics    │  │ Meta Tags    │  │ JSON-LD      │                  │
│  │ Script       │  │ (OG/SEO)     │  │ Schema       │                  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                  │
│         │                  │                  │                          │
├─────────┼──────────────────┼──────────────────┼──────────────────────────┤
│         │        Next.js Frontend (Port 3001)                            │
│         ↓                  ↓                  ↓                          │
│  ┌──────────────────────────────────────────────────────────────┐       │
│  │  app/layout.tsx                                              │       │
│  │  - <Script> component (analytics)                            │       │
│  │  - generateMetadata() (SEO/OG tags)                          │       │
│  │  - <script type="application/ld+json"> (JSON-LD)             │       │
│  └──────────────────────────────────────────────────────────────┘       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
│  │ app/         │  │ app/privacy  │  │ app/terms    │                  │
│  │ page.tsx     │  │ page.tsx     │  │ page.tsx     │                  │
│  └──────────────┘  └──────────────┘  └──────────────┘                  │
│  ┌────────────────────────────────────────────────────┐                 │
│  │  app/globals.css (Tailwind responsive styles)      │                 │
│  │  - Mobile-first breakpoints (sm:, md:, lg:, xl:)   │                 │
│  │  - Dark mode support (dark:)                       │                 │
│  └────────────────────────────────────────────────────┘                 │
│  ┌────────────────────────────────────────────────────┐                 │
│  │  components/                                        │                 │
│  │  - Improved loading states (Suspense/skeleton UI)  │                 │
│  │  - Error boundaries (error.tsx per route)          │                 │
│  └────────────────────────────────────────────────────┘                 │
├─────────────────────────────────────────────────────────────────────────┤
│                      Nginx Reverse Proxy                                │
│  - / → frontend:3001                                                    │
│  - /api/ → backend:3000                                                 │
│  - SSL termination (Let's Encrypt)                                      │
├─────────────────────────────────────────────────────────────────────────┤
│                   Rust/Axum Backend (Port 3000)                         │
│  - No changes required for launch features                              │
│  - Existing API routes continue to work                                 │
└─────────────────────────────────────────────────────────────────────────┘

External Services:
- Analytics: Plausible Cloud or Umami Self-hosted
- Legal: Static pages (no third-party service)
- Fonts: Google Fonts (already in use via next/font/google)
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| **Analytics Script** | Track page views and events without cookies | Next.js `<Script>` component with `strategy="afterInteractive"` |
| **SEO Metadata** | Provide title, description, OG tags for social sharing | `generateMetadata()` function in layout.tsx/page.tsx |
| **JSON-LD Schema** | Structured data for search engines and AI | `<script type="application/ld+json">` in layout.tsx |
| **Legal Pages** | Static privacy policy and terms of service | Next.js route pages (app/privacy/page.tsx, app/terms/page.tsx) |
| **Responsive CSS** | Mobile-first responsive design | Tailwind CSS breakpoints (already configured) |
| **Loading States** | Skeleton UI and loading feedback | React Suspense + loading.tsx files per route |
| **Error Handling** | Graceful error boundaries | error.tsx files per route + try-catch in components |

## Recommended Project Structure

### Current Structure (Existing)
```
frontend/
├── app/
│   ├── layout.tsx              # Root layout (MODIFY for analytics + metadata)
│   ├── page.tsx                # Landing page (MODIFY for JSON-LD)
│   ├── globals.css             # Global styles (REVIEW for responsive gaps)
│   ├── results/[token]/
│   │   └── page.tsx            # Results page (MODIFY for metadata)
│   ├── scan/[id]/
│   │   └── page.tsx            # Scan progress (ADD loading.tsx, error.tsx)
│   └── payment/success/
│       └── page.tsx            # Payment success (MODIFY for metadata)
├── components/
│   ├── scan-form.tsx           # (REVIEW for mobile UX)
│   ├── results-dashboard.tsx  # (REVIEW for mobile UX)
│   └── ...
├── lib/
│   └── types.ts
└── package.json
```

### New Structure (After Launch Features)
```
frontend/
├── app/
│   ├── layout.tsx              # ← MODIFIED: Analytics + base metadata
│   ├── page.tsx                # ← MODIFIED: JSON-LD schema
│   ├── globals.css             # ← MODIFIED: Enhanced responsive styles
│   ├── privacy/                # ← NEW: Legal page
│   │   └── page.tsx
│   ├── terms/                  # ← NEW: Legal page
│   │   └── page.tsx
│   ├── results/[token]/
│   │   └── page.tsx            # ← MODIFIED: Per-page metadata
│   ├── scan/[id]/
│   │   ├── page.tsx            # ← MODIFIED: Improved error handling
│   │   ├── loading.tsx         # ← NEW: Loading UI
│   │   └── error.tsx           # ← NEW: Error boundary
│   └── payment/success/
│       ├── page.tsx            # ← MODIFIED: Metadata
│       └── error.tsx           # ← NEW: Error boundary
├── components/
│   ├── analytics.tsx           # ← NEW: Analytics provider/wrapper (optional)
│   ├── json-ld.tsx             # ← NEW: Reusable JSON-LD component
│   ├── scan-form.tsx           # ← REVIEWED: Mobile-optimized
│   ├── results-dashboard.tsx  # ← REVIEWED: Mobile-optimized
│   └── ...
├── lib/
│   ├── types.ts
│   ├── metadata.ts             # ← NEW: Shared metadata utilities
│   └── schema.ts               # ← NEW: JSON-LD schema generators
└── package.json                # ← MODIFIED: Add next-plausible or analytics dep
```

### Structure Rationale

- **app/privacy/ and app/terms/**: Static route pages for legal content. No database or API required. SEO-friendly URLs (/privacy, /terms).
- **app/*/loading.tsx**: Next.js convention for route-level loading states. Automatically shown during navigation.
- **app/*/error.tsx**: Next.js convention for route-level error boundaries. Catches errors in that route segment.
- **components/analytics.tsx**: Optional abstraction for analytics provider (useful if switching between Plausible/Umami).
- **components/json-ld.tsx**: Reusable component for rendering JSON-LD scripts without hydration issues.
- **lib/metadata.ts**: Centralized metadata generation (avoid duplication across pages).
- **lib/schema.ts**: Centralized JSON-LD schema definitions (Organization, WebSite, SoftwareApplication).

## Architectural Patterns

### Pattern 1: Analytics Integration with Next.js Script Component

**What:** Use Next.js `<Script>` component to load analytics scripts with optimized loading strategy.

**When to use:** For any third-party script (analytics, chat widgets, etc.) that isn't critical for initial render.

**Trade-offs:**
- **Pros:** Optimized loading, defers script execution until appropriate time, prevents blocking page hydration
- **Cons:** Slightly more verbose than raw `<script>` tags, requires understanding of loading strategies

**Example:**
```typescript
// app/layout.tsx
import Script from 'next/script'

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <head>
        {/* Plausible Analytics */}
        <Script
          strategy="afterInteractive"
          data-domain="shipsecure.ai"
          src="https://plausible.io/js/script.js"
        />
      </head>
      <body>{children}</body>
    </html>
  )
}
```

**Alternatives:**
- **next-plausible library**: Provides `PlausibleProvider` wrapper with built-in proxying to bypass ad blockers
- **@next/third-parties**: Official Next.js library for Google Analytics, Google Tag Manager (but not Plausible/Umami)

**Recommendation:** Use next-plausible if choosing Plausible (provides proxy support). Use Script component directly for Umami.

### Pattern 2: Metadata Generation with generateMetadata()

**What:** Use Next.js App Router's `generateMetadata()` function to generate dynamic metadata per page.

**When to use:** For SEO meta tags, Open Graph tags, Twitter cards that need to be dynamic based on route parameters or data.

**Trade-offs:**
- **Pros:** Server-side rendering, automatic deduplication, type-safe with TypeScript, supports async data fetching
- **Cons:** Separate from static `metadata` export (can't mix), requires understanding of Next.js metadata API

**Example:**
```typescript
// app/results/[token]/page.tsx
import { Metadata } from 'next'

export async function generateMetadata({ params }): Promise<Metadata> {
  const { token } = params

  return {
    title: 'Security Scan Results - ShipSecure',
    description: 'Review your security scan findings and recommendations',
    openGraph: {
      title: 'Security Scan Results - ShipSecure',
      description: 'Free security scan completed',
      url: `https://shipsecure.ai/results/${token}`,
      type: 'website',
      images: [
        {
          url: 'https://shipsecure.ai/og-results.png',
          width: 1200,
          height: 630,
          alt: 'ShipSecure Security Scan Results',
        },
      ],
    },
    twitter: {
      card: 'summary_large_image',
      title: 'Security Scan Results - ShipSecure',
      description: 'Free security scan completed',
      images: ['https://shipsecure.ai/og-results.png'],
    },
  }
}
```

**Static Metadata Alternative:**
```typescript
// For pages without dynamic data
export const metadata: Metadata = {
  title: 'Privacy Policy - ShipSecure',
  description: 'Privacy policy for ShipSecure security scanning service',
}
```

**Shared Metadata Utility:**
```typescript
// lib/metadata.ts
export function createMetadata(page: string): Metadata {
  const titles = {
    home: 'ShipSecure - Security Scanning for Vibe-Coded Apps',
    privacy: 'Privacy Policy - ShipSecure',
    terms: 'Terms of Service - ShipSecure',
  }

  return {
    title: titles[page],
    description: 'Ship fast, stay safe. Free security scanning for AI-generated web applications.',
    openGraph: {
      siteName: 'ShipSecure',
      // ... shared config
    },
  }
}
```

### Pattern 3: JSON-LD Schema with Client Component Wrapper

**What:** Render JSON-LD structured data using a client component that prevents hydration mismatch.

**When to use:** For all pages where you want search engines to understand the content type (Organization, WebSite, SoftwareApplication, etc.).

**Trade-offs:**
- **Pros:** SEO benefits (rich results), AI/search engine understanding, no performance impact
- **Cons:** Requires client component wrapper to prevent React hydration issues, verbose JSON structure

**Example:**
```typescript
// components/json-ld.tsx
'use client'

import { useEffect } from 'react'

interface JsonLdProps {
  data: object
}

export function JsonLd({ data }: JsonLdProps) {
  useEffect(() => {
    // Only run on client to prevent hydration mismatch
  }, [])

  return (
    <script
      type="application/ld+json"
      dangerouslySetInnerHTML={{ __html: JSON.stringify(data) }}
    />
  )
}
```

**Usage:**
```typescript
// app/page.tsx
import { JsonLd } from '@/components/json-ld'

const schema = {
  '@context': 'https://schema.org',
  '@type': 'SoftwareApplication',
  name: 'ShipSecure',
  applicationCategory: 'SecurityApplication',
  offers: {
    '@type': 'Offer',
    price: '0',
    priceCurrency: 'USD',
  },
}

export default function Home() {
  return (
    <>
      <JsonLd data={schema} />
      {/* page content */}
    </>
  )
}
```

**Alternative (Simple):**
```typescript
// For static pages without hydration concerns
export default function Home() {
  return (
    <>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(schema) }}
      />
      {/* page content */}
    </>
  )
}
```

### Pattern 4: Loading States with Suspense and Skeleton UI

**What:** Use Next.js loading.tsx convention and React Suspense to show skeleton UI during data fetching.

**When to use:** For any page that fetches data server-side or client-side and needs loading feedback.

**Trade-offs:**
- **Pros:** Better UX, prevents layout shift, shows progress, reduces perceived loading time
- **Cons:** Requires designing skeleton UI, slightly more code

**Example:**
```typescript
// app/scan/[id]/loading.tsx
export default function Loading() {
  return (
    <div className="animate-pulse space-y-4">
      <div className="h-8 bg-gray-200 dark:bg-gray-700 rounded w-1/4"></div>
      <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded w-3/4"></div>
      <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded w-1/2"></div>
    </div>
  )
}
```

**Error Handling:**
```typescript
// app/scan/[id]/error.tsx
'use client'

export default function Error({
  error,
  reset,
}: {
  error: Error
  reset: () => void
}) {
  return (
    <div className="p-6 bg-red-50 dark:bg-red-950 border border-red-200 dark:border-red-800 rounded-lg">
      <h2 className="text-xl font-semibold text-red-800 dark:text-red-200 mb-2">
        Something went wrong
      </h2>
      <p className="text-red-600 dark:text-red-400 mb-4">{error.message}</p>
      <button
        onClick={reset}
        className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700"
      >
        Try again
      </button>
    </div>
  )
}
```

### Pattern 5: Responsive Design with Tailwind Mobile-First Breakpoints

**What:** Use Tailwind's mobile-first breakpoint system (sm:, md:, lg:, xl:) for responsive layouts.

**When to use:** Always. Default to mobile layout, progressively enhance for larger screens.

**Trade-offs:**
- **Pros:** Mobile-first forces simplicity, easier to scale up than down, matches user behavior (most traffic is mobile)
- **Cons:** Requires rethinking desktop-first designs, may feel counterintuitive initially

**Example:**
```typescript
// Mobile-first component
<div className="flex flex-col gap-4 md:flex-row md:gap-6 lg:gap-8">
  {/* Stacks vertically on mobile, horizontal on tablet+ */}
</div>

<button className="w-full py-3 md:w-auto md:px-6">
  {/* Full-width on mobile, auto-width on tablet+ */}
</button>
```

**Tailwind Breakpoints:**
- Default (no prefix): 0px - 640px (mobile)
- `sm:` 640px+ (large mobile)
- `md:` 768px+ (tablet)
- `lg:` 1024px+ (desktop)
- `xl:` 1280px+ (large desktop)
- `2xl:` 1536px+ (extra large desktop)

## Data Flow

### Analytics Event Flow

```
User Action (page view, button click)
    ↓
Next.js Script Component (afterInteractive strategy)
    ↓
Analytics Script Loads (Plausible/Umami)
    ↓
Event Sent to Analytics Service
    ↓
Analytics Dashboard (external)
```

**Note:** No backend integration required. Analytics is pure client-side.

### Metadata/SEO Flow

```
Request to Page
    ↓
Next.js Server (generateMetadata() runs)
    ↓
HTML with <meta> tags generated
    ↓
Browser receives HTML (no client-side JS needed for SEO)
    ↓
Search Engine Crawler sees meta tags
```

**Note:** Metadata is server-rendered. No runtime overhead.

### Legal Pages Flow

```
User clicks "Privacy Policy" link
    ↓
Browser navigates to /privacy
    ↓
Next.js renders app/privacy/page.tsx (static)
    ↓
HTML returned to browser (no API calls)
```

**Note:** Legal pages are static. No database or backend involvement.

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 0-10k users | All launch features work as-is. No changes needed. Analytics can handle this scale easily. |
| 10k-100k users | Consider self-hosting Umami if using cloud analytics (cost optimization). Add CDN for static assets. Consider edge caching for legal pages (though not necessary with Next.js static optimization). |
| 100k+ users | Analytics self-hosted becomes cost-effective. Consider adding real user monitoring (RUM) for performance insights. May want to separate analytics database from main database if self-hosting Umami with PostgreSQL. |

### Scaling Priorities

1. **First bottleneck:** Analytics cost (if using Plausible cloud at high traffic). **Fix:** Self-host Umami on same DigitalOcean droplet (minimal overhead).
2. **Second bottleneck:** Legal page load time (unlikely, but possible if traffic spikes). **Fix:** Static pages are already optimized. If needed, add Cloudflare CDN in front of Nginx.

## Anti-Patterns

### Anti-Pattern 1: Blocking Scripts in <head>

**What people do:** Add analytics scripts directly in <head> without async/defer or Script component.

**Why it's wrong:** Blocks page rendering, slows down First Contentful Paint (FCP), hurts Core Web Vitals.

**Do this instead:** Use Next.js `<Script>` component with `strategy="afterInteractive"` or `strategy="lazyOnload"`.

### Anti-Pattern 2: Client-Side-Only Metadata

**What people do:** Use React hooks (useEffect) to set document.title and meta tags client-side.

**Why it's wrong:** Search engine crawlers see original HTML without metadata. Social media link previews fail. SEO suffers.

**Do this instead:** Use Next.js `generateMetadata()` or static `metadata` export for server-side rendering.

### Anti-Pattern 3: Hardcoded JSON-LD Without Component Wrapper

**What people do:** Directly render `<script type="application/ld+json">` in server components without client wrapper.

**Why it's wrong:** Can cause React hydration mismatch errors in some Next.js setups. JSON stringification may differ between server and client.

**Do this instead:** Use client component wrapper (JsonLd component) that prevents hydration issues.

### Anti-Pattern 4: Desktop-First Responsive Design

**What people do:** Design for desktop first, then try to make it work on mobile with `max-width` media queries.

**Why it's wrong:** Mobile layouts get bloated with overrides. Harder to simplify complex desktop UI for mobile. Most users are on mobile.

**Do this instead:** Design mobile-first with Tailwind's min-width breakpoints. Add complexity progressively for larger screens.

### Anti-Pattern 5: No Loading States

**What people do:** Show blank screen or spinner during data fetching without indicating what's loading.

**Why it's wrong:** Poor UX. Users don't know if page is broken or loading. Increases bounce rate.

**Do this instead:** Use skeleton UI (Tailwind animate-pulse) that mimics content layout. Show specific loading messages ("Analyzing security headers...").

### Anti-Pattern 6: Copy-Paste Legal Pages

**What people do:** Copy another site's privacy policy and terms of service.

**Why it's wrong:** Copyright infringement (privacy policies are copyrighted). Legal liability if terms don't match your actual practices. GDPR/CCPA violations if inaccurate.

**Do this instead:** Use legal page generator (Termly, iubenda) or hire lawyer. Customize for your specific data practices.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| **Plausible Analytics** | Script tag in layout.tsx via Script component | Cloud: script.js from plausible.io. Self-hosted: your domain/js/script.js. Use next-plausible for proxy support. |
| **Umami Analytics** | Script tag in layout.tsx via Script component | Self-hosted only. Requires PostgreSQL database (can share with main app). Provides npm package `@umami/analytics` for events. |
| **Google Fonts** | Already integrated via next/font/google | No changes needed. Inter font already loaded in layout.tsx. |
| **Legal Page Generators** | Manual copy-paste or API integration | Termly/iubenda provide embed codes or static HTML. Manual copy to Next.js page.tsx works fine. |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| **Frontend ↔ Backend** | No changes required | Launch features are frontend-only. Existing API routes (/api/v1/*) unaffected. |
| **Nginx ↔ Frontend** | Proxy to frontend:3001 for / | Legal pages served via same route (e.g., https://shipsecure.ai/privacy → frontend). |
| **Analytics ↔ Frontend** | Client-side only | Analytics script loads on client. No server-side involvement. If self-hosting Umami, separate service. |
| **Metadata ↔ Backend** | Optional API call for dynamic data | If results page metadata needs scan details, fetch via existing backend API during generateMetadata(). |

## New Components vs Modified Components

### New Components (Build from Scratch)

| Component | File Path | Purpose |
|-----------|-----------|---------|
| Privacy Policy Page | `app/privacy/page.tsx` | Static legal page with privacy policy text |
| Terms of Service Page | `app/terms/page.tsx` | Static legal page with terms of service text |
| JSON-LD Wrapper | `components/json-ld.tsx` | Reusable client component for JSON-LD schema |
| Loading UI (Scan Page) | `app/scan/[id]/loading.tsx` | Skeleton UI for scan progress page |
| Error Boundary (Scan Page) | `app/scan/[id]/error.tsx` | Error UI for scan progress page |
| Error Boundary (Payment) | `app/payment/success/error.tsx` | Error UI for payment success page |
| Metadata Utilities | `lib/metadata.ts` | Shared metadata generation functions |
| Schema Utilities | `lib/schema.ts` | JSON-LD schema generators (Organization, SoftwareApplication, etc.) |

### Modified Components (Enhance Existing)

| Component | File Path | Changes |
|-----------|-----------|---------|
| Root Layout | `app/layout.tsx` | ADD: Analytics Script component. MODIFY: Base metadata (title template, OG defaults). ADD: JSON-LD Organization schema. |
| Landing Page | `app/page.tsx` | ADD: JSON-LD SoftwareApplication schema. MODIFY: Metadata (already has some, enhance OG tags). REVIEW: Responsive layout (already mobile-first, ensure polish). |
| Results Page | `app/results/[token]/page.tsx` | ADD: generateMetadata() for dynamic OG tags with scan results. REVIEW: Error handling (currently minimal). |
| Scan Progress Page | `app/scan/[id]/page.tsx` | IMPROVE: Error handling (set loading=false on API errors). ADD: Better loading states (currently shows polling progress). |
| Global Styles | `app/globals.css` | REVIEW: Ensure all responsive breakpoints covered. ADD: Any missing mobile-first styles. |
| Scan Form Component | `components/scan-form.tsx` | REVIEW: Mobile UX (input sizes, button touch targets). Already has loading state (good). |
| Results Dashboard | `components/results-dashboard.tsx` | REVIEW: Mobile layout (grouping toggles, accordion on small screens). Already responsive (good). |

## Build Order

### Phase 1: Analytics (Low Risk, High Value)

**Why first:** Analytics doesn't affect existing functionality. Can deploy independently. Provides immediate user insights.

1. **Add analytics dependency**: `npm install next-plausible` (if Plausible) or configure Umami
2. **Modify app/layout.tsx**: Add Script component for analytics
3. **Test**: Verify analytics events fire in browser dev tools (Network tab)
4. **Deploy**: Push to production, verify in analytics dashboard

**Dependencies:** None. Standalone.

**Estimated effort:** 1-2 hours (including testing)

### Phase 2: SEO Metadata (Low Risk, High Value)

**Why second:** Improves social sharing immediately. No dependencies on Phase 1.

1. **Modify app/layout.tsx**: Enhance base metadata with title template and OG defaults
2. **Add lib/metadata.ts**: Create shared metadata utilities
3. **Modify app/page.tsx**: Add full metadata (already has some)
4. **Add generateMetadata() to app/results/[token]/page.tsx**: Dynamic OG tags for scan results
5. **Add generateMetadata() to app/payment/success/page.tsx**: Payment success metadata
6. **Test**: Use Open Graph debugger (Facebook, Twitter, LinkedIn) to verify OG tags
7. **Deploy**: Push to production, verify social previews

**Dependencies:** None. Standalone.

**Estimated effort:** 3-4 hours (including OG image creation if not using placeholder)

### Phase 3: JSON-LD Schema (Low Risk, Medium Value)

**Why third:** SEO benefit is incremental. No user-facing changes. Can be done after metadata.

1. **Create components/json-ld.tsx**: Client wrapper for JSON-LD
2. **Create lib/schema.ts**: Schema generators (Organization, SoftwareApplication, WebSite)
3. **Modify app/layout.tsx**: Add Organization schema
4. **Modify app/page.tsx**: Add SoftwareApplication schema
5. **Test**: Use Google Rich Results Test to verify schema
6. **Deploy**: Push to production, verify in Google Search Console (takes days/weeks to index)

**Dependencies:** None. Standalone.

**Estimated effort:** 2-3 hours

### Phase 4: Legal Pages (Low Risk, Essential)

**Why fourth:** Required for GDPR/CCPA compliance but not urgent if no EU/CA traffic yet. Static pages are low risk.

1. **Create app/privacy/page.tsx**: Privacy policy page with static content
2. **Create app/terms/page.tsx**: Terms of service page with static content
3. **Modify app/page.tsx footer**: Add links to /privacy and /terms
4. **Review existing pages**: Ensure footer links exist on all pages
5. **Test**: Navigate to /privacy and /terms, verify rendering
6. **Deploy**: Push to production

**Dependencies:** Legal text (from lawyer or generator). If not ready, can use placeholder and update later.

**Estimated effort:** 2-3 hours (excluding legal text creation, which is external)

### Phase 5: Responsive CSS Polish (Medium Risk, Medium Value)

**Why fifth:** Existing site is already responsive (using Tailwind). This is polish, not critical path.

1. **Audit existing pages**: Test on real mobile devices (iOS, Android) and Chrome DevTools
2. **Identify gaps**: Note any layout breaks, small text, tiny buttons, overflow issues
3. **Fix issues**: Update Tailwind classes (e.g., `text-sm md:text-base`, `py-2 md:py-3`)
4. **Test**: Verify fixes on multiple viewports (320px, 375px, 768px, 1024px)
5. **Deploy**: Push to production, test on real devices

**Dependencies:** None. Standalone. (But benefits from completing Phases 1-4 first so analytics tracks any bounce rate improvements.)

**Estimated effort:** 4-6 hours (depends on number of issues found)

### Phase 6: UX Improvements (Medium Risk, High Value)

**Why sixth:** Improves user experience but touches existing components. Should be done after analytics is live to measure impact.

1. **Create app/scan/[id]/loading.tsx**: Skeleton UI for scan progress
2. **Create app/scan/[id]/error.tsx**: Error boundary with retry button
3. **Create app/payment/success/error.tsx**: Error boundary for payment page
4. **Modify app/scan/[id]/page.tsx**: Improve error handling (currently sets loading=false but no user-friendly message)
5. **Review components/scan-form.tsx**: Already has good loading state, verify mobile UX
6. **Review components/results-dashboard.tsx**: Test mobile layout (grouping toggles, accordions)
7. **Test**: Trigger errors (invalid URLs, network failures) and verify error UI
8. **Deploy**: Push to production, monitor error rates in analytics

**Dependencies:** Phase 1 (analytics) should be live to measure before/after UX metrics.

**Estimated effort:** 4-6 hours

## Build Order Summary

```
Phase 1: Analytics (1-2h)
    ↓
Phase 2: SEO Metadata (3-4h)
    ↓
Phase 3: JSON-LD Schema (2-3h)
    ↓
Phase 4: Legal Pages (2-3h, excluding legal text)
    ↓
Phase 5: Responsive CSS Polish (4-6h)
    ↓
Phase 6: UX Improvements (4-6h)

Total: 16-24 hours
```

**Critical Path:** Analytics → SEO Metadata (can be done in any order, but analytics first provides tracking for subsequent phases)

**Optional/Deferrable:** JSON-LD Schema (nice-to-have), Responsive CSS Polish (if existing layout is acceptable)

**Blockers:** Legal Pages depend on legal text availability (external). Can use placeholder.

## Integration with Existing Architecture

### No Backend Changes Required

All launch readiness features are **frontend-only**. The Rust/Axum backend does not need any modifications.

**Reason:** Analytics, SEO metadata, JSON-LD, legal pages, responsive CSS, and UX improvements are all client-side or static rendering concerns.

**Existing backend API routes continue to work unchanged:**
- `POST /api/v1/scans` (scan submission)
- `GET /api/v1/scans/:id` (scan status polling)
- `GET /api/v1/scans/:id/findings` (results)
- `POST /api/v1/checkout` (Stripe checkout)
- `POST /api/v1/webhooks/stripe` (Stripe webhooks)
- `GET /api/v1/stats/scan-count` (social proof)

### Nginx Configuration

**Current setup (from docker-compose.prod.yml and known Nginx config):**
- `/api/*` → proxied to backend:3000
- `/*` → proxied to frontend:3001

**No changes needed.**

Legal pages (`/privacy`, `/terms`) will be served by frontend:3001 via Next.js routing. Nginx doesn't need specific rules.

### Docker Compose Changes

**Minimal changes:**

1. **Frontend environment variables** (if using cloud analytics):
   ```yaml
   # docker-compose.prod.yml
   frontend:
     environment:
       NEXT_PUBLIC_PLAUSIBLE_DOMAIN: shipsecure.ai
       # OR for Umami:
       NEXT_PUBLIC_UMAMI_WEBSITE_ID: xxxxxxxx
       NEXT_PUBLIC_UMAMI_HOST: https://analytics.shipsecure.ai
   ```

2. **Optional: Self-hosted Umami** (if chosen):
   ```yaml
   # docker-compose.prod.yml
   umami:
     image: ghcr.io/umami-software/umami:postgresql-latest
     environment:
       DATABASE_URL: ${UMAMI_DATABASE_URL}
       DATABASE_TYPE: postgresql
       APP_SECRET: ${UMAMI_APP_SECRET}
     ports:
       - "127.0.0.1:3002:3000"
     depends_on:
       - db
   ```

   Then configure Nginx to proxy `/analytics/*` to `umami:3000` (or use subdomain like `analytics.shipsecure.ai`).

**If using cloud analytics (Plausible):** No Docker changes. Just environment variables for Next.js.

### Deployment Process

**Current process (from v1.1 milestones):**
1. Build Docker images locally or via GitHub Actions
2. Push to GHCR
3. SSH to DigitalOcean droplet
4. Pull images
5. `docker compose -f docker-compose.prod.yml down`
6. `docker compose -f docker-compose.prod.yml up -d`
7. Verify via https://shipsecure.ai

**No changes needed for launch features.** Follow existing deployment process.

**Additional step (one-time for legal pages):**
- Ensure footer component includes links to `/privacy` and `/terms` before deploying

## Risk Assessment

| Feature | Risk Level | Mitigation |
|---------|------------|------------|
| Analytics | LOW | No user-facing changes. Script loads asynchronously. If analytics service is down, site continues working. Use `strategy="afterInteractive"` to prevent blocking. |
| SEO Metadata | LOW | Pure HTML `<meta>` tags. No JavaScript runtime dependency. If metadata is wrong, site still works (just bad social previews). Test with OG debuggers before deploy. |
| JSON-LD Schema | LOW | Search engines ignore invalid schema. No user-facing impact. Test with Google Rich Results Test. |
| Legal Pages | LOW | Static pages. No dynamic data. Risk is legal accuracy (not technical). Use placeholder and update with lawyer-reviewed text later. |
| Responsive CSS | MEDIUM | Could break layouts if Tailwind classes conflict. Mitigate: Test on real devices before deploy. Have rollback plan. |
| UX Improvements | MEDIUM | Touches existing components. Could introduce bugs. Mitigate: Comprehensive testing. Deploy UX changes after analytics is live (to measure impact). |

**Overall risk: LOW to MEDIUM.** Most features are additive (analytics, metadata, legal pages) with minimal risk of breaking existing functionality.

## Monitoring and Validation

### Post-Deploy Checks

| Feature | Validation Method | Expected Result |
|---------|-------------------|------------------|
| Analytics | Check analytics dashboard for live page views | Page views appear within 1-2 minutes |
| SEO Metadata | Use Facebook Sharing Debugger, Twitter Card Validator, LinkedIn Post Inspector | OG tags render correctly with title, description, image |
| JSON-LD | Use Google Rich Results Test | Schema validates without errors |
| Legal Pages | Navigate to /privacy and /terms | Pages load without errors, content renders |
| Responsive CSS | Test on real mobile devices (iOS, Android) | No horizontal scroll, readable text, tappable buttons (44px min) |
| UX Improvements | Trigger errors (network disconnect during scan), check loading states | Error boundaries render, loading skeletons show, retry buttons work |

### Analytics Events to Track (Optional)

If implementing event tracking (beyond page views):
- Scan form submission (`scan_started`)
- Scan completion (`scan_completed`)
- Paid audit button click (`upgrade_clicked`)
- Stripe checkout initiation (`checkout_started`)
- Payment success (`payment_completed`)

**Implementation:** Use `window.plausible()` or `window.umami.track()` in client components.

## Sources

### Next.js Metadata & SEO
- [Functions: generateMetadata | Next.js](https://nextjs.org/docs/app/api-reference/functions/generate-metadata)
- [Getting Started: Metadata and OG images | Next.js](https://nextjs.org/docs/app/getting-started/metadata-and-og-images)
- [How to Configure SEO in Next.js 16 (the Right Way)](https://jsdevspace.substack.com/p/how-to-configure-seo-in-nextjs-16)
- [Next.js SEO Optimization Guide (2026 Edition)](https://www.djamware.com/post/697a19b07c935b6bb054313e/next-js-seo-optimization-guide--2026-edition)

### Analytics Integration
- [How to add the script to your NextJS site | Plausible docs](https://plausible.io/docs/nextjs-integration)
- [GitHub - 4lejandrito/next-plausible](https://github.com/4lejandrito/next-plausible)
- [How to Setup and Integrate Umami to Your Next.js Site](https://dev.to/yehezkielgunawan/how-to-setup-and-integrate-umami-to-your-nextjs-site-ahf)
- [Using the Umami Analytics Provider in Next.js](https://makerkit.dev/docs/next-supabase-turbo/analytics/umami-analytics-provider)

### Next.js Script Component
- [Components: Script Component | Next.js](https://nextjs.org/docs/app/api-reference/components/script)
- [Optimizing third-party script loading in Next.js | Chrome for Developers](https://developer.chrome.com/blog/script-component)
- [A Next.js package for managing third-party libraries | Chrome for Developers](https://developer.chrome.com/blog/next-third-parties)

### JSON-LD & Structured Data
- [Guides: JSON-LD | Next.js](https://nextjs.org/docs/app/guides/json-ld)
- [Implementing JSON-LD in Next.js for SEO](https://www.wisp.blog/blog/implementing-json-ld-in-nextjs-for-seo)
- [Add Structured Data to your Next.js site with JSON-LD](https://mikebifulco.com/posts/structured-data-json-ld-for-next-js-sites)

### Privacy & GDPR Compliance
- [Best Privacy-Compliant Analytics Tools for 2026](https://www.mitzu.io/post/best-privacy-compliant-analytics-tools-for-2026)
- [The 9 best GDPR-compliant analytics tools](https://posthog.com/blog/best-gdpr-compliant-analytics-tools)
- [Plausible vs Umami: Which One Is Right for Your Website Analytics?](https://vemetric.com/blog/plausible-vs-umami)

### Legal Pages
- [Using Termageddon with React and Next.js](https://termageddon.com/using-termageddon-with-react-and-next-js/)
- [Privacy Policy for Next.js: How To Create One](https://termly.io/resources/articles/privacy-policy-for-nextjs/)

### Responsive Design & Tailwind
- [Responsive design - Core concepts - Tailwind CSS](https://tailwindcss.com/docs/responsive-design)
- [Install Tailwind CSS with Next.js](https://tailwindcss.com/docs/guides/nextjs)
- [Breakpoint: Responsive Design Breakpoints in 2025](https://www.browserstack.com/guide/responsive-design-breakpoints)
- [Responsive Design Breakpoints: 2025 Playbook](https://dev.to/gerryleonugroho/responsive-design-breakpoints-2025-playbook-53ih)

### UX & Error Handling
- [Leveraging Suspense and Error Boundaries in Next.js 15](https://medium.com/@sureshdotariya/leveraging-suspense-and-error-boundaries-in-next-js-034aff10df4f)
- [Best Practices for Loading States in Next.js](https://www.getfishtank.com/insights/best-practices-for-loading-states-in-nextjs)
- [Next.js Error Handling Patterns](https://betterstack.com/community/guides/scaling-nodejs/error-handling-nextjs/)
- [Next.js 15: Error Handling best practices](https://devanddeliver.com/blog/frontend/next-js-15-error-handling-best-practices-for-code-and-routes)

---
*Architecture research for: Launch Readiness Features Integration*
*Researched: 2026-02-08*
