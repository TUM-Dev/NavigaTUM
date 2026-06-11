<script setup lang="ts">
import { mdiMapOutline } from "@mdi/js";

// Shared inline-pill treatment of the Browse-map bridges (see `webclient/CONTEXT.md`).
defineProps<{
  readonly to: string;
}>();
</script>

<template>
  <Btn :to="to" variant="link" size="text-xs gap-1 rounded">
    <MdiIcon :path="mdiMapOutline" :size="14" class="my-auto" aria-hidden="true" />
    <slot />
  </Btn>
</template>
