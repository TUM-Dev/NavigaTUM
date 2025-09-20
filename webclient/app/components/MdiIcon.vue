<script setup lang="ts">
interface Props {
  path: string;
  size?: number;
}

const props = withDefaults(defineProps<Props>(), {
  size: 24
});
</script>

<template>
  <svg
    :width="props.size"
    :height="props.size"
    viewBox="0 0 24 24"
    class="inline-block"
  >
    <path fill="currentColor" :d="props.path" />
  </svg>
</template>
