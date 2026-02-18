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
