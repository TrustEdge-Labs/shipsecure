# Stack Research: Launch Readiness Features

**Domain:** SaaS security scanner - launch readiness additions
**Researched:** 2026-02-08
**Confidence:** HIGH

## Executive Summary

Launch readiness features require **minimal stack additions**. Next.js 16.1.6's built-in Metadata API eliminates the need for SEO libraries like `next-seo`. Tailwind CSS 4 already provides responsive utilities. The primary additions are:

1. **Analytics**: Plausible (hosted) or Umami (self-hosted) - both lightweight, privacy-friendly
2. **UX Polish**: Sonner toast notifications (lightweight, modern)
3. **Legal Pages**: Static markdown files (no code generation libraries needed)
4. **SEO**: Built-in Next.js Metadata API + sitemap/robots generation

**Key Decision:** Avoid over-engineering. Use Next.js built-ins wherever possible. No heavy UI libraries needed.

## Stack Additions for Launch Readiness

### Analytics (NEW)

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **Plausible Analytics** | Cloud (self-hosted option) | Privacy-friendly analytics without cookies | Hosted solution, €9/month, minimal setup, no cookie banners required. Recommended for quick launch. |
| **Umami Analytics** | Cloud/Self-hosted | Open-source privacy analytics | Free if self-hosted with PostgreSQL (existing DB), $9/month hosted. Choose if you want data ownership and already manage infrastructure. |
| `next-plausible` | ^3.12.5 | Plausible integration for Next.js | Official integration, supports App Router, proxy bypass for ad blockers, custom events API |

**Recommendation:** **Plausible (hosted)** for fast launch. Umami if you have strong privacy/data ownership requirements and technical capacity to manage self-hosting.

### UX Polish (NEW)

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| **Sonner** | ^1.x (latest) | Toast notifications | Lightweight (small bundle), TypeScript-first, designed for React 18+, works seamlessly with Next.js Server Components. Modern, opinionated, zero config. |

**Alternative Considered:** `react-hot-toast` (5KB) - also excellent but Sonner is more modern with better Server Component support.

**Avoid:** `react-toastify` (16KB) - heavier, older API patterns.

### SEO & Metadata (NO NEW DEPENDENCIES)

| Technology | Version | Purpose | Why Use Built-in |
|------------|---------|---------|------------------|
| **Next.js Metadata API** | Built-in (16.1.6) | SEO meta tags, OpenGraph, Twitter cards | No external dependencies needed. TypeScript support, automatic deduplication, dynamic generation via `generateMetadata`. |
| **Sitemap/Robots** | Built-in | `app/sitemap.ts` and `app/robots.ts` | Native Next.js file conventions. Programmatic generation without libraries. |

**Alternative Considered:** `next-seo` package - **NOT NEEDED**. Next.js 15+ Metadata API supersedes it entirely.

**Alternative Considered:** `next-sitemap` package - Only needed for **very large sites** with 50K+ URLs requiring split sitemaps. ShipSecure has ~10 pages - built-in sitemap.ts is sufficient.

### Responsive Design (NO NEW DEPENDENCIES)

| Technology | Version | Purpose | Why Already Available |
|------------|---------|---------|----------------------|
| **Tailwind CSS 4** | ^4 (existing) | Mobile-first responsive utilities | Already installed. Mobile-first breakpoints (sm:, md:, lg:, xl:, 2xl:) cover all responsive needs. |

**Pattern:** Mobile-first means unprefixed classes apply to all sizes, `md:` and above apply at breakpoints and up.

**Breakpoints:**
- `sm:` 640px (40rem)
- `md:` 768px (48rem)
- `lg:` 1024px (64rem)
- `xl:` 1280px (80rem)
- `2xl:` 1536px (96rem)

### Favicon Generation (TOOLING, NO RUNTIME DEPENDENCY)

| Tool | Purpose | Notes |
|------|---------|-------|
| **Real Favicon Generator** | Create multi-platform favicons | Free web tool. Generates all sizes (16x16 to 512x512) + Apple Touch icons. Next.js integration guide available. Use once during design phase. |
| **RJL.io Favicon Generator** | Modern 2026 favicon generator | 100% client-side processing, supports emoji, Next.js App Router code snippets. |

**Next.js Integration:** Place generated files in `app/` directory:
- `app/icon.png` or `app/icon.svg` - automatically served as favicon
- `app/apple-icon.png` - Apple Touch icon
- Can programmatically generate with `app/icon.tsx` using `ImageResponse` API

### Legal Pages (NO CODE LIBRARIES)

| Approach | Tool | Why |
|----------|------|-----|
| **Static Markdown** | None - manual authoring | Best practice: hire lawyer or use generator as starting point, customize for your business. |
| **Template Generators** | GetTerms.io, TermsFeed, Termly | Use to generate initial drafts. **DO NOT use as-is** - customize for ShipSecure specifics (scanning disclaimers, CFAA compliance, data retention). |

**Key Point:** Legal pages are **business logic, not code dependencies**. Generate templates with tools above, review/customize manually, store as static markdown or React components.

## Installation

### For Plausible Analytics (Hosted)

```bash
npm install next-plausible
```

**Configuration** (in `app/layout.tsx`):

```typescript
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

**With Proxy** (bypass ad blockers) - add to `next.config.mjs`:

```javascript
import { withPlausibleProxy } from 'next-plausible'

export default withPlausibleProxy()({
  // your Next.js config
})
```

### For Umami Analytics (Self-hosted)

**Setup:**
1. Deploy Umami with Docker Compose (uses PostgreSQL - can share existing DB)
2. Create website in Umami admin panel, get tracking script

**Integration:**

```bash
# No package needed for basic integration - use Next.js Script component
```

**Configuration** (in `app/layout.tsx`):

```typescript
import Script from 'next/script'

export default function RootLayout({ children }) {
  return (
    <html>
      <head>
        <Script
          src="https://analytics.yourdomain.com/script.js"
          data-website-id="your-website-id"
          strategy="afterInteractive"
        />
      </head>
      <body>{children}</body>
    </html>
  )
}
```

### For Sonner Toast Notifications

```bash
npm install sonner
```

**Configuration** (in `app/layout.tsx`):

```typescript
import { Toaster } from 'sonner'

export default function RootLayout({ children }) {
  return (
    <html>
      <body>
        {children}
        <Toaster richColors closeButton position="top-right" />
      </body>
    </html>
  )
}
```

**Usage** (in any client component):

```typescript
'use client'
import { toast } from 'sonner'

export function ScanButton() {
  const handleScan = async () => {
    toast.promise(
      scanUrl(url),
      {
        loading: 'Scanning...',
        success: 'Scan complete!',
        error: 'Scan failed',
      }
    )
  }

  return <button onClick={handleScan}>Scan</button>
}
```

### No Installation Needed

**SEO Metadata** - use built-in `app/layout.tsx` or `app/page.tsx`:

```typescript
import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'ShipSecure - Security Scanner for Vibe-Coded Apps',
  description: 'Find vulnerabilities before hackers do. Free security scans for your web apps.',
  metadataBase: new URL('https://shipsecure.ai'),
  openGraph: {
    title: 'ShipSecure - Security Scanner',
    description: 'Find vulnerabilities before hackers do.',
    url: 'https://shipsecure.ai',
    siteName: 'ShipSecure',
    images: [
      {
        url: '/og-image.png', // resolves to https://shipsecure.ai/og-image.png
        width: 1200,
        height: 630,
      },
    ],
    locale: 'en_US',
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'ShipSecure - Security Scanner',
    description: 'Find vulnerabilities before hackers do.',
    images: ['/og-image.png'],
  },
  robots: {
    index: true,
    follow: true,
  },
}
```

**Sitemap** - create `app/sitemap.ts`:

```typescript
import { MetadataRoute } from 'next'

export default function sitemap(): MetadataRoute.Sitemap {
  return [
    {
      url: 'https://shipsecure.ai',
      lastModified: new Date(),
      changeFrequency: 'monthly',
      priority: 1,
    },
    {
      url: 'https://shipsecure.ai/privacy',
      lastModified: new Date(),
      changeFrequency: 'yearly',
      priority: 0.5,
    },
    {
      url: 'https://shipsecure.ai/terms',
      lastModified: new Date(),
      changeFrequency: 'yearly',
      priority: 0.5,
    },
  ]
}
```

**Robots.txt** - create `app/robots.ts`:

```typescript
import { MetadataRoute } from 'next'

export default function robots(): MetadataRoute.Robots {
  return {
    rules: {
      userAgent: '*',
      allow: '/',
      disallow: '/api/',
    },
    sitemap: 'https://shipsecure.ai/sitemap.xml',
  }
}
```

**Responsive Design** - use existing Tailwind utilities:

```typescript
// Mobile-first approach (already available)
<div className="w-full md:w-1/2 lg:w-1/3">
  <h1 className="text-2xl md:text-4xl lg:text-5xl">
    Find vulnerabilities before hackers do
  </h1>
</div>
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| **Plausible (hosted)** | Umami (self-hosted) | Strong privacy requirements, existing PostgreSQL infrastructure, technical team comfortable managing services |
| **Plausible/Umami** | Google Analytics | Never for privacy-focused SaaS targeting developers. GA requires cookie banners, tracks users, contradicts "privacy-first security" positioning |
| **Sonner** | react-hot-toast | Both excellent. Sonner preferred for Next.js 15+ with Server Components. react-hot-toast (5KB) slightly smaller. |
| **Next.js Metadata API** | next-seo package | Never. next-seo is obsolete for Next.js 15+. Built-in API is superior (type-safe, automatic optimization, no dependencies). |
| **Built-in sitemap.ts** | next-sitemap package | Sites with 50K+ URLs needing split sitemaps. ShipSecure has ~10 pages - overkill. |
| **Static legal pages** | Legal generator APIs (iubenda) | Ongoing compliance monitoring for large companies. ShipSecure MVP: generate once, review manually. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| **next-seo** | Obsolete. Next.js 15+ Metadata API replaced it entirely. | Next.js built-in `Metadata` type and `generateMetadata` function |
| **Google Analytics** | Privacy invasion, cookie banners, developer backlash. Contradicts ShipSecure's privacy positioning. | Plausible or Umami |
| **react-toastify** | 16KB (3x heavier than alternatives). Older API patterns. | Sonner (modern) or react-hot-toast (smallest) |
| **Skeleton UI libraries** (Material Tailwind, Flowbite, daisyUI) | Heavyweight component libraries. Tailwind already has `animate-pulse` for simple skeletons. | Custom Tailwind skeletons with `animate-pulse` class |
| **Heavy UI libraries** (Chakra, Material-UI) | Bundle bloat. ShipSecure has minimal UI needs. Tailwind covers everything. | Tailwind CSS utilities |
| **JSON-LD generator libraries** | ShipSecure doesn't need rich snippets (no blog, recipes, products). If needed later, manually add `<script type="application/ld+json">`. | Manual JSON-LD in components if needed |

## Stack Patterns by Use Case

**If launching on Hacker News/Reddit (developer audience):**
- **Use Plausible** - hosted, privacy-first, no cookies. Mention "privacy-friendly analytics" in Show HN post
- **Avoid Google Analytics** - developers use ad blockers, will notice GA and critique it
- **Emphasize**: "No tracking, no cookies, privacy-first" positioning

**If budget-constrained:**
- **Use Umami self-hosted** - free with existing PostgreSQL database
- **Skip toast library initially** - use browser `alert()` or simple custom component for MVP
- **Generate legal pages once** - use free generators (GetTerms.io), review manually, store as markdown

**If prioritizing speed to launch:**
- **Use Plausible hosted** - 5-minute setup, no infrastructure
- **Use Sonner** - zero configuration, works immediately
- **Use built-in Next.js features** - no external packages for SEO/sitemap/robots

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| next-plausible ^3.12.5 | Next.js 14+ | App Router and Pages Router support. Proxy feature requires Next.js 12.1+ |
| Sonner ^1.x | React 18+, Next.js 13+ | Designed for modern React. Server Component aware. |
| Tailwind CSS 4 | Next.js 16.1.6 | Already integrated via `@tailwindcss/postcss`. CSS-first configuration (breaking change from v3). |

## Integration Points with Existing Stack

### Analytics Integration
- **Frontend only** - add Script component or PlausibleProvider to `app/layout.tsx`
- **No backend changes** - analytics are client-side page view tracking
- **No database changes** - analytics data stored externally (Plausible cloud or Umami instance)

### Toast Notifications Integration
- **Add Toaster to layout** - renders toast container
- **Import toast function** - use in existing scan forms, payment flows, error handlers
- **Replace existing alerts** - if currently using `alert()` or custom modals for success/error messages

### SEO Metadata Integration
- **Add to app/layout.tsx** - site-wide metadata (title template, base URL, OpenGraph defaults)
- **Add to app/page.tsx** - homepage-specific metadata
- **Add to app/scan/[id]/page.tsx** - dynamic metadata for scan result pages (if making them shareable)

### Responsive Design Integration
- **Audit existing components** - add `md:`, `lg:` breakpoint prefixes to fixed-width elements
- **Test on mobile viewports** - Chrome DevTools responsive mode (375px, 768px, 1024px)
- **Priority areas**: Landing page hero, scan input form, results dashboard, payment modal

### Legal Pages Integration
- **Create routes**: `app/privacy/page.tsx`, `app/terms/page.tsx`, `app/acceptable-use/page.tsx`
- **Link from footer** - add links to existing frontend footer component
- **Update robots.ts** - allow indexing of legal pages
- **Update sitemap.ts** - include legal page URLs

## Sources

### Analytics
- [Plausible vs Umami comparison (2026)](https://www.mitzu.io/post/best-privacy-compliant-analytics-tools-for-2026) — HIGH confidence
- [Plausible Analytics Next.js integration](https://plausible.io/docs/nextjs-integration) — Official docs, HIGH confidence
- [next-plausible npm package](https://www.npmjs.com/package/next-plausible) — Version 3.12.5, HIGH confidence
- [Umami Analytics self-hosting guide](https://aaronjbecker.com/posts/umami-vs-plausible-vs-matomo-self-hosted-analytics/) — MEDIUM confidence

### SEO & Metadata
- [Next.js Metadata API official docs](https://nextjs.org/docs/app/api-reference/functions/generate-metadata) — Official docs, HIGH confidence
- [Next.js 15 SEO guide (2026)](https://www.djamware.com/post/697a19b07c935b6bb054313e/next-js-seo-optimization-guide--2026-edition) — MEDIUM confidence
- [next-seo vs Metadata API discussion](https://github.com/vercel/next.js/discussions/51392) — GitHub discussion, MEDIUM confidence
- [Next.js sitemap.xml official docs](https://nextjs.org/docs/app/api-reference/file-conventions/metadata/sitemap) — Official docs, HIGH confidence
- [Next.js robots.txt official docs](https://nextjs.org/docs/app/api-reference/file-conventions/metadata/robots) — Official docs, HIGH confidence

### UX & Notifications
- [Sonner toast component](https://github.com/emilkowalski/sonner) — GitHub repo, HIGH confidence
- [Top React notification libraries (2026)](https://knock.app/blog/the-top-notification-libraries-for-react) — MEDIUM confidence
- [React toast libraries comparison (2025)](https://blog.logrocket.com/react-toast-libraries-compared-2025/) — MEDIUM confidence

### Responsive Design
- [Tailwind CSS responsive design](https://tailwindcss.com/docs/responsive-design) — Official docs, HIGH confidence
- [Tailwind CSS 4 breakpoints](https://bordermedia.org/blog/tailwind-css-4-breakpoint-override) — MEDIUM confidence
- [Tailwind best practices (2025-2026)](https://www.frontendtools.tech/blog/tailwind-css-best-practices-design-system-patterns) — MEDIUM confidence

### Legal Pages
- [GetTerms.io SaaS generator](https://getterms.io/privacy-policy-generator/saas) — Tool reference, MEDIUM confidence
- [Privacy policy generators for SaaS (2026)](https://blocksurvey.io/privacy-guides/privacy-policy-generators-for-saas) — MEDIUM confidence
- [Terms and conditions generators (2026)](https://cybernews.com/privacy-compliance-tools/best-terms-and-conditions-generator/) — MEDIUM confidence

### Favicon
- [Real Favicon Generator for Next.js](https://realfavicongenerator.net/favicon-generator/nextjs) — Tool reference, MEDIUM confidence
- [Next.js favicon file conventions](https://nextjs.org/docs/app/api-reference/file-conventions/metadata/app-icons) — Official docs, HIGH confidence
- [Complete favicon guide (2026)](https://devconsole.dev/blog/complete-guide-favicons-2026) — MEDIUM confidence

---

*Stack research for: ShipSecure launch readiness features*
*Researched: 2026-02-08*
*Overall confidence: HIGH (built-ins), MEDIUM (external packages)*
