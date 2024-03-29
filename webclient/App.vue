<script setup lang="ts">
import AppFooter from "./components/AppFooter.vue";
import { useGlobalStore } from "./stores/global";
import FeedbackModal from "./components/feedback/FeedbackModal.vue";
import AppSearchBar from "./components/AppSearchBar.vue";
import AppNavHeader from "./components/AppNavHeader.vue";
import { ref } from "vue";

const global = useGlobalStore();
const searchBarFocused = ref(false);
</script>

<template>
  <AppNavHeader>
    <AppSearchBar v-model:searchBarFocused="searchBarFocused" />
  </AppNavHeader>

  <!-- Page content container -->
  <div
    class="mx-auto mb-16 mt-16 min-h-[calc(100vh-400px)] max-w-4xl transition-opacity"
    :class="{ 'opacity-70': searchBarFocused }"
  >
    <div class="mx-5">
      <RouterView />
    </div>
  </div>

  <AppFooter />
  <FeedbackModal v-if="global.feedback.open" />
</template>
