"use client"

import { useEffect, useState } from "react"
import { useRouter } from "next/navigation"
import Link from "next/link"
import { PageContainer } from "@/components/page-container"
import { SupplyChainSummary } from "@/components/supply-chain-summary"
import { SupplyChainFindings } from "@/components/supply-chain-findings"
import { SupplyChainScanResponse } from "@/lib/supply-chain-types"

const INLINE_STORAGE_KEY = "supply-chain-inline-results"

export function InlineResultsLoader() {
  const router = useRouter()
  const [scanResponse, setScanResponse] = useState<SupplyChainScanResponse | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const raw = sessionStorage.getItem(INLINE_STORAGE_KEY)
    if (!raw) {
      router.replace("/supply-chain")
      return
    }
    try {
      const parsed: SupplyChainScanResponse = JSON.parse(raw)
      setScanResponse(parsed)
    } catch {
      router.replace("/supply-chain")
      return
    }
    setLoading(false)
  }, [router])

  if (loading || !scanResponse) {
    return (
      <div className="min-h-screen bg-surface-secondary flex items-center justify-center">
        <div className="inline-block animate-spin rounded-full h-10 w-10 border-b-2 border-brand-primary" />
      </div>
    )
  }

  const results = scanResponse.results

  return (
    <PageContainer maxWidth="max-w-4xl" className="py-8">
      <div className="bg-warning-bg border border-warning-border rounded-(card) p-4 mb-6 text-sm text-warning-text">
        Share link unavailable — results shown inline only
      </div>

      {/* Header card */}
      <div className="bg-surface-elevated rounded-(card) shadow-md p-6 mb-6">
        <h1 className="text-2xl font-bold text-text-primary mb-4">
          Supply Chain Scan Results
        </h1>
        <div className="text-sm text-text-secondary">
          Inline results — these are not saved or shareable.
        </div>
      </div>

      {/* Summary cards */}
      <SupplyChainSummary results={results} />

      {/* Findings */}
      <div className="bg-surface-elevated rounded-(card) shadow-md p-6 my-6">
        <h2 className="text-xl font-bold text-text-primary mb-4">Findings</h2>
        <SupplyChainFindings results={results} />
      </div>

      {/* Actions */}
      <div className="flex gap-4 flex-wrap">
        <Link
          href="/supply-chain"
          className="inline-flex items-center justify-center gap-2 px-6 py-2 min-h-[44px] w-full sm:w-auto bg-brand-primary text-white rounded-md hover:bg-brand-primary/90 transition-colors"
        >
          Scan another project
        </Link>
      </div>
    </PageContainer>
  )
}
