import type { NextConfig } from "next";
import { withPlausibleProxy } from "next-plausible";

const nextConfig: NextConfig = {
  output: 'standalone',
};

export default withPlausibleProxy()(nextConfig);
