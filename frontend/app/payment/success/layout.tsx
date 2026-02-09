import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'Payment Successful - ShipSecure',
  description: 'Your deep security audit is processing. You will receive a PDF report by email.',
  robots: {
    index: false,
    follow: true,
  },
}

export default function PaymentSuccessLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
