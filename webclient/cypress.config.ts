import { defineConfig } from "cypress";
import { initPlugin } from "@frsource/cypress-plugin-visual-regression-diff/plugins";

export default defineConfig({
  projectId: "u6fxcx",
  e2e: {
    experimentalStudio: true,
    setupNodeEvents(on, config) {
      initPlugin(on, config);
    },
  },
});
