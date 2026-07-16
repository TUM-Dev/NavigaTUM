<script setup lang="ts">
// `grid-template-rows: 0fr→1fr` animates to auto height; the child needs `min-h-0` to shrink.
defineProps<{ open: boolean }>();
</script>

<template>
  <div
    class="grid transition-[grid-template-rows] duration-200 ease-out motion-reduce:transition-none"
    :class="open ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
  >
    <div class="min-h-0 overflow-hidden">
      <slot />
    </div>
  </div>
</template>
