import Link from 'next/link'

export function Footer() {
  const currentYear = new Date().getFullYear()

  return (
    <footer className="border-t border-gray-200 dark:border-gray-800 py-8 mt-auto">
      <div className="container mx-auto px-4">
        <nav className="flex flex-col sm:flex-row items-center justify-center gap-2 sm:gap-4 mb-3">
          <Link
            href="/privacy"
            className="text-sm text-gray-500 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 transition"
          >
            Privacy Policy
          </Link>
          <span className="hidden sm:inline text-gray-400 dark:text-gray-600">•</span>
          <Link
            href="/terms"
            className="text-sm text-gray-500 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 transition"
          >
            Terms of Service
          </Link>
        </nav>
        <p className="text-center text-sm text-gray-500 dark:text-gray-400">
          &copy; {currentYear} ShipSecure. All rights reserved.
        </p>
        <div className="mt-4 text-center text-xs text-gray-400 dark:text-gray-500">
          <p className="mb-1">Powered by open source:</p>
          <div className="flex flex-wrap justify-center gap-x-3 gap-y-1">
            <span>
              <a href="https://github.com/projectdiscovery/nuclei" target="_blank" rel="noopener noreferrer" className="hover:text-blue-600 dark:hover:text-blue-400 underline">Nuclei</a>
              {' '}by{' '}
              <a href="https://projectdiscovery.io" target="_blank" rel="noopener noreferrer" className="hover:text-blue-600 dark:hover:text-blue-400">ProjectDiscovery</a>
              {' '}(<a href="https://github.com/projectdiscovery/nuclei/blob/main/LICENSE.md" target="_blank" rel="noopener noreferrer" className="hover:text-blue-600 dark:hover:text-blue-400">MIT</a>)
            </span>
            <span className="hidden sm:inline text-gray-400 dark:text-gray-600">·</span>
            <span>
              <a href="https://testssl.sh" target="_blank" rel="noopener noreferrer" className="hover:text-blue-600 dark:hover:text-blue-400 underline">testssl.sh</a>
              {' '}(<a href="https://github.com/testssl/testssl.sh/blob/3.3dev/LICENSE" target="_blank" rel="noopener noreferrer" className="hover:text-blue-600 dark:hover:text-blue-400">GPLv2</a>)
            </span>
          </div>
        </div>
      </div>
    </footer>
  )
}
