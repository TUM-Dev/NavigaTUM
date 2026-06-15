<script setup lang="ts">
import { useEditProposal } from "~/composables/editProposal";
import { useFeedback } from "~/composables/feedback";

const feedback = useFeedback();
const editProposal = useEditProposal();

const i18nHead = useLocaleHead({ dir: true, seo: true });
useHead({
  htmlAttrs: {
    lang: i18nHead.value.htmlAttrs?.lang || "en",
  },
  link: [...(i18nHead.value.link || [])],
  meta: [...(i18nHead.value.meta || [])],
});

const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <a
    href="#main-content"
    class="sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-4 focus:z-30 focus:rounded focus:bg-blue-600 dark:focus:bg-blue-300 focus:px-4 focus:py-2 focus:text-white dark:focus:text-black focus:shadow-md focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400 focus:ring-offset-2"
  >
    {{ t("skip_to_content") }}
  </a>
  <!-- Full-viewport navigation view: no global header or footer. -->
  <main id="main-content" tabindex="-1" class="h-[100dvh] overflow-hidden">
    <slot />
  </main>

  <ClientOnly>
    <LazyFeedbackModal v-if="feedback.open" />
    <LazyAddProposalModal v-if="editProposal.addOpen" />
  </ClientOnly>
</template>

<i18n lang="yaml">
de:
  skip_to_content: Zum Hauptinhalt springen
en:
  skip_to_content: Skip to main content
</i18n>
