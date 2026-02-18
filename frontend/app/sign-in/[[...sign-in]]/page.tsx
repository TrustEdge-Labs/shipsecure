import { SignIn } from '@clerk/nextjs'

export default function SignInPage() {
  return (
    <main className="flex min-h-[calc(100vh-var(--header-height))] items-center justify-center">
      <SignIn />
    </main>
  )
}
