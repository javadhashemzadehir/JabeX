import type { NextConfig } from "next";

/**
 * Production builds use static export for Tauri webview consumption.
 * Dev mode still uses `next dev` and is loaded via localhost in tauri.conf.json.
 */
const nextConfig: NextConfig = {
  output: "export",
  // Hide the Next.js Dev Tools badge in the desktop webview; errors still surface.
  devIndicators: false,
  images: {
    unoptimized: true,
  },
  reactCompiler: true,
  turbopack: {
    rules: {
      "*.css": {
        loaders: ["@tailwindcss/turbopack"],
        as: "*.css",
      },
    },
  },
};

export default nextConfig;
