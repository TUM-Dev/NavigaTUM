<script setup lang="ts">
import "swagger-ui-dist/swagger-ui.css";

window.setTimeout(() => {
  // we need to make sure, that swagger-ui exists, otherwise the following command will fail
  // therefore waiting is effective
  import("swagger-ui-dist").then(({ SwaggerUIBundle }) =>
    SwaggerUIBundle({
      url: "/cdn/openapi.yaml",
      dom_id: "#swagger-ui",
      presets: [SwaggerUIBundle.presets.apis],
    })
  );
}, 10);
</script>

<template>
  <div id="swagger-ui"></div>
</template>

<style lang="scss">
@import "../assets/variables";

.swagger-ui {
  // we cannot apply loading-lg to this external dependency
  .loading-container .loading {
    min-height: 2rem;
  }

  .loading-container .loading::after {
    height: 1.6rem;
    margin-left: -0.8rem;
    margin-top: -0.8rem;
    width: 1.6rem;
  }
}
</style>
