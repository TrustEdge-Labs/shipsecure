'use client'

import { useState, useRef } from 'react'
import { useRouter } from 'next/navigation'
import { submitSupplyChainScan } from '@/app/actions/supply-chain-scan'

type ActiveTab = 'github' | 'upload' | 'paste'

export function SupplyChainForm() {
  const router = useRouter()
  const [activeTab, setActiveTab] = useState<ActiveTab>('github')
  const [githubUrl, setGithubUrl] = useState('')
  const [pasteContent, setPasteContent] = useState('')
  const [uploadFile, setUploadFile] = useState<File | null>(null)
  const [isDragging, setIsDragging] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [validationError, setValidationError] = useState<string | null>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)

  function handleTabChange(tab: ActiveTab) {
    setActiveTab(tab)
    setError(null)
    setValidationError(null)
  }

  function handleDragOver(e: React.DragEvent<HTMLDivElement>) {
    e.preventDefault()
    setIsDragging(true)
  }

  function handleDragLeave(e: React.DragEvent<HTMLDivElement>) {
    e.preventDefault()
    setIsDragging(false)
  }

  function handleDrop(e: React.DragEvent<HTMLDivElement>) {
    e.preventDefault()
    setIsDragging(false)
    const file = e.dataTransfer.files[0]
    if (file && file.name.endsWith('.json')) {
      setUploadFile(file)
      setValidationError(null)
    } else {
      setValidationError('Please drop a .json file')
    }
  }

  function handleFileChange(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0]
    if (file) {
      setUploadFile(file)
      setValidationError(null)
    }
  }

  function validate(): boolean {
    if (activeTab === 'github') {
      if (!githubUrl.trim()) {
        setValidationError('Please enter a GitHub repository URL')
        return false
      }
      if (!githubUrl.startsWith('https://github.com/')) {
        setValidationError('URL must start with https://github.com/')
        return false
      }
    } else if (activeTab === 'paste') {
      if (!pasteContent.trim()) {
        setValidationError('Please paste your package-lock.json content')
        return false
      }
    } else if (activeTab === 'upload') {
      if (!uploadFile) {
        setValidationError('Please select a package-lock.json file')
        return false
      }
    }
    return true
  }

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault()
    setError(null)
    setValidationError(null)

    if (!validate()) return

    setIsLoading(true)

    // Plausible analytics event
    window.plausible?.('supply_chain_scan_started', { props: { input_method: activeTab } })

    try {
      let result
      if (activeTab === 'github') {
        result = await submitSupplyChainScan({ mode: 'github', value: githubUrl.trim() })
      } else if (activeTab === 'paste') {
        result = await submitSupplyChainScan({ mode: 'paste', value: pasteContent.trim() })
      } else {
        result = await submitSupplyChainScan({ mode: 'upload', value: uploadFile! })
      }

      if ('error' in result) {
        setError(result.error)
        return
      }

      const { data } = result

      if (data.share_url) {
        router.push(data.share_url)
      } else {
        // Store inline results when share link is unavailable
        if (typeof window !== 'undefined') {
          sessionStorage.setItem('supply-chain-inline-results', JSON.stringify(data))
        }
        router.push('/supply-chain/results/inline')
      }
    } finally {
      setIsLoading(false)
    }
  }

  const tabButtonClass = (tab: ActiveTab) =>
    [
      'flex-1 py-2.5 px-4 text-sm font-medium rounded-md transition-colors min-h-[44px]',
      activeTab === tab
        ? 'bg-surface-hover text-text-primary'
        : 'bg-transparent text-text-secondary hover:text-text-primary hover:bg-surface-elevated',
    ].join(' ')

  if (isLoading) {
    return (
      <div className="flex flex-col items-center justify-center py-16 space-y-4">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-brand-primary" />
        <p className="text-text-secondary text-base">Scanning dependencies...</p>
      </div>
    )
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-5">
      {/* Error banner */}
      {error && (
        <div className="bg-danger-bg border border-danger-border rounded-lg p-3 flex items-start justify-between gap-3">
          <p className="text-danger-text text-sm">{error}</p>
          <button
            type="button"
            onClick={() => setError(null)}
            className="text-danger-text/70 hover:text-danger-text flex-shrink-0 text-xs font-medium"
            aria-label="Dismiss error"
          >
            Try again
          </button>
        </div>
      )}

      {/* Tab bar */}
      <div className="flex gap-1 p-1 bg-surface-secondary rounded-lg border border-border-subtle">
        <button type="button" className={tabButtonClass('github')} onClick={() => handleTabChange('github')}>
          GitHub URL
        </button>
        <button type="button" className={tabButtonClass('upload')} onClick={() => handleTabChange('upload')}>
          Upload File
        </button>
        <button type="button" className={tabButtonClass('paste')} onClick={() => handleTabChange('paste')}>
          Paste Content
        </button>
      </div>

      {/* Tab content */}
      <div className="space-y-2">
        {activeTab === 'github' && (
          <div>
            <label htmlFor="github-url" className="block text-sm font-medium text-text-secondary mb-1.5">
              GitHub repository URL
            </label>
            <input
              id="github-url"
              type="text"
              value={githubUrl}
              onChange={(e) => { setGithubUrl(e.target.value); setValidationError(null) }}
              placeholder="https://github.com/owner/repo"
              className="w-full px-4 py-3 rounded-lg border border-border-default bg-surface-elevated text-text-primary focus:ring-2 focus:ring-focus-ring focus:border-focus-ring outline-none transition placeholder:text-text-muted"
            />
            <p className="mt-1.5 text-xs text-text-tertiary">
              We&apos;ll fetch the package-lock.json from the default branch (main or master).
            </p>
          </div>
        )}

        {activeTab === 'upload' && (
          <div>
            <label className="block text-sm font-medium text-text-secondary mb-1.5">
              package-lock.json file
            </label>
            <div
              className={[
                'border-2 border-dashed rounded-lg p-8 text-center cursor-pointer transition-colors',
                isDragging
                  ? 'border-brand-primary bg-surface-elevated'
                  : 'border-border-default hover:border-border-default/80 hover:bg-surface-elevated/50',
              ].join(' ')}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              onDrop={handleDrop}
              onClick={() => fileInputRef.current?.click()}
              role="button"
              aria-label="Upload package-lock.json file"
              tabIndex={0}
              onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') fileInputRef.current?.click() }}
            >
              {uploadFile ? (
                <div className="space-y-1">
                  <p className="text-text-primary font-medium text-sm">{uploadFile.name}</p>
                  <p className="text-text-tertiary text-xs">{(uploadFile.size / 1024).toFixed(1)} KB — click to change</p>
                </div>
              ) : (
                <div className="space-y-1">
                  <p className="text-text-secondary text-sm">Drop package-lock.json here or click to browse</p>
                  <p className="text-text-tertiary text-xs">.json files only, max 5 MB</p>
                </div>
              )}
            </div>
            <input
              ref={fileInputRef}
              type="file"
              accept=".json"
              onChange={handleFileChange}
              className="hidden"
              aria-label="Select package-lock.json file"
            />
          </div>
        )}

        {activeTab === 'paste' && (
          <div>
            <label htmlFor="paste-content" className="block text-sm font-medium text-text-secondary mb-1.5">
              package-lock.json content
            </label>
            <textarea
              id="paste-content"
              value={pasteContent}
              onChange={(e) => { setPasteContent(e.target.value); setValidationError(null) }}
              placeholder="Paste your package-lock.json content here..."
              rows={10}
              className="w-full px-4 py-3 rounded-lg border border-border-default bg-surface-elevated text-text-primary focus:ring-2 focus:ring-focus-ring focus:border-focus-ring outline-none transition font-mono text-sm placeholder:text-text-muted resize-y"
            />
          </div>
        )}

        {/* Validation error */}
        {validationError && (
          <p className="text-danger-primary text-sm">{validationError}</p>
        )}
      </div>

      {/* Submit */}
      <button
        type="submit"
        disabled={isLoading}
        className="bg-brand-primary hover:bg-brand-primary-hover disabled:bg-brand-accent text-white font-semibold rounded-lg py-3 px-6 text-lg w-full transition"
      >
        Scan Dependencies
      </button>
    </form>
  )
}
