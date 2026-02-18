import { CheckCircle2, Clock, AlertTriangle, XCircle } from 'lucide-react'

interface DomainBadgeProps {
  status: 'pending' | 'verified'
  expiresAt: string | null
}

export function DomainBadge({ status, expiresAt }: DomainBadgeProps) {
  const baseClasses = 'inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium'

  if (status === 'verified' && expiresAt) {
    const expires = new Date(expiresAt)
    const now = new Date()
    const days = Math.ceil((expires.getTime() - now.getTime()) / (1000 * 60 * 60 * 24))

    if (days <= 0) {
      return (
        <span className={`${baseClasses} bg-danger-bg text-danger-text border border-danger-border`}>
          <XCircle className="w-3 h-3" aria-hidden="true" />
          Expired
        </span>
      )
    }

    if (days <= 7) {
      return (
        <span className={`${baseClasses} bg-caution-bg text-caution-text border border-caution-border`}>
          <AlertTriangle className="w-3 h-3" aria-hidden="true" />
          Expires in {days}d
        </span>
      )
    }

    return (
      <span className={`${baseClasses} bg-success-bg text-success-text border border-success-border`}>
        <CheckCircle2 className="w-3 h-3" aria-hidden="true" />
        Verified
      </span>
    )
  }

  // status === 'pending' or verified with no expiresAt (shouldn't happen but safe fallback)
  return (
    <span className={`${baseClasses} bg-info-bg text-info-text border border-info-border`}>
      <Clock className="w-3 h-3" aria-hidden="true" />
      Pending
    </span>
  )
}
