import { ScanForm } from '@/components/scan-form'

async function getScanCount(): Promise<number | null> {
  try {
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:3000'
    const res = await fetch(`${backendUrl}/api/v1/stats/scan-count`, {
      next: { revalidate: 60 }
    })
    if (!res.ok) return null
    const data = await res.json()
    return data.count
  } catch {
    return null
  }
}

export default async function Home() {
  const scanCount = await getScanCount()

  return (
    <div className="min-h-screen bg-white dark:bg-gray-950 text-gray-900 dark:text-gray-100">
      {/* Hero Section */}
      <main className="container mx-auto px-4 py-16 sm:py-24 max-w-4xl">
        <div className="text-center mb-12">
          <h1 className="text-4xl sm:text-5xl md:text-6xl font-bold mb-4 bg-gradient-to-r from-blue-600 to-blue-800 dark:from-blue-400 dark:to-blue-600 bg-clip-text text-transparent">
            Ship fast, stay safe.
          </h1>
          <p className="text-lg sm:text-xl text-gray-600 dark:text-gray-400 mb-2">
            Free security scanning for vibe-coded web apps.
          </p>
          <p className="text-sm text-gray-500 dark:text-gray-500">
            Catch security flaws before they become breaches.
          </p>
        </div>

        {/* Scan Form Card */}
        <div className="bg-gray-50 dark:bg-gray-900 rounded-2xl shadow-lg border border-gray-200 dark:border-gray-800 p-6 sm:p-8 mb-12">
          <ScanForm />
        </div>

        {/* What We Check */}
        <div className="text-center mb-12">
          <h2 className="text-2xl font-semibold mb-4 text-gray-900 dark:text-gray-100">
            What we check
          </h2>
          <div className="grid sm:grid-cols-2 gap-4 max-w-2xl mx-auto text-left">
            <div className="flex gap-3">
              <div className="text-blue-600 dark:text-blue-400 text-xl">&#x1F512;</div>
              <div>
                <h3 className="font-semibold text-gray-900 dark:text-gray-100">Security Headers</h3>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  CSP, HSTS, X-Frame-Options, and more
                </p>
              </div>
            </div>
            <div className="flex gap-3">
              <div className="text-blue-600 dark:text-blue-400 text-xl">&#x1F511;</div>
              <div>
                <h3 className="font-semibold text-gray-900 dark:text-gray-100">TLS Configuration</h3>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Certificate validity, protocol versions, cipher strength
                </p>
              </div>
            </div>
            <div className="flex gap-3">
              <div className="text-blue-600 dark:text-blue-400 text-xl">&#x1F4C4;</div>
              <div>
                <h3 className="font-semibold text-gray-900 dark:text-gray-100">Exposed Files</h3>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  .env, .git, config files, and sensitive data
                </p>
              </div>
            </div>
            <div className="flex gap-3">
              <div className="text-blue-600 dark:text-blue-400 text-xl">&#x1F50D;</div>
              <div>
                <h3 className="font-semibold text-gray-900 dark:text-gray-100">JavaScript Secrets</h3>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  API keys, tokens, and credentials in client code
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* Social Proof */}
        {scanCount !== null && scanCount > 0 && (
          <div className="text-center text-sm text-gray-500 dark:text-gray-500">
            <span className="font-semibold text-blue-600 dark:text-blue-400">
              {scanCount.toLocaleString()}
            </span>{' '}
            scans completed
          </div>
        )}
      </main>

      {/* Footer */}
      <footer className="border-t border-gray-200 dark:border-gray-800 py-8">
        <div className="container mx-auto px-4 text-center text-sm text-gray-600 dark:text-gray-400">
          <p>&copy; {new Date().getFullYear()} TrustEdge Audit</p>
        </div>
      </footer>
    </div>
  )
}
