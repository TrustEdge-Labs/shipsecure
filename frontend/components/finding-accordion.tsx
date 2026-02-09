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
        return 'bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-300'
      case 'high':
        return 'bg-orange-100 dark:bg-orange-900 text-orange-700 dark:text-orange-300'
      case 'medium':
        return 'bg-yellow-100 dark:bg-yellow-900 text-yellow-700 dark:text-yellow-300'
      case 'low':
        return 'bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300'
      default:
        return 'bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300'
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
    <div className="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden mb-3">
      {/* Header - always visible */}
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full px-4 py-3 flex flex-wrap items-center gap-2 sm:gap-3 bg-white dark:bg-gray-900 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors text-left"
      >
        <span className={`px-2 py-1 text-xs font-medium rounded uppercase ${getSeverityStyles(finding.severity)}`}>
          {finding.severity}
        </span>
        {finding.vibe_code && (
          <span className="px-2 py-0.5 text-xs font-medium rounded bg-purple-100 dark:bg-purple-900 text-purple-700 dark:text-purple-300">
            Vibe-Code
          </span>
        )}
        <span className="flex-1 min-w-0 font-medium text-gray-900 dark:text-gray-100 break-words">
          {finding.title}
        </span>
        <span className="hidden sm:inline text-xs text-gray-500 dark:text-gray-400">
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
        <div className="px-4 py-4 bg-gray-50 dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-700 dark:text-gray-300 mb-4 break-words">
            {finding.description}
          </p>
          <h4 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-2">
            How to Fix
          </h4>
          <p className="text-sm text-gray-700 dark:text-gray-300 whitespace-pre-line break-words">
            {finding.remediation}
          </p>
        </div>
      </div>
    </div>
  )
}
