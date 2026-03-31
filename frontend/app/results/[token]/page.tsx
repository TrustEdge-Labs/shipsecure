import { notFound } from 'next/navigation'
import { auth } from '@clerk/nextjs/server'
import { GradeSummary } from '@/components/grade-summary'
import { ResultsDashboard } from '@/components/results-dashboard'
import { ShareButton } from '@/components/share-button'
import { ScanResponse } from '@/lib/types'

const DEMO_TARGET_HOST = 'demo.owasp-juice.shop'

interface ResultsPageProps {
  params: Promise<{
    token: string
  }>
}

export async function generateMetadata({ params }: ResultsPageProps) {
  const { token } = await params

  try {
    const BACKEND_URL = process.env.BACKEND_URL || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:3000'
    const { getToken } = await auth()
    const sessionToken = await getToken()
    const metaHeaders: Record<string, string> = {}
    if (sessionToken) {
      metaHeaders['Authorization'] = `Bearer ${sessionToken}`
    }
    const res = await fetch(`${BACKEND_URL}/api/v1/results/${token}`, {
      cache: 'no-store',
      headers: metaHeaders,
    })

    if (!res.ok) {
      return {
        title: 'Results Not Found - ShipSecure',
        robots: {
          index: false,
          follow: false,
          nocache: true,
        },
      }
    }

    const data: ScanResponse = await res.json()

    // Extract domain from target_url for title
    const domain = (() => {
      try { return new URL(data.target_url).hostname } catch { return data.target_url }
    })()

    // In-progress scans keep simple title (no OG enrichment)
    if (data.status !== 'completed' && data.status !== 'expired') {
      return {
        title: 'Security Scan: In Progress - ShipSecure',
        robots: { index: false, follow: false, nocache: true },
      }
    }

    // Expired scans get a tombstone title
    if (data.status === 'expired') {
      return {
        title: `${domain} - Results Expired | ShipSecure`,
        description: `These scan results have expired. Scan ${domain} again free at shipsecure.ai.`,
        openGraph: {
          title: `${domain} - Results Expired | ShipSecure`,
          description: `These scan results have expired. Scan ${domain} again free at shipsecure.ai.`,
          type: 'website',
          siteName: 'ShipSecure',
          url: `https://shipsecure.ai/results/${token}`,
        },
        twitter: {
          card: 'summary' as const,
          title: `${domain} - Results Expired | ShipSecure`,
          description: `These scan results have expired. Scan ${domain} again free at shipsecure.ai.`,
        },
        robots: { index: false, follow: false, nocache: true },
      }
    }

    // Completed scan — enrich OG tags with grade and finding counts
    const title = data.score
      ? `${domain} - Grade ${data.score} | ShipSecure`
      : `${domain} - Security Scan | ShipSecure`

    const highCount = data.summary?.high || 0
    const criticalCount = data.summary?.critical || 0
    const totalCount = data.summary?.total || 0
    const severeCount = highCount + criticalCount
    const description = totalCount > 0
      ? `Security scan found ${totalCount} issues. ${severeCount} high/critical severity. Scan your app free at shipsecure.ai.`
      : `Security scan complete. Scan your app free at shipsecure.ai.`

    return {
      title,
      description,
      openGraph: {
        title,
        description,
        type: 'website',
        siteName: 'ShipSecure',
        url: `https://shipsecure.ai/results/${token}`,
      },
      twitter: {
        card: 'summary' as const,
        title,
        description,
      },
      robots: { index: false, follow: false, nocache: true },
    }
  } catch {
    return {
      title: 'Results Not Found - ShipSecure',
      robots: {
        index: false,
        follow: false,
        nocache: true,
      },
    }
  }
}

export default async function ResultsPage({ params }: ResultsPageProps) {
  const { token } = await params

  // Extract Clerk session token (if authenticated) to forward to backend
  const { getToken } = await auth()
  const sessionToken = await getToken()
  const requestHeaders: Record<string, string> = {}
  if (sessionToken) {
    requestHeaders['Authorization'] = `Bearer ${sessionToken}`
  }

  // Fetch results server-side
  const BACKEND_URL = process.env.BACKEND_URL || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:3000'

  let data: ScanResponse
  try {
    const res = await fetch(`${BACKEND_URL}/api/v1/results/${token}`, {
      cache: 'no-store',
      headers: requestHeaders,
    })

    if (!res.ok) {
      notFound()
    }

    data = await res.json()
  } catch (error) {
    console.error('Error fetching results:', error)
    notFound()
  }

  // Expired results: dedicated page with scan-again CTA
  if (data.status === 'expired') {
    return (
      <div className="min-h-screen bg-surface-secondary py-8 px-4">
        <div className="max-w-md mx-auto mt-16">
          <div className="bg-surface-elevated rounded-(card) shadow-md p-8 text-center">
            <div className="text-4xl mb-4">
              <svg className="w-12 h-12 mx-auto text-text-secondary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <h1 className="text-xl font-semibold text-text-primary mb-2">
              These results have expired
            </h1>
            <p className="text-text-secondary mb-6">
              Free scan results are available for 24 hours.
            </p>
            <a
              href={`/?url=${encodeURIComponent(data.target_url)}`}
              className="inline-flex items-center justify-center gap-2 px-6 py-3 min-h-[44px] w-full bg-brand-primary text-white rounded-md hover:bg-brand-primary/90 transition-colors font-medium"
            >
              Scan again
            </a>
            <p className="text-text-tertiary text-sm mt-4">
              <a href="/sign-up" className="text-brand-primary hover:underline">Sign up</a> for 30-day scan history
            </p>
          </div>
        </div>
      </div>
    )
  }

  // If scan not completed yet, show in-progress message
  if (data.status !== 'completed') {
    return (
      <div className="min-h-screen bg-surface-secondary flex items-center justify-center p-4">
        <div className="bg-surface-elevated rounded-(card) shadow-md p-8 max-w-md w-full text-center">
          <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-brand-primary mb-4"></div>
          <h1 className="text-xl font-semibold text-text-primary mb-2">
            Scan Still In Progress
          </h1>
          <p className="text-text-secondary mb-6">
            Your scan is still running. Please check back in a few moments.
          </p>
          <a
            href={`/scan/${data.id}`}
            className="inline-block bg-brand-primary text-white px-6 py-2 rounded-md hover:bg-brand-primary/90 transition-colors"
          >
            View Progress
          </a>
        </div>
      </div>
    )
  }

  // Format dates
  const formatDate = (dateStr: string | null) => {
    if (!dateStr) return 'N/A'
    return new Date(dateStr).toLocaleString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
  }

  const getExpiryWarning = (expiresAt: string | null) => {
    if (!expiresAt) return null

    const expiry = new Date(expiresAt)
    const now = new Date()
    const hoursUntilExpiry = (expiry.getTime() - now.getTime()) / (1000 * 60 * 60)

    if (hoursUntilExpiry < 24) {
      return 'bg-danger-bg border-danger-border text-danger-text'
    }
    return 'bg-info-bg border-info-border text-info-text'
  }

  const downloadUrl = `/api/v1/results/${token}/download`
  const isDemoTarget = (() => {
    try {
      return new URL(data.target_url).hostname === DEMO_TARGET_HOST
    } catch {
      return false
    }
  })()
  const isAnonymous = !sessionToken

  return (
    <div className="min-h-screen bg-surface-secondary py-8 px-4">
      <div className="max-w-4xl mx-auto">
        {/* Header */}
        <div className="bg-surface-elevated rounded-(card) shadow-md p-6 mb-6">
          <h1 className="text-3xl font-bold text-text-primary mb-4">
            {isDemoTarget && isAnonymous
              ? `Live Scan Results for ${DEMO_TARGET_HOST}`
              : 'Security Scan Results'}
          </h1>

          <div className="space-y-2 text-sm">
            <div>
              <span className="text-text-secondary">Target: </span>
              <a
                href={data.target_url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-brand-primary hover:underline font-mono break-all"
              >
                {data.target_url}
              </a>
            </div>
            <div>
              <span className="text-text-secondary">Scanned: </span>
              <span className="text-text-primary">{formatDate(data.completed_at)}</span>
            </div>
            <div>
              <span className="text-text-secondary">Scan type: </span>
              {data.tier === 'free' ? (
                <span className="inline-flex items-center gap-1">
                  <span className="text-xs font-medium px-2 py-0.5 rounded-full bg-surface-secondary text-text-secondary border border-border-subtle">Basic scan</span>
                  <a href="/sign-up" className="text-xs text-brand-primary hover:underline ml-1">Sign up for deeper analysis</a>
                </span>
              ) : (
                <span className="text-xs font-medium px-2 py-0.5 rounded-full bg-brand-primary/10 text-brand-primary border border-brand-primary/20">Enhanced scan</span>
              )}
            </div>
            {data.expires_at && (
              <div className={`block sm:inline-block px-3 py-1 rounded-md border ${getExpiryWarning(data.expires_at)}`}>
                <span className="font-medium">Results available until: </span>
                {formatDate(data.expires_at)}
              </div>
            )}
          </div>
        </div>

        {/* Grade Summary */}
        <div className="mb-6">
          <GradeSummary
            grade={data.score || 'N/A'}
            summary={data.summary}
            framework={data.detected_framework}
            platform={data.detected_platform}
          />
        </div>

        {/* Findings Dashboard */}
        <div className="bg-surface-elevated rounded-(card) shadow-md p-6 mb-6">
          <h2 className="text-xl font-bold text-text-primary mb-4">
            Security Findings
          </h2>
          <ResultsDashboard findings={data.findings} />
        </div>

        {/* Actions */}
        <div className="flex gap-4 flex-wrap">
          <ShareButton url={`https://shipsecure.ai/results/${token}`} />

          {data.owner_verified && (
            <a
              href={downloadUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center justify-center gap-2 px-6 py-2 min-h-[44px] w-full sm:w-auto border-2 border-brand-primary text-brand-primary rounded-md hover:bg-info-bg transition-colors"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
              </svg>
              Download Markdown Report
            </a>
          )}

          <a
            href={`/?url=${encodeURIComponent(data.target_url)}`}
            className="inline-flex items-center justify-center gap-2 px-6 py-2 min-h-[44px] w-full sm:w-auto bg-brand-primary text-white rounded-md hover:bg-brand-primary/90 transition-colors"
          >
            Fixed some issues? Scan again
          </a>
        </div>

        {/* Demo target CTA for anonymous users */}
        {isDemoTarget && isAnonymous && (
          <div className="bg-info-bg border border-info-border rounded-(card) p-6 mt-6">
            <p className="text-sm text-info-text mb-3">
              <a
                href={`https://${DEMO_TARGET_HOST}`}
                target="_blank"
                rel="noopener noreferrer"
                className="font-medium text-brand-primary hover:underline"
              >
                OWASP Juice Shop
              </a>{' '}
              is a real, interactive website. You can visit the site and try to find these vulnerabilities yourself!
            </p>
            <p className="text-sm text-info-text">
              When you&apos;re ready,{' '}
              <a href="/sign-up" className="font-semibold text-brand-primary hover:underline">
                sign up
              </a>{' '}
              to protect your own applications.
            </p>
          </div>
        )}
      </div>
    </div>
  )
}
