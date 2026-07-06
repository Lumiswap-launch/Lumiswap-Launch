/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  
  webpack: (config) => {
    config.resolve.fallback = {
      ...config.resolve.fallback,
      fs: false,
      net: false,
      tls: false,
    };
    return config;
  },
  
  images: {
    domains: ['stellar.org'],
  },
  
  env: {
    NEXT_PUBLIC_APP_NAME: 'Lumiswap Launch',
    NEXT_PUBLIC_APP_VERSION: '1.0.0',
  },
};

module.exports = nextConfig;
