import { defineConfig } from "cypress";

export default defineConfig({
  projectId: 'u6fxcx',
  e2e: {},
  component: {
    devServer: {
      framework: "vue",
      bundler: "vite",
    },
  },
});
