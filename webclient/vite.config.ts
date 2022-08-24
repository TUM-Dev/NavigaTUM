import { fileURLToPath, URL } from "node:url";

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import VueI18nPlugin from "@intlify/unplugin-vue-i18n/vite";
import path from "path";

// https://vitejs.dev/config/
export default defineConfig({
  build: {
    rollupOptions: {
      input: path.resolve(__dirname, "./index.html"),
    },
  },
  plugins: [
    vue(),
    VueI18nPlugin({
      include: path.resolve(__dirname, "./src/locales/**"),
    }),
  ],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
      vue: path.resolve(__dirname, "node_modules/vue/dist/vue.esm-bundler.js"),
    },
  },
});
