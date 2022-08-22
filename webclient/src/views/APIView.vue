<script setup lang="ts">
import TheWelcome from "@/components/TheWelcome.vue";

const head = document.getElementsByTagName("head")[0];
  // Add CSS first (required by swagger-ui)
  const elCSS = document.createElement("link");
  elCSS.rel = "stylesheet";
  elCSS.href = "css/swagger-ui.min.css";
  head.appendChild(elCSS);

  // JS should trigger init on load
  const elJS = document.createElement("script");
  elJS.src = "js/swagger-ui.min.js";
  elJS.onload = () => {
    window.setTimeout(() => {
      // we need to make sure, that swagger-ui exists, otherwise the following command will fail
      // therefore waiting is effective
      /* global SwaggerUIBundle */
      SwaggerUIBundle({
        url: "https://raw.githubusercontent.com/TUM-Dev/navigatum/main/openapi.yaml",
        dom_id: "#swagger-ui",
        presets: [
          SwaggerUIBundle.presets.apis,
        ],
      });
    }, 10);
  };
  head.appendChild(elJS);
</script>

<style lang="scss">
@import "src/assets/variables";

.swagger-ui {
    // we cannot apply loading-lg to this external dependency
    .loading-container .loading {
        min-height: 2rem;
    }

    .loading-container .loading::after {
        height: 1.6rem;
        margin-left: -.8rem;
        margin-top: -.8rem;
        width: 1.6rem;
    }
}
</style>

<template>
  <div id="swagger-ui"></div>
</template>
