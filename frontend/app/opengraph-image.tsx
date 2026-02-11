import { ImageResponse } from 'next/og'
import { readFile } from 'node:fs/promises'
import { join } from 'node:path'

export const alt = 'ShipSecure - Security Scanning for Vibe-Coded Apps'
export const size = { width: 1200, height: 630 }
export const contentType = 'image/png'

export default async function Image() {
  // Load logo as base64 data URI (ImageResponse supports data URIs)
  const logoBuffer = await readFile(join(process.cwd(), 'public/logo.png'))
  const logoBase64 = logoBuffer.toString('base64')
  const logoSrc = `data:image/png;base64,${logoBase64}`

  return new ImageResponse(
    (
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          // Dark branded background using design token equivalent
          // --primitive-gray-950 = oklch(0.145 0.006 265) ~ #0c0e12
          // Gradient from dark blue to near-black for visual depth
          background: 'linear-gradient(135deg, #0f172a 0%, #1e293b 50%, #0f172a 100%)',
          padding: '60px',
        }}
      >
        {/* Logo centered with generous size */}
        <img
          src={logoSrc}
          alt=""
          width={600}
          height={400}
          style={{
            objectFit: 'contain',
            marginBottom: '30px',
          }}
        />
        {/* Tagline below logo */}
        <div
          style={{
            fontSize: 32,
            color: 'rgba(255, 255, 255, 0.7)',
            fontFamily: 'system-ui, sans-serif',
            textAlign: 'center',
          }}
        >
          Security Scanning for Vibe-Coded Apps
        </div>
      </div>
    ),
    { ...size }
  )
}
