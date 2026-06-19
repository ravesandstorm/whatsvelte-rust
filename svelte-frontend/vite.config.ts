import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Tauri drives this dev server via beforeDevCommand; port must match devUrl in
// tauri.conf.json. strictPort so a port clash fails loudly instead of drifting.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
});
