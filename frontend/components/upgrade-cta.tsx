'use client'

import { useState } from 'react'

interface UpgradeCTAProps {
  scanId: string
  token: string
}

export function UpgradeCTA({ scanId, token }: UpgradeCTAProps) {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleUpgrade = async () => {
    setLoading(true)
    setError(null)

    try {
      const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:3000'
      const res = await fetch(`${BACKEND_URL}/api/v1/checkout`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ scan_id: scanId }),
      })

      if (!res.ok) {
        const errorData = await res.json().catch(() => ({ title: 'Failed to start checkout' }))
        throw new Error(errorData.title || 'Failed to start checkout')
      }

      const data = await res.json()

      // Redirect to Stripe Checkout
      window.location.href = data.checkout_url
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An error occurred')
      setLoading(false)
    }
  }

  return (
    <div className="bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 border-2 border-blue-200 dark:border-blue-700 rounded-lg p-6">
      <h3 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-3">
        Upgrade to Deep Audit
      </h3>

      <p className="text-gray-700 dark:text-gray-300 mb-4">
        Get a comprehensive security assessment with active probing and detailed PDF report.
      </p>

      <ul className="space-y-2 mb-6">
        <li className="flex items-start gap-2 text-gray-700 dark:text-gray-300">
          <svg className="w-5 h-5 text-blue-600 dark:text-blue-400 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
          </svg>
          <span><strong>10x more checks</strong> — active vulnerability probing</span>
        </li>
        <li className="flex items-start gap-2 text-gray-700 dark:text-gray-300">
          <svg className="w-5 h-5 text-blue-600 dark:text-blue-400 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
          </svg>
          <span><strong>Active probing</strong> — SQL injection, auth bypass, file inclusion tests</span>
        </li>
        <li className="flex items-start gap-2 text-gray-700 dark:text-gray-300">
          <svg className="w-5 h-5 text-blue-600 dark:text-blue-400 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
          </svg>
          <span><strong>PDF report</strong> — professional executive summary and remediation roadmap</span>
        </li>
        <li className="flex items-start gap-2 text-gray-700 dark:text-gray-300">
          <svg className="w-5 h-5 text-blue-600 dark:text-blue-400 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
          </svg>
          <span><strong>Extended JS analysis</strong> — scan 50 files vs 20 in free tier</span>
        </li>
      </ul>

      {error && (
        <div className="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
          <p className="text-sm text-red-800 dark:text-red-300">{error}</p>
        </div>
      )}

      <div className="flex items-center gap-4">
        <button
          onClick={handleUpgrade}
          disabled={loading}
          className="px-6 py-3 min-h-[44px] bg-blue-600 text-white font-semibold rounded-md hover:bg-blue-700 disabled:bg-blue-400 disabled:cursor-not-allowed transition-colors"
        >
          {loading ? 'Redirecting to checkout...' : 'Upgrade for $49'}
        </button>
        <span className="text-sm text-gray-600 dark:text-gray-400">One-time payment</span>
      </div>
    </div>
  )
}
