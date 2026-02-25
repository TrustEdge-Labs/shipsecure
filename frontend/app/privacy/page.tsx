import type { Metadata } from 'next'
import { PageContainer } from '@/components/page-container'

export const metadata: Metadata = {
  title: 'Privacy Policy - ShipSecure',
  description: 'How ShipSecure collects, uses, and protects your personal information. GDPR and CCPA compliant.',
  robots: {
    index: true,
    follow: true,
  },
}

export default function PrivacyPolicy() {
  return (
    <main><PageContainer maxWidth="max-w-4xl" className="py-16 pb-8">
        <div className="mb-8">
          <h1 className="text-4xl sm:text-5xl font-bold mb-4 text-text-primary">
            Privacy Policy
          </h1>
          <p className="text-sm text-text-secondary">
            Last Updated: February 2026
          </p>
        </div>

        <div className="prose dark:prose-invert max-w-none">
          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              Information We Collect
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                ShipSecure collects the following information when you use our service:
              </p>
              <ul className="list-disc pl-6 space-y-2">
                <li>
                  <strong className="text-text-primary">Email address:</strong> Provided through the scan form to deliver scan results to you
                </li>
                <li>
                  <strong className="text-text-primary">Website URL:</strong> The target website you submit for security scanning
                </li>
                <li>
                  <strong className="text-text-primary">Payment information:</strong> For paid audits, payment details are processed by Stripe. We do not store credit card numbers, CVV codes, or other complete payment card details on our servers.
                </li>
                <li>
                  <strong className="text-text-primary">Usage analytics:</strong> We use Plausible Analytics, a privacy-focused, cookie-less analytics service hosted in the EU (Germany). No personally identifiable information is collected through analytics.
                </li>
              </ul>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              How We Use Your Information
            </h2>
            <div className="text-text-secondary space-y-3">
              <ul className="list-disc pl-6 space-y-2">
                <li>
                  <strong className="text-text-primary">Email address:</strong> To deliver scan results and communicate important information about your scans
                </li>
                <li>
                  <strong className="text-text-primary">Website URL:</strong> To perform the requested security scanning services
                </li>
                <li>
                  <strong className="text-text-primary">Payment information:</strong> To process paid audit purchases through our payment processor, Stripe
                </li>
                <li>
                  <strong className="text-text-primary">Usage analytics:</strong> To understand how our service is used and improve the user experience
                </li>
              </ul>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              Legal Basis for Processing (GDPR)
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                For users in the European Union, we process your personal data based on:
              </p>
              <ul className="list-disc pl-6 space-y-2">
                <li>
                  <strong className="text-text-primary">Legitimate interest:</strong> Providing security scanning services to protect web applications
                </li>
                <li>
                  <strong className="text-text-primary">Consent:</strong> By submitting your email address through our form, you consent to receiving scan results
                </li>
                <li>
                  <strong className="text-text-primary">Contractual necessity:</strong> Processing payment information is necessary to fulfill paid audit services
                </li>
              </ul>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              Data Retention
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                Scan results and associated email addresses are retained for 12 months from the date of the scan, unless you request earlier deletion. Payment records are retained in accordance with Stripe's data retention policies and applicable legal requirements (typically 7 years for tax and financial compliance).
              </p>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              Third-Party Services
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                We use the following third-party services to operate ShipSecure:
              </p>
              <ul className="list-disc pl-6 space-y-2">
                <li>
                  <strong className="text-text-primary">Stripe (Payment Processing):</strong> Processes all payment transactions. Credit card information is handled directly by Stripe and never stored on our servers. See{' '}
                  <a
                    href="https://stripe.com/privacy"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-brand-primary hover:underline"
                  >
                    Stripe's Privacy Policy
                  </a>
                  {' '}and{' '}
                  <a
                    href="https://stripe.com/legal/dpa"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-brand-primary hover:underline"
                  >
                    Data Processing Agreement
                  </a>
                  .
                </li>
                <li>
                  <strong className="text-text-primary">Plausible Analytics (EU-hosted, Germany):</strong> Cookie-less, privacy-focused analytics. No personal data is collected. See{' '}
                  <a
                    href="https://plausible.io/data-policy"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-brand-primary hover:underline"
                  >
                    Plausible's Data Policy
                  </a>
                  .
                </li>
                <li>
                  <strong className="text-text-primary">Resend (Email Delivery):</strong> Delivers scan result emails from scans@shipsecure.ai. Email addresses are shared with Resend only for the purpose of delivering scan results.
                </li>
              </ul>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              Your Rights Under GDPR and CCPA
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                If you are a resident of the European Union or California, you have the following rights:
              </p>
              <ul className="list-disc pl-6 space-y-2">
                <li>
                  <strong className="text-text-primary">Right to access:</strong> Request a copy of the personal data we hold about you
                </li>
                <li>
                  <strong className="text-text-primary">Right to deletion:</strong> Request deletion of your personal data (subject to legal retention requirements)
                </li>
                <li>
                  <strong className="text-text-primary">Right to portability:</strong> Receive your personal data in a structured, machine-readable format
                </li>
                <li>
                  <strong className="text-text-primary">Right to object:</strong> Object to processing of your personal data based on legitimate interests
                </li>
              </ul>
              <p className="mt-4">
                To exercise these rights, email us at{' '}
                <a
                  href="mailto:privacy@shipsecure.ai"
                  className="text-brand-primary hover:underline"
                >
                  privacy@shipsecure.ai
                </a>
                {' '}with a clear subject line describing your request (e.g., "GDPR Data Access Request" or "CCPA Data Deletion Request"). We will respond within 30 days (GDPR) or 45 days (CCPA). Identity verification may be required to protect your privacy.
              </p>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              Data Security
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                We implement industry-standard security measures to protect your personal information:
              </p>
              <ul className="list-disc pl-6 space-y-2">
                <li>All connections are encrypted using HTTPS/TLS</li>
                <li>Payment card information is never stored on our servers</li>
                <li>Database access is restricted and secured with industry-standard encryption</li>
                <li>Regular security audits and vulnerability scanning</li>
              </ul>
              <p className="mt-4">
                However, no method of transmission over the internet or electronic storage is 100% secure. While we strive to protect your personal information, we cannot guarantee absolute security.
              </p>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              International Data Transfers
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                Your information may be transferred to and processed in countries other than your country of residence:
              </p>
              <ul className="list-disc pl-6 space-y-2">
                <li>
                  <strong className="text-text-primary">Plausible Analytics:</strong> EU-hosted in Germany, fully GDPR-compliant
                </li>
                <li>
                  <strong className="text-text-primary">Stripe and Resend:</strong> US-based services that use Standard Contractual Clauses (SCCs) approved by the European Commission for international data transfers
                </li>
              </ul>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              Children's Privacy
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                ShipSecure is not directed to children under the age of 13. We do not knowingly collect personal information from children under 13. If you believe we have collected information from a child under 13, please contact us immediately at{' '}
                <a
                  href="mailto:privacy@shipsecure.ai"
                  className="text-brand-primary hover:underline"
                >
                  privacy@shipsecure.ai
                </a>
                {' '}and we will delete it.
              </p>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              Changes to This Policy
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                We may update this Privacy Policy from time to time. Material changes will be communicated by updating the "Last Updated" date at the top of this page. We encourage you to review this policy periodically. Your continued use of ShipSecure after changes constitutes acceptance of the updated policy.
              </p>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              Contact
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                For privacy-related questions or requests, contact us at:
              </p>
              <p>
                <a
                  href="mailto:privacy@shipsecure.ai"
                  className="text-brand-primary hover:underline"
                >
                  privacy@shipsecure.ai
                </a>
              </p>
              <p>
                For general support inquiries:
              </p>
              <p>
                <a
                  href="mailto:support@shipsecure.ai"
                  className="text-brand-primary hover:underline"
                >
                  support@shipsecure.ai
                </a>
              </p>
            </div>
          </section>
        </div>

        {/* Back to Home Link */}
        <div className="mt-12 pt-8 border-t border-border-subtle">
          <a
            href="/"
            className="text-brand-primary hover:underline"
          >
            ← Back to Home
          </a>
        </div>
      </PageContainer></main>
  )
}
