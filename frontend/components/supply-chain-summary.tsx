"use client"

import { useEffect } from "react"
import { SupplyChainResults } from "@/lib/supply-chain-types"

interface SupplyChainSummaryProps {
  results: SupplyChainResults
}

const cards = [
  {
    key: "infected" as const,
    label: "Infected",
    color: "text-[#ef4444]",
    tintBg: "bg-[#ef4444]/10",
  },
  {
    key: "vulnerable" as const,
    label: "Vulnerable",
    color: "text-[#f59e0b]",
    tintBg: "bg-[#f59e0b]/10",
  },
  {
    key: "advisory" as const,
    label: "Advisory",
    color: "text-yellow-400/60",
    tintBg: "",
  },
  {
    key: "no_known_issues" as const,
    label: "No Known Issues",
    color: "text-[#22c55e]",
    tintBg: "",
  },
  {
    key: "unscanned" as const,
    label: "Unscanned",
    color: "text-[#71717a]",
    tintBg: "",
  },
]

function getCount(results: SupplyChainResults, key: typeof cards[number]["key"]): number {
  if (key === "no_known_issues") return results.no_known_issues.length
  if (key === "unscanned") return results.unscanned.length
  return results[key].length
}

export function SupplyChainSummary({ results }: SupplyChainSummaryProps) {
  useEffect(() => {
    window.plausible?.("supply_chain_scan_completed", {
      props: { total_deps: String(results.total_deps) },
    })
    if (results.infected.length > 0) {
      window.plausible?.("infected_found", {
        props: { count: String(results.infected.length) },
      })
    }
    if (results.vulnerable.length > 0) {
      window.plausible?.("vulnerable_found", {
        props: { count: String(results.vulnerable.length) },
      })
    }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3 mb-6">
      {cards.map((card) => {
        const count = getCount(results, card.key)
        const hasTint = count > 0 && card.tintBg
        return (
          <div
            key={card.key}
            className={`bg-surface-elevated rounded-(card) border border-border-default p-4 ${hasTint ? card.tintBg : ""}`}
          >
            <div className={`text-2xl font-bold ${card.color}`}>{count}</div>
            <div className="text-sm text-text-secondary mt-1">{card.label}</div>
          </div>
        )
      })}
    </div>
  )
}
