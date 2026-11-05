import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Unit tests run in Node; `fake-indexeddb` stands in for the webview's
// IndexedDB. The svelte plugin compiles the runes in `*.svelte.ts` stores.
export default defineConfig({
  plugins: [svelte()],
  resolve: { conditions: ["browser"] },
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
});
