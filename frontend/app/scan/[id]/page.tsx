'use client'

import { useEffect, useState, useRef, useCallback } from 'react'
import { useRouter, useParams } from 'next/navigation'
import { ProgressChecklist } from '@/components/progress-checklist'

const BASE_POLL_MS = 2000
const MAX_POLL_MS = 30000
const MAX_CONSECUTIVE_ERRORS = 20

interface ScanStatus {
  id: string
  target_url: string
  status: string
  results_token: string | null
  stage_detection: boolean
  stage_headers: boolean
  stage_tls: boolean
  stage_files: boolean
  stage_secrets: boolean
  stage_vibecode: boolean
  error_message: string | null
}

export default function ScanProgressPage() {
  const params = useParams()
  const router = useRouter()
  const scanId = params.id as string

  const [scan, setScan] = useState<ScanStatus | null>(null)
  const [loading, setLoading] = useState(true)
  const [errorCount, setErrorCount] = useState(0)
  const [gaveUp, setGaveUp] = useState(false)
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const stoppedRef = useRef(false)
  const consecutiveErrorsRef = useRef(0)

  const scheduleNext = useCallback((errors: number) => {
    if (stoppedRef.current) return
    const backoff = Math.min(BASE_POLL_MS * Math.pow(1.5, errors), MAX_POLL_MS)
    timeoutRef.current = setTimeout(fetchScan, backoff)
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const fetchScan = useCallback(async () => {
    if (stoppedRef.current) return
    try {
      const res = await fetch(`/api/v1/scans/${scanId}`, {
        cache: 'no-store',
      })

      if (!res.ok) {
        consecutiveErrorsRef.current += 1
        setErrorCount(consecutiveErrorsRef.current)
        setLoading(false)
        if (consecutiveErrorsRef.current >= MAX_CONSECUTIVE_ERRORS) {
          stoppedRef.current = true
          setGaveUp(true)
          return
        }
        scheduleNext(consecutiveErrorsRef.current)
        return
      }

      const data: ScanStatus = await res.json()
      setScan(data)
      setLoading(false)
      consecutiveErrorsRef.current = 0
      setErrorCount(0)

      if (data.status === 'completed' && data.results_token) {
        stoppedRef.current = true
        setTimeout(() => router.push(`/results/${data.results_token}`), 1000)
        return
      }

      if (data.status === 'failed') {
        stoppedRef.current = true
        return
      }

      scheduleNext(0)
    } catch {
      consecutiveErrorsRef.current += 1
      setErrorCount(consecutiveErrorsRef.current)
      setLoading(false)
      if (consecutiveErrorsRef.current >= MAX_CONSECUTIVE_ERRORS) {
        stoppedRef.current = true
        setGaveUp(true)
        return
      }
      scheduleNext(consecutiveErrorsRef.current)
    }
  }, [scanId, router, scheduleNext])

  useEffect(() => {
    stoppedRef.current = false
    consecutiveErrorsRef.current = 0
    fetchScan()

    const handleVisibility = () => {
      if (document.hidden) {
        if (timeoutRef.current) clearTimeout(timeoutRef.current)
      } else if (!stoppedRef.current) {
        fetchScan()
      }
    }
    document.addEventListener('visibilitychange', handleVisibility)

    return () => {
      stoppedRef.current = true
      if (timeoutRef.current) clearTimeout(timeoutRef.current)
      document.removeEventListener('visibilitychange', handleVisibility)
    }
  }, [fetchScan])

  if (loading) {
    return (
      <div className="min-h-screen bg-surface-secondary flex items-center justify-center p-4">
        <div className="bg-surface-elevated rounded-(card) shadow-md p-8 max-w-md w-full">
          <div className="flex items-center justify-center mb-4">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-brand-primary"></div>
          </div>
          <p className="text-center text-text-secondary">Connecting to scan service...</p>
        </div>
      </div>
    )
  }

  if (!scan) {
    return (
      <div className="min-h-screen bg-surface-secondary flex items-center justify-center p-4">
        <div className="bg-surface-elevated rounded-(card) shadow-md p-8 max-w-md w-full">
          <h1 className="text-xl font-semibold text-danger-primary mb-4">Scan Not Found</h1>
          <p className="text-text-secondary mb-2">
            This scan doesn't exist or has expired. Scan results are available for 30 days after completion.
          </p>
          <p className="text-sm text-text-tertiary mb-6">
            If you just submitted a scan, wait a few seconds and refresh this page.
          </p>
          <a
            href="/"
            className="inline-flex items-center justify-center min-h-[44px] bg-brand-primary hover:bg-brand-primary/90 text-white px-6 py-3 rounded-lg font-medium transition-colors"
          >
            Start New Scan
          </a>
        </div>
      </div>
    )
  }

  if (scan.status === 'failed') {
    return (
      <div className="min-h-screen bg-surface-secondary flex items-center justify-center p-4">
        <div className="bg-surface-elevated rounded-(card) shadow-md p-8 max-w-md w-full">
          <h1 className="text-xl font-semibold text-danger-primary mb-4">Scan Failed</h1>
          <p className="text-text-secondary mb-2">
            Unfortunately, the scan for <span className="font-mono text-sm break-all">{scan.target_url}</span> failed.
          </p>
          {scan.error_message && (
            <p className="text-sm text-text-tertiary mb-4 font-mono bg-surface-secondary p-3 rounded break-all">
              {scan.error_message}
            </p>
          )}
          <p className="text-sm text-text-secondary mb-6">
            Common causes: the target website may be unreachable, blocking automated requests, or experiencing downtime. Try scanning again or check that the URL is accessible.
          </p>
          <a
            href="/"
            className="inline-flex items-center justify-center min-h-[44px] bg-brand-primary hover:bg-brand-primary/90 text-white px-6 py-3 rounded-lg font-medium transition-colors"
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
    <div className="min-h-screen bg-surface-secondary flex items-center justify-center p-4">
      <div className="bg-surface-elevated rounded-(card) shadow-md p-8 max-w-md w-full">
        <div className="text-center mb-6">
          {isScanning && (
            <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-brand-primary mb-4"></div>
          )}
          {isComplete && (
            <div className="inline-block rounded-full h-12 w-12 bg-grade-a-bg flex items-center justify-center mb-4">
              <span className="text-2xl text-grade-a-text">✓</span>
            </div>
          )}
          <h1 className="text-2xl font-bold text-text-primary mb-2">
            {isScanning ? 'Scanning' : 'Scan Complete!'}
          </h1>
          <p className="text-sm text-text-secondary break-all">
            {scan.target_url}
          </p>
        </div>

        <div className="mb-8">
          <ProgressChecklist
            stages={{
              detection: scan.stage_detection,
              headers: scan.stage_headers,
              tls: scan.stage_tls,
              files: scan.stage_files,
              secrets: scan.stage_secrets,
              vibecode: scan.stage_vibecode,
            }}
            status={scan.status}
          />
        </div>

        {isComplete && (
          <div className="bg-success-bg border border-success-border rounded-md p-4 mb-6">
            <p className="text-sm text-success-text">
              Redirecting to results...
            </p>
          </div>
        )}

        {gaveUp && (
          <div className="bg-danger-bg border border-danger-border rounded-md p-4 mb-6">
            <p className="text-sm text-danger-text">
              Lost connection to our servers after multiple retries. Please refresh the page or check back later.
            </p>
          </div>
        )}

        {!gaveUp && errorCount >= 3 && isScanning && (
          <div className="bg-caution-bg border border-caution-border rounded-md p-4 mb-6">
            <p className="text-sm text-caution-text">
              Having trouble connecting to our servers. We're still trying -- you can also refresh the page or check back later.
            </p>
          </div>
        )}

        <div className="text-center text-sm text-text-tertiary border-t border-border-subtle pt-4">
          <p>You can close this tab. We'll email you when your scan is ready.</p>
        </div>
      </div>
    </div>
  )
}
