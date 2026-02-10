'use client'

import { useState } from 'react'
import { Finding } from '@/lib/types'
import { FindingAccordion } from './finding-accordion'

interface ResultsDashboardProps {
  findings: Finding[]
}

type GroupingMode = 'severity' | 'category'

export function ResultsDashboard({ findings }: ResultsDashboardProps) {
  const [grouping, setGrouping] = useState<GroupingMode>('severity')

  if (findings.length === 0) {
    return (
      <div className="text-center py-12 bg-success-bg border border-success-border rounded-lg">
        <div className="inline-block rounded-full h-16 w-16 bg-success-bg flex items-center justify-center mb-4">
          <span className="text-3xl text-success-primary">✓</span>
        </div>
        <h2 className="text-xl font-semibold text-text-primary mb-2">
          No Security Issues Found!
        </h2>
        <p className="text-text-secondary">
          Your application passed all checks.
        </p>
      </div>
    )
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

  const groupBySeverity = () => {
    const severityOrder = ['critical', 'high', 'medium', 'low']
    const groups: Record<string, Finding[]> = {
      critical: [],
      high: [],
      medium: [],
      low: [],
    }

    findings.forEach(finding => {
      groups[finding.severity].push(finding)
    })

    return severityOrder
      .filter(severity => groups[severity].length > 0)
      .map(severity => ({
        title: `${severity.charAt(0).toUpperCase() + severity.slice(1)} (${groups[severity].length})`,
        findings: groups[severity],
      }))
  }

  const groupByCategory = () => {
    const groups: Record<string, Finding[]> = {}

    findings.forEach(finding => {
      const category = finding.scanner_name
      if (!groups[category]) {
        groups[category] = []
      }
      groups[category].push(finding)
    })

    return Object.entries(groups).map(([category, findings]) => ({
      title: `${getScannerDisplayName(category)} (${findings.length})`,
      findings,
    }))
  }

  const groupedFindings = grouping === 'severity' ? groupBySeverity() : groupByCategory()

  return (
    <div>
      {/* Toggle buttons */}
      <div className="flex gap-2 mb-6">
        <button
          onClick={() => setGrouping('severity')}
          className={`px-4 py-2 min-h-[44px] text-sm font-medium rounded-md transition-colors ${
            grouping === 'severity'
              ? 'bg-brand-primary text-white'
              : 'bg-surface-secondary text-text-secondary hover:bg-surface-secondary/80'
          }`}
        >
          By Severity
        </button>
        <button
          onClick={() => setGrouping('category')}
          className={`px-4 py-2 min-h-[44px] text-sm font-medium rounded-md transition-colors ${
            grouping === 'category'
              ? 'bg-brand-primary text-white'
              : 'bg-surface-secondary text-text-secondary hover:bg-surface-secondary/80'
          }`}
        >
          By Category
        </button>
      </div>

      {/* Grouped findings */}
      <div className="space-y-6">
        {groupedFindings.map((group, idx) => (
          <div key={idx}>
            <h3 className="text-lg font-semibold text-text-primary mb-3">
              {group.title}
            </h3>
            <div>
              {group.findings.map(finding => (
                <FindingAccordion key={finding.id} finding={finding} />
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
