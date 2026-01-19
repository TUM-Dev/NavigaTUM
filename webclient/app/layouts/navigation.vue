<script setup lang="ts">
import { useFeedback } from "~/composables/feedback";

const feedback = useFeedback();

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
  <AppNavHeader />

  <!-- Page content container -->
  <main class="mx-auto mt-[60px] min-h-[calc(100vh-150px)] transition-opacity">
    <slot />
  </main>

  <AppFooter />
  <ClientOnly>
    <LazyFeedbackModal v-if="feedback.open" />
  </ClientOnly>
</template>
