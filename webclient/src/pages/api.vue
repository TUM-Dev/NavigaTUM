<script setup lang="ts">
import "swagger-ui/dist/swagger-ui.css";
import { useDark } from "@vueuse/core";
// TODO: this is reaaaly hacky, but I have no idea how to
//  - convince vue to allow conditional css imports
//  - postcss to allow for imports under a selector
const dark = useDark({ storageKey: "theme" });
if (dark.value) {
  import("swaggerdark/SwaggerDark.css");
}
window.setTimeout(() => {
  // we need to make sure, that swagger-ui exists, otherwise the following command will fail
  // therefore waiting is effective
  import("swagger-ui").then((SwaggerUI) =>
    SwaggerUI({
      url: `${import.meta.env.VITE_APP_URL}/cdn/openapi.yaml`,
      dom_id: "#swagger-ui",
    }),
  );
}, 10);
</script>

<template>
  <div id="swagger-ui" class="pt-0.5" />
</template>

<style lang="postcss" scoped>
/* we cannot apply loading-lg to this external dependency */
.loading-container .loading {
  @apply min-h-8 after:-ml-2.5 after:-mt-3.5 after:h-6 after:w-6;
}
</style>
