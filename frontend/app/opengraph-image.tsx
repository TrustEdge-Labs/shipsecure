import { ImageResponse } from 'next/og'

export const runtime = 'edge'
export const alt = 'ShipSecure - Security Scanning for Vibe-Coded Apps'
export const size = { width: 1200, height: 630 }
export const contentType = 'image/png'

export default async function Image() {
  return new ImageResponse(
    (
      <div
        style={{
          background: 'linear-gradient(135deg, #1e3a5f 0%, #2563eb 50%, #1e40af 100%)',
          width: '100%',
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          color: 'white',
          fontFamily: 'system-ui, sans-serif',
        }}
      >
        <div
          style={{
            fontSize: 80,
            fontWeight: 'bold',
            marginBottom: 20,
          }}
        >
          ShipSecure
        </div>
        <div
          style={{
            fontSize: 36,
            opacity: 0.85,
          }}
        >
          Security Scanning for Vibe-Coded Apps
        </div>
        <div
          style={{
            fontSize: 24,
            opacity: 0.6,
            marginTop: 40,
          }}
        >
          Ship fast, stay safe.
        </div>
      </div>
    ),
    {
      ...size,
    }
  )
}
