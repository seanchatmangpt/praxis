/** @type {import('next').NextConfig} */
const nextConfig = {
  // Use server mode for better client-side component support
  // output: 'export' mode has limitations with Web Workers and dynamic code
  staticPageGenerationTimeout: 120,
};

export default nextConfig;
