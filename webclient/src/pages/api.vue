<script setup lang="ts">
import "swagger-ui-dist/swagger-ui.css";

window.setTimeout(() => {
  // we need to make sure, that swagger-ui exists, otherwise the following command will fail
  // therefore waiting is effective
  import("swagger-ui-dist").then(({ SwaggerUIBundle }) =>
    SwaggerUIBundle({
      url: `${import.meta.env.VITE_APP_URL}/cdn/openapi.yaml`,
      dom_id: "#swagger-ui",
      presets: [SwaggerUIBundle.presets.apis],
    }),
  );
}, 10);
</script>

<template>
  <div id="swagger-ui" />
</template>

<style lang="postcss" scoped>
/* we cannot apply loading-lg to this external dependency */
.loading-container .loading {
  @apply min-h-8 after:-ml-2.5 after:-mt-3.5 after:h-6 after:w-6;
}
</style>
