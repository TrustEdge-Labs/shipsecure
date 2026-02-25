import { type ReactNode } from 'react'

interface PageContainerProps {
  children: ReactNode
  maxWidth?: string
  className?: string
}

export function PageContainer({
  children,
  maxWidth = 'max-w-4xl',
  className,
}: PageContainerProps) {
  const classes = ['container mx-auto px-4', maxWidth, className]
    .filter(Boolean)
    .join(' ')
  return <div className={classes}>{children}</div>
}
