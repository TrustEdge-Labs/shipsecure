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
    { key: 'detection', label: 'Detecting Framework', done: stages.detection },
    { key: 'headers', label: 'Security Headers', done: stages.headers },
    { key: 'tls', label: 'TLS Configuration', done: stages.tls },
    { key: 'files', label: 'Exposed Files', done: stages.files },
    { key: 'secrets', label: 'JavaScript Secrets', done: stages.secrets },
    { key: 'vibecode', label: 'Vibe-Code Scan', done: stages.vibecode },
  ]

  return (
    <ul className="space-y-3">
      {items.map(item => (
        <li key={item.key} className="flex items-center gap-3">
          <span className={`flex-shrink-0 w-6 h-6 rounded-full flex items-center justify-center text-sm
            ${item.done
              ? 'bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-400'
              : status === 'failed'
                ? 'bg-red-100 dark:bg-red-900 text-red-600 dark:text-red-400'
                : 'bg-gray-100 dark:bg-gray-800 text-gray-400 dark:text-gray-500 animate-pulse'
            }`}
          >
            {item.done ? '✓' : status === 'failed' ? '✗' : '○'}
          </span>
          <span className={item.done ? 'text-gray-900 dark:text-gray-100' : 'text-gray-500 dark:text-gray-400'}>
            {item.label}
          </span>
        </li>
      ))}
    </ul>
  )
}
