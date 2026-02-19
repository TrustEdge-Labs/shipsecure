# Phase 09: SEO & Discoverability - Research

**Researched:** 2026-02-08
**Domain:** Next.js SEO, Open Graph, Structured Data, Social Sharing
**Confidence:** HIGH

## Summary

Phase 9 focuses on implementing comprehensive SEO and social sharing features for ShipSecure using Next.js 16.1.6 App Router's built-in Metadata API. The codebase already uses the App Router with basic metadata in `app/layout.tsx`, so this phase extends that foundation with page-specific metadata, Open Graph tags, structured data, and crawler controls.

The Next.js Metadata API provides a type-safe, server-side approach to SEO that eliminates the need for third-party libraries like next-seo. Key implementation areas include: unique metadata for each page route (landing, results, payment success), Open Graph tags for social sharing with proper image generation, noindex/nofollow directives for private scan results, JSON-LD structured data for Organization and SoftwareApplication schemas, and automated sitemap/robots.txt generation.

**Primary recommendation:** Use Next.js built-in Metadata API with static metadata objects for landing/success pages and generateMetadata() for dynamic scan result pages. Generate Open Graph images using next/og ImageResponse API for dynamic branding. Implement noindex via robots metadata field in generateMetadata for scan result pages.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Next.js | 16.1.6 | App Router with Metadata API | Built-in SEO features, no external library needed |
| next/og | Built-in | OG image generation (ImageResponse) | Official Next.js solution, 5x faster than previous versions |
| TypeScript | Latest | Type safety for Metadata objects | Prevents configuration errors, IDE autocomplete |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @vercel/og | 0.6+ | Alternative OG generation | When not using Next.js or need standalone solution |
| Sharp | Latest | Advanced image processing | Complex image manipulation beyond ImageResponse |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Metadata API | next-seo | Third-party dependency, less type-safe, not needed for App Router |
| next/og | @vercel/og | Older approach, next/og is preferred for Next.js projects |
| Dynamic sitemap | Static XML | Manual updates vs automatic, dynamic better for content changes |

**Installation:**
No additional packages needed - Next.js 16.1.6 includes all required functionality.

## Architecture Patterns

### Recommended Project Structure
```
frontend/app/
├── layout.tsx                    # Root metadata, default title/description
├── page.tsx                      # Landing page (static metadata export)
├── results/[token]/
│   ├── page.tsx                  # Scan results (generateMetadata for dynamic)
│   └── opengraph-image.tsx       # Optional: custom OG image generation
├── payment/success/
│   └── page.tsx                  # Success page (static metadata export)
├── sitemap.ts                    # Dynamic sitemap generation
├── robots.ts                     # Dynamic robots.txt generation
└── api/og/route.tsx              # Optional: API route for OG images
```

### Pattern 1: Static Metadata for Fixed Pages
**What:** Export a Metadata object directly from page.tsx for pages with fixed content
**When to use:** Landing page, payment success page, about page, pricing page
**Example:**
```typescript
// Source: https://nextjs.org/docs/app/api-reference/functions/generate-metadata
import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'ShipSecure - Security Scanning for Vibe-Coded Apps',
  description: 'Ship fast, stay safe. Free security scanning for AI-generated web applications. Catch security flaws before they become breaches.',
  openGraph: {
    title: 'ShipSecure - Security Scanning for Vibe-Coded Apps',
    description: 'Ship fast, stay safe. Free security scanning for AI-generated web applications.',
    url: 'https://shipsecure.ai',
    siteName: 'ShipSecure',
    images: [
      {
        url: 'https://shipsecure.ai/og-image.png',
        width: 1200,
        height: 630,
        alt: 'ShipSecure - Security Scanning for Vibe-Coded Apps',
      },
    ],
    locale: 'en_US',
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'ShipSecure - Security Scanning for Vibe-Coded Apps',
    description: 'Ship fast, stay safe. Free security scanning for AI-generated web applications.',
    images: ['https://shipsecure.ai/og-image.png'],
  },
}
```

### Pattern 2: Dynamic Metadata for User-Generated Content
**What:** Use generateMetadata() function for pages with dynamic data (scan results)
**When to use:** Dynamic routes, pages with user-specific content, private pages
**Example:**
```typescript
// Source: https://nextjs.org/docs/app/api-reference/functions/generate-metadata
import type { Metadata } from 'next'

export async function generateMetadata({ params }: { params: Promise<{ token: string }> }): Promise<Metadata> {
  const { token } = await params

  try {
    const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:3000'
    const res = await fetch(`${BACKEND_URL}/api/v1/results/${token}`, { cache: 'no-store' })

    if (!res.ok) {
      return {
        title: 'Results Not Found - ShipSecure',
        robots: { index: false, follow: false },
      }
    }

    const data = await res.json()

    return {
      title: `Security Scan: ${data.score || 'In Progress'} Grade - ShipSecure`,
      description: `Security scan results for ${data.target_url}. See detailed findings and recommendations.`,
      robots: {
        index: false,        // CRITICAL: Prevent indexing of private results
        follow: false,       // Don't follow links from results pages
        nocache: true,       // Don't cache this page
      },
    }
  } catch (error) {
    return {
      title: 'Results Not Found - ShipSecure',
      robots: { index: false, follow: false },
    }
  }
}
```

### Pattern 3: Dynamic Sitemap Generation
**What:** Generate sitemap.xml programmatically using MetadataRoute.Sitemap
**When to use:** When you have dynamic content or need to include database-driven URLs
**Example:**
```typescript
// Source: https://nextjs.org/docs/app/api-reference/file-conventions/metadata/sitemap
import type { MetadataRoute } from 'next'

export default function sitemap(): MetadataRoute.Sitemap {
  return [
    {
      url: 'https://shipsecure.ai',
      lastModified: new Date(),
      changeFrequency: 'daily',
      priority: 1.0,
    },
    {
      url: 'https://shipsecure.ai/pricing',
      lastModified: new Date(),
      changeFrequency: 'weekly',
      priority: 0.8,
    },
    // Note: Don't include /results/* pages (they're noindex)
  ]
}
```

### Pattern 4: Robots.txt Configuration
**What:** Configure crawler behavior using MetadataRoute.Robots
**When to use:** Control which paths crawlers can access, reference sitemap
**Example:**
```typescript
// Source: https://nextjs.org/docs/app/api-reference/file-conventions/metadata/robots
import type { MetadataRoute } from 'next'

export default function robots(): MetadataRoute.Robots {
  return {
    rules: {
      userAgent: '*',
      allow: '/',
      disallow: ['/results/', '/scan/', '/api/'],
    },
    sitemap: 'https://shipsecure.ai/sitemap.xml',
  }
}
```

### Pattern 5: JSON-LD Structured Data
**What:** Embed structured data in page using script tag with JSON-LD
**When to use:** Landing page for Organization and SoftwareApplication schemas
**Example:**
```typescript
// Source: https://schema.org/Organization, https://schema.org/SoftwareApplication
export default function Home() {
  const organizationSchema = {
    '@context': 'https://schema.org',
    '@type': 'Organization',
    name: 'ShipSecure',
    url: 'https://shipsecure.ai',
    description: 'Security scanning for AI-generated web applications',
    contactPoint: {
      '@type': 'ContactPoint',
      email: 'support@shipsecure.ai',
      contactType: 'Customer Support',
    },
  }

  const softwareSchema = {
    '@context': 'https://schema.org',
    '@type': 'SoftwareApplication',
    name: 'ShipSecure',
    applicationCategory: 'SecurityApplication',
    operatingSystem: 'Web',
    offers: {
      '@type': 'Offer',
      price: '0',
      priceCurrency: 'USD',
      description: 'Free security scanning',
    },
    url: 'https://shipsecure.ai',
    description: 'Free security scanning for vibe-coded web apps',
    featureList: [
      'Security header analysis',
      'TLS configuration scanning',
      'Exposed file detection',
      'JavaScript secret scanning',
    ],
  }

  return (
    <div>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(organizationSchema) }}
      />
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(softwareSchema) }}
      />
      {/* Page content */}
    </div>
  )
}
```

### Pattern 6: Dynamic OG Image Generation
**What:** Generate OG images on-demand using ImageResponse API
**When to use:** Custom branding, dynamic content in social previews
**Example:**
```typescript
// Source: https://nextjs.org/docs/app/api-reference/functions/image-response
import { ImageResponse } from 'next/og'

export const runtime = 'edge'
export const size = { width: 1200, height: 630 }

export default async function Image() {
  return new ImageResponse(
    (
      <div
        style={{
          fontSize: 60,
          background: 'linear-gradient(to bottom, #2563eb, #1e40af)',
          width: '100%',
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          color: 'white',
        }}
      >
        <h1>ShipSecure</h1>
        <p style={{ fontSize: 30 }}>Security Scanning for Vibe-Coded Apps</p>
      </div>
    ),
    {
      ...size,
    }
  )
}
```

### Anti-Patterns to Avoid
- **Client-side metadata updates:** Never use useEffect to modify meta tags - all metadata must be server-side
- **Mixing static and dynamic metadata:** Don't export both `metadata` object and `generateMetadata()` from same file
- **Missing metadataBase:** Relative URLs in Open Graph won't resolve without metadataBase in root layout
- **Forgetting noindex on private content:** Default is to index all pages - must explicitly set robots: { index: false }
- **Over-relying on Twitter Card tags when OG exists:** Twitter falls back to OG tags, but include both for full control
- **Blocking noindex pages in robots.txt:** If robots.txt blocks a page, crawlers never see the noindex tag

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| OG image generation | Custom Canvas/Sharp image builder | next/og ImageResponse | Built-in, Edge Runtime, 5x faster, supports JSX syntax |
| Sitemap generation | Manual XML file maintenance | sitemap.ts with MetadataRoute.Sitemap | Auto-updates, type-safe, supports dynamic content |
| Meta tag injection | Custom <Head> component logic | Metadata API export | Framework-native, SSR by default, prevents duplicates |
| Robots.txt logic | Static file with manual edits | robots.ts with MetadataRoute.Robots | Dynamic rules, environment-aware, type-safe |
| Schema markup builder | String concatenation for JSON-LD | Type-safe objects with JSON.stringify | Prevents syntax errors, easier to maintain |

**Key insight:** Next.js 13+ App Router provides first-class SEO primitives that eliminate the need for third-party libraries. Using framework-native solutions ensures better performance (Edge Runtime support), automatic caching, and future compatibility with Next.js updates.

## Common Pitfalls

### Pitfall 1: Private Content Appearing in Search Results
**What goes wrong:** Scan result pages get indexed by Google, exposing user's private security findings in search results
**Why it happens:** Next.js default is to index all pages unless explicitly configured otherwise
**How to avoid:** Always set `robots: { index: false, follow: false }` in generateMetadata for user-generated content pages
**Warning signs:**
- Search for "site:shipsecure.ai results" returns result pages
- Google Search Console shows /results/* pages indexed
- Users report their scan URLs appearing in search

### Pitfall 2: Metadata Not Inheriting from Parent
**What goes wrong:** Child pages missing Open Graph images or other shared metadata
**Why it happens:** Metadata uses shallow merge - if child defines openGraph at all, it replaces parent's entirely
**How to avoid:** Either duplicate shared metadata in each page, or use generateMetadata with parent parameter to extend
**Warning signs:**
- Social sharing previews show broken images on some pages
- Title format inconsistent across pages
- Missing og:site_name on child pages

### Pitfall 3: Relative URLs in Open Graph Tags
**What goes wrong:** Open Graph images don't display when shared on social media
**Why it happens:** Social media crawlers need absolute URLs, relative paths like `/og-image.png` don't work
**How to avoid:** Set `metadataBase: new URL('https://shipsecure.ai')` in root layout, or use full URLs in metadata
**Warning signs:**
- Facebook Sharing Debugger shows "Could not retrieve image"
- Twitter Card Validator shows broken image
- Social shares show no preview image

### Pitfall 4: Title/Description Truncation
**What goes wrong:** Titles cut off mid-sentence in search results with "..." or descriptions don't show
**Why it happens:** Google truncates titles >60 characters (580 pixels), descriptions >160 characters (920 pixels on desktop)
**How to avoid:** Keep titles under 60 characters, descriptions under 155 characters, front-load important keywords
**Warning signs:**
- Search results show "..." in title
- Key brand name cut off in search snippet
- Google rewrites title completely (76% rewrite rate in 2025)

### Pitfall 5: Missing Canonical URLs Leading to Duplicate Content
**What goes wrong:** Google indexes multiple versions of the same page (http/https, www/non-www, trailing slash variants)
**Why it happens:** Next.js doesn't automatically set canonical tags - you must configure explicitly
**How to avoid:** Set `alternates.canonical` in metadata for all pages, ensure consistency with actual deployed URL
**Warning signs:**
- Google Search Console warns "Duplicate, Google chose different canonical than user"
- Multiple URLs for same content in search results
- PageRank diluted across duplicate pages

### Pitfall 6: Robots.txt Blocking Noindex Pages
**What goes wrong:** Pages meant to have noindex tag still appear in search results
**Why it happens:** If robots.txt blocks a page, crawlers never access it to see the noindex meta tag
**How to avoid:** Don't block pages in robots.txt if they need noindex - use only noindex for private content
**Warning signs:**
- Pages with robots: { index: false } still appearing in search
- Google Search Console shows "Blocked by robots.txt" instead of "Excluded by noindex"

### Pitfall 7: OG Image Generation Exceeding 500KB Bundle Limit
**What goes wrong:** ImageResponse fails to generate, social previews broken
**Why it happens:** ImageResponse has 500KB bundle limit including fonts, images, code
**How to avoid:** Use system fonts or single custom font, optimize any embedded images, keep JSX simple
**Warning signs:**
- Build warnings about route segment size
- OG image endpoints returning 500 errors
- Social media crawlers time out when fetching image

### Pitfall 8: Character Limits in 2026 Standards
**What goes wrong:** Meta descriptions or titles don't display as expected in search results
**Why it happens:** Updated 2026 character limits differ from older SEO advice
**How to avoid:** Follow 2026 limits - titles: 50-60 chars (580px), descriptions: 150-160 chars (920px desktop, 680px mobile ~120 chars)
**Warning signs:**
- Descriptions truncated earlier than expected
- Mobile previews cut off significantly more than desktop
- Inconsistent display across different search engines

## Code Examples

Verified patterns from official sources:

### Example 1: Complete Landing Page Metadata
```typescript
// Source: https://nextjs.org/docs/app/api-reference/functions/generate-metadata
// app/page.tsx
import type { Metadata } from 'next'

export const metadata: Metadata = {
  metadataBase: new URL('https://shipsecure.ai'),
  title: 'ShipSecure - Security Scanning for Vibe-Coded Apps',
  description: 'Ship fast, stay safe. Free security scanning for AI-generated web applications. Catch security flaws before they become breaches.',
  keywords: ['security scanning', 'AI apps', 'vibe coding', 'web security', 'vulnerability scanner'],
  authors: [{ name: 'ShipSecure' }],
  alternates: {
    canonical: '/',
  },
  openGraph: {
    title: 'ShipSecure - Security Scanning for Vibe-Coded Apps',
    description: 'Ship fast, stay safe. Free security scanning for AI-generated web applications.',
    url: 'https://shipsecure.ai',
    siteName: 'ShipSecure',
    images: [
      {
        url: '/og-image.png', // Resolves to https://shipsecure.ai/og-image.png with metadataBase
        width: 1200,
        height: 630,
        alt: 'ShipSecure - Security Scanning for Vibe-Coded Apps',
      },
    ],
    locale: 'en_US',
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'ShipSecure - Security Scanning for Vibe-Coded Apps',
    description: 'Ship fast, stay safe. Free security scanning for AI-generated web applications.',
    images: ['/og-image.png'],
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      'max-image-preview': 'large',
      'max-snippet': -1,
    },
  },
}

export default function Home() {
  // Page content
}
```

### Example 2: Dynamic Scan Results Page with Noindex
```typescript
// Source: https://nextjs.org/docs/app/api-reference/functions/generate-metadata
// app/results/[token]/page.tsx
import type { Metadata } from 'next'

export async function generateMetadata({
  params
}: {
  params: Promise<{ token: string }>
}): Promise<Metadata> {
  const { token } = await params

  try {
    const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:3000'
    const res = await fetch(`${BACKEND_URL}/api/v1/results/${token}`, {
      cache: 'no-store',
    })

    if (!res.ok) {
      return {
        title: 'Results Not Found - ShipSecure',
        robots: { index: false, follow: false },
      }
    }

    const data = await res.json()

    return {
      title: `Security Scan: ${data.score || 'In Progress'} Grade - ShipSecure`,
      description: `Security scan results for ${data.target_url}`,
      robots: {
        index: false,        // CRITICAL: Private content
        follow: false,
        nocache: true,
      },
      // Explicitly no OpenGraph - don't want results previewed in social shares
    }
  } catch (error) {
    return {
      title: 'Results Not Found - ShipSecure',
      robots: { index: false, follow: false },
    }
  }
}
```

### Example 3: Payment Success Page Metadata
```typescript
// Source: https://nextjs.org/docs/app/api-reference/functions/generate-metadata
// app/payment/success/page.tsx
import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'Payment Successful - ShipSecure',
  description: 'Your deep security audit is now processing. You'll receive an email with your PDF report when complete.',
  robots: {
    index: false,  // Transactional page, no SEO value
    follow: true,  // Can follow links back to main site
  },
}

export default function PaymentSuccessPage() {
  // Page content
}
```

### Example 4: Dynamic Sitemap
```typescript
// Source: https://nextjs.org/docs/app/api-reference/file-conventions/metadata/sitemap
// app/sitemap.ts
import type { MetadataRoute } from 'next'

export default function sitemap(): MetadataRoute.Sitemap {
  const baseUrl = 'https://shipsecure.ai'

  return [
    {
      url: baseUrl,
      lastModified: new Date(),
      changeFrequency: 'daily',
      priority: 1.0,
    },
    {
      url: `${baseUrl}/pricing`,
      lastModified: new Date(),
      changeFrequency: 'weekly',
      priority: 0.8,
    },
    // Note: Do NOT include /results/* or /scan/* (private/dynamic content)
    // Note: Do NOT include /payment/* (transactional pages)
  ]
}
```

### Example 5: Robots.txt Generation
```typescript
// Source: https://nextjs.org/docs/app/api-reference/file-conventions/metadata/robots
// app/robots.ts
import type { MetadataRoute } from 'next'

export default function robots(): MetadataRoute.Robots {
  return {
    rules: {
      userAgent: '*',
      allow: '/',
      disallow: [
        '/results/',     // Private scan results
        '/scan/',        // In-progress scans
        '/api/',         // API endpoints
        '/payment/',     // Transactional pages
      ],
    },
    sitemap: 'https://shipsecure.ai/sitemap.xml',
  }
}
```

### Example 6: JSON-LD Structured Data in Landing Page
```typescript
// Source: https://schema.org/Organization, https://schema.org/SoftwareApplication
// app/page.tsx
export default function Home() {
  const organizationSchema = {
    '@context': 'https://schema.org',
    '@type': 'Organization',
    name: 'ShipSecure',
    url: 'https://shipsecure.ai',
    description: 'Security scanning for AI-generated web applications',
  }

  const softwareSchema = {
    '@context': 'https://schema.org',
    '@type': 'SoftwareApplication',
    name: 'ShipSecure',
    applicationCategory: 'SecurityApplication',
    operatingSystem: 'Web',
    offers: {
      '@type': 'Offer',
      price: '0',
      priceCurrency: 'USD',
      description: 'Free security scanning',
    },
    aggregateRating: undefined, // Add when you have user reviews
    url: 'https://shipsecure.ai',
    description: 'Free security scanning for vibe-coded web apps. Catch security flaws before they become breaches.',
    featureList: [
      'Security header analysis',
      'TLS configuration scanning',
      'Exposed file detection',
      'JavaScript secret scanning',
    ],
  }

  return (
    <div>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(organizationSchema) }}
      />
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(softwareSchema) }}
      />
      {/* Page content */}
    </div>
  )
}
```

### Example 7: Static OG Image (Simplest Approach)
```typescript
// Source: https://nextjs.org/docs/app/api-reference/file-conventions/metadata/opengraph-image
// app/opengraph-image.tsx
import { ImageResponse } from 'next/og'

export const runtime = 'edge'
export const alt = 'ShipSecure - Security Scanning for Vibe-Coded Apps'
export const size = { width: 1200, height: 630 }
export const contentType = 'image/png'

export default async function Image() {
  return new ImageResponse(
    (
      <div
        style={{
          fontSize: 64,
          background: 'linear-gradient(to bottom, #2563eb, #1e40af)',
          width: '100%',
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          color: 'white',
          fontFamily: 'system-ui, sans-serif',
        }}
      >
        <div style={{ fontSize: 80, fontWeight: 'bold', marginBottom: 20 }}>
          ShipSecure
        </div>
        <div style={{ fontSize: 36, opacity: 0.9 }}>
          Security Scanning for Vibe-Coded Apps
        </div>
      </div>
    ),
    {
      ...size,
    }
  )
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| next-seo package | Built-in Metadata API | Next.js 13 (Oct 2022) | No external dependency, type-safe, SSR by default |
| Pages Router Head component | App Router metadata exports | Next.js 13 (Oct 2022) | Server-side metadata, automatic deduplication |
| @vercel/og package | next/og (ImageResponse) | Next.js 14 (Oct 2023) | Framework-integrated, better caching |
| Static sitemap.xml | Dynamic sitemap.ts | Next.js 13.3 (Apr 2023) | Auto-updates, type-safe, database-driven |
| Manual robots.txt | Dynamic robots.ts | Next.js 13.3 (Apr 2023) | Environment-aware rules, type-safe |
| Twitter Card vs OG separation | OG fallback standard | ~2020 | Twitter uses OG tags if twitter:* missing |
| Title limit 70 chars | Title limit 60 chars (580px) | 2025-2026 | Google truncates earlier now |
| Description 200 chars | Description 155-160 chars | 2026 | Mobile limits stricter (120 chars) |

**Deprecated/outdated:**
- **next-seo:** Still maintained but unnecessary for App Router - built-in Metadata API is superior
- **Manual <Head> manipulation:** Pages Router approach, replaced by metadata exports in App Router
- **Assuming Twitter doesn't use OG tags:** Twitter falls back to OG tags if twitter:* tags missing, but include both for full control
- **Character limits from pre-2025 SEO guides:** Updated limits in 2026 are stricter, especially mobile

## Open Questions

1. **Static vs Dynamic OG Images for ShipSecure**
   - What we know: Static opengraph-image.tsx is simplest, API route allows dynamic branding
   - What's unclear: Do we want scan result grade in OG image when sharing result pages? (Conflicts with noindex approach)
   - Recommendation: Start with static OG image for landing page only. Results pages shouldn't be shared (noindex), so no OG tags needed there.

2. **Additional Pages Beyond Requirements**
   - What we know: Requirements mention landing, results, payment success pages
   - What's unclear: Are there pricing, about, or other marketing pages that need metadata?
   - Recommendation: Implement for documented pages first, pattern is reusable for future pages

3. **Multilingual/i18n Support**
   - What we know: Metadata API supports `alternates.languages` for hreflang tags
   - What's unclear: Is internationalization planned for ShipSecure?
   - Recommendation: Skip for MVP (Phase 9), but architecture supports it via alternates.languages if needed later

4. **Google Rich Results Eligibility**
   - What we know: Organization and SoftwareApplication schemas exist in schema.org
   - What's unclear: Does Google actually show rich results for security SaaS tools? Not all schemas trigger rich results
   - Recommendation: Implement JSON-LD anyway (good practice), validate with Rich Results Test, but don't expect fancy search result cards - security tools rarely qualify

## Sources

### Primary (HIGH confidence)
- [Next.js generateMetadata Documentation](https://nextjs.org/docs/app/api-reference/functions/generate-metadata) - Complete Metadata API reference, verified January 2026
- [Next.js Sitemap Documentation](https://nextjs.org/docs/app/api-reference/file-conventions/metadata/sitemap) - Dynamic sitemap generation
- [Next.js Robots.txt Documentation](https://nextjs.org/docs/app/api-reference/file-conventions/metadata/robots) - Robots.txt configuration
- [Next.js ImageResponse Documentation](https://nextjs.org/docs/app/api-reference/functions/image-response) - OG image generation
- [Schema.org Organization](https://schema.org/Organization) - Organization schema properties
- [Schema.org SoftwareApplication](https://schema.org/SoftwareApplication) - SoftwareApplication schema properties
- [Google Rich Results Test](https://search.google.com/test/rich-results) - Official validation tool

### Secondary (MEDIUM confidence)
- [Next.js SEO Optimization Guide (2026 Edition)](https://www.djamware.com/post/697a19b07c935b6bb054313e/next-js-seo-optimization-guide--2026-edition) - Current best practices
- [Open Graph Image Guide](https://www.opengraph.xyz/blog/the-ultimate-guide-to-open-graph-images) - OG image specifications
- [Meta Title and Description Character Limit (2026 Guidelines)](https://www.wscubetech.com/blog/meta-title-description-length/) - Updated character limits
- [Google Block Search Indexing with noindex](https://developers.google.com/search/docs/crawling-indexing/block-indexing) - Official noindex guidance
- [JavaScript SEO In 2026: 7 Mistakes Killing Your Rankings](https://zumeirah.com/javascript-seo-in-2026/) - Common pitfalls for JS frameworks

### Tertiary (LOW confidence)
- Community examples from DEV.to and Medium - Used for pattern inspiration only, cross-verified with official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Next.js 16.1.6 official docs verified, framework-native approach
- Architecture: HIGH - Official docs provide complete examples, verified on codebase structure
- Pitfalls: MEDIUM-HIGH - Combination of official warnings + community experience reports
- OG image best practices: MEDIUM - Industry standards but not officially mandated by platforms
- Character limits: MEDIUM - 2026 guides cite current behavior but Google behavior changes frequently

**Research date:** 2026-02-08
**Valid until:** ~60 days (SEO practices stable, but Google algorithm updates can change priorities)
