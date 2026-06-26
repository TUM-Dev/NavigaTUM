<script setup lang="ts">
// Renders inline emphasis from `**…**` markers as real <strong> text nodes (never v-html), so
// translators can mark the words that matter inside a single i18n string. Odd split segments are
// the emphasized runs; unbalanced markers degrade to plain text rather than breaking.
const props = defineProps<{ text: string }>();

const segments = computed(() =>
  props.text.split("**").map((part, index) => ({ text: part, bold: index % 2 === 1 }))
);
</script>

<template>
  <span>
    <template v-for="(segment, index) in segments" :key="index">
      <strong v-if="segment.bold" class="text-zinc-800 dark:text-zinc-100 font-semibold">{{ segment.text }}</strong>
      <template v-else>{{ segment.text }}</template>
    </template>
  </span>
</template>
