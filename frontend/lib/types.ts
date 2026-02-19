export interface Scan {
  id: string
  target_url: string
  email: string
  status: 'pending' | 'in_progress' | 'completed' | 'failed'
  score: string | null
  results_token: string | null
  expires_at: string | null
  stage_detection: boolean
  stage_headers: boolean
  stage_tls: boolean
  stage_files: boolean
  stage_secrets: boolean
  stage_vibecode: boolean
  detected_framework: string | null
  detected_platform: string | null
  created_at: string
  started_at: string | null
  completed_at: string | null
  error_message: string | null
}

export interface Finding {
  id: string
  title: string
  description: string | null
  severity: 'critical' | 'high' | 'medium' | 'low'
  remediation: string | null
  scanner_name: string
  vibe_code: boolean
  gated?: boolean
}

export interface ScanResponse {
  id: string
  target_url: string
  status: string
  score: string | null
  tier: string
  results_token: string | null
  expires_at: string | null
  stage_detection: boolean
  stage_headers: boolean
  stage_tls: boolean
  stage_files: boolean
  stage_secrets: boolean
  stage_vibecode: boolean
  detected_framework: string | null
  detected_platform: string | null
  created_at: string
  started_at: string | null
  completed_at: string | null
  findings: Finding[]
  summary: {
    total: number
    critical: number
    high: number
    medium: number
    low: number
  }
  owner_verified: boolean
}

export interface CreateScanResponse {
  id: string
  status: string
  url: string
}

export interface VerifiedDomain {
  id: string
  domain: string
  status: 'pending' | 'verified'
  verified_at: string | null
  expires_at: string | null
  created_at: string
}

export interface VerifyStartResponse {
  domain: string
  token: string
  meta_tag: string
  already_verified?: boolean
  expires_in_days?: number
}

export interface VerifyConfirmResponse {
  verified: boolean
  domain: string
  expires_at?: string
  failure_reason?: string
}

export interface VerifyCheckResponse {
  found: boolean
  domain: string
  message: string
}

export interface QuotaResponse {
  used: number
  limit: number
  resets_at: string
}

export interface ScanHistoryItem {
  id: string
  target_url: string
  status: 'pending' | 'in_progress' | 'completed' | 'failed'
  results_token: string | null
  expires_at: string | null
  tier: string
  created_at: string
  critical_count: number
  high_count: number
  medium_count: number
  low_count: number
}

export interface ScanHistoryResponse {
  scans: ScanHistoryItem[]
  active_scans: ScanHistoryItem[]
  total: number
  page: number
  per_page: number
  total_pages: number
}
