import { ScanForm } from '@/components/scan-form'
import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'ShipSecure - Security Scanning for Vibe-Coded Apps',
  description: 'Free vulnerability detection for AI-generated web apps. Detects exposed .env files, weak TLS, hardcoded API keys, and framework misconfigurations. No signup required.',
  keywords: ['security scanning', 'vibe coding', 'web security', 'vulnerability scanner', 'AI-generated apps'],
  alternates: {
    canonical: '/',
  },
  openGraph: {
    title: 'ShipSecure - Security Scanning for Vibe-Coded Apps',
    description: 'Free vulnerability detection for AI-generated web apps. Detects exposed .env files, weak TLS, hardcoded API keys, and framework misconfigurations. No signup required.',
    url: 'https://shipsecure.ai',
    siteName: 'ShipSecure',
    images: [
      {
        url: '/opengraph-image',
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
    description: 'Free vulnerability detection for AI-generated web apps. Detects exposed .env files, weak TLS, hardcoded API keys, and framework misconfigurations. No signup required.',
    images: ['/opengraph-image'],
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

async function getScanCount(): Promise<number | null> {
  try {
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:3000'
    const res = await fetch(`${backendUrl}/api/v1/stats/scan-count`, {
      next: { revalidate: 60 }
    })
    if (!res.ok) return null
    const data = await res.json()
    return data.count
  } catch {
    return null
  }
}

export default async function Home() {
  const scanCount = await getScanCount()

  const organizationSchema = {
    '@context': 'https://schema.org',
    '@type': 'Organization',
    name: 'ShipSecure',
    url: 'https://shipsecure.ai',
    description: 'Security scanning for AI-generated web applications',
  }

  const softwareApplicationSchema = {
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
    description: 'Free security scanning for vibe-coded web apps. Catch security flaws before they become breaches.',
    featureList: [
      'Security header analysis',
      'TLS configuration scanning',
      'Exposed file detection',
      'JavaScript secret scanning',
    ],
  }

  return (
    <>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(organizationSchema) }}
      />
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(softwareApplicationSchema) }}
      />
      {/* Hero Section */}
      <main className="container mx-auto px-4 py-16 sm:py-24 max-w-4xl bg-white dark:bg-gray-950 text-gray-900 dark:text-gray-100">
        <div className="text-center mb-12">
          <h1 className="text-4xl sm:text-5xl md:text-6xl font-bold mb-4 bg-gradient-to-r from-blue-600 to-blue-800 dark:from-blue-400 dark:to-blue-600 bg-clip-text text-transparent">
            Security scanning for AI-generated web apps
          </h1>
          <p className="text-lg sm:text-xl text-gray-600 dark:text-gray-400 mb-2">
            Free vulnerability detection for vibe-coded projects. Detects exposed .env files, weak TLS ciphers, hardcoded API keys, and framework-specific misconfigurations.
          </p>
          <p className="text-sm text-gray-500 dark:text-gray-500">
            No signup required. Results in ~60 seconds.
          </p>
        </div>

        {/* Scan Form Card */}
        <div className="bg-gray-50 dark:bg-gray-900 rounded-2xl shadow-lg border border-gray-200 dark:border-gray-800 p-6 sm:p-8 mb-12">
          <ScanForm />
        </div>

        {/* What We Check */}
        <div className="text-center mb-12">
          <h2 className="text-2xl font-semibold mb-4 text-gray-900 dark:text-gray-100">
            What we check
          </h2>
          <div className="grid sm:grid-cols-2 gap-4 max-w-2xl mx-auto text-left">
            <div className="flex gap-3">
              <div className="text-blue-600 dark:text-blue-400 text-xl">&#x1F512;</div>
              <div>
                <h3 className="font-semibold text-gray-900 dark:text-gray-100">Security Headers</h3>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Analyzes CSP, HSTS, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy
                </p>
              </div>
            </div>
            <div className="flex gap-3">
              <div className="text-blue-600 dark:text-blue-400 text-xl">&#x1F511;</div>
              <div>
                <h3 className="font-semibold text-gray-900 dark:text-gray-100">TLS Configuration</h3>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Certificate chain validation, TLS 1.2/1.3 protocol versions, cipher suite strength via SSL Labs API
                </p>
              </div>
            </div>
            <div className="flex gap-3">
              <div className="text-blue-600 dark:text-blue-400 text-xl">&#x1F4C4;</div>
              <div>
                <h3 className="font-semibold text-gray-900 dark:text-gray-100">Exposed Files</h3>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Probes for .env, .git/config, /debug, /admin, wp-config.php, and 20+ sensitive paths
                </p>
              </div>
            </div>
            <div className="flex gap-3">
              <div className="text-blue-600 dark:text-blue-400 text-xl">&#x1F50D;</div>
              <div>
                <h3 className="font-semibold text-gray-900 dark:text-gray-100">JavaScript Secrets</h3>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Pattern-matches bundled JS for AWS keys, Stripe tokens, Firebase credentials, and API secrets
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* How It Works */}
        <div className="mb-12">
          <h2 className="text-2xl font-semibold mb-6 text-center text-gray-900 dark:text-gray-100">
            How it works
          </h2>
          <div className="grid md:grid-cols-3 gap-6 mb-6">
            <div className="text-center">
              <div className="text-3xl font-bold text-blue-600 dark:text-blue-400 mb-2">1</div>
              <h3 className="font-semibold text-gray-900 dark:text-gray-100 mb-2">Submit your URL</h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Enter any URL. No signup, no API key. Scans start immediately.
              </p>
            </div>
            <div className="text-center">
              <div className="text-3xl font-bold text-blue-600 dark:text-blue-400 mb-2">2</div>
              <h3 className="font-semibold text-gray-900 dark:text-gray-100 mb-2">Automated analysis</h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Four scanners run in parallel: HTTP header analysis, TLS configuration check via SSL Labs, sensitive file probing, and JavaScript static analysis.
              </p>
            </div>
            <div className="text-center">
              <div className="text-3xl font-bold text-blue-600 dark:text-blue-400 mb-2">3</div>
              <h3 className="font-semibold text-gray-900 dark:text-gray-100 mb-2">Prioritized results</h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Get an A-F security grade with severity-ranked findings and copy-paste remediation steps. Paid tier adds Nuclei-powered framework-specific checks.
              </p>
            </div>
          </div>
          <details className="text-sm text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
            <summary className="cursor-pointer hover:text-blue-600 dark:hover:text-blue-400 text-center">
              Scan methodology
            </summary>
            <ul className="mt-4 space-y-2 list-disc list-inside">
              <li><strong>Security headers:</strong> Passive HTTP response header analysis (CSP, HSTS, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy)</li>
              <li><strong>TLS configuration:</strong> Qualys SSL Labs API — certificate chain, protocol versions, cipher strength, known vulnerabilities</li>
              <li><strong>Exposed files:</strong> Active HTTP probes for common sensitive paths (.env, .git/config, /debug, /admin, backup files)</li>
              <li><strong>JavaScript secrets:</strong> Regex pattern matching on bundled JavaScript for API keys, tokens, and credentials</li>
              <li><strong>Vibe-code scanning (paid):</strong> Nuclei with custom templates for Supabase RLS bypass, Firebase security rules, Next.js/React misconfigurations</li>
            </ul>
          </details>
        </div>

        {/* Social Proof */}
        {scanCount !== null && scanCount > 0 && (
          <div className="text-center text-sm text-gray-500 dark:text-gray-500">
            <span className="font-semibold text-blue-600 dark:text-blue-400">
              {scanCount.toLocaleString()}
            </span>{' '}
            scans completed
          </div>
        )}
      </main>
    </>
  )
}
