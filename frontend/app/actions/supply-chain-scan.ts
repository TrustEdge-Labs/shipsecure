import type {
  SupplyChainScanResponse,
  ApiErrorResponse,
} from '@/lib/supply-chain-types'

type SupplyChainScanMode = 'github' | 'upload' | 'paste'

type SupplyChainScanInput =
  | { mode: 'github'; value: string }
  | { mode: 'paste'; value: string }
  | { mode: 'upload'; value: File }

type SupplyChainScanResult =
  | { data: SupplyChainScanResponse }
  | { error: string }

function mapErrorResponse(status: number, body: Partial<ApiErrorResponse>): string {
  const errorType = body.type ?? ''

  if (status === 400) {
    if (errorType.includes('invalid-lockfile')) {
      return "This doesn't look like a valid package-lock.json file"
    }
    if (errorType.includes('too-many-deps')) {
      return 'Too many dependencies (max 5,000)'
    }
    return body.detail ?? 'Invalid request'
  }

  if (status === 429) {
    return body.detail ?? "You've reached the scan limit. Please try again later."
  }

  if (status === 502) {
    return 'Something went wrong. Try again or upload your lockfile directly'
  }

  if (status === 504) {
    return 'Scan timed out. Try a smaller lockfile'
  }

  return 'An unexpected error occurred'
}

export async function submitSupplyChainScan(
  input: SupplyChainScanInput
): Promise<SupplyChainScanResult> {
  const url = '/api/v1/scans/supply-chain'

  try {
    let response: Response

    if (input.mode === 'github') {
      response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ github_url: input.value }),
      })
    } else if (input.mode === 'paste') {
      response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ lockfile_content: input.value }),
      })
    } else {
      const formData = new FormData()
      formData.append('lockfile', input.value)
      response = await fetch(url, {
        method: 'POST',
        body: formData,
      })
    }

    if (!response.ok) {
      const errorBody = await response.json().catch(() => ({}) as Partial<ApiErrorResponse>)
      const message = mapErrorResponse(response.status, errorBody)
      return { error: message }
    }

    const data = (await response.json()) as SupplyChainScanResponse
    return { data }
  } catch {
    return { error: 'Unable to connect to the scanning service. Please try again later.' }
  }
}
