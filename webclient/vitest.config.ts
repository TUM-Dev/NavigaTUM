import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

// Tests live outside `app/` to stay clear of Nuxt's auto-import scanning.
export default defineConfig({
  resolve: {
    alias: { "~": fileURLToPath(new URL("./app", import.meta.url)) },
  },
  test: {
    include: ["test/**/*.test.ts"],
    environment: "node",
  },
});
