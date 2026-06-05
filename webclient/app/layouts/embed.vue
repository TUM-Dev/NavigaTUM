<script setup lang="ts">
const i18nHead = useLocaleHead({ dir: true, seo: true });
useHead({
  htmlAttrs: {
    lang: i18nHead.value.htmlAttrs?.lang || "en",
  },
  link: [...(i18nHead.value.link || [])],
  meta: [...(i18nHead.value.meta || [])],
});
</script>

<template>
  <main class="h-screen w-screen overflow-hidden bg-zinc-50 dark:bg-zinc-900">
    <slot />
  </main>
</template>
