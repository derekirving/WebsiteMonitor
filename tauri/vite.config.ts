import type { UserConfig } from 'vite'
import path from 'path'
import { fileURLToPath } from 'node:url'
import { defineConfig } from "vite";
import { glob } from "glob";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  clearScreen: false,
  root: path.join(__dirname, "src"),
  build: {
    rollupOptions: {
      input: glob.sync(path.resolve(__dirname, "src", "*.html")),
    },
    outDir: path.join(__dirname, "dist"),
    //emptyOutDir: true,
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
        protocol: "ws",
        host,
        port: 1421,
      }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  css: {
    preprocessorOptions: {
      scss: {
        silenceDeprecations: [
          'import',
          'color-functions',
          'global-builtin',
          'legacy-js-api',
        ],
      },
    },
  }
})) satisfies UserConfig;
