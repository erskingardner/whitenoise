import { defineConfig } from "vite";
import { visualizer } from "rollup-plugin-visualizer";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import * as path from "path";

const SRC_DIR = path.resolve(__dirname, "./src");
const PUBLIC_DIR = path.resolve(__dirname, "./public");
const BUILD_DIR = path.resolve(__dirname, "./build");

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
    plugins: [svelte(), visualizer()],
    root: SRC_DIR,
    base: "",
    publicDir: PUBLIC_DIR,
    build: {
        outDir: BUILD_DIR,
        assetsInlineLimit: 0,
        emptyOutDir: true,
        rollupOptions: {
            treeshake: true,
        },
    },

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    clearScreen: false,
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
            // 3. tell vite to ignore watching `src-tauri`
            ignored: ["**/src-tauri/**"],
        },
    },
}));
