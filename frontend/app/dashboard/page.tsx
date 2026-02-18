import { auth, currentUser } from '@clerk/nextjs/server'
import { redirect } from 'next/navigation'
import Link from 'next/link'

export default async function DashboardPage() {
  const { userId } = await auth()
  if (!userId) redirect('/sign-in')

  const user = await currentUser()

  return (
    <main className="container mx-auto px-4 py-16 max-w-4xl">
      <h1 className="text-3xl font-bold text-text-primary mb-2">
        Welcome, {user?.firstName ?? 'there'}
      </h1>
      <p className="text-text-secondary mb-8">
        Your security dashboard. Verify a domain to start scanning.
      </p>
      <Link
        href="/verify-domain"
        className="inline-flex items-center px-6 py-3 bg-brand-primary hover:bg-brand-primary-hover text-text-inverse font-semibold rounded-lg transition text-base"
      >
        Verify your domain
      </Link>
    </main>
  )
}
