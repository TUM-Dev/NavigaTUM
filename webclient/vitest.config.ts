import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

// Unit tests cover the pure logic in `app/utils` and the framework-free composables (zod schemas),
// so they live outside `app/` to stay clear of Nuxt's auto-import scanning. The `~` alias mirrors
// Nuxt's so those modules' `~/...` imports resolve under plain Vitest.
export default defineConfig({
  resolve: {
    alias: { "~": fileURLToPath(new URL("./app", import.meta.url)) },
  },
  test: {
    include: ["test/**/*.test.ts"],
    environment: "node",
  },
});
