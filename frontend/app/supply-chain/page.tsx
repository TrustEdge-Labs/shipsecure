import type { Metadata } from 'next'
import Link from 'next/link'
import { PageContainer } from '@/components/page-container'
import { SupplyChainForm } from '@/components/supply-chain-form'

export const metadata: Metadata = {
  title: 'Supply Chain Scanner - ShipSecure',
  description:
    'Check your npm dependencies for known compromised packages, vulnerabilities, and malware. Paste a GitHub URL or upload your package-lock.json.',
  alternates: { canonical: '/supply-chain' },
  openGraph: {
    title: 'Supply Chain Scanner - ShipSecure',
    description:
      'Check your npm dependencies for known compromised packages, vulnerabilities, and malware.',
    url: 'https://shipsecure.ai/supply-chain',
    siteName: 'ShipSecure',
    type: 'website',
  },
}

export default function SupplyChainPage() {
  return (
    <main className="min-h-screen pt-24 pb-16">
      <PageContainer maxWidth="max-w-2xl">
        <div className="space-y-8">
          {/* Breadcrumb */}
          <nav className="text-sm text-text-tertiary">
            <Link href="/" className="hover:text-text-secondary transition">
              ShipSecure
            </Link>
            <span className="mx-2">/</span>
            <span className="text-text-secondary">Supply Chain Scanner</span>
          </nav>

          {/* Header */}
          <div className="text-center space-y-3">
            <h1 className="text-3xl sm:text-4xl font-bold text-text-primary">
              Check your npm dependencies
            </h1>
            <p className="text-text-secondary text-base sm:text-lg max-w-xl mx-auto">
              Scan your package-lock.json for compromised packages, known vulnerabilities, and malware. No signup required.
            </p>
          </div>

          {/* Form card */}
          <div
            className="bg-surface-secondary rounded-[var(--card-radius)] shadow-lg border border-border-subtle p-6 sm:p-8"
          >
            <SupplyChainForm />
          </div>

          {/* Supported ecosystems */}
          <div className="text-center space-y-2">
            <p className="text-xs text-text-tertiary">
              Currently supports <span className="font-medium text-text-secondary">npm</span> (package-lock.json v1, v2, v3).
              {' '}More ecosystems coming soon.
            </p>
          </div>
        </div>
      </PageContainer>
    </main>
  )
}
