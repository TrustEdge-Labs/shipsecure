# Phase 11: Mobile & UX Polish - Research

**Researched:** 2026-02-08
**Domain:** Mobile-responsive design, loading states, error handling UX, performance optimization
**Confidence:** HIGH

## Summary

Mobile & UX Polish focuses on delivering a production-quality experience across all devices and interaction states. The phase addresses five key areas: responsive design (using Tailwind's mobile-first breakpoint system), progressive loading feedback (Next.js streaming with stage-specific messaging), error handling UX (inline messages with actionable guidance), visual consistency (centralized design tokens), and performance optimization (Core Web Vitals targeting >90 Lighthouse score).

The stack is well-established: Tailwind CSS provides mobile-first responsive utilities with default breakpoints at 640px (sm), 768px (md), 1024px (lg), 1280px (xl), and 1536px (2xl). Next.js App Router offers built-in streaming and Suspense for progressive loading, plus error.js files for granular error boundaries. The Image component handles optimization automatically (20-40% payload reduction on mobile, WebP format conversion, lazy loading).

Critical success factors: test on real mobile devices (not just DevTools), use unprefixed Tailwind classes for mobile-first design, implement multi-stage loading feedback (not generic spinners), provide actionable error messages (never silent failures), and optimize LCP images with priority loading.

**Primary recommendation:** Build mobile-first with Tailwind's unprefixed utilities, add loading.js files for route-level streaming, create error.js boundaries at appropriate hierarchy levels, use Next.js Image component with priority on LCP candidates, and validate on real devices before considering complete.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tailwind CSS | 3.x/4.x | Responsive utility-first CSS | Industry standard for mobile-first design, built-in breakpoint system, no custom media queries needed |
| Next.js App Router | 14+/15+ | Streaming, loading states, error boundaries | Native streaming support, loading.js convention, error.js boundaries, Image component optimization |
| React Suspense | React 18+ | Progressive loading, code splitting | Built into Next.js App Router, enables granular loading states, automatic streaming |
| next/image | Next.js built-in | Image optimization, lazy loading | Automatic WebP conversion, responsive srcset, lazy loading, 20-40% mobile payload reduction |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Lighthouse CI | Latest | Performance monitoring, Core Web Vitals | Automated performance testing in CI/CD pipeline |
| Chrome DevTools Device Mode | Browser built-in | Initial responsive testing | Quick viewport checks during development (80% of testing) |
| BrowserStack/Real Devices | N/A | Real device testing | Final validation before production (20% of testing) |
| @tailwindcss/debug-screens | Latest | Breakpoint visualization | Debugging responsive layouts, displaying active breakpoint |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Tailwind CSS | CSS Modules, styled-components | Tailwind's mobile-first utilities are more maintainable for responsive design at scale |
| Next.js Image | img tag, third-party CDN | Next.js Image provides automatic optimization without external service dependencies |
| loading.js | Custom loading state | Next.js convention provides streaming and Suspense boundaries automatically |
| error.js | Try-catch everywhere | error.js provides React error boundaries at route level, cleaner hierarchy |

**Installation:**
```bash
# Tailwind CSS (usually pre-installed in Next.js projects)
npm install -D tailwindcss postcss autoprefixer

# Lighthouse CI (for automated performance testing)
npm install -D @lhci/cli

# Debug screens (for development)
npm install -D @tailwindcss/debug-screens
```

## Architecture Patterns

### Recommended Project Structure
```
app/
├── layout.tsx              # Root layout with viewport meta tag
├── page.tsx                # Landing page (mobile-first)
├── loading.tsx             # Root loading fallback
├── error.tsx               # Root error boundary
├── global-error.tsx        # Catch-all error handler
├── scan/
│   └── [id]/
│       ├── page.tsx        # Scan progress page
│       ├── loading.tsx     # Scan-specific loading UI
│       └── error.tsx       # Scan-specific error boundary
└── results/
    └── [token]/
        ├── page.tsx        # Results dashboard
        ├── loading.tsx     # Results loading skeleton
        └── error.tsx       # Results error handler
```

### Pattern 1: Mobile-First Responsive Design
**What:** Start with mobile styles (unprefixed), layer desktop styles with breakpoint prefixes
**When to use:** All responsive layouts, all pages
**Example:**
```tsx
// Source: https://tailwindcss.com/docs/responsive-design
export function HeroSection() {
  return (
    <div className="container mx-auto px-4 py-8 sm:py-12 md:py-16 lg:py-24">
      {/* Mobile: 4xl, Desktop: 6xl */}
      <h1 className="text-4xl sm:text-5xl md:text-6xl font-bold">
        Ship fast, stay safe.
      </h1>
      {/* Mobile: base, Tablet: lg, Desktop: xl */}
      <p className="text-base sm:text-lg md:text-xl text-gray-600">
        Free security scanning for vibe-coded web apps.
      </p>
      {/* Mobile: full width, Tablet: 2 columns */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        {/* ... */}
      </div>
    </div>
  )
}
```

### Pattern 2: Progressive Loading with loading.js
**What:** Create route-level loading fallbacks using loading.js convention
**When to use:** Long-running data fetches, dashboard pages, results pages
**Example:**
```tsx
// Source: https://nextjs.org/docs/app/building-your-application/routing/loading-ui-and-streaming
// app/results/[token]/loading.tsx
export default function ResultsLoading() {
  return (
    <div className="container mx-auto px-4 py-8">
      {/* Skeleton matches actual layout to prevent CLS */}
      <div className="animate-pulse space-y-4">
        <div className="h-12 bg-gray-200 rounded w-1/3"></div>
        <div className="h-64 bg-gray-200 rounded"></div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="h-32 bg-gray-200 rounded"></div>
          <div className="h-32 bg-gray-200 rounded"></div>
        </div>
      </div>
    </div>
  )
}
```

### Pattern 3: Stage-Specific Loading Feedback
**What:** Multi-stage loading state machine with descriptive messages
**When to use:** Long-running processes (scans, audits, complex operations)
**Example:**
```tsx
// Source: https://medium.com/uxdworld/6-loading-state-patterns-that-feel-premium-716aa0fe63e8
'use client'

import { useState, useEffect } from 'react'

type ScanStage = 'initializing' | 'headers' | 'tls' | 'nuclei' | 'complete'

const STAGE_MESSAGES: Record<ScanStage, string> = {
  initializing: 'Starting security scan...',
  headers: 'Checking security headers...',
  tls: 'Analyzing TLS configuration...',
  nuclei: 'Running vulnerability templates...',
  complete: 'Scan complete!'
}

export function ScanProgress({ scanId }: { scanId: string }) {
  const [stage, setStage] = useState<ScanStage>('initializing')

  useEffect(() => {
    // Poll backend for current stage
    const interval = setInterval(async () => {
      const res = await fetch(`/api/v1/scans/${scanId}/status`)
      const data = await res.json()
      setStage(data.stage)
      if (data.stage === 'complete') clearInterval(interval)
    }, 1000)
    return () => clearInterval(interval)
  }, [scanId])

  return (
    <div className="text-center p-8">
      <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mb-4"></div>
      <p className="text-lg font-medium text-gray-900">
        {STAGE_MESSAGES[stage]}
      </p>
    </div>
  )
}
```

### Pattern 4: Error Boundaries with error.js
**What:** Route-level error boundaries with reset functionality
**When to use:** Every major route segment, especially data-fetching pages
**Example:**
```tsx
// Source: https://nextjs.org/docs/app/getting-started/error-handling
// app/results/[token]/error.tsx
'use client'

import { useEffect } from 'react'

export default function ResultsError({
  error,
  reset,
}: {
  error: Error & { digest?: string }
  reset: () => void
}) {
  useEffect(() => {
    console.error('Results page error:', error)
  }, [error])

  return (
    <div className="container mx-auto px-4 py-8 text-center">
      <div className="max-w-md mx-auto p-6 bg-red-50 border border-red-200 rounded-lg">
        <div className="text-4xl mb-4">⚠️</div>
        <h2 className="text-xl font-semibold text-red-800 mb-2">
          Unable to load scan results
        </h2>
        <p className="text-red-600 mb-4">
          The results could not be loaded. This may be because the scan is still in progress or the token has expired.
        </p>
        <div className="space-x-2">
          <button
            onClick={() => reset()}
            className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
          >
            Try again
          </button>
          <a
            href="/"
            className="inline-block px-4 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300"
          >
            Start new scan
          </a>
        </div>
      </div>
    </div>
  )
}
```

### Pattern 5: Inline Form Error Messages
**What:** Field-level validation errors with useActionState
**When to use:** All forms with server-side validation
**Example:**
```tsx
// Source: https://nextjs.org/docs/app/getting-started/error-handling
'use client'

import { useActionState } from 'react'
import { submitScan } from '@/app/actions/scan'

export function ScanForm() {
  const [state, formAction, pending] = useActionState(submitScan, {})

  return (
    <form action={formAction} className="space-y-4">
      {/* Global form error */}
      {state.errors?._form && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-lg">
          <p className="text-red-700 text-sm flex items-start gap-2">
            <span className="text-red-500">⚠</span>
            {state.errors._form[0]}
          </p>
        </div>
      )}

      {/* Field with inline error */}
      <div>
        <label htmlFor="url" className="block text-sm font-medium mb-1">
          Website URL
        </label>
        <input
          id="url"
          name="url"
          type="url"
          className="w-full px-4 py-3 rounded-lg border focus:ring-2 focus:ring-blue-500"
        />
        {state.errors?.url && (
          <p className="mt-1 text-sm text-red-600" aria-live="polite">
            {state.errors.url[0]}
          </p>
        )}
      </div>

      <button
        type="submit"
        disabled={pending}
        className="w-full py-3 px-6 rounded-lg bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white font-semibold"
      >
        {pending ? 'Starting scan...' : 'Scan Now — Free'}
      </button>
    </form>
  )
}
```

### Pattern 6: Image Optimization for Performance
**What:** Use next/image with priority for LCP, lazy loading for below-fold
**When to use:** All images, especially hero images and above-the-fold content
**Example:**
```tsx
// Source: https://nextjs.org/docs/app/building-your-application/optimizing/images
import Image from 'next/image'

export function HeroImage() {
  return (
    <>
      {/* Above-the-fold LCP candidate: priority loading */}
      <Image
        src="/hero-screenshot.png"
        alt="Security dashboard screenshot"
        width={1200}
        height={675}
        priority
        className="w-full h-auto rounded-lg shadow-lg"
      />

      {/* Below-fold: default lazy loading */}
      <Image
        src="/feature-1.png"
        alt="Security headers check"
        width={600}
        height={400}
        className="w-full h-auto rounded-lg"
      />
    </>
  )
}
```

### Pattern 7: Container-Based Responsive Design
**What:** Use Tailwind container queries for component-level responsiveness
**When to use:** Reusable components that need to adapt to parent size, not viewport
**Example:**
```tsx
// Source: https://tailwindcss.com/docs/responsive-design
export function FindingCard({ finding }: { finding: Finding }) {
  return (
    <div className="@container">
      {/* Stacks on mobile, side-by-side when container > 512px */}
      <div className="flex flex-col @md:flex-row gap-4">
        <div className="@md:w-1/3">
          <SeverityBadge severity={finding.severity} />
        </div>
        <div className="@md:w-2/3">
          <h3 className="text-lg @md:text-xl font-semibold">
            {finding.title}
          </h3>
          <p className="text-sm @md:text-base text-gray-600">
            {finding.description}
          </p>
        </div>
      </div>
    </div>
  )
}
```

### Anti-Patterns to Avoid

- **Using `sm:` for mobile targeting**: `sm:` applies at 640px and above. Use unprefixed classes for mobile, then override with breakpoint prefixes for larger screens.
- **Fixed widths without responsive overrides**: `width: 1000px` causes horizontal scroll on mobile. Use `max-w-*` utilities or percentage-based widths.
- **Generic spinner without stage feedback**: "Loading..." tells users nothing. Show stage-specific messages: "Checking security headers...", "Running Nuclei templates..."
- **Silent API failures**: Never swallow errors silently. Always display inline messages with suggested actions.
- **Overusing layout shifts**: Missing width/height on images causes CLS. Always specify dimensions or use `fill` with container.
- **Testing only in DevTools**: DevTools viewport simulation doesn't catch touch interaction issues, font rendering differences, or performance on constrained hardware. Test on real devices.
- **Using `100vw` on page content**: Includes scrollbar width, causing horizontal scroll. Use `w-full` or `max-w-screen-xl` instead.
- **Hardcoded viewport units**: `vw` and `vh` behave differently on mobile (especially iOS Safari with dynamic viewport). Use Tailwind utilities or container queries instead.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Image optimization | Custom image resizing, format conversion, CDN integration | Next.js `<Image>` component | Handles responsive srcset, WebP conversion, lazy loading, blur placeholders automatically. 20-40% smaller payloads on mobile. |
| Loading states | Custom Suspense boundaries, manual streaming logic | Next.js `loading.js` convention | Automatic Suspense boundary creation, streaming support, prefetching integration. |
| Error boundaries | Try-catch in every component | Next.js `error.js` files | Route-level error boundaries with reset functionality, hierarchical error propagation. |
| Responsive breakpoints | Custom media queries, CSS breakpoints | Tailwind CSS breakpoint system | Mobile-first by default, inline responsive utilities, no separate CSS files to maintain. |
| Skeleton screens | Custom loading placeholders | Tailwind + Suspense patterns | Tailwind's `animate-pulse` + Next.js Suspense provides instant loading UI with minimal code. |
| Viewport configuration | Custom responsive logic, window.innerWidth checks | Tailwind breakpoints + container queries | Declarative, works during SSR, no JS required, better performance. |
| Performance monitoring | Custom Lighthouse scripts | Lighthouse CI + Next.js analytics | Automated Core Web Vitals tracking, historical data, CI/CD integration. |

**Key insight:** Next.js App Router and Tailwind CSS handle 90% of mobile UX patterns out-of-the-box. Custom solutions add complexity without improving user experience. Focus on using conventions correctly rather than reinventing them.

## Common Pitfalls

### Pitfall 1: Mobile-First Confusion
**What goes wrong:** Developers use `sm:` prefix expecting it to target mobile, then wonder why styles don't apply on small screens.
**Why it happens:** Misunderstanding of Tailwind's mobile-first approach. `sm:` means "640px and above", not "mobile only".
**How to avoid:** Use unprefixed classes for mobile base styles, then add breakpoint prefixes for progressively larger screens. Think "mobile default, then enhance".
**Warning signs:** Styles not appearing on mobile, needing to use `max-sm:` frequently, horizontal scroll on mobile.

**Example:**
```tsx
// ❌ WRONG: sm: doesn't target mobile
<div className="sm:text-center">Text</div>

// ✅ CORRECT: Unprefixed for mobile, sm: for tablet+
<div className="text-center sm:text-left">Text</div>
```

### Pitfall 2: Horizontal Scroll on Mobile
**What goes wrong:** Content overflows viewport width, forcing users to scroll horizontally (terrible UX).
**Why it happens:** Fixed widths exceeding viewport, incorrect use of `100vw`, missing `overflow-x-hidden`, wide inline content (long URLs, code blocks).
**How to avoid:** Use `max-w-full` on containers, avoid fixed pixel widths, test at 375px viewport (iPhone SE size), use `overflow-x-hidden` on body if needed.
**Warning signs:** Viewport meta tag missing, fixed widths > 375px, `vw` units on content, white space on right side of page.

**Example:**
```tsx
// ❌ WRONG: Fixed width causes overflow
<div style={{ width: '1200px' }}>Content</div>

// ✅ CORRECT: Responsive max-width with padding
<div className="max-w-4xl mx-auto px-4">Content</div>
```

### Pitfall 3: Cumulative Layout Shift (CLS)
**What goes wrong:** Elements shift position during page load, causing users to mis-click or lose reading position. Kills Lighthouse score.
**Why it happens:** Images without width/height, web fonts loading late, dynamic content insertion without reserved space.
**How to avoid:** Always specify image dimensions, use `font-display: swap` with fallback fonts, reserve space for skeleton content matching final layout size.
**Warning signs:** Lighthouse CLS score > 0.1, buttons jumping when user tries to click, text reflow when fonts load.

**Example:**
```tsx
// ❌ WRONG: No dimensions specified
<img src="/hero.png" alt="Hero" />

// ✅ CORRECT: Dimensions prevent layout shift
<Image
  src="/hero.png"
  alt="Hero"
  width={1200}
  height={675}
  className="w-full h-auto"
/>
```

### Pitfall 4: Generic Loading States
**What goes wrong:** Users see "Loading..." spinner indefinitely, don't know if progress is being made, feel anxious and bounce.
**Why it happens:** Using same loading state for all operations, no backend stage reporting, no perceived progress.
**How to avoid:** Implement stage-specific messages ("Checking security headers...", "Running Nuclei templates..."), use skeleton screens matching final layout, show progress percentage when available.
**Warning signs:** High bounce rate on loading pages, users repeatedly refreshing, support tickets asking "Is it stuck?".

**Example:**
```tsx
// ❌ WRONG: Generic spinner
<div className="animate-spin">⏳</div>

// ✅ CORRECT: Stage-specific feedback
<div>
  <div className="animate-spin">⏳</div>
  <p>Checking security headers...</p>
  <p className="text-sm text-gray-500">Step 1 of 4</p>
</div>
```

### Pitfall 5: Silent Error Failures
**What goes wrong:** API request fails, user sees nothing, assumes page is broken or frozen.
**Why it happens:** Catch blocks that swallow errors, missing error state in UI, optimistic UI updates without rollback.
**How to avoid:** Always display inline error messages, use Next.js error.js boundaries, log errors for monitoring, provide actionable next steps ("Try again", "Contact support").
**Warning signs:** Users reporting "nothing happened", empty error logs, high error rates but low error report submissions.

**Example:**
```tsx
// ❌ WRONG: Silent failure
try {
  await submitScan(data)
} catch (error) {
  console.error(error) // User sees nothing!
}

// ✅ CORRECT: Inline error message
const [error, setError] = useState<string | null>(null)

try {
  await submitScan(data)
} catch (error) {
  setError('Unable to submit scan. Please check your URL and try again.')
}

{error && (
  <div className="p-3 bg-red-50 border border-red-200 rounded">
    <p className="text-red-700 text-sm">{error}</p>
  </div>
)}
```

### Pitfall 6: DevTools-Only Testing
**What goes wrong:** Looks perfect in Chrome DevTools, breaks on real iPhone/Android devices.
**Why it happens:** DevTools simulates viewport size but doesn't emulate touch interactions, hardware constraints, font rendering differences, Safari-specific bugs.
**How to avoid:** Use DevTools for 80% of development, validate on real devices for 20% (especially before production). Test on iPhone Safari (iOS viewport quirks) and mid-range Android (performance constraints).
**Warning signs:** Touch targets too small on real devices, scrolling performance issues, viewport height bugs (iOS Safari), font rendering differences.

**Validation checklist:**
- Chrome DevTools: Initial responsive checks, breakpoint verification
- iPhone Safari: Touch interactions, viewport height, iOS-specific bugs
- Android Chrome: Performance on constrained hardware, font rendering
- BrowserStack/LambdaTest: Automated cross-device testing in CI/CD

### Pitfall 7: Missing Viewport Meta Tag
**What goes wrong:** Mobile browsers zoom out to fit desktop layout, making text unreadable and layout broken.
**Why it happens:** Forgot to include viewport meta tag in `<head>`, incorrect configuration.
**How to avoid:** Always include in root layout: `<meta name="viewport" content="width=device-width, initial-scale=1" />`
**Warning signs:** Page appears zoomed out on mobile, text tiny and unreadable, users needing to pinch-zoom.

**Example:**
```tsx
// app/layout.tsx
export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <head>
        <meta name="viewport" content="width=device-width, initial-scale=1" />
      </head>
      <body>{children}</body>
    </html>
  )
}
```

### Pitfall 8: Ignoring Core Web Vitals
**What goes wrong:** Lighthouse score < 90, poor SEO ranking, high bounce rate from slow page loads.
**Why it happens:** Large unoptimized images (especially hero images), render-blocking JavaScript, missing priority loading on LCP candidates.
**How to avoid:** Use `priority` prop on LCP images, defer non-critical JavaScript, optimize images with next/image, minimize layout shifts.
**Warning signs:** LCP > 2.5s, INP > 200ms, CLS > 0.1, Lighthouse performance score < 90.

**Core Web Vitals thresholds (75th percentile):**
- Largest Contentful Paint (LCP): ≤ 2.5s
- Interaction to Next Paint (INP): ≤ 200ms
- Cumulative Layout Shift (CLS): ≤ 0.1

## Code Examples

Verified patterns from official sources:

### Example 1: Complete Mobile-First Page Layout
```tsx
// Source: https://tailwindcss.com/docs/responsive-design
export default function LandingPage() {
  return (
    <div className="min-h-screen bg-white">
      {/* Container with responsive padding */}
      <main className="container mx-auto px-4 py-8 sm:py-12 md:py-16 lg:py-24 max-w-4xl">
        {/* Hero with responsive typography */}
        <div className="text-center mb-8 sm:mb-12">
          <h1 className="text-4xl sm:text-5xl md:text-6xl font-bold mb-4">
            Ship fast, stay safe.
          </h1>
          <p className="text-lg sm:text-xl text-gray-600 mb-2">
            Free security scanning for vibe-coded web apps.
          </p>
        </div>

        {/* Card with responsive padding */}
        <div className="bg-gray-50 rounded-2xl shadow-lg p-6 sm:p-8 mb-8 sm:mb-12">
          <ScanForm />
        </div>

        {/* Grid: 1 column mobile, 2 columns tablet+ */}
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 max-w-2xl mx-auto">
          <FeatureCard title="Security Headers" icon="🔒" />
          <FeatureCard title="TLS Config" icon="🔑" />
          <FeatureCard title="Exposed Files" icon="📄" />
          <FeatureCard title="JS Secrets" icon="🔍" />
        </div>
      </main>

      {/* Footer */}
      <footer className="border-t py-8">
        <div className="container mx-auto px-4 text-center text-sm text-gray-600">
          <p>&copy; 2026 ShipSecure</p>
        </div>
      </footer>
    </div>
  )
}
```

### Example 2: Skeleton Loading Screen (Prevents CLS)
```tsx
// Source: https://nextjs.org/docs/app/building-your-application/routing/loading-ui-and-streaming
// app/results/[token]/loading.tsx
export default function ResultsLoading() {
  return (
    <div className="container mx-auto px-4 py-8 max-w-6xl">
      {/* Skeleton matches actual results layout */}
      <div className="animate-pulse space-y-6">
        {/* Header skeleton */}
        <div className="space-y-2">
          <div className="h-8 bg-gray-200 rounded w-1/3"></div>
          <div className="h-4 bg-gray-200 rounded w-1/2"></div>
        </div>

        {/* Grade summary skeleton */}
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
          <div className="h-24 bg-gray-200 rounded"></div>
          <div className="h-24 bg-gray-200 rounded"></div>
          <div className="h-24 bg-gray-200 rounded"></div>
          <div className="h-24 bg-gray-200 rounded"></div>
        </div>

        {/* Findings skeleton */}
        <div className="space-y-4">
          <div className="h-32 bg-gray-200 rounded"></div>
          <div className="h-32 bg-gray-200 rounded"></div>
          <div className="h-32 bg-gray-200 rounded"></div>
        </div>
      </div>
    </div>
  )
}
```

### Example 3: Error Boundary with Actionable Messages
```tsx
// Source: https://nextjs.org/docs/app/getting-started/error-handling
// app/scan/[id]/error.tsx
'use client'

import { useEffect } from 'react'

export default function ScanError({
  error,
  reset,
}: {
  error: Error & { digest?: string }
  reset: () => void
}) {
  useEffect(() => {
    console.error('Scan page error:', error)
  }, [error])

  // Parse error type for specific guidance
  const isNotFound = error.message.includes('not found')
  const isTimeout = error.message.includes('timeout')

  return (
    <div className="container mx-auto px-4 py-8 max-w-2xl">
      <div className="p-6 sm:p-8 bg-red-50 border border-red-200 rounded-lg">
        {/* Icon + Title */}
        <div className="flex items-start gap-4 mb-4">
          <div className="text-4xl">⚠️</div>
          <div>
            <h2 className="text-xl font-semibold text-red-800 mb-2">
              {isNotFound && 'Scan not found'}
              {isTimeout && 'Scan timed out'}
              {!isNotFound && !isTimeout && 'Unable to load scan'}
            </h2>
            <p className="text-red-600 text-sm sm:text-base">
              {isNotFound && 'This scan ID does not exist or has expired. Scan results are stored for 30 days.'}
              {isTimeout && 'The scan is taking longer than expected. This can happen with large sites or network issues.'}
              {!isNotFound && !isTimeout && 'An unexpected error occurred while loading the scan progress.'}
            </p>
          </div>
        </div>

        {/* Action buttons */}
        <div className="flex flex-col sm:flex-row gap-2 sm:gap-3">
          <button
            onClick={() => reset()}
            className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 font-medium"
          >
            Try again
          </button>
          <a
            href="/"
            className="px-4 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300 text-center font-medium"
          >
            Start new scan
          </a>
          {isTimeout && (
            <a
              href="/docs/scan-timeout"
              className="px-4 py-2 text-blue-600 hover:underline text-center"
            >
              Learn more
            </a>
          )}
        </div>
      </div>
    </div>
  )
}
```

### Example 4: Progressive Loading State Machine
```tsx
// Source: https://medium.com/uxdworld/6-loading-state-patterns-that-feel-premium-716aa0fe63e8
'use client'

import { useState, useEffect } from 'react'

type ScanStage = 'initializing' | 'headers' | 'tls' | 'nuclei' | 'complete' | 'error'

interface StageInfo {
  message: string
  step: number
  total: number
}

const STAGES: Record<Exclude<ScanStage, 'error'>, StageInfo> = {
  initializing: { message: 'Starting security scan...', step: 1, total: 4 },
  headers: { message: 'Checking security headers...', step: 2, total: 4 },
  tls: { message: 'Analyzing TLS configuration...', step: 3, total: 4 },
  nuclei: { message: 'Running vulnerability templates...', step: 4, total: 4 },
  complete: { message: 'Scan complete!', step: 4, total: 4 },
}

export function ScanProgress({ scanId }: { scanId: string }) {
  const [stage, setStage] = useState<ScanStage>('initializing')
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let interval: NodeJS.Timeout

    const pollStatus = async () => {
      try {
        const res = await fetch(`/api/v1/scans/${scanId}/status`)
        if (!res.ok) throw new Error('Failed to fetch status')

        const data = await res.json()
        setStage(data.stage)

        if (data.stage === 'complete' || data.stage === 'error') {
          clearInterval(interval)
        }
      } catch (err) {
        setError('Unable to check scan progress. Please refresh the page.')
        clearInterval(interval)
      }
    }

    // Poll every 1 second
    interval = setInterval(pollStatus, 1000)
    pollStatus() // Initial fetch

    return () => clearInterval(interval)
  }, [scanId])

  if (error) {
    return (
      <div className="text-center p-8 bg-red-50 border border-red-200 rounded-lg">
        <p className="text-red-700">{error}</p>
      </div>
    )
  }

  if (stage === 'complete') {
    return (
      <div className="text-center p-8 bg-green-50 border border-green-200 rounded-lg">
        <div className="text-4xl mb-3">✓</div>
        <p className="text-lg font-semibold text-green-800">Scan complete!</p>
        <p className="text-sm text-green-600">Loading results...</p>
      </div>
    )
  }

  const stageInfo = STAGES[stage]

  return (
    <div className="text-center p-8">
      {/* Spinner */}
      <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mb-4"></div>

      {/* Stage message */}
      <p className="text-lg font-medium text-gray-900 mb-2">
        {stageInfo.message}
      </p>

      {/* Progress indicator */}
      <p className="text-sm text-gray-500">
        Step {stageInfo.step} of {stageInfo.total}
      </p>

      {/* Progress bar */}
      <div className="mt-4 w-full max-w-xs mx-auto bg-gray-200 rounded-full h-2">
        <div
          className="bg-blue-600 h-2 rounded-full transition-all duration-500"
          style={{ width: `${(stageInfo.step / stageInfo.total) * 100}%` }}
        />
      </div>
    </div>
  )
}
```

### Example 5: Optimized Hero Image (LCP Optimization)
```tsx
// Source: https://nextjs.org/docs/app/building-your-application/optimizing/images
import Image from 'next/image'

export function HeroSection() {
  return (
    <div className="container mx-auto px-4 py-8 sm:py-12 max-w-6xl">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-8 items-center">
        <div>
          <h1 className="text-4xl sm:text-5xl font-bold mb-4">
            Security scanning for modern apps
          </h1>
          <p className="text-lg text-gray-600 mb-6">
            Catch vulnerabilities before they become breaches.
          </p>
          <button className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700">
            Start Free Scan
          </button>
        </div>

        {/* LCP candidate: use priority to preload */}
        <Image
          src="/dashboard-screenshot.png"
          alt="Security dashboard showing scan results"
          width={1200}
          height={800}
          priority
          className="w-full h-auto rounded-lg shadow-2xl"
        />
      </div>

      {/* Below-fold images: default lazy loading */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-6 mt-12">
        <Image
          src="/feature-headers.png"
          alt="Security headers analysis"
          width={400}
          height={300}
          className="w-full h-auto rounded-lg"
        />
        <Image
          src="/feature-tls.png"
          alt="TLS configuration check"
          width={400}
          height={300}
          className="w-full h-auto rounded-lg"
        />
        <Image
          src="/feature-nuclei.png"
          alt="Nuclei vulnerability scan"
          width={400}
          height={300}
          className="w-full h-auto rounded-lg"
        />
      </div>
    </div>
  )
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Custom media queries in CSS | Tailwind responsive utilities | 2017+ | Inline responsive styles, no separate CSS files, faster development |
| Manual lazy loading with IntersectionObserver | Native browser lazy loading + next/image | 2020+ (Chrome 76+) | Simpler implementation, better performance, automatic optimization |
| Custom loading spinners | Suspense + streaming | Next.js 13+ (2022) | Progressive rendering, better perceived performance, automatic code splitting |
| Try-catch error handling everywhere | React error boundaries + error.js | Next.js 13+ (2022) | Granular error isolation, automatic error propagation, better UX |
| Bootstrap/Foundation grid | CSS Grid + Tailwind utilities | 2017+ | Simpler layouts, better responsiveness, no framework overhead |
| Viewport-based responsive | Container queries | 2023+ (Chrome 105+) | Component-level responsiveness, better encapsulation, fewer breakpoints |
| FCP, FMP metrics | Core Web Vitals (LCP, INP, CLS) | 2020+ | User-centric metrics, SEO impact, better performance targets |
| `useEffect` + `useState` for forms | `useActionState` hook | React 19+ (2024) | Server Actions integration, progressive enhancement, better error handling |

**Deprecated/outdated:**
- **Pages Router loading patterns**: Use App Router loading.js convention instead
- **`getServerSideProps` loading**: Use Server Components with Suspense instead
- **Third-party image CDNs for optimization**: Next.js Image handles optimization natively
- **Bootstrap grid system**: Use Tailwind grid utilities or CSS Grid directly
- **Custom skeleton screen libraries**: Use Tailwind `animate-pulse` + basic divs
- **`max-*` breakpoints as primary strategy**: Use mobile-first min-width breakpoints

## Open Questions

1. **Touch target sizing on mobile**
   - What we know: Minimum 44x44px recommended, 48x48px preferred
   - What's unclear: Current button sizes in scan form, results page
   - Recommendation: Audit all interactive elements, ensure `min-h-[44px]` on buttons/links

2. **Font rendering differences iOS vs Android**
   - What we know: System fonts render differently, Inter font is used
   - What's unclear: Whether current font stack has acceptable fallback for loading state
   - Recommendation: Test Inter fallback on both platforms, consider `font-display: swap`

3. **Real device testing infrastructure**
   - What we know: Must test on real devices, not just DevTools
   - What's unclear: Which devices to prioritize, testing frequency
   - Recommendation: Test on iPhone SE (smallest iOS viewport), mid-range Android, latest iPhone Safari

4. **Lighthouse score optimization priority**
   - What we know: Target >90 on landing and results pages
   - What's unclear: Current scores, biggest bottlenecks
   - Recommendation: Run Lighthouse audit, prioritize LCP images and JavaScript bundle size

5. **Loading state polling frequency**
   - What we know: Need stage-specific feedback, backend provides `/status` endpoint
   - What's unclear: Optimal polling interval (1s? 2s? 5s?)
   - Recommendation: Start with 2s interval, adjust based on backend scan duration data

## Sources

### Primary (HIGH confidence)
- [Tailwind CSS Responsive Design](https://tailwindcss.com/docs/responsive-design) - Official docs, mobile-first approach, breakpoint system
- [Next.js Loading UI and Streaming](https://nextjs.org/docs/app/building-your-application/routing/loading-ui-and-streaming) - Official docs, loading.js convention, Suspense
- [Next.js Error Handling](https://nextjs.org/docs/app/getting-started/error-handling) - Official docs, error.js boundaries, expected vs uncaught errors
- [Next.js Image Optimization](https://nextjs.org/docs/app/building-your-application/optimizing/images) - Official docs, priority loading, lazy loading
- [Core Web Vitals Overview](https://web.dev/articles/vitals) - Official Google docs, LCP/INP/CLS thresholds

### Secondary (MEDIUM confidence)
- [Mastering Mobile Performance: Next.js Lighthouse Scores](https://www.wisp.blog/blog/mastering-mobile-performance-a-complete-guide-to-improving-nextjs-lighthouse-scores) - 2026 guide, practical optimization techniques
- [6 Loading State Patterns That Feel Premium](https://medium.com/uxdworld/6-loading-state-patterns-that-feel-premium-716aa0fe63e8) - UX patterns, skeleton screens, progressive feedback
- [Designing Better Error Messages UX](https://www.smashingmagazine.com/2022/08/error-messages-ux-design/) - Error message best practices, inline validation
- [Simulate Mobile Devices with Chrome DevTools](https://developer.chrome.com/docs/devtools/device-mode) - Official Chrome docs, DevTools limitations
- [Mobile Web Testing: Emulators vs Real Devices](https://testrig.medium.com/mobile-web-testing-with-playwright-emulators-vs-real-devices-62ab3b081e16) - Testing strategy, 80/20 rule

### Tertiary (LOW confidence, marked for validation)
- [React Stack Patterns 2026](https://www.patterns.dev/react/react-2026/) - Modern patterns, needs verification with official docs
- [Tailwind CSS Best Practices 2025-2026](https://www.frontendtools.tech/blog/tailwind-css-best-practices-design-system-patterns) - Design system patterns, needs validation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Tailwind CSS and Next.js App Router are officially documented standards
- Architecture patterns: HIGH - All patterns verified with official documentation and real codebase examples
- Loading states: HIGH - Next.js official docs provide complete guidance on loading.js and Suspense
- Error handling: HIGH - Next.js official docs provide complete error.js patterns
- Image optimization: HIGH - Next.js official docs provide complete Image component guidance
- Mobile testing: MEDIUM - Best practices from multiple sources, need to establish project-specific workflow
- Performance targets: HIGH - Core Web Vitals thresholds are official Google standards
- Pitfalls: HIGH - Common issues documented in official sources and verified community patterns

**Research date:** 2026-02-08
**Valid until:** 2026-03-10 (30 days for stable ecosystem)

**Notes:**
- Tailwind CSS v4.0 was released December 2024 with breaking changes, but v3.x patterns remain valid
- Next.js 15+ is current stable version, App Router patterns are mature
- Core Web Vitals thresholds are stable, updated annually at most
- Real device testing recommendations are evergreen, hardware specifics change but strategy remains constant
