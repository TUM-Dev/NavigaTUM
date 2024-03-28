import { defineConfig } from "vite";
import Vue from "@vitejs/plugin-vue";
import VueI18nPlugin from "@intlify/unplugin-vue-i18n/vite";
import Markdown from "unplugin-vue-markdown/vite";
import prism from "markdown-it-prism";
import path from "path";

export default defineConfig({
  envDir: "env",
  appType: "spa",
  server: {
    port: 3000,
    strictPort: true,
    open: false,
    proxy: {
      "^/api/[cf].*": {
        target: "https://nav.tum.de",
      },
      "^/api/[^cf].*": {
        target: "http://127.0.0.1:3003",
        secure: false,
      },
      "/cdn": {
        target: "https://nav.tum.de",
      },
    },
  },
  build: {
    sourcemap: true,
    rollupOptions: {
      input: path.resolve("./index.html"),
      output: {
        manualChunks: {
          maplibre_gl: ["maplibre-gl"],
          swagger_ui: ["swagger-ui"],
        },
      },
    },
  },
  plugins: [
    Vue({
      include: [/\.vue$/, /\.md$/],
    }),
    VueI18nPlugin({
      include: path.resolve("./locales/**"),
      fullInstall: false,
    }),
    Markdown({
      markdownItUses: [prism],
    }),
  ],
  resolve: {
    alias: {
      vue: path.resolve("node_modules/vue/dist/vue.esm-bundler.js"),
    },
  },
});
