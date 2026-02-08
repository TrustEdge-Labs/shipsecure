import { notFound } from 'next/navigation'
import { GradeSummary } from '@/components/grade-summary'
import { ResultsDashboard } from '@/components/results-dashboard'
import { UpgradeCTA } from '@/components/upgrade-cta'
import { ScanResponse } from '@/lib/types'

interface ResultsPageProps {
  params: Promise<{
    token: string
  }>
}

export async function generateMetadata({ params }: ResultsPageProps) {
  const { token } = await params

  try {
    const BACKEND_URL = process.env.BACKEND_URL || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:3000'
    const res = await fetch(`${BACKEND_URL}/api/v1/results/${token}`, {
      cache: 'no-store',
    })

    if (!res.ok) {
      return {
        title: 'Results Not Found - ShipSecure',
        robots: { index: false },
      }
    }

    const data: ScanResponse = await res.json()

    return {
      title: `Security Scan: ${data.score || 'In Progress'} Grade - ShipSecure`,
      description: `Security scan results for ${data.target_url}`,
      robots: { index: false },
    }
  } catch (error) {
    return {
      title: 'Results Not Found - ShipSecure',
      robots: { index: false },
    }
  }
}

export default async function ResultsPage({ params }: ResultsPageProps) {
  const { token } = await params

  // Fetch results server-side
  const BACKEND_URL = process.env.BACKEND_URL || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:3000'

  let data: ScanResponse
  try {
    const res = await fetch(`${BACKEND_URL}/api/v1/results/${token}`, {
      cache: 'no-store',
    })

    if (!res.ok) {
      notFound()
    }

    data = await res.json()
  } catch (error) {
    console.error('Error fetching results:', error)
    notFound()
  }

  // If scan not completed yet, show in-progress message
  if (data.status !== 'completed') {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-950 flex items-center justify-center p-4">
        <div className="bg-white dark:bg-gray-900 rounded-lg shadow-md p-8 max-w-md w-full text-center">
          <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mb-4"></div>
          <h1 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
            Scan Still In Progress
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mb-6">
            Your scan is still running. Please check back in a few moments.
          </p>
          <a
            href={`/scan/${data.id}`}
            className="inline-block bg-blue-600 text-white px-6 py-2 rounded-md hover:bg-blue-700 transition-colors"
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
      return 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800 text-red-800 dark:text-red-300'
    }
    return 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800 text-blue-800 dark:text-blue-300'
  }

  const downloadUrl = `${process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:3000'}/api/v1/results/${token}/download`

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 py-8 px-4">
      <div className="max-w-4xl mx-auto">
        {/* Header */}
        <div className="bg-white dark:bg-gray-900 rounded-lg shadow-md p-6 mb-6">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-4">
            Security Scan Results
          </h1>

          <div className="space-y-2 text-sm">
            <div>
              <span className="text-gray-600 dark:text-gray-400">Target: </span>
              <a
                href={data.target_url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 dark:text-blue-400 hover:underline font-mono"
              >
                {data.target_url}
              </a>
            </div>
            <div>
              <span className="text-gray-600 dark:text-gray-400">Scanned: </span>
              <span className="text-gray-900 dark:text-gray-100">{formatDate(data.completed_at)}</span>
            </div>
            {data.expires_at && (
              <div className={`inline-block px-3 py-1 rounded-md border ${getExpiryWarning(data.expires_at)}`}>
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
        <div className="bg-white dark:bg-gray-900 rounded-lg shadow-md p-6 mb-6">
          <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100 mb-4">
            Security Findings
          </h2>
          <ResultsDashboard findings={data.findings} />
        </div>

        {/* Upgrade CTA (only for free tier) */}
        {data.tier === 'free' && (
          <div className="mb-6">
            <UpgradeCTA scanId={data.id} token={token} />
          </div>
        )}

        {/* Actions */}
        <div className="flex gap-4 flex-wrap">
          <a
            href={downloadUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 px-6 py-2 border-2 border-blue-600 text-blue-600 dark:text-blue-400 dark:border-blue-400 rounded-md hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-colors"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
            Download Markdown Report
          </a>

          <a
            href={`/?url=${encodeURIComponent(data.target_url)}`}
            className="inline-flex items-center gap-2 px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
          >
            Fixed some issues? Scan again
          </a>
        </div>
      </div>
    </div>
  )
}
