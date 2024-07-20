<script setup lang="ts">
import { useFeedback } from "~/composables/feedback";

const searchBarFocused = ref(false);
const feedback = useFeedback();

const i18nHead = useLocaleHead({
  addDirAttribute: true,
  addSeoAttributes: true,
});
useHead({
  htmlAttrs: {
    lang: i18nHead.value.htmlAttrs!.lang,
  },
  link: [...(i18nHead.value.link || [])],
  meta: [...(i18nHead.value.meta || [])],
});
</script>

<template>
  <AppNavHeader>
    <AppSearchBar v-model:searchBarFocused="searchBarFocused" />
  </AppNavHeader>

  <!-- Page content container -->
  <div
    class="mx-auto mt-16 min-h-[calc(100vh-360px)] max-w-4xl pb-10 transition-opacity"
    :class="{ 'opacity-70': searchBarFocused }"
  >
    <div class="mx-5">
      <slot />
    </div>
  </div>

  <AppFooter :class="searchBarFocused ? 'opacity-70' : ''" />
  <ClientOnly>
    <LazyFeedbackModal v-if="feedback.open" />
  </ClientOnly>
</template>
