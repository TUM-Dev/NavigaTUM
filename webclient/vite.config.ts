import { URL, fileURLToPath } from "node:url";

import { defineConfig } from "vite";
import Vue from "@vitejs/plugin-vue";
import VueI18nPlugin from "@intlify/unplugin-vue-i18n/vite";
import Markdown from "vite-plugin-md";
import link from "@yankeeinlondon/link-builder";
import path from "path";
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import pluginRewriteAll from "vite-plugin-rewrite-all";

export default defineConfig({
  envDir: path.resolve(__dirname, "./env"),
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
        target: "http://127.0.0.1:8080",
        secure: false,
      },
    },
  },
  build: {
    sourcemap: true,
    rollupOptions: {
      input: path.resolve(__dirname, "./index.html"),
      output: {
        manualChunks: {
          maplibre_gl: ["maplibre-gl"],
          swagger_ui_dist: ["swagger-ui-dist"],
        },
      },
    },
  },
  plugins: [
    Vue({
      include: [/\.vue$/, /\.md$/],
    }),
    VueI18nPlugin({
      include: path.resolve(__dirname, "./src/locales/**"),
      fullInstall: false,
    }),
    Markdown({
      builders: [link()],
    }),
    //The next one is included due to https://github.com/vitejs/vite/issues/2415
    // otherwise the router won't serve the details pages, as they include dots
    pluginRewriteAll(),
  ],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
      vue: path.resolve(__dirname, "node_modules/vue/dist/vue.esm-bundler.js"),
    },
  },
});
