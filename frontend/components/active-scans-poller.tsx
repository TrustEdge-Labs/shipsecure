'use client'

import { useEffect } from 'react'
import { useRouter } from 'next/navigation'

interface ActiveScansPollertProps {
  hasActiveScans: boolean
}

export function ActiveScansPoller({ hasActiveScans }: ActiveScansPollertProps) {
  const router = useRouter()

  useEffect(() => {
    if (!hasActiveScans) return

    const interval = setInterval(() => {
      router.refresh()
    }, 7000)

    return () => clearInterval(interval)
  }, [hasActiveScans, router])

  return null
}
