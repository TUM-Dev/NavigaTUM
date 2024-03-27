// vite.config.ts
import { defineConfig } from "file:///workspaces/NavigaTUM/webclient/node_modules/.pnpm/vite@5.2.3_@types+node@20.11.30_sass@1.72.0/node_modules/vite/dist/node/index.js";
import Vue from "file:///workspaces/NavigaTUM/webclient/node_modules/.pnpm/@vitejs+plugin-vue@5.0.4_vite@5.2.3_vue@3.4.21/node_modules/@vitejs/plugin-vue/dist/index.mjs";
import VueI18nPlugin from "file:///workspaces/NavigaTUM/webclient/node_modules/.pnpm/@intlify+unplugin-vue-i18n@4.0.0_vue-i18n@9.10.2/node_modules/@intlify/unplugin-vue-i18n/lib/vite.mjs";
import Markdown from "file:///workspaces/NavigaTUM/webclient/node_modules/.pnpm/unplugin-vue-markdown@0.26.0_vite@5.2.3/node_modules/unplugin-vue-markdown/dist/vite.js";
import prism from "file:///workspaces/NavigaTUM/webclient/node_modules/.pnpm/markdown-it-prism@2.3.0/node_modules/markdown-it-prism/build/index.js";
import path from "path";
var vite_config_default = defineConfig({
  envDir: "env",
  appType: "spa",
  server: {
    port: 3e3,
    strictPort: true,
    open: false,
    proxy: {
      "^/api/[cf].*": {
        target: "https://nav.tum.de"
      },
      "^/api/[^cf].*": {
        target: "http://127.0.0.1:3003",
        secure: false
      },
      "/cdn": {
        target: "https://nav.tum.de"
      }
    }
  },
  build: {
    sourcemap: true,
    rollupOptions: {
      input: path.resolve("./index.html"),
      output: {
        manualChunks: {
          maplibre_gl: ["maplibre-gl"],
          swagger_ui: ["swagger-ui"]
        }
      }
    }
  },
  plugins: [
    Vue({
      include: [/\.vue$/, /\.md$/]
    }),
    VueI18nPlugin({
      include: path.resolve("./locales/**"),
      fullInstall: false
    }),
    Markdown({
      markdownItUses: [prism]
    })
    // currently the devtools fucks MAJORLY with the darkmode with no way of configuring it
    // (darkmode will get randomly enabled/disabled)
    // VueDevTools(),
  ],
  resolve: {
    alias: {
      vue: path.resolve("node_modules/vue/dist/vue.esm-bundler.js")
    }
  }
});
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcudHMiXSwKICAic291cmNlc0NvbnRlbnQiOiBbImNvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9kaXJuYW1lID0gXCIvd29ya3NwYWNlcy9OYXZpZ2FUVU0vd2ViY2xpZW50XCI7Y29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2ZpbGVuYW1lID0gXCIvd29ya3NwYWNlcy9OYXZpZ2FUVU0vd2ViY2xpZW50L3ZpdGUuY29uZmlnLnRzXCI7Y29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2ltcG9ydF9tZXRhX3VybCA9IFwiZmlsZTovLy93b3Jrc3BhY2VzL05hdmlnYVRVTS93ZWJjbGllbnQvdml0ZS5jb25maWcudHNcIjtpbXBvcnQgeyBkZWZpbmVDb25maWcgfSBmcm9tIFwidml0ZVwiO1xuaW1wb3J0IFZ1ZSBmcm9tIFwiQHZpdGVqcy9wbHVnaW4tdnVlXCI7XG5pbXBvcnQgVnVlSTE4blBsdWdpbiBmcm9tIFwiQGludGxpZnkvdW5wbHVnaW4tdnVlLWkxOG4vdml0ZVwiO1xuaW1wb3J0IE1hcmtkb3duIGZyb20gXCJ1bnBsdWdpbi12dWUtbWFya2Rvd24vdml0ZVwiO1xuaW1wb3J0IHByaXNtIGZyb20gXCJtYXJrZG93bi1pdC1wcmlzbVwiO1xuaW1wb3J0IHBhdGggZnJvbSBcInBhdGhcIjtcbi8vaW1wb3J0IFZ1ZURldlRvb2xzIGZyb20gXCJ2aXRlLXBsdWdpbi12dWUtZGV2dG9vbHNcIjtcblxuZXhwb3J0IGRlZmF1bHQgZGVmaW5lQ29uZmlnKHtcbiAgZW52RGlyOiBcImVudlwiLFxuICBhcHBUeXBlOiBcInNwYVwiLFxuICBzZXJ2ZXI6IHtcbiAgICBwb3J0OiAzMDAwLFxuICAgIHN0cmljdFBvcnQ6IHRydWUsXG4gICAgb3BlbjogZmFsc2UsXG4gICAgcHJveHk6IHtcbiAgICAgIFwiXi9hcGkvW2NmXS4qXCI6IHtcbiAgICAgICAgdGFyZ2V0OiBcImh0dHBzOi8vbmF2LnR1bS5kZVwiLFxuICAgICAgfSxcbiAgICAgIFwiXi9hcGkvW15jZl0uKlwiOiB7XG4gICAgICAgIHRhcmdldDogXCJodHRwOi8vMTI3LjAuMC4xOjMwMDNcIixcbiAgICAgICAgc2VjdXJlOiBmYWxzZSxcbiAgICAgIH0sXG4gICAgICBcIi9jZG5cIjoge1xuICAgICAgICB0YXJnZXQ6IFwiaHR0cHM6Ly9uYXYudHVtLmRlXCIsXG4gICAgICB9LFxuICAgIH0sXG4gIH0sXG4gIGJ1aWxkOiB7XG4gICAgc291cmNlbWFwOiB0cnVlLFxuICAgIHJvbGx1cE9wdGlvbnM6IHtcbiAgICAgIGlucHV0OiBwYXRoLnJlc29sdmUoXCIuL2luZGV4Lmh0bWxcIiksXG4gICAgICBvdXRwdXQ6IHtcbiAgICAgICAgbWFudWFsQ2h1bmtzOiB7XG4gICAgICAgICAgbWFwbGlicmVfZ2w6IFtcIm1hcGxpYnJlLWdsXCJdLFxuICAgICAgICAgIHN3YWdnZXJfdWk6IFtcInN3YWdnZXItdWlcIl0sXG4gICAgICAgIH0sXG4gICAgICB9LFxuICAgIH0sXG4gIH0sXG4gIHBsdWdpbnM6IFtcbiAgICBWdWUoe1xuICAgICAgaW5jbHVkZTogWy9cXC52dWUkLywgL1xcLm1kJC9dLFxuICAgIH0pLFxuICAgIFZ1ZUkxOG5QbHVnaW4oe1xuICAgICAgaW5jbHVkZTogcGF0aC5yZXNvbHZlKFwiLi9sb2NhbGVzLyoqXCIpLFxuICAgICAgZnVsbEluc3RhbGw6IGZhbHNlLFxuICAgIH0pLFxuICAgIE1hcmtkb3duKHtcbiAgICAgIG1hcmtkb3duSXRVc2VzOiBbcHJpc21dLFxuICAgIH0pLFxuICAgIC8vIGN1cnJlbnRseSB0aGUgZGV2dG9vbHMgZnVja3MgTUFKT1JMWSB3aXRoIHRoZSBkYXJrbW9kZSB3aXRoIG5vIHdheSBvZiBjb25maWd1cmluZyBpdFxuICAgIC8vIChkYXJrbW9kZSB3aWxsIGdldCByYW5kb21seSBlbmFibGVkL2Rpc2FibGVkKVxuICAgIC8vIFZ1ZURldlRvb2xzKCksXG4gIF0sXG4gIHJlc29sdmU6IHtcbiAgICBhbGlhczoge1xuICAgICAgdnVlOiBwYXRoLnJlc29sdmUoXCJub2RlX21vZHVsZXMvdnVlL2Rpc3QvdnVlLmVzbS1idW5kbGVyLmpzXCIpLFxuICAgIH0sXG4gIH0sXG59KTtcbiJdLAogICJtYXBwaW5ncyI6ICI7QUFBK1EsU0FBUyxvQkFBb0I7QUFDNVMsT0FBTyxTQUFTO0FBQ2hCLE9BQU8sbUJBQW1CO0FBQzFCLE9BQU8sY0FBYztBQUNyQixPQUFPLFdBQVc7QUFDbEIsT0FBTyxVQUFVO0FBR2pCLElBQU8sc0JBQVEsYUFBYTtBQUFBLEVBQzFCLFFBQVE7QUFBQSxFQUNSLFNBQVM7QUFBQSxFQUNULFFBQVE7QUFBQSxJQUNOLE1BQU07QUFBQSxJQUNOLFlBQVk7QUFBQSxJQUNaLE1BQU07QUFBQSxJQUNOLE9BQU87QUFBQSxNQUNMLGdCQUFnQjtBQUFBLFFBQ2QsUUFBUTtBQUFBLE1BQ1Y7QUFBQSxNQUNBLGlCQUFpQjtBQUFBLFFBQ2YsUUFBUTtBQUFBLFFBQ1IsUUFBUTtBQUFBLE1BQ1Y7QUFBQSxNQUNBLFFBQVE7QUFBQSxRQUNOLFFBQVE7QUFBQSxNQUNWO0FBQUEsSUFDRjtBQUFBLEVBQ0Y7QUFBQSxFQUNBLE9BQU87QUFBQSxJQUNMLFdBQVc7QUFBQSxJQUNYLGVBQWU7QUFBQSxNQUNiLE9BQU8sS0FBSyxRQUFRLGNBQWM7QUFBQSxNQUNsQyxRQUFRO0FBQUEsUUFDTixjQUFjO0FBQUEsVUFDWixhQUFhLENBQUMsYUFBYTtBQUFBLFVBQzNCLFlBQVksQ0FBQyxZQUFZO0FBQUEsUUFDM0I7QUFBQSxNQUNGO0FBQUEsSUFDRjtBQUFBLEVBQ0Y7QUFBQSxFQUNBLFNBQVM7QUFBQSxJQUNQLElBQUk7QUFBQSxNQUNGLFNBQVMsQ0FBQyxVQUFVLE9BQU87QUFBQSxJQUM3QixDQUFDO0FBQUEsSUFDRCxjQUFjO0FBQUEsTUFDWixTQUFTLEtBQUssUUFBUSxjQUFjO0FBQUEsTUFDcEMsYUFBYTtBQUFBLElBQ2YsQ0FBQztBQUFBLElBQ0QsU0FBUztBQUFBLE1BQ1AsZ0JBQWdCLENBQUMsS0FBSztBQUFBLElBQ3hCLENBQUM7QUFBQTtBQUFBO0FBQUE7QUFBQSxFQUlIO0FBQUEsRUFDQSxTQUFTO0FBQUEsSUFDUCxPQUFPO0FBQUEsTUFDTCxLQUFLLEtBQUssUUFBUSwwQ0FBMEM7QUFBQSxJQUM5RDtBQUFBLEVBQ0Y7QUFDRixDQUFDOyIsCiAgIm5hbWVzIjogW10KfQo=
