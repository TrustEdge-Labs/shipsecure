'use client'

import { useEffect } from 'react'
import Link from 'next/link'
import { usePlausible } from 'next-plausible'

export default function PaymentSuccessPage() {
  const plausible = usePlausible()

  useEffect(() => {
    plausible('Audit Purchased', { props: { value: '49', currency: 'USD' } })
  }, [plausible])

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 flex items-center justify-center p-4">
      <div className="bg-white dark:bg-gray-900 rounded-lg shadow-md p-8 max-w-md w-full text-center">
        {/* Success Checkmark */}
        <div className="mx-auto w-16 h-16 bg-green-100 dark:bg-green-900/20 rounded-full flex items-center justify-center mb-4">
          <svg className="w-10 h-10 text-green-600 dark:text-green-400" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
          </svg>
        </div>

        <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-3">
          Payment Successful!
        </h1>

        <p className="text-gray-700 dark:text-gray-300 mb-2">
          Your deep security audit is now processing.
        </p>

        <p className="text-gray-600 dark:text-gray-400 mb-6">
          Typically takes 5-10 minutes. You'll receive an email with your PDF report when complete.
        </p>

        <Link
          href="/"
          className="inline-block bg-blue-600 text-white px-6 py-3 rounded-md hover:bg-blue-700 transition-colors font-medium"
        >
          Return to Home
        </Link>
      </div>
    </div>
  )
}
