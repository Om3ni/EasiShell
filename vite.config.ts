import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Tauri provides this when targeting a physical device on the LAN.
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],

  // Vite options tailored for Tauri development, applied during `tauri dev`/`tauri build`.
  clearScreen: false, // 1. don't obscure Rust compiler errors
  server: {
    port: 1420, // 2. Tauri expects this fixed port
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    // 3. ignore the Rust side AND the workspace-root Cargo build output.
    //    (As a Cargo workspace, `target/` lives at the repo root, not under
    //    src-tauri/ — watching it causes EBUSY crashes on locked build artifacts.)
    watch: { ignored: ["**/src-tauri/**", "**/target/**"] },
  },
});
