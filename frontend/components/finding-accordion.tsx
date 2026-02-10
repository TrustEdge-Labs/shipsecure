'use client'

import { useState } from 'react'
import { Finding } from '@/lib/types'

interface FindingAccordionProps {
  finding: Finding
  defaultExpanded?: boolean
}

export function FindingAccordion({ finding, defaultExpanded = false }: FindingAccordionProps) {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded)

  const getSeverityStyles = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'bg-severity-critical-bg text-severity-critical-text'
      case 'high':
        return 'bg-severity-high-bg text-severity-high-text'
      case 'medium':
        return 'bg-severity-medium-bg text-severity-medium-text'
      case 'low':
        return 'bg-severity-info-bg text-severity-info-text'
      default:
        return 'bg-severity-none-bg text-severity-none-text'
    }
  }

  const getScannerDisplayName = (scannerName: string) => {
    const mapping: Record<string, string> = {
      security_headers: 'Headers',
      tls: 'TLS',
      exposed_files: 'Exposed Files',
      js_secrets: 'JavaScript Secrets',
      vibecode: 'Vibe-Code',
    }
    return mapping[scannerName] || scannerName
  }

  return (
    <div className="border border-border-subtle rounded-lg overflow-hidden mb-3">
      {/* Header - always visible */}
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full px-4 py-3 flex flex-wrap items-center gap-2 sm:gap-3 bg-surface-elevated hover:bg-surface-secondary transition-colors text-left"
      >
        <span className={`px-2 py-1 text-xs font-medium rounded uppercase ${getSeverityStyles(finding.severity)}`}>
          {finding.severity}
        </span>
        {finding.vibe_code && (
          <span className="px-2 py-0.5 text-xs font-medium rounded bg-category-bg text-category-text">
            Vibe-Code
          </span>
        )}
        <span className="flex-1 min-w-0 font-medium text-text-primary break-words">
          {finding.title}
        </span>
        <span className="hidden sm:inline text-xs text-text-tertiary">
          {getScannerDisplayName(finding.scanner_name)}
        </span>
        <span className={`transform transition-transform ${isExpanded ? 'rotate-180' : ''}`}>
          <svg className="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
          </svg>
        </span>
      </button>

      {/* Expandable body */}
      <div
        className={`transition-all duration-200 ease-in-out ${
          isExpanded ? 'max-h-[1000px] opacity-100' : 'max-h-0 opacity-0'
        } overflow-hidden`}
      >
        <div className="px-4 py-4 bg-surface-secondary border-t border-border-subtle">
          <p className="text-sm text-text-secondary mb-4 break-words">
            {finding.description}
          </p>
          <h4 className="text-sm font-semibold text-text-primary mb-2">
            How to Fix
          </h4>
          <p className="text-sm text-text-secondary whitespace-pre-line break-words">
            {finding.remediation}
          </p>
        </div>
      </div>
    </div>
  )
}
