import { defineConfig } from "vitest/config";

// Unit tests cover the pure logic in `app/utils` (no Nuxt runtime needed), so they live outside
// `app/` to stay clear of Nuxt's auto-import scanning.
export default defineConfig({
  test: {
    include: ["test/**/*.test.ts"],
    environment: "node",
  },
});
