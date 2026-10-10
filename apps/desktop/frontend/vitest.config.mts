import path from "node:path";
import { fileURLToPath } from "node:url";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

const rootDir = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    setupFiles: ["./vitest.setup.ts"],
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
  },
  resolve: {
    alias: {
      "@": path.resolve(rootDir, "./src"),
      "@components": path.resolve(rootDir, "./src/components"),
      "@features": path.resolve(rootDir, "./src/features"),
      "@lib": path.resolve(rootDir, "./src/lib"),
    },
  },
});
