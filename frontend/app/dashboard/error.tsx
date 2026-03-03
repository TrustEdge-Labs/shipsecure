'use client'

import { useEffect } from 'react'
import Link from 'next/link'

export default function DashboardError({
  error,
  reset,
}: {
  error: Error & { digest?: string }
  reset: () => void
}) {
  useEffect(() => {
    console.error('Dashboard error boundary caught:', error)
  }, [error])

  return (
    <div className="min-h-screen bg-surface-secondary flex items-center justify-center p-4">
      <div className="bg-surface-elevated rounded-(card) shadow-md p-8 max-w-md w-full text-center">
        <h1 className="text-2xl font-bold text-text-primary mb-2">
          Could not load dashboard
        </h1>
        <p className="text-text-secondary mb-6">
          Something went wrong loading your dashboard data. This is usually temporary.
        </p>
        <div className="flex flex-col gap-3">
          <button
            onClick={reset}
            className="min-h-[44px] bg-brand-primary hover:bg-brand-primary/90 text-white px-6 py-3 rounded-lg font-medium transition-colors"
          >
            Try again
          </button>
          <Link
            href="/"
            className="min-h-[44px] flex items-center justify-center border border-border-default text-text-secondary px-6 py-3 rounded-lg font-medium hover:bg-surface-secondary transition-colors"
          >
            Return to Home
          </Link>
        </div>
      </div>
    </div>
  )
}
