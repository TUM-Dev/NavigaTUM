<script setup lang="ts">
import { useFeedback } from "~/composables/feedback";
import ManyChangesToast from "~/components/ManyChangesToast.vue";

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
    class="mx-auto mt-16 min-h-[calc(100vh-400px)] max-w-4xl pb-10 transition-opacity"
    :class="{ 'opacity-70': searchBarFocused }"
  >
    <div class="mx-5">
      <div class="-mb-1 flex flex-col gap-4 pt-5">
        <ManyChangesToast />
      </div>
      <slot />
    </div>
  </div>

  <AppFooter :class="searchBarFocused ? 'opacity-70' : ''" />
  <ClientOnly>
    <FeedbackModal v-if="feedback.open" />
  </ClientOnly>
</template>
