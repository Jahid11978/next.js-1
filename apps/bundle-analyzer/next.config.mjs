/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',
  distDir: 'dist',
  rewrites() {
    return [
      {
        source: '/data/:path*',
        destination: 'http://localhost:4000/data/:path*',
      },
    ]
  },
}

export default nextConfig
