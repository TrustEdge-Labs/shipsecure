"use client"

import { useState } from "react"

export function ShareButton({ url }: { url: string }) {
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    await navigator.clipboard.writeText(url)
    window.plausible?.('Share Clicked')
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <button
      onClick={handleCopy}
      className="inline-flex items-center justify-center gap-2 px-6 py-2 min-h-[44px] w-full sm:w-auto bg-surface-elevated text-text-primary border border-border-default rounded-md hover:bg-surface-hover transition-colors"
      aria-label="Copy results link to clipboard"
    >
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
      </svg>
      {copied ? "Copied!" : "Copy Link"}
      {copied && (
        <span className="sr-only" aria-live="polite">Link copied to clipboard</span>
      )}
    </button>
  )
}
