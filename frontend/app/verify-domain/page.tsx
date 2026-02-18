'use client'

import { useState } from 'react'
import { useAuth } from '@clerk/nextjs'
import { useRouter } from 'next/navigation'
import Link from 'next/link'
import { CheckCircle2, XCircle, Loader2, AlertTriangle } from 'lucide-react'
import { MetaTagSnippet } from '@/components/meta-tag-snippet'
import { verifyStart, verifyConfirm, verifyCheck } from '@/lib/api'
import type { VerifyStartResponse } from '@/lib/types'

type WizardStep = 'input' | 'snippet' | 'verifying' | 'success' | 'failed'

// Shared-hosting root TLDs that cannot be verified (subdomains are allowed)
const BLOCKED_ROOT_TLDS = ['github.io', 'vercel.app', 'netlify.app', 'pages.dev']

function normalizeDomain(raw: string): string {
  let d = raw.trim().toLowerCase()
  // Strip scheme
  d = d.replace(/^https?:\/\//, '')
  // Strip path and query
  d = d.split('/')[0].split('?')[0]
  // Strip www.
  if (d.startsWith('www.')) d = d.slice(4)
  return d
}

function isBlockedRootTld(domain: string): string | null {
  for (const tld of BLOCKED_ROOT_TLDS) {
    if (domain === tld) return tld
  }
  return null
}

export default function VerifyDomainPage() {
  const { getToken } = useAuth()
  const router = useRouter()

  const [step, setStep] = useState<WizardStep>('input')
  const [domain, setDomain] = useState('')
  const [verifyData, setVerifyData] = useState<VerifyStartResponse | null>(null)
  const [confirmedExpiresAt, setConfirmedExpiresAt] = useState<string | null>(null)
  const [failureReason, setFailureReason] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const [testResult, setTestResult] = useState<{ found: boolean; message: string } | null>(null)
  const [alreadyVerified, setAlreadyVerified] = useState<{ domain: string; expiresInDays: number } | null>(null)

  async function handleStart(e: React.FormEvent) {
    e.preventDefault()
    setError(null)
    setAlreadyVerified(null)

    const normalized = normalizeDomain(domain)
    if (!normalized) {
      setError('Please enter a domain name.')
      return
    }

    const blocked = isBlockedRootTld(normalized)
    if (blocked) {
      setError(
        `'${normalized}' is a shared hosting platform. Enter your app's subdomain instead (e.g., myapp.${normalized}).`
      )
      return
    }

    setLoading(true)
    try {
      const token = await getToken()
      if (!token) throw new Error('Not authenticated')
      const data = await verifyStart(normalized, token)

      if (data.already_verified) {
        setAlreadyVerified({
          domain: data.domain,
          expiresInDays: data.expires_in_days ?? 0,
        })
        return
      }

      setVerifyData(data)
      setStep('snippet')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Something went wrong. Please try again.')
    } finally {
      setLoading(false)
    }
  }

  async function handleTestTag() {
    if (!verifyData) return
    setTestResult(null)
    setLoading(true)
    try {
      const token = await getToken()
      if (!token) throw new Error('Not authenticated')
      const result = await verifyCheck(verifyData.domain, token)
      setTestResult({ found: result.found, message: result.message })
    } catch (err) {
      setTestResult({ found: false, message: err instanceof Error ? err.message : 'Could not check tag.' })
    } finally {
      setLoading(false)
    }
  }

  async function handleVerifyNow() {
    if (!verifyData) return
    setStep('verifying')
    try {
      const token = await getToken()
      if (!token) throw new Error('Not authenticated')
      const result = await verifyConfirm(verifyData.domain, token)
      if (result.verified) {
        setConfirmedExpiresAt(result.expires_at ?? null)
        setStep('success')
      } else {
        setFailureReason(
          result.failure_reason ??
            "Verification failed. Please check that the meta tag is in your page's <head> section."
        )
        setStep('failed')
      }
    } catch (err) {
      setFailureReason(err instanceof Error ? err.message : 'An unexpected error occurred during verification.')
      setStep('failed')
    }
  }

  return (
    <main className="container mx-auto px-4 py-16 max-w-lg">
      <div className="bg-surface-elevated border border-border-subtle rounded-xl p-8 shadow-sm">
        <h1 className="text-2xl font-bold text-text-primary mb-6">
          Verify Domain Ownership
        </h1>

        {/* Step: Input */}
        {step === 'input' && (
          <div>
            {alreadyVerified ? (
              <div className="mb-6 p-4 bg-success-bg border border-success-border rounded-lg">
                <p className="text-success-text font-medium">
                  This domain is already verified. Expires in {alreadyVerified.expiresInDays} days.
                </p>
                <Link
                  href="/dashboard"
                  className="mt-3 inline-block text-sm text-brand-primary hover:underline"
                >
                  Return to Dashboard
                </Link>
              </div>
            ) : (
              <p className="text-text-secondary mb-6">
                Enter your domain to receive a meta tag snippet. Place it in your site&apos;s{' '}
                <code className="bg-surface-secondary px-1 py-0.5 rounded text-sm font-mono">&lt;head&gt;</code>{' '}
                to prove you control the domain.
              </p>
            )}

            <form onSubmit={handleStart} className="space-y-4">
              <div>
                <label htmlFor="domain" className="block text-sm font-medium text-text-primary mb-1.5">
                  Domain
                </label>
                <input
                  id="domain"
                  type="text"
                  value={domain}
                  onChange={(e) => {
                    setDomain(e.target.value)
                    setError(null)
                    setAlreadyVerified(null)
                  }}
                  placeholder="myapp.vercel.app"
                  className="w-full px-3 py-2 border border-border-default rounded-lg bg-surface-primary text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-2 focus:ring-focus-ring"
                  disabled={loading}
                  autoFocus
                />
                {error && (
                  <p className="mt-2 text-sm text-danger-text flex items-start gap-1.5">
                    <AlertTriangle className="w-4 h-4 mt-0.5 shrink-0" aria-hidden="true" />
                    {error}
                  </p>
                )}
              </div>

              <button
                type="submit"
                disabled={loading || !domain.trim()}
                className="w-full px-4 py-2.5 bg-brand-primary hover:bg-brand-primary-hover text-text-inverse font-semibold rounded-lg transition disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {loading ? (
                  <span className="flex items-center justify-center gap-2">
                    <Loader2 className="w-4 h-4 animate-spin" aria-hidden="true" />
                    Starting verification...
                  </span>
                ) : (
                  'Start Verification'
                )}
              </button>
            </form>
          </div>
        )}

        {/* Step: Snippet */}
        {step === 'snippet' && verifyData && (
          <div className="space-y-5">
            <p className="text-text-secondary">
              Add this meta tag to the{' '}
              <code className="bg-surface-secondary px-1 py-0.5 rounded text-sm font-mono">&lt;head&gt;</code>{' '}
              section of your site at{' '}
              <span className="font-medium text-text-primary">https://{verifyData.domain}/</span>
            </p>

            <MetaTagSnippet metaTag={verifyData.meta_tag} />

            {testResult && (
              <div
                className={`p-3 rounded-lg border text-sm flex items-start gap-2 ${
                  testResult.found
                    ? 'bg-success-bg border-success-border text-success-text'
                    : 'bg-danger-bg border-danger-border text-danger-text'
                }`}
              >
                {testResult.found ? (
                  <CheckCircle2 className="w-4 h-4 mt-0.5 shrink-0" aria-hidden="true" />
                ) : (
                  <AlertTriangle className="w-4 h-4 mt-0.5 shrink-0" aria-hidden="true" />
                )}
                {testResult.message}
              </div>
            )}

            <div className="flex gap-3">
              <button
                onClick={handleTestTag}
                disabled={loading}
                className="flex-1 px-4 py-2.5 border border-border-default text-text-primary font-medium rounded-lg hover:bg-surface-secondary transition disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {loading ? (
                  <span className="flex items-center justify-center gap-2">
                    <Loader2 className="w-4 h-4 animate-spin" aria-hidden="true" />
                    Checking...
                  </span>
                ) : (
                  'Test my tag'
                )}
              </button>

              <button
                onClick={handleVerifyNow}
                disabled={loading}
                className="flex-1 px-4 py-2.5 bg-brand-primary hover:bg-brand-primary-hover text-text-inverse font-semibold rounded-lg transition disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Verify now
              </button>
            </div>
          </div>
        )}

        {/* Step: Verifying */}
        {step === 'verifying' && (
          <div className="flex flex-col items-center py-8 gap-4">
            <Loader2 className="w-10 h-10 animate-spin text-brand-primary" aria-hidden="true" />
            <p className="text-text-secondary font-medium">Verifying your domain...</p>
          </div>
        )}

        {/* Step: Success */}
        {step === 'success' && verifyData && (
          <div className="flex flex-col items-center text-center py-4 gap-4">
            <CheckCircle2 className="w-12 h-12 text-success-primary" aria-hidden="true" />
            <div>
              <h2 className="text-xl font-bold text-text-primary">Domain verified!</h2>
              <p className="mt-1 text-text-secondary font-medium">{verifyData.domain}</p>
              {confirmedExpiresAt && (
                <p className="mt-1 text-sm text-text-tertiary">
                  Expires{' '}
                  {new Date(confirmedExpiresAt).toLocaleDateString(undefined, {
                    year: 'numeric',
                    month: 'long',
                    day: 'numeric',
                  })}
                </p>
              )}
            </div>
            <button
              onClick={() => router.push('/dashboard')}
              className="mt-2 px-6 py-2.5 bg-brand-primary hover:bg-brand-primary-hover text-text-inverse font-semibold rounded-lg transition"
            >
              Go to Dashboard
            </button>
          </div>
        )}

        {/* Step: Failed */}
        {step === 'failed' && (
          <div className="flex flex-col items-center text-center py-4 gap-4">
            <XCircle className="w-12 h-12 text-danger-primary" aria-hidden="true" />
            <div>
              <h2 className="text-xl font-bold text-text-primary">Verification failed</h2>
              {failureReason && (
                <p className="mt-2 text-sm text-text-secondary max-w-sm">{failureReason}</p>
              )}
            </div>
            <button
              onClick={() => setStep('snippet')}
              className="mt-2 px-6 py-2.5 border border-border-default text-text-primary font-medium rounded-lg hover:bg-surface-secondary transition"
            >
              Try again
            </button>
          </div>
        )}
      </div>
    </main>
  )
}
