# Phase 10: Legal Compliance - Research

**Researched:** 2026-02-08
**Domain:** Legal documentation, privacy compliance, consent mechanisms, Next.js static pages
**Confidence:** MEDIUM-HIGH

## Summary

Legal compliance for SaaS security scanners requires three core components: privacy-compliant documentation (GDPR/CCPA), Computer Fraud and Abuse Act (CFAA) protection through explicit authorization, and accessible legal links throughout the site. The 2026 landscape introduces new CCPA enhancements (mandatory opt-out confirmations, extended data access periods back to January 2022, and risk assessments for automated decision-making) and the California Delete Act requiring data broker registration by January 31, 2026 (not applicable to ShipSecure as it's not a data broker). Implementation follows standard Next.js App Router patterns: static pages at `/privacy` and `/terms`, persistent footer in root layout, and React Hook Form checkbox validation for explicit scanning consent.

**Primary recommendation:** Implement privacy policy and terms as standalone Next.js pages with comprehensive third-party disclosures (Stripe, Plausible, email collection), add explicit CFAA authorization checkbox to scan form with required validation, and place legal links in root layout footer for global visibility. Legal text should use SaaS templates as foundation but must receive legal review before production deployment.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Next.js | 16.1.6 | Static legal pages | Already in use, built-in metadata API for SEO |
| Zod | 4.3.6 | Consent checkbox validation | Already in use for form validation |
| React | 19.2.3 | UI components | Already in use for frontend |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| react-hook-form | 7.x | Form validation (optional) | If upgrading scan form validation patterns |
| marked | 17.0.1 | Markdown rendering (optional) | If storing legal docs as markdown |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Static Next.js pages | External links to Termly/iubenda | Lose control, higher cost, but auto-updates for law changes |
| Zod validation | React Hook Form | More features but adds dependency (current Zod sufficient) |
| Root layout footer | Per-page footers | Duplication, inconsistency, maintenance burden |

**Installation:**
No additional dependencies required — all necessary tools already installed.

## Architecture Patterns

### Recommended Project Structure
```
frontend/
├── app/
│   ├── privacy/
│   │   └── page.tsx          # Privacy Policy page
│   ├── terms/
│   │   └── page.tsx          # Terms of Service page
│   ├── layout.tsx            # Modified: add Footer component
│   └── page.tsx              # Modified: link to legal docs
├── components/
│   ├── footer.tsx            # NEW: Global footer with legal links
│   └── scan-form.tsx         # Modified: add consent checkbox
```

### Pattern 1: Next.js Static Legal Pages
**What:** Create legal pages as standard Next.js App Router pages with metadata for SEO and accessibility.

**When to use:** All static legal documentation (Privacy Policy, Terms of Service, Acceptable Use Policy).

**Example:**
```typescript
// Source: Next.js official docs + current codebase pattern
// app/privacy/page.tsx
import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'Privacy Policy - ShipSecure',
  description: 'ShipSecure privacy policy covering email collection, analytics, and GDPR/CCPA rights.',
  robots: {
    index: true,  // Allow indexing (builds trust)
    follow: true,
  },
}

export default function PrivacyPage() {
  return (
    <div className="min-h-screen bg-white dark:bg-gray-950 text-gray-900 dark:text-gray-100">
      <main className="container mx-auto px-4 py-16 max-w-4xl">
        <h1 className="text-4xl font-bold mb-8">Privacy Policy</h1>

        <section className="prose dark:prose-invert max-w-none">
          <p className="text-gray-600 dark:text-gray-400">
            Last Updated: {new Date().toLocaleDateString()}
          </p>

          <h2>1. Data We Collect</h2>
          {/* Legal content */}
        </section>
      </main>
    </div>
  )
}
```

### Pattern 2: Persistent Footer in Root Layout
**What:** Add Footer component to root layout so it appears on all pages without duplication.

**When to use:** Global UI elements (header, footer, navigation) that must be consistent across entire site.

**Example:**
```typescript
// Source: https://nextjs.org/docs/app/api-reference/file-conventions/layout
// app/layout.tsx
export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className={`${inter.variable} font-sans antialiased`}>
        {children}
        <Footer />  {/* Global footer on all pages */}
      </body>
    </html>
  );
}
```

**Key insight:** The `children` prop contains page content that changes on navigation, while footer persists without re-rendering. Layouts are cached during client-side navigation for performance.

### Pattern 3: Required Consent Checkbox with Zod
**What:** Checkbox input with Zod validation requiring `true` value for explicit authorization.

**When to use:** CFAA protection, GDPR consent, terms acceptance where implicit agreement is insufficient.

**Example:**
```typescript
// Source: https://jasonwatmore.com/post/2020/10/05/react-required-checkbox-example-with-react-hook-form
// Modified for Zod validation (already in use)
import { z } from 'zod'

const scanFormSchema = z.object({
  url: z.string().url(),
  email: z.string().email(),
  authorization: z.boolean().refine(val => val === true, {
    message: 'You must confirm authorization to scan this website'
  })
})

// In component:
<div className="flex items-start gap-2">
  <input
    type="checkbox"
    id="authorization"
    name="authorization"
    required
    className="mt-1"
  />
  <label htmlFor="authorization" className="text-sm">
    I confirm I own this website or have explicit authorization to scan it.
    Unauthorized scanning may violate the Computer Fraud and Abuse Act (CFAA).
  </label>
</div>
{state.errors?.authorization && (
  <p className="text-sm text-red-600">{state.errors.authorization[0]}</p>
)}
```

### Pattern 4: Footer Component Structure
**What:** Reusable Footer component with legal links, copyright, and responsive layout.

**When to use:** Global footer referenced in root layout.

**Example:**
```typescript
// components/footer.tsx
import Link from 'next/link'

export function Footer() {
  return (
    <footer className="border-t border-gray-200 dark:border-gray-800 py-8 mt-auto">
      <div className="container mx-auto px-4 text-center text-sm text-gray-600 dark:text-gray-400">
        <div className="flex flex-col sm:flex-row justify-center items-center gap-4 mb-4">
          <Link href="/privacy" className="hover:text-blue-600 dark:hover:text-blue-400">
            Privacy Policy
          </Link>
          <span className="hidden sm:inline">•</span>
          <Link href="/terms" className="hover:text-blue-600 dark:hover:text-blue-400">
            Terms of Service
          </Link>
        </div>
        <p>&copy; {new Date().getFullYear()} ShipSecure. All rights reserved.</p>
      </div>
    </footer>
  )
}
```

### Anti-Patterns to Avoid
- **Storing legal text in database:** Legal docs are static, change infrequently, and benefit from version control. Database adds complexity without value.
- **Implicit consent through button click:** CFAA liability requires explicit checkbox authorization separate from form submission.
- **Generic "I accept" checkboxes:** Must clearly state what user is agreeing to (scanning authorization, not generic terms).
- **Missing last-updated dates:** GDPR/CCPA require disclosure of when policies changed; timestamps are mandatory.
- **Copy-pasting legal text without review:** Templates are starting points; must be customized for actual data practices and reviewed by legal.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Legal text generation | Custom privacy policy generator | SaaS templates from TermsFeed, Termly, or iubenda | Legal language requires precision; misstatements create liability. Templates are lawyer-reviewed. |
| GDPR/CCPA compliance tracking | Custom consent management platform | Document procedures + email-based deletion requests (MVP scale) | Full CMP is overkill for email-only collection; procedural documentation sufficient at current scale. |
| Data deletion automation | Custom multi-database purge scripts | Manual email-triggered deletion with audit log | Low volume (email-only collection) doesn't justify automation; human review prevents accidental deletion. |
| Cookie consent banners | Custom banner implementation | None needed (Plausible is cookie-less) | Plausible uses no cookies/local storage; GDPR/CCPA consent banners not required. |

**Key insight:** Legal compliance requirements are well-documented and templated. Custom legal text introduces unnecessary risk. Use templates as foundation, customize for actual data practices, then obtain legal review. For ShipSecure's simple data model (email + scan results), complex consent management platforms are premature optimization.

## Common Pitfalls

### Pitfall 1: Incomplete Third-Party Disclosures
**What goes wrong:** Privacy policy omits Stripe, Plausible, or Resend data processing, creating GDPR violation.

**Why it happens:** Developers focus on first-party data collection and forget third-party processors are data sharing under GDPR/CCPA.

**How to avoid:** Audit ALL external services that touch user data: Stripe (payment processing, PCI-DSS), Plausible (analytics, EU-hosted), Resend (email delivery). Document what data each receives, where it's stored, and link to their DPAs.

**Warning signs:** Privacy policy doesn't mention "Stripe," "Plausible," or "email service provider." No section on "Third-Party Services" or "Data Processors."

### Pitfall 2: Vague CFAA Authorization Language
**What goes wrong:** Checkbox says "I agree to terms" instead of explicit scanning authorization, providing no CFAA protection.

**Why it happens:** Developers treat scanning authorization like generic T&C acceptance, missing the criminal statute context.

**How to avoid:** Checkbox must explicitly state: (1) user owns website OR has authorization, (2) unauthorized scanning is illegal (cite CFAA), (3) consent is required. Link to Terms section explaining scanning authorization requirements.

**Warning signs:** Checkbox text under 20 words. No mention of "authorization," "own," or "legal right to scan." CFAA not referenced.

### Pitfall 3: Missing GDPR Data Subject Rights
**What goes wrong:** Privacy policy doesn't explain how users exercise rights (access, deletion, portability), creating compliance gap.

**Why it happens:** Developers document what data is collected but forget users must know HOW to act on their rights.

**How to avoid:** Privacy policy must include section "Your Rights Under GDPR/CCPA" with explicit instructions: "To request data deletion, email privacy@shipsecure.ai with subject line 'Data Deletion Request.'" Provide contact method, expected timeline (45 days CCPA, 30 days GDPR), and verification procedure.

**Warning signs:** Privacy policy has "data subject rights" section but no contact email. Rights listed but no procedure described. Uses passive voice ("rights may be exercised") instead of actionable instructions.

### Pitfall 4: Outdated Refund Policy for One-Time Payments
**What goes wrong:** Terms of Service uses subscription-based refund language (prorated refunds, cancellation periods) for one-time $49 audit.

**Why it happens:** Copy-pasting SaaS templates designed for monthly subscriptions without adapting for one-time payment model.

**How to avoid:** One-time payment refund policy differs from subscriptions: no prorated periods, simpler time window (e.g., "full refund within 24 hours if scan hasn't completed"), clear non-refundable conditions. Stripe handles chargeback disputes but policy sets customer expectations.

**Warning signs:** Terms mention "billing cycle," "monthly fee," or "cancellation" for one-time product. No clear refund window stated. Contradictory language about partial refunds.

### Pitfall 5: Footer Missing from Specific Pages
**What goes wrong:** Footer implemented per-page instead of root layout, causing inconsistent legal link availability.

**Why it happens:** Developers add footer to home page but forget it must appear on /scan/[id], /payment/success, /results/[id], etc.

**How to avoid:** Place Footer component in root layout.tsx so it renders on ALL pages automatically. Use Next.js layout hierarchy — root layout wraps entire app.

**Warning signs:** Footer component imported per-page. Legal links present on homepage but missing on dynamic routes. Different footer implementations across pages.

### Pitfall 6: Ignoring 2026 CCPA Enhancements
**What goes wrong:** Privacy policy doesn't address mandatory opt-out confirmations or extended data access windows required as of January 1, 2026.

**Why it happens:** Using pre-2026 privacy policy templates without updating for new CCPA requirements.

**How to avoid:** 2026 CCPA changes: (1) Mandatory opt-out confirmation UI ("Tracking Disabled" visible message), (2) Extended data access back to January 2022 if maintained, (3) Risk assessments for automated decision-making. ShipSecure impact: Plausible opt-out confirmation not needed (cookie-less), no ADMT requiring assessment, scan data retention policy determines access window.

**Warning signs:** Privacy policy dated before 2026. No mention of "opt-out confirmation" or "automated decision-making." Data retention policy undefined (affects access rights window).

## Code Examples

Verified patterns from official sources and current codebase:

### Privacy Policy Page Implementation
```typescript
// Source: Next.js official docs + ShipSecure codebase pattern
// app/privacy/page.tsx
import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'Privacy Policy - ShipSecure',
  description: 'How ShipSecure collects, uses, and protects your personal information.',
  robots: {
    index: true,
    follow: true,
  },
}

export default function PrivacyPage() {
  return (
    <div className="min-h-screen bg-white dark:bg-gray-950">
      <main className="container mx-auto px-4 py-16 max-w-4xl">
        <h1 className="text-4xl font-bold mb-2">Privacy Policy</h1>
        <p className="text-gray-600 dark:text-gray-400 mb-8">
          Last Updated: February 8, 2026
        </p>

        <div className="prose dark:prose-invert max-w-none">
          <h2>1. Information We Collect</h2>
          <p>ShipSecure collects the following personal information:</p>
          <ul>
            <li><strong>Email Address:</strong> Collected via scan submission form for results notification</li>
            <li><strong>Website URL:</strong> Target website submitted for security scanning</li>
            <li><strong>Payment Information:</strong> Processed by Stripe for paid audits (we do not store card details)</li>
            <li><strong>Usage Analytics:</strong> Plausible Analytics (cookie-less, EU-hosted, privacy-focused)</li>
          </ul>

          <h2>2. How We Use Your Information</h2>
          {/* Continue with legal sections */}

          <h2>8. Your Rights Under GDPR and CCPA</h2>
          <p>You have the right to:</p>
          <ul>
            <li>Access your personal data</li>
            <li>Request data deletion</li>
            <li>Object to data processing</li>
            <li>Data portability</li>
          </ul>
          <p>
            To exercise these rights, email{' '}
            <a href="mailto:privacy@shipsecure.ai">privacy@shipsecure.ai</a>{' '}
            with your request. We will respond within 30 days (GDPR) or 45 days (CCPA).
          </p>

          <h2>9. Third-Party Services</h2>
          <h3>Stripe (Payment Processing)</h3>
          <p>
            We use Stripe to process payments. Stripe collects payment card information,
            billing details, and transaction metadata. See Stripe's{' '}
            <a href="https://stripe.com/privacy">Privacy Policy</a> and{' '}
            <a href="https://stripe.com/legal/dpa">Data Processing Agreement</a>.
          </p>

          <h3>Plausible Analytics</h3>
          <p>
            We use Plausible Analytics for privacy-friendly website analytics. Plausible
            is GDPR/CCPA compliant, uses no cookies, and stores all data in the EU (Germany).
            See <a href="https://plausible.io/data-policy">Plausible's Data Policy</a>.
          </p>
        </div>
      </main>
    </div>
  )
}
```

### Terms of Service with CFAA Language
```typescript
// app/terms/page.tsx
export default function TermsPage() {
  return (
    <div className="prose dark:prose-invert max-w-none">
      <h2>2. Acceptable Use and Scanning Authorization</h2>
      <h3>2.1 Authorization Requirement</h3>
      <p>
        By submitting a website for scanning, you represent and warrant that:
      </p>
      <ul>
        <li>You own the target website, OR</li>
        <li>You have explicit written authorization from the website owner to conduct security scanning</li>
      </ul>

      <h3>2.2 Computer Fraud and Abuse Act (CFAA) Compliance</h3>
      <p>
        Unauthorized access to computer systems is illegal under the Computer Fraud and
        Abuse Act (18 U.S.C. § 1030) and equivalent state laws. Users are solely responsible
        for obtaining all necessary permissions before submitting a website for scanning.
      </p>
      <p>
        <strong>ShipSecure will immediately terminate accounts and may report to law enforcement
        any suspected unauthorized scanning activity.</strong>
      </p>

      <h3>2.3 Prohibited Uses</h3>
      <p>You may NOT use ShipSecure to:</p>
      <ul>
        <li>Scan websites you do not own or have authorization to test</li>
        <li>Conduct penetration testing without proper authorization</li>
        <li>Violate any applicable laws or regulations</li>
      </ul>

      <h2>5. Limitation of Liability</h2>
      <h3>5.1 Disclaimer of Warranties</h3>
      <p>
        ShipSecure is provided "as is" without warranties of any kind. We do not guarantee
        that our scanning will detect all vulnerabilities or that results are error-free.
      </p>

      <h3>5.2 Liability Cap</h3>
      <p>
        To the maximum extent permitted by law, ShipSecure's total liability for any claim
        arising from use of the service shall not exceed the amount you paid for the service
        (maximum $49 for one-time audits, $0 for free scans).
      </p>

      <h3>5.3 Excluded Damages</h3>
      <p>
        We are not liable for indirect, incidental, special, consequential, or punitive
        damages including lost profits, data loss, or business interruption.
      </p>

      <h2>6. Refund Policy</h2>
      <p>
        For one-time paid audits ($49):
      </p>
      <ul>
        <li><strong>Full refund:</strong> Within 24 hours if scan has not started processing</li>
        <li><strong>No refund:</strong> After scan completes or PDF report is delivered</li>
      </ul>
      <p>
        Free scans are not eligible for refunds. To request a refund, email{' '}
        support@shipsecure.ai within 24 hours of payment.
      </p>
    </div>
  )
}
```

### Consent Checkbox in Scan Form
```typescript
// components/scan-form.tsx (modified)
'use client'

import { useActionState } from 'react'

export function ScanForm() {
  const [state, formAction, pending] = useActionState(submitScan, {} as ScanFormState)

  return (
    <form action={formAction} className="space-y-4">
      {/* Existing URL and email fields */}

      {/* NEW: Authorization checkbox */}
      <div className="border-t border-gray-200 dark:border-gray-700 pt-4">
        <div className="flex items-start gap-3">
          <input
            type="checkbox"
            id="authorization"
            name="authorization"
            required
            className="mt-1 w-4 h-4 rounded border-gray-300 dark:border-gray-600
                       text-blue-600 focus:ring-blue-500"
          />
          <label
            htmlFor="authorization"
            className="text-sm text-gray-700 dark:text-gray-300"
          >
            I confirm I own this website or have explicit authorization from the owner
            to conduct security scanning. I understand that unauthorized scanning may
            violate the{' '}
            <a
              href="/terms#acceptable-use"
              target="_blank"
              className="text-blue-600 dark:text-blue-400 underline"
            >
              Computer Fraud and Abuse Act (CFAA)
            </a>.
          </label>
        </div>
        {state.errors?.authorization && (
          <p className="mt-1 text-sm text-red-600 dark:text-red-400">
            {state.errors.authorization[0]}
          </p>
        )}
      </div>

      <button
        type="submit"
        disabled={pending}
        className="w-full py-3 px-6 rounded-lg bg-blue-600 hover:bg-blue-700
                   disabled:bg-blue-400 text-white font-semibold transition"
      >
        {pending ? 'Starting scan...' : 'Scan Now — Free'}
      </button>

      <p className="text-xs text-gray-500 dark:text-gray-500 text-center">
        By submitting, you agree to our{' '}
        <a href="/terms" className="underline">Terms of Service</a> and{' '}
        <a href="/privacy" className="underline">Privacy Policy</a>.
      </p>
    </form>
  )
}
```

### Server Action Validation (Zod)
```typescript
// app/actions/scan.ts (modified)
import { z } from 'zod'

const scanFormSchema = z.object({
  url: z.string().url('Please enter a valid URL'),
  email: z.string().email('Please enter a valid email'),
  authorization: z
    .string()
    .transform(val => val === 'on' || val === 'true')  // HTML checkbox values
    .refine(val => val === true, {
      message: 'You must confirm authorization to scan this website'
    })
})

export async function submitScan(
  prevState: ScanFormState,
  formData: FormData
): Promise<ScanFormState> {
  const rawData = {
    url: formData.get('url'),
    email: formData.get('email'),
    authorization: formData.get('authorization'),
  }

  const validation = scanFormSchema.safeParse(rawData)

  if (!validation.success) {
    return {
      errors: validation.error.flatten().fieldErrors,
    }
  }

  // Continue with scan submission
}
```

### Global Footer Component
```typescript
// components/footer.tsx
import Link from 'next/link'

export function Footer() {
  const currentYear = new Date().getFullYear()

  return (
    <footer className="border-t border-gray-200 dark:border-gray-800 py-8 mt-auto">
      <div className="container mx-auto px-4">
        <div className="flex flex-col sm:flex-row justify-between items-center gap-4">
          {/* Legal Links */}
          <nav className="flex gap-4 text-sm text-gray-600 dark:text-gray-400">
            <Link
              href="/privacy"
              className="hover:text-blue-600 dark:hover:text-blue-400 transition"
            >
              Privacy Policy
            </Link>
            <span className="hidden sm:inline">•</span>
            <Link
              href="/terms"
              className="hover:text-blue-600 dark:hover:text-blue-400 transition"
            >
              Terms of Service
            </Link>
          </nav>

          {/* Copyright */}
          <p className="text-sm text-gray-600 dark:text-gray-400">
            &copy; {currentYear} ShipSecure. All rights reserved.
          </p>
        </div>
      </div>
    </footer>
  )
}
```

### Root Layout with Footer
```typescript
// app/layout.tsx (modified)
import { Footer } from '@/components/footer'

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <head>{/* Plausible scripts */}</head>
      <body className={`${inter.variable} font-sans antialiased`}>
        <div className="flex flex-col min-h-screen">
          <div className="flex-1">
            {children}
          </div>
          <Footer />
        </div>
      </body>
    </html>
  );
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Cookie consent banners for all analytics | Cookie-less analytics (Plausible) | 2021+ | No consent banner needed; simpler compliance |
| Generic "I agree" checkboxes | Explicit authorization language citing CFAA | Post-Van Buren (2021) | Better legal protection for security tools |
| Manual privacy policy updates | Automated compliance tools (Termly, iubenda) | 2018+ (GDPR) | Reduces legal risk but adds cost ($10-30/mo) |
| 30-day CCPA response window | 45-day window with mandatory confirmation | 2026 (CCPA amendments) | Longer response time, visible opt-out UI required |
| Privacy policies as PDFs | HTML pages with anchor links | 2015+ | Better SEO, accessibility, mobile UX |

**Deprecated/outdated:**
- **EU-US Privacy Shield:** Invalidated by Schrems II (2020); use Standard Contractual Clauses (SCCs) or EU-hosted services (Plausible is EU-based, compliant)
- **Implied consent via browsing:** GDPR requires explicit opt-in for tracking; Plausible's cookie-less approach avoids this entirely
- **"We may update this policy" without notification:** CCPA 2026 amendments require notification of material changes to privacy practices

## Open Questions

1. **Legal review source and timeline**
   - What we know: Privacy policy and TOS templates are available from TermsFeed, Termly, iubenda, and legal blogs
   - What's unclear: Whether founder has legal counsel for review, budget for legal services, or acceptable launch timeline with vs. without legal review
   - Recommendation: Flag legal review as pre-production blocker in plan. Legal docs can be implemented and staged but should not go live without review. Minimum: founder self-review against GDPR/CCPA checklists. Optimal: attorney review ($500-1500 one-time).

2. **Data deletion procedure implementation**
   - What we know: GDPR/CCPA require data deletion within 45 days of request; ShipSecure stores emails + scan results in PostgreSQL
   - What's unclear: Whether manual email-triggered deletion is acceptable or if automated self-service portal is required for compliance
   - Recommendation: MVP uses manual email-based deletion (privacy@shipsecure.ai). Document procedure: (1) Verify requester identity, (2) Delete from scans table (email + results), (3) Confirm deletion to requester within 45 days. Automated portal deferred to post-launch based on volume.

3. **Stripe Data Processing Agreement acceptance**
   - What we know: Stripe provides DPA for GDPR compliance; business users must accept it
   - What's unclear: Whether founder has already accepted Stripe DPA for the account
   - Recommendation: Verify Stripe DPA acceptance during implementation. Privacy policy links to Stripe's DPA but ShipSecure must have accepted it. Check Stripe Dashboard > Settings > Legal for DPA status.

4. **Email retention policy**
   - What we know: CCPA 2026 requires access to data back to January 2022 "if maintained"
   - What's unclear: How long ShipSecure retains email addresses and scan results (indefinite vs. time-limited)
   - Recommendation: Define retention policy in Privacy Policy (e.g., "We retain scan results and email addresses for 12 months unless you request deletion"). Shorter retention = less compliance burden. Document in privacy policy's "Data Retention" section.

5. **Contact email setup**
   - What we know: Privacy policies must provide contact method for data requests; common pattern is privacy@domain or support@domain
   - What's unclear: Whether privacy@shipsecure.ai, support@shipsecure.ai, or legal@shipsecure.ai is preferred
   - Recommendation: Use privacy@shipsecure.ai for GDPR/CCPA requests (clearer purpose) and support@shipsecure.ai for general customer service. Set up email aliases via Resend (already in use) and ensure both route to founder during MVP phase.

## Sources

### Primary (HIGH confidence)
- [Next.js Layout Documentation](https://nextjs.org/docs/app/api-reference/file-conventions/layout) - Official pattern for persistent UI elements (footer implementation)
- [CCPA Requirements 2026: Complete Compliance Guide](https://secureprivacy.ai/blog/ccpa-requirements-2026-complete-compliance-guide) - Mandatory opt-out confirmations, enhanced requirements effective January 1, 2026
- [GDPR CCPA data deletion request procedure SaaS implementation 2026](https://secureprivacy.ai/blog/ccpa-requirements-2026-complete-compliance-guide) - 45-day response timeline, California Delete Act (DROP platform for data brokers)
- [Plausible: GDPR, CCPA and cookie law compliant site analytics](https://plausible.io/data-policy) - Cookie-less analytics, EU-hosted, no consent banner required
- [Stripe Payment Compliance: Complete Financial Data Protection for SaaS Companies](https://complydog.com/blog/stripe-payment-compliance-financial-data-protection-saas) - Stripe DPA requirements, GDPR/CCPA disclosure obligations

### Secondary (MEDIUM confidence)
- [SaaS Terms & Conditions Template - TermsFeed](https://www.termsfeed.com/blog/sample-saas-terms-conditions-template/) - Liability limits, acceptable use policy patterns (verified against multiple sources)
- [Terms of Service - Website Security Scanner](https://webscansec.com/legal/terms) - Real-world example of security scanner TOS with CFAA authorization language
- [Security Scanning and Monitoring Terms of Use - H-X tech portal](https://service.h-x.technology/scan-terms-of-use) - Authorization requirements, user warranties for scanning services
- [React - Required Checkbox Example with React Hook Form](https://jasonwatmore.com/post/2020/10/05/react-required-checkbox-example-with-react-hook-form) - Checkbox validation pattern (adapted for Zod)
- [SaaS Refund Policy - TermsFeed](https://www.termsfeed.com/blog/saas-refund-policy/) - One-time payment refund policy patterns vs. subscription models

### Tertiary (LOW confidence - verify before production)
- [Understanding the Bounds of the Computer Fraud and Abuse Act in the Wake of Van Buren](https://www.floridabar.org/the-florida-bar-journal/understanding-the-bounds-of-the-computer-fraud-and-abuse-act-in-the-wake-of-van-buren/) - Legal analysis of CFAA authorization boundaries (academic source, not official guidance)
- [Email Privacy Laws & Regulations 2026: GDPR, CCPA Guide](https://www.getmailbird.com/email-privacy-laws-regulations-compliance/) - Email collection disclosure requirements (source credibility uncertain)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries already in use, no new dependencies required
- Architecture patterns: HIGH - Next.js layout/page patterns verified from official docs and current codebase
- Legal content requirements: MEDIUM-HIGH - GDPR/CCPA requirements verified from multiple authoritative sources (SecurePrivacy, official Plausible/Stripe docs), but legal text templates require attorney review
- CFAA authorization: MEDIUM - Legal analysis available but not official DOJ guidance; explicit authorization pattern verified from multiple security scanner TOS examples
- Implementation pitfalls: MEDIUM - Based on WebSearch cross-verification and common SaaS compliance issues, but not exhaustive

**Research date:** February 8, 2026
**Valid until:** March 10, 2026 (30 days - privacy laws stable, but 2026 CCPA changes are new and may see clarifications)

**Critical gaps requiring validation:**
1. Legal review process and timeline (founder decision needed)
2. Data deletion procedure automation requirements (manual vs. self-service)
3. Stripe DPA acceptance status (verify in Stripe Dashboard)
4. Email retention policy duration (founder policy decision)
5. CFAA authorization checkbox exact wording (legal review recommended)

**Research quality notes:**
- No CONTEXT.md existed, so full discretion applied
- Multiple cross-verified sources for GDPR/CCPA requirements
- Official Next.js documentation used for architecture patterns
- Real security scanner TOS examples reviewed for CFAA language
- Current codebase examined to ensure compatibility (Zod validation, Plausible integration, existing form patterns)
- 2026 CCPA amendments specifically researched (mandatory opt-out confirmations, Delete Act)
- Plausible cookie-less approach confirmed via official data policy (no consent banner needed)
