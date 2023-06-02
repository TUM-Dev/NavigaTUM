import { fileURLToPath, URL } from "node:url";

import { defineConfig } from "vite";
import Vue from "@vitejs/plugin-vue";
import VueI18nPlugin from "@intlify/unplugin-vue-i18n/vite";
import Markdown from "vite-plugin-md";
import link from "@yankeeinlondon/link-builder";
import path from "path";
import pluginRewriteAll from "vite-plugin-rewrite-all";
import { sentryVitePlugin } from "@sentry/vite-plugin";

// https://vitejs.dev/config/
export default defineConfig({
  envDir: path.resolve(__dirname, "./env"),
  appType: "spa",
  server: {
    port: 8000,
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
    sentryVitePlugin({
      org: "rbg",
      project: "navigatum",
      authToken: "3279def15c0543e797ec3550b0273fbf58e4eb2e67e64a5ba5474bd83d5fa149",
      url: "https://sentry.mm.rbg.tum.de/",
      release: {
        name: process.env.GIT_COMMIT_MESSAGE || "development",
        //setCommits: {
        //  repo: "TUM-Dev/NavigaTUM",
        //  commit: process.env.GIT_COMMIT_SHA || "development",
        //  auto: false,
        //},
        deploy: {
          env: process.env.GIT_COMMIT_SHA ? "production" : "staging",
          started: new Date().getTime(),
          url: "https://nav.tum.de",
        },
      },
    }),
  ],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
      vue: path.resolve(__dirname, "node_modules/vue/dist/vue.esm-bundler.js"),
    },
  },
});
