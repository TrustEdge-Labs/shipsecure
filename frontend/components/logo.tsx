import Image from 'next/image'

interface LogoProps {
  size: 'small' | 'medium' | 'large'
  className?: string
}

const sizeMap = {
  small: { width: 96, height: 64 },
  medium: { width: 384, height: 256 },
  large: { width: 768, height: 512 },
} as const

export function Logo({ size, className }: LogoProps) {
  const { width, height } = sizeMap[size]

  return (
    <Image
      src="/logo.png"
      alt="ShipSecure"
      width={width}
      height={height}
      className={className}
      priority={size === 'large'}
    />
  )
}
