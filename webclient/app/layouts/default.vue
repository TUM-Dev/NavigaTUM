<script setup lang="ts">
import { useFeedback } from "~/composables/feedback";

const searchBarFocused = ref(false);
const feedback = useFeedback();
const route = useRoute();

function handleKeyDown(event: KeyboardEvent) {
  const isIndexPage = route.path === "/" || route.path === "/en";
  
  // Only handle on index page, when feedback is closed and search bar is not focused
  if (!isIndexPage || feedback.value.open || searchBarFocused.value) {
    return;
  }
  
  // Ignore special keys (Ctrl, Alt, Meta, Shift, Escape, Tab, etc.)
  if (event.ctrlKey || event.altKey || event.metaKey || event.shiftKey) {
    return;
  }
  
  // Ignore non-printable keys
  const ignoredKeys = ['Escape', 'Tab', 'Enter', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End', 'PageUp', 'PageDown', 'Delete', 'Backspace'];
  if (ignoredKeys.includes(event.key)) {
    return;
  }
  
  // Focus the search bar when user starts typing
  if (event.key.length === 1) {
    document.getElementById("search")?.focus();
  }
}

onMounted(() => {
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
</script>

<template>
  <AppNavHeader>
    <AppSearchBar v-model:search-bar-focused="searchBarFocused" />
  </AppNavHeader>

  <!-- Page content container -->
  <div
    class="mx-auto mt-16 min-h-[calc(100vh-360px)] max-w-4xl pb-10 transition-opacity"
    :class="{ 'opacity-70': searchBarFocused }"
  >
    <main class="mx-5">
      <slot />
    </main>
  </div>

  <AppFooter :class="searchBarFocused ? 'opacity-70' : ''" />
  <ClientOnly>
    <LazyFeedbackModal v-if="feedback.open" />
  </ClientOnly>
</template>
