<script setup lang="ts">
const searchBarFocused = ref(false);

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
  <AppNavHeader>
    <AppSearchBar v-model:search-bar-focused="searchBarFocused" />
  </AppNavHeader>
  <!-- Page content container -->
  <main :class="{ 'opacity-70': searchBarFocused }">
    <slot />
  </main>
</template>
