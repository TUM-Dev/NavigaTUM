<script setup lang="ts">
const props = defineProps<{ image: string; name: string }>();

// Fall back to the glyph when the image is absent or fails to load.
const failed = ref(!props.image);
</script>

<template>
  <div
    class="box-border size-10 cursor-pointer overflow-hidden rounded-full border-2 border-blue-500 bg-blue-500 shadow-md ring-2 ring-white"
    :class="{ 'flex items-center justify-center text-white': failed }"
  >
    <!-- [filter:none] keeps the photo from being inverted in dark mode. -->
    <img
      v-if="!failed"
      :src="image"
      :alt="name"
      :draggable="false"
      loading="lazy"
      decoding="async"
      class="block size-full rounded-full object-cover [filter:none]"
      @error="failed = true"
    />
    <svg
      v-else
      class="size-1/2"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <rect x="3" y="4" width="18" height="18" rx="2" />
      <path d="M16 2v4M8 2v4M3 10h18" />
    </svg>
  </div>
</template>
