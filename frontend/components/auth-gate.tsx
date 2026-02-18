'use client'

import { useClerk } from '@clerk/nextjs'

interface AuthGateProps {
  gated: boolean
  severity: string
  scannerName: string
  children: React.ReactNode
}

export function AuthGate({ gated, severity, scannerName, children }: AuthGateProps) {
  const { openSignUp } = useClerk()

  if (!gated) {
    return <>{children}</>
  }

  return (
    <div className="relative">
      <div className="absolute inset-0 flex flex-col items-center justify-center
                      bg-surface-elevated/90 backdrop-blur-sm rounded-b-lg
                      border-t border-border-subtle py-8 px-4">
        <div className="text-center">
          <div className="inline-flex items-center justify-center w-10 h-10 rounded-full bg-severity-high-bg mb-3">
            <svg className="w-5 h-5 text-severity-high-text" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m0 0v2m0-2h2m-2 0H10m2-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
          </div>
          <p className="text-sm font-medium text-text-primary mb-1">
            {severity.charAt(0).toUpperCase() + severity.slice(1)} severity finding
          </p>
          <p className="text-xs text-text-tertiary mb-4">
            {scannerName}
          </p>
          <button
            onClick={() => openSignUp({})}
            className="px-5 py-2 bg-brand-primary text-white text-sm
                       font-semibold rounded-md hover:bg-brand-primary/90
                       transition-colors focus:outline-none focus:ring-2
                       focus:ring-brand-primary focus:ring-offset-2"
          >
            Sign up free to view
          </button>
        </div>
      </div>
      {/* Invisible spacer to maintain layout height */}
      <div className="invisible py-8 px-4">
        <p className="text-sm mb-4">Placeholder for description height</p>
        <h4 className="text-sm font-semibold mb-2">How to Fix</h4>
        <p className="text-sm">Placeholder for remediation height</p>
      </div>
    </div>
  )
}
