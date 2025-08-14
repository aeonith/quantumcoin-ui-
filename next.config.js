/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    appDir: false, // Use pages directory
  },
  typescript: {
    ignoreBuildErrors: true, // Allow builds even with TS errors initially
  },
  eslint: {
    ignoreDuringBuilds: true,
  },
  // Output configuration for Vercel
  output: 'export',
  trailingSlash: false,
  distDir: 'out',
  images: {
    unoptimized: true
  },
  
  // Environment variables
  env: {
    NEXT_PUBLIC_BTC_ADDRESS: process.env.NEXT_PUBLIC_BTC_ADDRESS || 'bc1qv7tpdxqvgwutfrhf53nhwgp77j5lv7whnk433y',
    EXCHANGE_AVAILABLE_FLOAT: process.env.EXCHANGE_AVAILABLE_FLOAT || '250000',
    QTC_USD_PRICE: process.env.QTC_USD_PRICE || '1.00',
    NEXT_PUBLIC_REVSTOP_DEFAULT_ON: process.env.NEXT_PUBLIC_REVSTOP_DEFAULT_ON || 'true',
  },

  // Static file serving (for legacy HTML files)
  async rewrites() {
    return [
      {
        source: '/legacy/:path*',
        destination: '/:path*',
      },
    ];
  },

  // Security headers
  async headers() {
    return [
      {
        source: '/(.*)',
        headers: [
          {
            key: 'X-Frame-Options',
            value: 'DENY',
          },
          {
            key: 'X-Content-Type-Options',
            value: 'nosniff',
          },
          {
            key: 'Referrer-Policy',
            value: 'origin-when-cross-origin',
          },
          {
            key: 'X-DNS-Prefetch-Control',
            value: 'on',
          },
        ],
      },
    ];
  },

  // API routes configuration
  async redirects() {
    return [
      {
        source: '/old-wallet',
        destination: '/wallet',
        permanent: true,
      },
    ];
  },
};

module.exports = nextConfig;
