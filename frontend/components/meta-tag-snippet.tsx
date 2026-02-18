'use client'

import { useState } from 'react'
import { Copy, Check } from 'lucide-react'

interface MetaTagSnippetProps {
  metaTag: string
}

export function MetaTagSnippet({ metaTag }: MetaTagSnippetProps) {
  const [copied, setCopied] = useState(false)

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(metaTag)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch {
      // Fallback: select text manually
    }
  }

  return (
    <div className="relative bg-gray-900 rounded-lg p-4 font-mono text-sm text-green-400">
      <pre className="overflow-x-auto whitespace-pre-wrap break-all pr-10">
        {metaTag}
      </pre>
      <button
        onClick={handleCopy}
        aria-label="Copy meta tag"
        className="absolute top-3 right-3 p-1.5 rounded hover:bg-gray-700 transition-colors text-gray-400 hover:text-gray-200"
      >
        {copied ? (
          <Check className="w-4 h-4 text-green-400" aria-hidden="true" />
        ) : (
          <Copy className="w-4 h-4" aria-hidden="true" />
        )}
      </button>
    </div>
  )
}
