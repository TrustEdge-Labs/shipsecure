import { CreateScanResponse, ScanResponse, VerifiedDomain, VerifyCheckResponse, VerifyConfirmResponse, VerifyStartResponse } from './types'

export async function createScan(url: string, email: string): Promise<CreateScanResponse> {
  const res = await fetch(`/api/v1/scans`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ url, email }),
  })
  if (!res.ok) {
    const error = await res.json()
    throw new Error(error.detail || 'Failed to start scan')
  }
  return res.json()
}

export async function getScan(id: string): Promise<ScanResponse> {
  const res = await fetch(`/api/v1/scans/${id}`, { cache: 'no-store' })
  if (!res.ok) throw new Error('Scan not found')
  return res.json()
}

export async function getScanByToken(token: string): Promise<ScanResponse> {
  const res = await fetch(`/api/v1/results/${token}`, { cache: 'no-store' })
  if (!res.ok) throw new Error('Results not found or expired')
  return res.json()
}

export async function getScanCount(): Promise<number> {
  const res = await fetch(`/api/v1/stats/scan-count`, { next: { revalidate: 60 } })
  if (!res.ok) return 0
  const data = await res.json()
  return data.count
}

export async function verifyStart(domain: string, authToken: string): Promise<VerifyStartResponse> {
  const res = await fetch(`/api/v1/domains/verify-start`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${authToken}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ domain }),
  })
  if (!res.ok) {
    const error = await res.json().catch(() => ({}))
    throw new Error(error.detail || 'Failed to start domain verification')
  }
  return res.json()
}

export async function verifyConfirm(domain: string, authToken: string): Promise<VerifyConfirmResponse> {
  const res = await fetch(`/api/v1/domains/verify-confirm`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${authToken}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ domain }),
  })
  if (!res.ok) {
    const error = await res.json().catch(() => ({}))
    throw new Error(error.detail || 'Failed to confirm domain verification')
  }
  return res.json()
}

export async function verifyCheck(domain: string, authToken: string): Promise<VerifyCheckResponse> {
  const res = await fetch(`/api/v1/domains/verify-check`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${authToken}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ domain }),
  })
  if (!res.ok) {
    const error = await res.json().catch(() => ({}))
    throw new Error(error.detail || 'Failed to check domain tag')
  }
  return res.json()
}

export async function listDomains(authToken: string): Promise<VerifiedDomain[]> {
  try {
    const res = await fetch(`/api/v1/domains`, {
      headers: { 'Authorization': `Bearer ${authToken}` },
    })
    if (!res.ok) return []
    return res.json()
  } catch {
    return []
  }
}
