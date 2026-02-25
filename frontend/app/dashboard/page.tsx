import { auth, currentUser } from '@clerk/nextjs/server'
import { redirect } from 'next/navigation'
import Link from 'next/link'
import { Loader2 } from 'lucide-react'
import { DomainBadge } from '@/components/domain-badge'
import { ScanHistoryTable } from '@/components/scan-history-table'
import { ActiveScansPoller } from '@/components/active-scans-poller'
import { PageContainer } from '@/components/page-container'
import type { VerifiedDomain, QuotaResponse, ScanHistoryResponse } from '@/lib/types'

interface DashboardPageProps {
  searchParams: Promise<{ page?: string }>
}

function getQuotaStyle(used: number, limit: number) {
  const ratio = used / limit
  if (ratio >= 1.0) return 'bg-danger-bg text-danger-text border-danger-border'
  if (ratio >= 0.6) return 'bg-caution-bg text-caution-text border-caution-border'
  return 'bg-success-bg text-success-text border-success-border'
}

function formatResetDate(resetsAt: string): string {
  return new Date(resetsAt).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })
}

function extractHostname(targetUrl: string): string {
  try {
    return new URL(targetUrl).hostname
  } catch {
    return targetUrl
  }
}

export default async function DashboardPage({ searchParams }: DashboardPageProps) {
  const { userId, getToken } = await auth()
  if (!userId) redirect('/sign-in')

  const user = await currentUser()
  const firstName = user?.firstName ?? 'there'

  const { page: pageParam } = await searchParams
  const page = Math.max(1, parseInt(pageParam ?? '1', 10) || 1)

  const sessionToken = await getToken()
  const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:3000'

  const [scansRes, quotaRes, domainsRes] = await Promise.all([
    fetch(`${BACKEND_URL}/api/v1/users/me/scans?page=${page}`, {
      cache: 'no-store',
      headers: sessionToken ? { 'Authorization': `Bearer ${sessionToken}` } : {},
    }),
    fetch(`${BACKEND_URL}/api/v1/quota`, {
      cache: 'no-store',
      headers: sessionToken ? { 'Authorization': `Bearer ${sessionToken}` } : {},
    }),
    fetch(`${BACKEND_URL}/api/v1/domains`, {
      cache: 'no-store',
      headers: sessionToken ? { 'Authorization': `Bearer ${sessionToken}` } : {},
    }),
  ])

  const scanHistory: ScanHistoryResponse | null = scansRes.ok ? await scansRes.json() : null
  const quota: QuotaResponse | null = quotaRes.ok ? await quotaRes.json() : null
  const domains: VerifiedDomain[] = domainsRes.ok ? await domainsRes.json() : []

  const activeScans = scanHistory?.active_scans ?? []
  const completedScans = scanHistory?.scans ?? []
  const totalPages = scanHistory?.total_pages ?? 1
  const isEmpty = completedScans.length === 0 && activeScans.length === 0

  return (
    <main><PageContainer maxWidth="max-w-6xl" className="py-8">
      <h1 className="text-3xl font-bold text-text-primary mb-1">Dashboard</h1>
      <p className="text-text-secondary mb-8">Welcome, {firstName}</p>

      <div className="flex flex-col lg:flex-row gap-8">
        {/* Main content */}
        <div className="flex-1 min-w-0">

          {/* Active scans section */}
          {activeScans.length > 0 && (
            <section className="mb-8">
              <h2 className="text-lg font-semibold text-text-primary mb-3">Active Scans</h2>
              <div className="border border-brand-primary/20 rounded-(card) bg-surface-elevated divide-y divide-border-subtle">
                {activeScans.map((scan) => (
                  <div key={scan.id} className="flex items-center gap-3 px-5 py-4">
                    <Loader2 className="w-4 h-4 text-brand-primary animate-spin shrink-0" aria-hidden="true" />
                    <div className="min-w-0">
                      <p className="font-mono text-sm font-medium text-text-primary truncate">
                        {extractHostname(scan.target_url)}
                      </p>
                      <p className="text-xs text-text-secondary">
                        {scan.status === 'in_progress' ? 'Scanning...' : 'Queued'} &middot;{' '}
                        {new Date(scan.created_at).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            </section>
          )}

          {/* Scan history section */}
          <section>
            <h2 className="text-lg font-semibold text-text-primary mb-3">Scan History</h2>

            {isEmpty ? (
              <div className="border border-border-subtle rounded-(card) p-8 bg-surface-secondary text-center">
                {domains.length === 0 ? (
                  <>
                    <p className="text-text-secondary mb-4">Verify a domain to start scanning.</p>
                    <Link
                      href="/verify-domain"
                      className="inline-flex items-center px-4 py-2 bg-brand-primary hover:bg-brand-primary-hover text-text-inverse font-semibold rounded-lg transition text-sm"
                    >
                      Verify a Domain
                    </Link>
                  </>
                ) : (
                  <>
                    <p className="text-text-secondary mb-4">No scans yet. Run your first scan.</p>
                    <Link
                      href="/"
                      className="inline-flex items-center px-4 py-2 bg-brand-primary hover:bg-brand-primary-hover text-text-inverse font-semibold rounded-lg transition text-sm"
                    >
                      Run a Scan
                    </Link>
                  </>
                )}
              </div>
            ) : (
              <div className="border border-border-subtle rounded-(card) p-5 bg-surface-elevated">
                <ScanHistoryTable
                  scans={completedScans}
                  currentPage={page}
                  totalPages={totalPages}
                />
              </div>
            )}
          </section>
        </div>

        {/* Sidebar */}
        <div className="lg:w-72 shrink-0 space-y-5">

          {/* Quota card */}
          {quota && (
            <div className="border border-border-subtle rounded-(card) p-5 bg-surface-elevated">
              <h2 className="text-sm font-semibold text-text-primary mb-3">Scan Quota</h2>
              <p className={`text-sm font-medium mb-4 px-2 py-1 rounded border inline-flex ${getQuotaStyle(quota.used, quota.limit)}`}>
                {quota.used} of {quota.limit} scans used — resets {formatResetDate(quota.resets_at)}
              </p>

              {quota.used >= quota.limit ? (
                <div>
                  <span className="inline-flex items-center justify-center w-full px-4 py-2 text-sm font-semibold rounded-lg border border-border-subtle text-text-secondary opacity-50 cursor-not-allowed pointer-events-none">
                    New Scan
                  </span>
                  <p className="text-xs text-text-secondary text-center mt-2">
                    Resets {formatResetDate(quota.resets_at)}
                  </p>
                </div>
              ) : (
                <Link
                  href="/"
                  className="inline-flex items-center justify-center w-full px-4 py-2 text-sm font-semibold rounded-lg border border-border-subtle text-text-primary hover:bg-surface-secondary transition"
                >
                  New Scan
                </Link>
              )}
            </div>
          )}

          {/* Verified domains card */}
          <div className="border border-border-subtle rounded-(card) p-5 bg-surface-elevated">
            <h2 className="text-sm font-semibold text-text-primary mb-3">Verified Domains</h2>

            {domains.length === 0 ? (
              <div className="text-center py-4">
                <p className="text-sm text-text-secondary mb-3">No domains verified yet.</p>
                <Link
                  href="/verify-domain"
                  className="text-sm text-brand-primary hover:underline font-medium"
                >
                  Verify a Domain
                </Link>
              </div>
            ) : (
              <>
                <ul className="space-y-2 mb-3">
                  {domains.map((d) => {
                    const isExpiredOrExpiring =
                      d.status === 'verified' &&
                      d.expires_at !== null &&
                      Math.ceil(
                        (new Date(d.expires_at).getTime() - Date.now()) / (1000 * 60 * 60 * 24)
                      ) <= 7

                    return (
                      <li key={d.id} className="flex items-center justify-between gap-2">
                        <div className="flex items-center gap-2 min-w-0">
                          <span className="font-mono text-xs text-text-primary truncate">{d.domain}</span>
                          <DomainBadge status={d.status} expiresAt={d.expires_at} />
                        </div>
                        {isExpiredOrExpiring && (
                          <Link
                            href="/verify-domain"
                            className="text-xs text-brand-primary hover:underline shrink-0"
                          >
                            Re-verify
                          </Link>
                        )}
                      </li>
                    )
                  })}
                </ul>
                <Link
                  href="/verify-domain"
                  className="text-sm text-brand-primary hover:underline font-medium"
                >
                  Verify a Domain
                </Link>
              </>
            )}
          </div>
        </div>
      </div>
      <ActiveScansPoller hasActiveScans={activeScans.length > 0} />
      </PageContainer></main>
  )
}
