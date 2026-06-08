<script setup lang="ts">
// Shared styling for one button in a segmented control. Designed to sit inside a
// Headless UI `<Tab as="template">`, which merges its ARIA and keyboard props onto
// the button via attribute fall-through. Only the box sizing (width and padding)
// varies between controls, so callers pass that as a `class` (e.g. `w-full px-3
// py-2`); the ring and selected/unselected styling is kept identical across the
// app so the controls never drift apart.
defineProps<{ selected: boolean }>();
</script>

<template>
  <button
    type="button"
    :class="[
      'rounded-md text-sm font-medium leading-5 transition-all',
      'ring-white/60 dark:ring-black/60 ring-offset-2 ring-offset-blue-400 dark:ring-offset-blue-500 focus:outline-none focus:ring-2',
      selected
        ? 'bg-white dark:bg-black text-zinc-700 dark:text-zinc-200 shadow'
        : 'text-zinc-500 dark:text-zinc-400 hover:bg-white/[0.12] dark:hover:bg-black/[0.12] hover:text-zinc-700 dark:hover:text-zinc-200',
    ]"
  >
    <slot />
  </button>
</template>
