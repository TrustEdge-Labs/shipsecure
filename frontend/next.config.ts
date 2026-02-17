import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: 'standalone',
  experimental: {
    testProxy: process.env.PLAYWRIGHT_TEST === '1',
  },
};

export default nextConfig;
