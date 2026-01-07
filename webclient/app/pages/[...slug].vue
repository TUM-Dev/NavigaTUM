<script setup lang="ts">
const route = useRoute();

// Set proper 404 status code for catch-all routes
if (import.meta.server) {
  const event = useRequestEvent();
  if (event) {
    setResponseStatus(event, 404);
  }
}
</script>

<template>
  <NotFound :path="route.fullPath" />
</template>
