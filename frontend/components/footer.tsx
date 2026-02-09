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
      </div>
    </footer>
  )
}
