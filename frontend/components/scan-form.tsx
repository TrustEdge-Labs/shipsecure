'use client'

import { useActionState, useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { submitScan, type ScanFormState } from '@/app/actions/scan'

interface ScanFormProps {
  isAuthenticated?: boolean
}

export function ScanForm({ isAuthenticated = false }: ScanFormProps) {
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
      <div className="text-center p-8 bg-success-bg rounded-xl border border-success-border">
        <div className="text-4xl mb-3">&#10003;</div>
        <h2 className="text-xl font-semibold text-success-text mb-2">
          Scan started!
        </h2>
        <p className="text-success-primary">
          Redirecting to your scan progress...
        </p>
      </div>
    )
  }

  return (
    <form action={formAction} className="space-y-4">
      {state.errors?._form && (
        <div className="p-3 bg-danger-bg border border-danger-border rounded-lg text-danger-text text-sm">
          {state.errors._form[0].startsWith('DOMAIN_VERIFICATION_REQUIRED:') ? (
            <>
              Please verify ownership of <strong>{state.errors._form[0].split(':')[1]}</strong> first.{' '}
              <a href="/verify-domain" className="underline font-medium hover:text-danger-text/80">
                Verify your domain to get started
              </a>
            </>
          ) : state.errors._form[0].startsWith('RATE_LIMITED:') ? (
            <>
              {state.errors._form[0].replace('RATE_LIMITED:', '')}
              {!isAuthenticated && (
                <>
                  {' '}
                  <a href="/sign-in" className="underline font-medium hover:text-danger-text/80">
                    Sign in for 5 scans/month
                  </a>
                </>
              )}
            </>
          ) : (
            state.errors._form[0]
          )}
        </div>
      )}

      <div>
        <label htmlFor="url" className="block text-sm font-medium text-text-secondary mb-1">
          Website URL
        </label>
        <input
          id="url"
          name="url"
          type="url"
          placeholder="https://your-app.vercel.app"
          required
          className="w-full px-4 py-3 rounded-lg border border-border-default bg-surface-elevated text-text-primary focus:ring-2 focus:ring-focus-ring focus:border-focus-ring outline-none transition"
        />
        {state.errors?.url && (
          <p className="mt-1 text-sm text-danger-primary">{state.errors.url[0]}</p>
        )}
      </div>

      <div>
        <label htmlFor="email" className="block text-sm font-medium text-text-secondary mb-1">
          Email (for results notification)
        </label>
        <input
          id="email"
          name="email"
          type="email"
          placeholder="you@example.com"
          required
          className="w-full px-4 py-3 rounded-lg border border-border-default bg-surface-elevated text-text-primary focus:ring-2 focus:ring-focus-ring focus:border-focus-ring outline-none transition"
        />
        {state.errors?.email && (
          <p className="mt-1 text-sm text-danger-primary">{state.errors.email[0]}</p>
        )}
      </div>

      <div className="border-t border-border-default pt-4 mt-2">
        <div className="flex items-start gap-3">
          <input
            type="checkbox"
            id="authorization"
            name="authorization"
            required
            className="mt-1 w-4 h-4 rounded border-border-default text-brand-primary focus:ring-focus-ring"
          />
          <label htmlFor="authorization" className="text-sm text-text-secondary">
            I confirm I own this website or have explicit authorization from the owner to conduct security scanning. Unauthorized scanning may violate the <a href="/terms#acceptable-use" target="_blank" className="text-brand-primary underline">Computer Fraud and Abuse Act (CFAA)</a>.
          </label>
        </div>
        {state.errors?.authorization && (
          <p className="mt-1 text-sm text-danger-primary">{state.errors.authorization[0]}</p>
        )}
      </div>

      <button
        type="submit"
        disabled={pending}
        className="w-full py-3 px-6 rounded-lg bg-brand-primary hover:bg-brand-primary-hover disabled:bg-brand-accent text-white font-semibold transition text-lg"
      >
        {pending ? 'Starting scan...' : 'Scan Now — Free'}
      </button>

      <div className="text-xs text-text-tertiary text-center space-y-1">
        <p>
          {isAuthenticated
            ? '5 scans per month included with your account.'
            : '1 free scan per day per email. Sign in for 5 scans/month.'
          }
        </p>
        <p>
          By submitting, you agree to our <a href="/terms" className="underline">Terms of Service</a> and <a href="/privacy" className="underline">Privacy Policy</a>.
        </p>
      </div>
    </form>
  )
}
