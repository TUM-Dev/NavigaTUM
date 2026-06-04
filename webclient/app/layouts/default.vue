<script setup lang="ts">
import { useEditProposal } from "~/composables/editProposal";
import { useFeedback } from "~/composables/feedback";

const searchBarFocused = ref(false);
const feedback = useFeedback();
const editProposal = useEditProposal();
const route = useRoute();
const searchElement = ref<HTMLElement | null>(null);

const IGNORED_KEYS = new Set([
  "Escape",
  "Tab",
  "Enter",
  "ArrowUp",
  "ArrowDown",
  "ArrowLeft",
  "ArrowRight",
  "Home",
  "End",
  "PageUp",
  "PageDown",
  "Delete",
  "Backspace",
]);

function handleKeyDown(event: KeyboardEvent) {
  const isIndexPage = route.path === "/" || route.path === "/en";

  // Only handle on index page, when feedback is closed and search bar is not focused
  if (!isIndexPage || feedback.value.open || searchBarFocused.value) {
    return;
  }

  // Ignore special keys (Ctrl, Alt, Meta - but allow Shift for uppercase/symbols)
  if (event.ctrlKey || event.altKey || event.metaKey) {
    return;
  }

  // Ignore non-printable keys
  if (IGNORED_KEYS.has(event.key)) {
    return;
  }

  // Focus the search bar when user starts typing
  if (event.key.length === 1 && searchElement.value) {
    searchElement.value.focus();
  }
}

onMounted(() => {
  searchElement.value = document.getElementById("search");
  document.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeyDown);
});

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
  <AppNavHeader>
    <AppSearchBar v-model:search-bar-focused="searchBarFocused" />
  </AppNavHeader>

  <!-- Page content container -->
  <div
    class="mx-auto mt-16 min-h-[calc(100vh-360px)] max-w-4xl pb-10 transition-opacity"
    :class="{ 'opacity-70': searchBarFocused }"
  >
    <main id="main-content" class="mx-5" tabindex="-1">
      <slot />
    </main>
  </div>

  <AppFooter :class="searchBarFocused ? 'opacity-70' : ''" />
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
