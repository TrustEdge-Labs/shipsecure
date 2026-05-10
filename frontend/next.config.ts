import type { NextConfig } from "next";

const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL ?? '';
const extraConnectSrc = backendUrl ? ` ${backendUrl}` : '';

const cspHeader = [
  "default-src 'self'",
  "script-src 'self' 'unsafe-inline' 'unsafe-eval' https://plausible.io https://*.clerk.accounts.dev https://clerk.shipsecure.ai https://accounts.shipsecure.ai",
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data: https://img.clerk.com https://*.clerk.com https://clerk.shipsecure.ai",
  "font-src 'self' data:",
  `connect-src 'self' https://*.clerk.accounts.dev https://clerk.shipsecure.ai https://accounts.shipsecure.ai https://plausible.io${extraConnectSrc}`,
  "frame-src 'self' https://*.clerk.accounts.dev https://clerk.shipsecure.ai https://accounts.shipsecure.ai",
  "object-src 'none'",
  "base-uri 'self'",
  "form-action 'self'",
  "frame-ancestors 'none'",
  "upgrade-insecure-requests",
].join('; ');

const nextConfig: NextConfig = {
  output: 'standalone',
  experimental: {
    testProxy: process.env.PLAYWRIGHT_TEST === '1',
  },
  headers: async () => [
    {
      source: '/(.*)',
      headers: [
        {
          key: 'Content-Security-Policy',
          value: cspHeader,
        },
        {
          key: 'X-Content-Type-Options',
          value: 'nosniff',
        },
        {
          key: 'Referrer-Policy',
          value: 'strict-origin-when-cross-origin',
        },
        {
          key: 'Permissions-Policy',
          value: 'camera=(), microphone=(), geolocation=()',
        },
      ],
    },
  ],
};

export default nextConfig;
