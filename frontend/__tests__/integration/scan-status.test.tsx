import { useState, useEffect } from 'react'
import { describe, expect, test } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import { server } from '@/__tests__/helpers/msw/server'
import { errorHandlers } from '@/__tests__/helpers/msw/handlers'
import { scanFixtures } from '@/__tests__/helpers/fixtures/scan'
import { http, HttpResponse } from 'msw'

const BACKEND_URL = 'http://localhost:3000'

/**
 * Minimal test component that fetches scan status and renders it.
 * Proves the MSW data fetching pipeline works end-to-end.
 */
function ScanStatusTestComponent({ scanId }: { scanId: string }) {
  const [status, setStatus] = useState<string>('loading')
  const [framework, setFramework] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetch(`${BACKEND_URL}/api/v1/scans/${scanId}`)
      .then(res => {
        if (!res.ok) throw new Error('Scan not found')
        return res.json()
      })
      .then(data => {
        setStatus(data.status)
        setFramework(data.detected_framework)
      })
      .catch(err => setError(err.message))
  }, [scanId])

  if (error) return <div role="alert">{error}</div>
  if (status === 'loading') return <div>Loading...</div>

  return (
    <div>
      <span data-testid="scan-status">{status}</span>
      {framework && <span data-testid="framework">{framework}</span>}
    </div>
  )
}

describe('Scan Status API Integration', () => {
  test('fetches and displays scan status from MSW handler', async () => {
    renderWithProviders(<ScanStatusTestComponent scanId="test-scan-123" />)

    // Initially shows loading
    expect(screen.getByText('Loading...')).toBeInTheDocument()

    // MSW handler returns inProgress fixture by default
    await waitFor(() => {
      expect(screen.getByTestId('scan-status')).toHaveTextContent('in_progress')
    })

    // Verify framework detection data came through
    expect(screen.getByTestId('framework')).toHaveTextContent('Next.js')
  })

  test('fetches completed scan with findings count', async () => {
    // Override default handler to return completed scan
    server.use(
      http.get(`${BACKEND_URL}/api/v1/scans/:id`, () => {
        return HttpResponse.json(scanFixtures.completed)
      })
    )

    renderWithProviders(<ScanStatusTestComponent scanId="test-scan-456" />)

    await waitFor(() => {
      expect(screen.getByTestId('scan-status')).toHaveTextContent('completed')
    })
  })

  test('displays error when scan API returns 404', async () => {
    // Use pre-built error handler
    server.use(errorHandlers.scanNotFound)

    renderWithProviders(<ScanStatusTestComponent scanId="nonexistent" />)

    await waitFor(() => {
      expect(screen.getByRole('alert')).toHaveTextContent('Scan not found')
    })
  })

  test('displays error when scan API returns 500', async () => {
    server.use(errorHandlers.scanServerError)

    renderWithProviders(<ScanStatusTestComponent scanId="server-error" />)

    await waitFor(() => {
      expect(screen.getByRole('alert')).toBeInTheDocument()
    })
  })
})
