'use client'

import { useEffect } from 'react'
import { useSearchParams } from 'next/navigation'

export function SignupTracker() {
  const searchParams = useSearchParams()

  useEffect(() => {
    if (searchParams.get('source') === 'signup') {
      window.plausible?.('Signup Completed')
    }
  }, [searchParams])

  return null
}
