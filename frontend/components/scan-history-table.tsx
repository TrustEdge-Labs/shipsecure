import Link from 'next/link'
import type { ScanHistoryItem } from '@/lib/types'

interface ScanHistoryTableProps {
  scans: ScanHistoryItem[]
  currentPage: number
  totalPages: number
}

function SeverityBadge({ count, level }: { count: number; level: 'critical' | 'high' | 'medium' | 'low' }) {
  if (count === 0) return null

  const styles: Record<string, string> = {
    critical: 'bg-danger-bg text-danger-text border-danger-border',
    high: 'bg-caution-bg text-caution-text border-caution-border',
    medium: 'bg-info-bg text-info-text border-info-border',
    low: 'bg-success-bg text-success-text border-success-border',
  }

  return (
    <span className={`inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium border ${styles[level]}`}>
      {count}
    </span>
  )
}

function TierBadge({ tier }: { tier: string }) {
  const isEnhanced = tier === 'authenticated'
  return (
    <span
      className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border ${
        isEnhanced
          ? 'bg-brand-primary/10 text-brand-primary border-brand-primary/20'
          : 'bg-surface-secondary text-text-secondary border-border-subtle'
      }`}
    >
      {isEnhanced ? 'Enhanced' : 'Basic'}
    </span>
  )
}

function formatExpiry(expiresAt: string | null, status: string): React.ReactNode {
  if (status === 'failed') {
    return (
      <span className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border bg-danger-bg text-danger-text border-danger-border">
        Failed
      </span>
    )
  }

  if (expiresAt === null) {
    return <span className="text-text-secondary">—</span>
  }

  const days = Math.ceil((new Date(expiresAt).getTime() - Date.now()) / (1000 * 60 * 60 * 24))

  if (days <= 0) {
    return (
      <span className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border bg-surface-secondary text-text-secondary border-border-subtle">
        Expired
      </span>
    )
  }

  if (days <= 3) {
    return (
      <span className="text-xs font-medium text-caution-text">
        {days} day{days === 1 ? '' : 's'} left
      </span>
    )
  }

  return (
    <span className="text-xs text-text-secondary">
      {days} days left
    </span>
  )
}

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  })
}

function extractHostname(targetUrl: string): string {
  try {
    return new URL(targetUrl).hostname
  } catch {
    return targetUrl
  }
}

function Pagination({ currentPage, totalPages }: { currentPage: number; totalPages: number }) {
  if (totalPages <= 1) return null

  return (
    <div className="inline-flex items-center gap-1 mt-4">
      {Array.from({ length: totalPages }, (_, i) => i + 1).map((n) => {
        if (n === currentPage) {
          return (
            <span key={n} className="px-3 py-1.5 text-sm font-bold text-text-primary">
              {n}
            </span>
          )
        }
        return (
          <Link
            key={n}
            href={`/dashboard?page=${n}`}
            className="px-3 py-1.5 text-sm text-brand-primary hover:underline"
          >
            {n}
          </Link>
        )
      })}
    </div>
  )
}

export function ScanHistoryTable({ scans, currentPage, totalPages }: ScanHistoryTableProps) {
  if (scans.length === 0) {
    return (
      <p className="text-text-secondary text-sm py-4">No completed scans yet.</p>
    )
  }

  return (
    <div>
      {/* Desktop table */}
      <div className="hidden sm:block overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-border-subtle text-left">
              <th className="pb-3 pr-4 font-semibold text-text-secondary text-xs uppercase tracking-wide">Domain</th>
              <th className="pb-3 pr-4 font-semibold text-text-secondary text-xs uppercase tracking-wide">Date</th>
              <th className="pb-3 pr-4 font-semibold text-text-secondary text-xs uppercase tracking-wide">Severity</th>
              <th className="pb-3 pr-4 font-semibold text-text-secondary text-xs uppercase tracking-wide">Expiry</th>
              <th className="pb-3 pr-4 font-semibold text-text-secondary text-xs uppercase tracking-wide">Tier</th>
              <th className="pb-3 font-semibold text-text-secondary text-xs uppercase tracking-wide">Action</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-border-subtle">
            {scans.map((scan) => {
              const isExpired =
                scan.expires_at !== null &&
                Math.ceil((new Date(scan.expires_at).getTime() - Date.now()) / (1000 * 60 * 60 * 24)) <= 0
              const isFailed = scan.status === 'failed'
              const isClickable = !isExpired && !isFailed && scan.results_token !== null

              const severityContent = (
                <div className="flex gap-1">
                  <SeverityBadge count={scan.critical_count} level="critical" />
                  <SeverityBadge count={scan.high_count} level="high" />
                  <SeverityBadge count={scan.medium_count} level="medium" />
                  <SeverityBadge count={scan.low_count} level="low" />
                  {scan.critical_count === 0 && scan.high_count === 0 && scan.medium_count === 0 && scan.low_count === 0 && (
                    <span className="text-text-secondary text-xs">—</span>
                  )}
                </div>
              )

              if (isClickable) {
                // Clickable row: use relative positioning with an overlay link for full-row click
                // Plus explicit View button in action column
                return (
                  <tr
                    key={scan.id}
                    className="hover:bg-surface-secondary transition-colors relative"
                  >
                    <td className="py-3 pr-4 font-mono text-text-primary relative">
                      {/* Overlay link covers the entire row */}
                      <Link
                        href={`/results/${scan.results_token}`}
                        className="absolute inset-0 -mx-0"
                        aria-label={`View results for ${extractHostname(scan.target_url)}`}
                      />
                      {extractHostname(scan.target_url)}
                    </td>
                    <td className="py-3 pr-4 text-text-secondary whitespace-nowrap">
                      {formatDate(scan.created_at)}
                    </td>
                    <td className="py-3 pr-4">
                      {severityContent}
                    </td>
                    <td className="py-3 pr-4">{formatExpiry(scan.expires_at, scan.status)}</td>
                    <td className="py-3 pr-4">
                      <TierBadge tier={scan.tier} />
                    </td>
                    <td className="py-3 relative z-10">
                      <Link
                        href={`/results/${scan.results_token}`}
                        className="text-xs font-medium text-brand-primary hover:underline"
                      >
                        View
                      </Link>
                    </td>
                  </tr>
                )
              }

              return (
                <tr
                  key={scan.id}
                  className={isExpired ? 'opacity-60' : ''}
                >
                  <td className="py-3 pr-4 font-mono text-text-primary">
                    {extractHostname(scan.target_url)}
                  </td>
                  <td className="py-3 pr-4 text-text-secondary whitespace-nowrap">
                    {formatDate(scan.created_at)}
                  </td>
                  <td className="py-3 pr-4">
                    {severityContent}
                  </td>
                  <td className="py-3 pr-4">{formatExpiry(scan.expires_at, scan.status)}</td>
                  <td className="py-3 pr-4">
                    <TierBadge tier={scan.tier} />
                  </td>
                  <td className="py-3">
                    {isExpired ? (
                      <span className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border bg-surface-secondary text-text-secondary border-border-subtle">
                        Expired
                      </span>
                    ) : null}
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>

      {/* Mobile cards */}
      <div className="sm:hidden space-y-3">
        {scans.map((scan) => {
          const isExpired =
            scan.expires_at !== null &&
            Math.ceil((new Date(scan.expires_at).getTime() - Date.now()) / (1000 * 60 * 60 * 24)) <= 0
          const isFailed = scan.status === 'failed'
          const isClickable = !isExpired && !isFailed && scan.results_token !== null

          const cardContent = (
            <div className={`border border-border-subtle rounded-lg p-4 bg-surface-elevated ${isExpired ? 'opacity-60' : ''}`}>
              <div className="flex items-center justify-between mb-2">
                <span className="font-mono text-sm font-semibold text-text-primary truncate">
                  {extractHostname(scan.target_url)}
                </span>
                <TierBadge tier={scan.tier} />
              </div>
              <p className="text-xs text-text-secondary mb-2">{formatDate(scan.created_at)}</p>
              <div className="flex gap-1 mb-2">
                <SeverityBadge count={scan.critical_count} level="critical" />
                <SeverityBadge count={scan.high_count} level="high" />
                <SeverityBadge count={scan.medium_count} level="medium" />
                <SeverityBadge count={scan.low_count} level="low" />
                {scan.critical_count === 0 && scan.high_count === 0 && scan.medium_count === 0 && scan.low_count === 0 && (
                  <span className="text-text-secondary text-xs">No findings</span>
                )}
              </div>
              <div>{formatExpiry(scan.expires_at, scan.status)}</div>
            </div>
          )

          if (isClickable) {
            return (
              <Link key={scan.id} href={`/results/${scan.results_token}`}>
                {cardContent}
              </Link>
            )
          }

          return <div key={scan.id}>{cardContent}</div>
        })}
      </div>

      <Pagination currentPage={currentPage} totalPages={totalPages} />
    </div>
  )
}
