<script setup lang="ts">
const runtimeConfig = useRuntimeConfig();
</script>

<template>
  <ClientOnly>
    <SwaggerUI :url="`${runtimeConfig.public.apiURL}/cdn/openapi.yaml`" />
  </ClientOnly>
</template>
