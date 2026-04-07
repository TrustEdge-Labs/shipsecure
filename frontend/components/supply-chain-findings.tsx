"use client"

import { SupplyChainResults, SupplyChainFinding, UnscannedDep } from "@/lib/supply-chain-types"

interface SupplyChainFindingsProps {
  results: SupplyChainResults
}

const tierConfig = {
  Infected: {
    color: "text-[#ef4444]",
    badgeBg: "bg-[#ef4444]/10 text-[#ef4444]",
    fixLabel: "Remove immediately",
    fixColor: "text-[#ef4444]",
  },
  Vulnerable: {
    color: "text-[#f59e0b]",
    badgeBg: "bg-[#f59e0b]/10 text-[#f59e0b]",
    fixLabel: "Update package",
    fixColor: "text-[#22c55e]",
  },
  Advisory: {
    color: "text-yellow-400/60",
    badgeBg: "bg-yellow-400/10 text-yellow-400/60",
    fixLabel: "Update package",
    fixColor: "text-[#22c55e]",
  },
}

const sourceBadgeColors: Record<UnscannedDep["source"], string> = {
  Git: "bg-surface-secondary text-text-tertiary border border-border-subtle",
  File: "bg-surface-secondary text-text-tertiary border border-border-subtle",
  Link: "bg-surface-secondary text-text-tertiary border border-border-subtle",
  Tarball: "bg-surface-secondary text-text-tertiary border border-border-subtle",
}

function FindingRow({
  finding,
  isLast,
}: {
  finding: SupplyChainFinding
  isLast: boolean
}) {
  const config = tierConfig[finding.tier]
  return (
    <div className={`py-3 ${isLast ? "" : "border-b border-border-subtle"}`}>
      <div className="flex flex-wrap items-start gap-x-3 gap-y-1 mb-1">
        <span className="font-mono text-text-primary font-medium">{finding.name}</span>
        <span className="font-mono text-text-secondary text-sm">{finding.version}</span>
        <a
          href={`https://osv.dev/vulnerability/${finding.osv_id}`}
          target="_blank"
          rel="noopener noreferrer"
          className="text-info text-sm hover:underline font-mono"
        >
          {finding.osv_id}
        </a>
      </div>
      <p className="text-sm text-text-secondary line-clamp-2 mb-1">
        {finding.description}
      </p>
      <span className={`text-xs font-medium ${config.fixColor} inline-flex items-center gap-1`}>
        <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
        </svg>
        {config.fixLabel}
      </span>
    </div>
  )
}

function TierSection({
  tier,
  findings,
}: {
  tier: "Infected" | "Vulnerable" | "Advisory"
  findings: SupplyChainFinding[]
}) {
  if (findings.length === 0) return null
  const config = tierConfig[tier]
  return (
    <div className="mb-6">
      <div className="flex items-center gap-2 mb-3">
        <span className={`px-2 py-0.5 rounded text-xs font-semibold uppercase tracking-wide ${config.badgeBg}`}>
          {tier}
        </span>
        <span className="text-sm text-text-secondary">
          {findings.length} {findings.length === 1 ? "package" : "packages"}
        </span>
      </div>
      <div>
        {findings.map((finding, idx) => (
          <FindingRow
            key={`${finding.name}-${finding.osv_id}`}
            finding={finding}
            isLast={idx === findings.length - 1}
          />
        ))}
      </div>
    </div>
  )
}

export function SupplyChainFindings({ results }: SupplyChainFindingsProps) {
  const hasFindings =
    results.infected.length > 0 ||
    results.vulnerable.length > 0 ||
    results.advisory.length > 0

  if (!hasFindings) {
    return (
      <div className="flex flex-col items-center justify-center py-10 text-center">
        <svg
          className="w-12 h-12 text-[#22c55e] mb-4"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          aria-hidden="true"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1.5}
            d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z"
          />
        </svg>
        <h3 className="text-xl font-semibold text-text-primary mb-2">
          No compromised packages found
        </h3>
        <p className="text-text-secondary">
          {results.no_known_issues.length} {results.no_known_issues.length === 1 ? "package" : "packages"} checked, all clear
        </p>
      </div>
    )
  }

  return (
    <div>
      <TierSection tier="Infected" findings={results.infected} />
      <TierSection tier="Vulnerable" findings={results.vulnerable} />
      <TierSection tier="Advisory" findings={results.advisory} />

      {results.unscanned.length > 0 && (
        <details className="mt-4">
          <summary className="cursor-pointer text-sm text-text-secondary hover:text-text-primary py-2 flex items-center gap-2">
            <span className="font-medium text-[#71717a]">
              Unscanned ({results.unscanned.length})
            </span>
            <span className="text-xs text-text-tertiary">— git/file/link/tarball deps, npm registry only</span>
          </summary>
          <div className="mt-2 space-y-1 pl-2">
            {results.unscanned.map((dep) => (
              <div key={`${dep.name}-${dep.version}`} className="flex items-center gap-2 py-1">
                <span className="font-mono text-text-tertiary text-sm">
                  {dep.name}@{dep.version}
                </span>
                <span className={`text-xs px-1.5 py-0.5 rounded font-mono ${sourceBadgeColors[dep.source]}`}>
                  {dep.source}
                </span>
                {dep.is_dev && (
                  <span className="text-xs text-text-tertiary">(dev)</span>
                )}
              </div>
            ))}
          </div>
        </details>
      )}
    </div>
  )
}
