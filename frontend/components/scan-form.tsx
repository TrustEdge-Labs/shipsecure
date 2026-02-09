'use client'

import { useActionState, useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { submitScan, type ScanFormState } from '@/app/actions/scan'

export function ScanForm() {
  const [state, formAction, pending] = useActionState(submitScan, {} as ScanFormState)
  const router = useRouter()

  useEffect(() => {
    if (state.scanId) {
      window.plausible?.('Scan Submitted', { props: { scan_type: 'url' } })
      const timer = setTimeout(() => {
        router.push(`/scan/${state.scanId}`)
      }, 2500)
      return () => clearTimeout(timer)
    }
  }, [state.scanId, router])

  if (state.scanId) {
    return (
      <div className="text-center p-8 bg-green-50 dark:bg-green-950 rounded-xl border border-green-200 dark:border-green-800">
        <div className="text-4xl mb-3">&#10003;</div>
        <h2 className="text-xl font-semibold text-green-800 dark:text-green-200 mb-2">
          Scan started!
        </h2>
        <p className="text-green-600 dark:text-green-400">
          Redirecting to your scan progress...
        </p>
      </div>
    )
  }

  return (
    <form action={formAction} className="space-y-4">
      {state.errors?._form && (
        <div className="p-3 bg-red-50 dark:bg-red-950 border border-red-200 dark:border-red-800 rounded-lg text-red-700 dark:text-red-300 text-sm">
          {state.errors._form[0]}
        </div>
      )}

      <div>
        <label htmlFor="url" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          Website URL
        </label>
        <input
          id="url"
          name="url"
          type="url"
          placeholder="https://your-app.vercel.app"
          required
          className="w-full px-4 py-3 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition"
        />
        {state.errors?.url && (
          <p className="mt-1 text-sm text-red-600 dark:text-red-400">{state.errors.url[0]}</p>
        )}
      </div>

      <div>
        <label htmlFor="email" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          Email (for results notification)
        </label>
        <input
          id="email"
          name="email"
          type="email"
          placeholder="you@example.com"
          required
          className="w-full px-4 py-3 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition"
        />
        {state.errors?.email && (
          <p className="mt-1 text-sm text-red-600 dark:text-red-400">{state.errors.email[0]}</p>
        )}
      </div>

      <div className="border-t border-gray-200 dark:border-gray-700 pt-4 mt-2">
        <div className="flex items-start gap-3">
          <input
            type="checkbox"
            id="authorization"
            name="authorization"
            required
            className="mt-1 w-4 h-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
          />
          <label htmlFor="authorization" className="text-sm text-gray-700 dark:text-gray-300">
            I confirm I own this website or have explicit authorization from the owner to conduct security scanning. Unauthorized scanning may violate the <a href="/terms#acceptable-use" target="_blank" className="text-blue-600 dark:text-blue-400 underline">Computer Fraud and Abuse Act (CFAA)</a>.
          </label>
        </div>
        {state.errors?.authorization && (
          <p className="mt-1 text-sm text-red-600 dark:text-red-400">{state.errors.authorization[0]}</p>
        )}
      </div>

      <button
        type="submit"
        disabled={pending}
        className="w-full py-3 px-6 rounded-lg bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white font-semibold transition text-lg"
      >
        {pending ? 'Starting scan...' : 'Scan Now — Free'}
      </button>

      <p className="text-xs text-gray-500 text-center">
        By submitting, you agree to our <a href="/terms" className="underline">Terms of Service</a> and <a href="/privacy" className="underline">Privacy Policy</a>.
      </p>
    </form>
  )
}
