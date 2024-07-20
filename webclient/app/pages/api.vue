<script setup lang="ts">
const runtimeConfig = useRuntimeConfig();
</script>

<template>
  <ClientOnly>
    <SwaggerUI :url="`${runtimeConfig.public.cdnURL}/cdn/openapi.yaml`" />
  </ClientOnly>
</template>
