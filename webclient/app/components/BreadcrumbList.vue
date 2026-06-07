<script setup lang="ts">
// `to` is optional: an ancestor without a canonical entity route renders as plain text.
type Item = { to?: string; name: string };
const props = withDefaults(defineProps<{ items: Item[]; class?: string }>(), {
  class: "",
});
</script>

<template>
  <ol
    vocab="https://schema.org/"
    typeof="BreadcrumbList"
    class="flex flex-row flex-wrap gap-2 text-sm"
    :class="props.class"
  >
    <template v-for="(item, i) in props.items" :key="i">
      <span v-if="i > 0" aria-hidden="true" class="text-zinc-500 dark:text-zinc-400">/</span>
      <li property="itemListElement" typeof="ListItem">
        <NuxtLinkLocale
          v-if="item.to"
          :to="item.to"
          property="item"
          typeof="WebPage"
          class="focusable rounded-sm hover:underline"
          :class="{
            'visited:text-blue-500 dark:visited:text-blue-400': i > 0,
            'visited:text-zinc-500 dark:visited:text-zinc-400': i === 0,
            'hover:text-blue-600 dark:hover:text-blue-300': i > 0,
            'hover:text-zinc-600 dark:hover:text-zinc-300': i === 0,
            'text-blue-500 dark:text-blue-400': i > 0,
            'text-zinc-500 dark:text-zinc-400': i === 0,
          }"
        >
          <span property="name">{{ item.name }}</span>
        </NuxtLinkLocale>
        <span v-else property="name" class="text-zinc-500 dark:text-zinc-400">{{ item.name }}</span>
        <meta property="position" :content="`${i + 1}`" />
      </li>
    </template>
  </ol>
</template>
