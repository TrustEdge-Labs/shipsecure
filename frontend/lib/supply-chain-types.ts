// Match DepFinding from src/scanners/lockfile_parser.rs
export interface SupplyChainFinding {
  name: string
  version: string
  osv_id: string
  description: string
  tier: 'Infected' | 'Vulnerable' | 'Advisory'
}

// Match ParsedDep for unscanned items
export interface UnscannedDep {
  name: string
  version: string
  source: 'Git' | 'File' | 'Link' | 'Tarball'
  is_dev: boolean
}

// Match SupplyChainScanResult from backend
export interface SupplyChainResults {
  total_deps: number
  infected: SupplyChainFinding[]
  vulnerable: SupplyChainFinding[]
  advisory: SupplyChainFinding[]
  no_known_issues: string[] // "name@version" strings
  unscanned: UnscannedDep[]
  scanned_at: string
}

// POST /api/v1/scans/supply-chain response
export interface SupplyChainScanResponse {
  status: string
  results_token: string | null
  share_url: string | null
  share_unavailable: boolean
  results: SupplyChainResults
}

// GET /api/v1/results/:token response for supply chain scans
export interface SupplyChainResultsPageData {
  id: string
  target_url: string
  status: string
  kind: string
  expires_at: string | null
  created_at: string
  completed_at: string | null
  supply_chain_results: SupplyChainResults | null
}

// Error response (RFC 7807)
export interface ApiErrorResponse {
  type: string
  title: string
  status: number
  detail: string
}
