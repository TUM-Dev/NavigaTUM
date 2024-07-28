<script setup lang="ts">
import "swagger-ui/dist/swagger-ui.css";

const props = defineProps<{ class?: string; url: string }>();

// TODO: this is reaaaly hacky, but I have no idea how to
//  - convince vue to allow conditional css imports
//  - postcss to allow for imports under a selector
const colorMode = useColorMode();
if (colorMode.value === "dark") {
  //@ts-expect-error swaggerdark/SwaggerDark.css does not have types
  import("swaggerdark/SwaggerDark.css");
}
setTimeout(async () => {
  // we need to make sure, that swagger-ui exists, otherwise the following command will fail
  // therefore waiting is effective
  const SwaggerUI = await import("swagger-ui");
  SwaggerUI.default({
    url: props.url,
    dom_id: "#swagger-ui",
  });
}, 10);
</script>

<template>
  <div id="swagger-ui" :class="props.class" />
</template>

<style lang="postcss" scoped>
/* we cannot apply loading-lg to this external dependency */
.loading-container .loading {
  @apply min-h-8 after:-ml-2.5 after:-mt-3.5 after:h-6 after:w-6;
}
</style>
