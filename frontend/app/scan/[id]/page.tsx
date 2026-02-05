'use client'

import { useEffect, useState } from 'react'
import { useRouter, useParams } from 'next/navigation'
import { ProgressChecklist } from '@/components/progress-checklist'

interface ScanStatus {
  id: string
  target_url: string
  status: string
  results_token: string | null
  stage_headers: boolean
  stage_tls: boolean
  stage_files: boolean
  stage_secrets: boolean
  error_message: string | null
}

export default function ScanProgressPage() {
  const params = useParams()
  const router = useRouter()
  const scanId = params.id as string

  const [scan, setScan] = useState<ScanStatus | null>(null)
  const [loading, setLoading] = useState(true)
  const [errorCount, setErrorCount] = useState(0)

  useEffect(() => {
    const BACKEND_URL = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:3000'

    const fetchScan = async () => {
      try {
        const res = await fetch(`${BACKEND_URL}/api/v1/scans/${scanId}`, {
          cache: 'no-store',
        })

        if (!res.ok) {
          setErrorCount(prev => prev + 1)
          return
        }

        const data: ScanStatus = await res.json()
        setScan(data)
        setLoading(false)
        setErrorCount(0)

        // Auto-redirect when scan completes
        if (data.status === 'completed' && data.results_token) {
          setTimeout(() => {
            router.push(`/results/${data.results_token}`)
          }, 1000)
        }

        // Stop polling if scan is in final state
        if (data.status === 'completed' || data.status === 'failed') {
          clearInterval(interval)
        }
      } catch (error) {
        console.error('Error fetching scan:', error)
        setErrorCount(prev => prev + 1)
      }
    }

    // Initial fetch
    fetchScan()

    // Poll every 2 seconds
    const interval = setInterval(fetchScan, 2000)

    return () => clearInterval(interval)
  }, [scanId, router])

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-950 flex items-center justify-center p-4">
        <div className="bg-white dark:bg-gray-900 rounded-lg shadow-md p-8 max-w-md w-full">
          <div className="flex items-center justify-center mb-4">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          </div>
          <p className="text-center text-gray-600 dark:text-gray-400">Loading scan status...</p>
        </div>
      </div>
    )
  }

  if (!scan) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-950 flex items-center justify-center p-4">
        <div className="bg-white dark:bg-gray-900 rounded-lg shadow-md p-8 max-w-md w-full">
          <h1 className="text-xl font-semibold text-red-600 dark:text-red-400 mb-4">Scan Not Found</h1>
          <p className="text-gray-600 dark:text-gray-400 mb-6">
            The scan you're looking for doesn't exist or has expired.
          </p>
          <a
            href="/"
            className="inline-block bg-blue-600 text-white px-6 py-2 rounded-md hover:bg-blue-700 transition-colors"
          >
            Start New Scan
          </a>
        </div>
      </div>
    )
  }

  if (scan.status === 'failed') {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-950 flex items-center justify-center p-4">
        <div className="bg-white dark:bg-gray-900 rounded-lg shadow-md p-8 max-w-md w-full">
          <h1 className="text-xl font-semibold text-red-600 dark:text-red-400 mb-4">Scan Failed</h1>
          <p className="text-gray-600 dark:text-gray-400 mb-2">
            Unfortunately, the scan for <span className="font-mono text-sm">{scan.target_url}</span> failed.
          </p>
          {scan.error_message && (
            <p className="text-sm text-gray-500 dark:text-gray-500 mb-6 font-mono bg-gray-100 dark:bg-gray-800 p-3 rounded">
              {scan.error_message}
            </p>
          )}
          <a
            href="/"
            className="inline-block bg-blue-600 text-white px-6 py-2 rounded-md hover:bg-blue-700 transition-colors"
          >
            Try Again
          </a>
        </div>
      </div>
    )
  }

  const isScanning = scan.status === 'pending' || scan.status === 'in_progress'
  const isComplete = scan.status === 'completed'

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 flex items-center justify-center p-4">
      <div className="bg-white dark:bg-gray-900 rounded-lg shadow-md p-8 max-w-md w-full">
        <div className="text-center mb-6">
          {isScanning && (
            <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mb-4"></div>
          )}
          {isComplete && (
            <div className="inline-block rounded-full h-12 w-12 bg-green-100 dark:bg-green-900 flex items-center justify-center mb-4">
              <span className="text-2xl text-green-600 dark:text-green-400">✓</span>
            </div>
          )}
          <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">
            {isScanning ? 'Scanning' : 'Scan Complete!'}
          </h1>
          <p className="text-sm text-gray-600 dark:text-gray-400 break-all">
            {scan.target_url}
          </p>
        </div>

        <div className="mb-8">
          <ProgressChecklist
            stages={{
              headers: scan.stage_headers,
              tls: scan.stage_tls,
              files: scan.stage_files,
              secrets: scan.stage_secrets,
            }}
            status={scan.status}
          />
        </div>

        {isComplete && (
          <div className="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-md p-4 mb-6">
            <p className="text-sm text-green-800 dark:text-green-300">
              Redirecting to results...
            </p>
          </div>
        )}

        {errorCount >= 3 && isScanning && (
          <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-md p-4 mb-6">
            <p className="text-sm text-yellow-800 dark:text-yellow-300">
              Connection lost, retrying...
            </p>
          </div>
        )}

        <div className="text-center text-sm text-gray-500 dark:text-gray-400 border-t border-gray-200 dark:border-gray-800 pt-4">
          <p>You can close this tab. We'll email you when your scan is ready.</p>
        </div>
      </div>
    </div>
  )
}
