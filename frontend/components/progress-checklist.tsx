'use client'

interface ProgressChecklistProps {
  stages: {
    detection: boolean
    headers: boolean
    tls: boolean
    files: boolean
    secrets: boolean
    vibecode: boolean
  }
  status: string
}

export function ProgressChecklist({ stages, status }: ProgressChecklistProps) {
  const items = [
    { key: 'detection', label: 'Detecting Framework', description: 'Identifying your app framework and deployment platform...', done: stages.detection },
    { key: 'headers', label: 'Security Headers', description: 'Checking security headers like CSP, HSTS, X-Frame-Options...', done: stages.headers },
    { key: 'tls', label: 'TLS Configuration', description: 'Analyzing certificate validity, protocol versions, cipher strength...', done: stages.tls },
    { key: 'files', label: 'Exposed Files', description: 'Probing for .env, .git, config files, and sensitive data...', done: stages.files },
    { key: 'secrets', label: 'JavaScript Secrets', description: 'Scanning client-side code for API keys, tokens, and credentials...', done: stages.secrets },
    { key: 'vibecode', label: 'Vibe-Code Scan', description: 'Running vulnerability templates for common AI-generated code patterns...', done: stages.vibecode },
  ]

  // Find the active stage (first one that's not done and not failed)
  const activeIndex = (status === 'pending' || status === 'in_progress')
    ? items.findIndex(item => !item.done)
    : -1

  return (
    <ul className="space-y-3">
      {items.map((item, index) => {
        const isActive = index === activeIndex

        return (
          <li key={item.key} className="flex items-start gap-3">
            <span className={`flex-shrink-0 w-6 h-6 rounded-full flex items-center justify-center text-sm
              ${item.done
                ? 'bg-grade-a-bg text-grade-a-text'
                : status === 'failed'
                  ? 'bg-danger-bg text-danger-primary'
                  : 'bg-skeleton text-text-muted animate-pulse'
              }`}
            >
              {item.done ? '✓' : status === 'failed' ? '✗' : '○'}
            </span>
            <div className="flex-1">
              <span className={item.done ? 'text-text-primary' : 'text-text-tertiary'}>
                {item.label}
              </span>
              {isActive && (
                <div className="text-sm text-text-tertiary mt-1">
                  {item.description}
                </div>
              )}
            </div>
          </li>
        )
      })}
    </ul>
  )
}
