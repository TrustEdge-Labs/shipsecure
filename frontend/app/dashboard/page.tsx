import { auth, currentUser } from '@clerk/nextjs/server'
import { redirect } from 'next/navigation'
import Link from 'next/link'
import { DomainBadge } from '@/components/domain-badge'
import type { VerifiedDomain, QuotaResponse } from '@/lib/types'

export default async function DashboardPage() {
  const { userId, getToken } = await auth()
  if (!userId) redirect('/sign-in')

  const user = await currentUser()

  // Fetch verified domains server-side
  const sessionToken = await getToken()
  const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:3000'
  const domainsRes = await fetch(`${BACKEND_URL}/api/v1/domains`, {
    cache: 'no-store',
    headers: sessionToken ? { 'Authorization': `Bearer ${sessionToken}` } : {},
  })
  const domains: VerifiedDomain[] = domainsRes.ok ? await domainsRes.json() : []

  // Fetch quota info server-side
  const quotaRes = await fetch(`${BACKEND_URL}/api/v1/quota`, {
    cache: 'no-store',
    headers: sessionToken ? { 'Authorization': `Bearer ${sessionToken}` } : {},
  })
  const quota: QuotaResponse | null = quotaRes.ok ? await quotaRes.json() : null

  function getQuotaStyle(used: number, limit: number) {
    const ratio = used / limit
    if (ratio >= 1.0) return 'bg-danger-bg text-danger-text border-danger-border'
    if (ratio >= 0.6) return 'bg-caution-bg text-caution-text border-caution-border'
    return 'bg-success-bg text-success-text border-success-border'
  }

  return (
    <main className="container mx-auto px-4 py-16 max-w-4xl">
      <h1 className="text-3xl font-bold text-text-primary mb-2">
        Welcome, {user?.firstName ?? 'there'}
      </h1>
      <p className="text-text-secondary mb-4">
        Your security dashboard. Verify a domain to start scanning.
      </p>

      {quota && (
        <div className={`inline-flex items-center gap-2 px-3 py-1.5 rounded-lg border text-sm font-medium mb-6 ${getQuotaStyle(quota.used, quota.limit)}`}>
          <span>{quota.used}/{quota.limit} scans</span>
          <span className="text-xs opacity-75">this month</span>
        </div>
      )}

      {/* Verified Domains Section */}
      <section className="mb-10">
        <h2 className="text-xl font-semibold text-text-primary mb-4">Verified Domains</h2>

        {domains.length === 0 ? (
          <div className="border border-border-subtle rounded-xl p-6 bg-surface-secondary text-center">
            <p className="text-text-secondary mb-4">No domains verified yet.</p>
            <Link
              href="/verify-domain"
              className="inline-flex items-center px-4 py-2 bg-brand-primary hover:bg-brand-primary-hover text-text-inverse font-semibold rounded-lg transition text-sm"
            >
              Verify a Domain
            </Link>
          </div>
        ) : (
          <div className="border border-border-subtle rounded-xl overflow-hidden">
            <ul className="divide-y divide-border-subtle">
              {domains.map((d) => {
                const isExpiredOrExpiring =
                  d.status === 'verified' &&
                  d.expires_at !== null &&
                  Math.ceil(
                    (new Date(d.expires_at).getTime() - Date.now()) / (1000 * 60 * 60 * 24)
                  ) <= 7

                return (
                  <li key={d.id} className="flex items-center justify-between px-5 py-3.5 bg-surface-elevated">
                    <div className="flex items-center gap-3 min-w-0">
                      <span className="font-mono text-sm text-text-primary truncate">{d.domain}</span>
                      <DomainBadge status={d.status} expiresAt={d.expires_at} />
                    </div>
                    {isExpiredOrExpiring && (
                      <Link
                        href="/verify-domain"
                        className="ml-3 text-xs text-brand-primary hover:underline shrink-0"
                      >
                        Re-verify
                      </Link>
                    )}
                  </li>
                )
              })}
            </ul>
            <div className="px-5 py-3 bg-surface-secondary border-t border-border-subtle">
              <Link
                href="/verify-domain"
                className="text-sm text-brand-primary hover:underline font-medium"
              >
                Verify a Domain
              </Link>
            </div>
          </div>
        )}
      </section>
    </main>
  )
}
