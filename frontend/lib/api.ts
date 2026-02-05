import { CreateScanResponse, ScanResponse } from './types'

const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:3000'

export async function createScan(url: string, email: string): Promise<CreateScanResponse> {
  const res = await fetch(`${BACKEND_URL}/api/v1/scans`, {
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
  const res = await fetch(`${BACKEND_URL}/api/v1/scans/${id}`, { cache: 'no-store' })
  if (!res.ok) throw new Error('Scan not found')
  return res.json()
}

export async function getScanByToken(token: string): Promise<ScanResponse> {
  const res = await fetch(`${BACKEND_URL}/api/v1/results/${token}`, { cache: 'no-store' })
  if (!res.ok) throw new Error('Results not found or expired')
  return res.json()
}

export async function getScanCount(): Promise<number> {
  const res = await fetch(`${BACKEND_URL}/api/v1/stats/scan-count`, { next: { revalidate: 60 } })
  if (!res.ok) return 0
  const data = await res.json()
  return data.count
}
