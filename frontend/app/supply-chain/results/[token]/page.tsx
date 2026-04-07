import { notFound } from "next/navigation"
import Link from "next/link"
import { PageContainer } from "@/components/page-container"
import { ShareButton } from "@/components/share-button"
import { SupplyChainSummary } from "@/components/supply-chain-summary"
import { SupplyChainFindings } from "@/components/supply-chain-findings"
import { SupplyChainResultsPageData, SupplyChainResults } from "@/lib/supply-chain-types"
import { InlineResultsLoader } from "./inline-results-loader"

interface SupplyChainResultsPageProps {
  params: Promise<{
    token: string
  }>
}

export async function generateMetadata({ params }: SupplyChainResultsPageProps) {
  const { token } = await params

  if (token === "inline") {
    return {
      title: "Supply Chain Scan Results - ShipSecure",
      robots: { index: false, follow: false, nocache: true },
    }
  }

  try {
    const BACKEND_URL =
      process.env.BACKEND_URL ||
      process.env.NEXT_PUBLIC_BACKEND_URL ||
      "http://localhost:3000"

    const res = await fetch(`${BACKEND_URL}/api/v1/results/${token}`, {
      cache: "no-store",
    })

    if (!res.ok) {
      return {
        title: "Results Not Found - ShipSecure",
        robots: { index: false, follow: false, nocache: true },
      }
    }

    const data: SupplyChainResultsPageData = await res.json()

    if (data.status === "expired") {
      return {
        title: "Results Expired - ShipSecure",
        description: "These supply chain scan results have expired. Scan your dependencies again free at shipsecure.ai.",
        robots: { index: false, follow: false, nocache: true },
      }
    }

    if (data.kind !== "supply_chain" || !data.supply_chain_results) {
      return {
        title: "Results Not Found - ShipSecure",
        robots: { index: false, follow: false, nocache: true },
      }
    }

    const sc = data.supply_chain_results
    const infectedCount = sc.infected.length
    const vulnerableCount = sc.vulnerable.length
    const description =
      infectedCount > 0 || vulnerableCount > 0
        ? `Found ${infectedCount} infected and ${vulnerableCount} vulnerable packages in ${sc.total_deps} dependencies.`
        : `${sc.total_deps} dependencies scanned. No compromised packages found.`

    return {
      title: "Supply Chain Scan Results - ShipSecure",
      description,
      robots: { index: false, follow: false, nocache: true },
    }
  } catch {
    return {
      title: "Results Not Found - ShipSecure",
      robots: { index: false, follow: false, nocache: true },
    }
  }
}

function formatDate(dateStr: string | null): string {
  if (!dateStr) return "N/A"
  return new Date(dateStr).toLocaleString("en-US", {
    year: "numeric",
    month: "long",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  })
}

function isGitHubUrl(url: string): boolean {
  try {
    return new URL(url).hostname === "github.com"
  } catch {
    return false
  }
}

function ResultsView({
  data,
  token,
  isInline = false,
}: {
  data: SupplyChainResultsPageData
  token: string
  isInline?: boolean
}) {
  const supplyChainResults = data.supply_chain_results as SupplyChainResults

  return (
    <PageContainer maxWidth="max-w-4xl" className="py-8">
      {isInline && (
        <div className="bg-warning-bg border border-warning-border rounded-(card) p-4 mb-6 text-sm text-warning-text">
          Share link unavailable — results shown inline only
        </div>
      )}

      {/* Header card */}
      <div className="bg-surface-elevated rounded-(card) shadow-md p-6 mb-6">
        <h1 className="text-2xl font-bold text-text-primary mb-4">
          Supply Chain Scan Results
        </h1>
        <div className="space-y-2 text-sm">
          <div>
            <span className="text-text-secondary">Target: </span>
            {isGitHubUrl(data.target_url) ? (
              <a
                href={data.target_url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-brand-primary hover:underline font-mono break-all"
              >
                {data.target_url}
              </a>
            ) : (
              <span className="font-mono text-text-primary break-all">{data.target_url}</span>
            )}
          </div>
          <div>
            <span className="text-text-secondary">Scanned: </span>
            <span className="text-text-primary">{formatDate(data.completed_at)}</span>
          </div>
          {data.expires_at && (
            <div>
              <span className="text-text-secondary">Expires: </span>
              <span className="text-text-primary">{formatDate(data.expires_at)}</span>
            </div>
          )}
        </div>
      </div>

      {/* Summary cards */}
      <SupplyChainSummary results={supplyChainResults} />

      {/* Findings */}
      <div className="bg-surface-elevated rounded-(card) shadow-md p-6 my-6">
        <h2 className="text-xl font-bold text-text-primary mb-4">Findings</h2>
        <SupplyChainFindings results={supplyChainResults} />
      </div>

      {/* Actions */}
      <div className="flex gap-4 flex-wrap">
        {!isInline && (
          <ShareButton url={`https://shipsecure.ai/supply-chain/results/${token}`} />
        )}
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

export default async function SupplyChainResultsPage({
  params,
}: SupplyChainResultsPageProps) {
  const { token } = await params

  // Inline case: results from sessionStorage (client-side only)
  if (token === "inline") {
    return <InlineResultsLoader />
  }

  const BACKEND_URL =
    process.env.BACKEND_URL ||
    process.env.NEXT_PUBLIC_BACKEND_URL ||
    "http://localhost:3000"

  let data: SupplyChainResultsPageData
  try {
    const res = await fetch(`${BACKEND_URL}/api/v1/results/${token}`, {
      cache: "no-store",
    })

    if (!res.ok) {
      notFound()
    }

    data = await res.json()
  } catch (error) {
    console.error("Error fetching supply chain results:", error)
    notFound()
  }

  // Check kind
  if (data.kind !== "supply_chain") {
    notFound()
  }

  // Expired results
  if (data.status === "expired") {
    return (
      <div className="min-h-screen bg-surface-secondary py-8 px-4">
        <div className="max-w-md mx-auto mt-16">
          <div className="bg-surface-elevated rounded-(card) shadow-md p-8 text-center">
            <svg
              className="w-12 h-12 mx-auto text-text-secondary mb-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <h1 className="text-xl font-semibold text-text-primary mb-2">
              These results have expired
            </h1>
            <p className="text-text-secondary mb-6">
              Supply chain scan results are available for 30 days.
            </p>
            <Link
              href="/supply-chain"
              className="inline-flex items-center justify-center gap-2 px-6 py-3 min-h-[44px] w-full bg-brand-primary text-white rounded-md hover:bg-brand-primary/90 transition-colors font-medium"
            >
              Scan again
            </Link>
          </div>
        </div>
      </div>
    )
  }

  // Missing results data
  if (!data.supply_chain_results) {
    notFound()
  }

  return <ResultsView data={data} token={token} />
}
