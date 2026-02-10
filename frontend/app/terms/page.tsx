import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'Terms of Service - ShipSecure',
  description: 'Terms of Service for ShipSecure security scanning. Covers acceptable use, scanning authorization, and liability.',
  robots: {
    index: true,
    follow: true,
  },
}

export default function TermsOfService() {
  return (
    <main className="container mx-auto px-4 py-16 pb-8 max-w-4xl bg-surface-primary text-text-primary">
        <div className="mb-8">
          <h1 className="text-4xl sm:text-5xl font-bold mb-4 text-text-primary">
            Terms of Service
          </h1>
          <p className="text-sm text-text-secondary mb-4">
            Last Updated: February 2026
          </p>
          <p className="text-text-secondary">
            By using ShipSecure, you agree to these terms.
          </p>
        </div>

        <div className="prose dark:prose-invert max-w-none">
          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              1. Service Description
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                ShipSecure provides automated security scanning services for web applications. Our service includes:
              </p>
              <ul className="list-disc pl-6 space-y-2">
                <li>
                  <strong className="text-text-primary">Free Tier:</strong> Basic security scan with results delivered via email
                </li>
                <li>
                  <strong className="text-text-primary">Paid Tier:</strong> Comprehensive security audit ($49 one-time payment) with detailed PDF report
                </li>
              </ul>
            </div>
          </section>

          <section className="mb-8" id="acceptable-use">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              2. Acceptable Use and Scanning Authorization
            </h2>
            <div className="text-text-secondary space-y-3">
              <h3 className="text-xl font-semibold text-text-primary">
                2.1 Authorization Requirement
              </h3>
              <p>
                By submitting a website URL for scanning, you represent and warrant that you either:
              </p>
              <ul className="list-disc pl-6 space-y-2">
                <li>Own the target website, or</li>
                <li>Have obtained explicit written authorization from the website owner to perform security scanning</li>
              </ul>

              <h3 className="text-xl font-semibold text-text-primary mt-6">
                2.2 CFAA Compliance
              </h3>
              <p>
                Unauthorized access to computer systems is illegal under the Computer Fraud and Abuse Act (18 U.S.C. section 1030) and equivalent state and international laws. You are solely responsible for obtaining all necessary permissions before submitting a website for scanning.
              </p>
              <p>
                ShipSecure will terminate accounts that engage in unauthorized scanning activities and may report suspected violations to appropriate authorities.
              </p>

              <h3 className="text-xl font-semibold text-text-primary mt-6">
                2.3 Prohibited Uses
              </h3>
              <p>
                You may not use ShipSecure to:
              </p>
              <ul className="list-disc pl-6 space-y-2">
                <li>Scan websites without proper authorization</li>
                <li>Perform penetration testing or exploitation beyond the scope of authorized scanning</li>
                <li>Violate any applicable laws or regulations</li>
                <li>Interfere with or disrupt the operation of ShipSecure or other users' access to the service</li>
                <li>Attempt to reverse engineer, decompile, or disassemble any part of the service</li>
                <li>Use the service to harm, threaten, or harass any person or entity</li>
              </ul>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              3. User Responsibilities
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                As a user of ShipSecure, you are responsible for:
              </p>
              <ul className="list-disc pl-6 space-y-2">
                <li>Providing accurate and truthful information</li>
                <li>Ensuring you have authorization to scan all submitted URLs</li>
                <li>Complying with all applicable laws and regulations</li>
                <li>Using scan results responsibly and ethically</li>
                <li>Practicing responsible disclosure if vulnerabilities are discovered</li>
                <li>Maintaining the confidentiality of any scan results</li>
              </ul>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              4. Limitation of Liability
            </h2>
            <div className="text-text-secondary space-y-3">
              <h3 className="text-xl font-semibold text-text-primary">
                4.1 Service Disclaimer
              </h3>
              <p>
                ShipSecure is provided "as is" and "as available" without warranties of any kind, either express or implied, including but not limited to warranties of merchantability, fitness for a particular purpose, or non-infringement.
              </p>
              <p>
                We do not guarantee that:
              </p>
              <ul className="list-disc pl-6 space-y-2">
                <li>The service will detect all vulnerabilities or security issues</li>
                <li>Scan results will be error-free, complete, or accurate</li>
                <li>The service will be available without interruption</li>
                <li>Any vulnerabilities identified pose actual security risks</li>
              </ul>

              <h3 className="text-xl font-semibold text-text-primary mt-6">
                4.2 Liability Cap
              </h3>
              <p>
                To the maximum extent permitted by law, ShipSecure's total liability to you for any claims arising from or related to the service shall not exceed the amount you paid to ShipSecure in the 12 months preceding the claim. For free tier users, this amount is $0. For paid tier users, the maximum liability is $49.
              </p>

              <h3 className="text-xl font-semibold text-text-primary mt-6">
                4.3 Excluded Damages
              </h3>
              <p>
                In no event shall ShipSecure be liable for any indirect, incidental, special, consequential, or punitive damages, including but not limited to:
              </p>
              <ul className="list-disc pl-6 space-y-2">
                <li>Loss of profits, revenue, or business opportunities</li>
                <li>Loss of data or information</li>
                <li>Security breaches or vulnerabilities not detected by our service</li>
                <li>Damages resulting from unauthorized use of scan results</li>
                <li>Third-party claims arising from your use of the service</li>
              </ul>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              5. Refund Policy
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                For one-time paid audits ($49):
              </p>
              <ul className="list-disc pl-6 space-y-2">
                <li>
                  <strong className="text-text-primary">Full refund available:</strong> Within 24 hours of payment if your scan has not started processing
                </li>
                <li>
                  <strong className="text-text-primary">No refund available:</strong> After the scan completes or the PDF report is delivered
                </li>
                <li>
                  <strong className="text-text-primary">Free scans:</strong> Not eligible for refunds (no payment made)
                </li>
              </ul>
              <p className="mt-4">
                To request a refund, contact{' '}
                <a
                  href="mailto:support@shipsecure.ai"
                  className="text-brand-primary hover:underline"
                >
                  support@shipsecure.ai
                </a>
                {' '}within 24 hours of your payment. Include your payment receipt or transaction ID.
              </p>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              6. Intellectual Property
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                ShipSecure and its original content, features, and functionality are owned by ShipSecure and are protected by international copyright, trademark, patent, trade secret, and other intellectual property laws.
              </p>
              <p>
                Scan results generated for your website belong to you. You may use, share, or distribute your scan results as you see fit.
              </p>
              <p>
                ShipSecure uses open-source security tools including Nuclei and testssl.sh. We acknowledge and thank the maintainers of these projects for their contributions to the security community.
              </p>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              7. Termination
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                ShipSecure may suspend or terminate your access to the service immediately, without prior notice or liability, for any reason, including but not limited to:
              </p>
              <ul className="list-disc pl-6 space-y-2">
                <li>Violation of these Terms of Service</li>
                <li>Suspected unauthorized scanning activity</li>
                <li>Requests from law enforcement or government agencies</li>
                <li>Technical or security reasons</li>
              </ul>
              <p className="mt-4">
                You may stop using ShipSecure at any time. All provisions of these Terms which by their nature should survive termination shall survive, including ownership provisions, warranty disclaimers, and limitations of liability.
              </p>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              8. Governing Law
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                These Terms shall be governed by and construed in accordance with the laws of the United States, without regard to its conflict of law provisions.
              </p>
              <p>
                Any disputes arising from or relating to these Terms or your use of ShipSecure shall be resolved in the appropriate state or federal courts with jurisdiction over the matter.
              </p>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              9. Changes to Terms
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                We reserve the right to modify these Terms at any time. Material changes will be communicated by updating the "Last Updated" date at the top of this page. Your continued use of ShipSecure after changes constitutes acceptance of the modified Terms.
              </p>
            </div>
          </section>

          <section className="mb-8">
            <h2 className="text-2xl font-semibold mb-4 text-text-primary">
              10. Contact
            </h2>
            <div className="text-text-secondary space-y-3">
              <p>
                For questions about these Terms of Service, contact:
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
      </main>
  )
}
