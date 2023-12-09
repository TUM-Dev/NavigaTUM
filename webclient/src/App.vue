<script setup lang="ts">
import Footer from "@/components/AppFooter.vue";
import { useGlobalStore } from "@/stores/global";
import FeedbackModal from "@/components/feedback/FeedbackModal.vue";
import AppSearchBar from "@/components/AppSearchBar.vue";
import AppNavHeader from "@/components/AppNavHeader.vue";

const global = useGlobalStore();
</script>

<template>
  <AppNavHeader>
    <AppSearchBar />
  </AppNavHeader>

  <!-- General error message toast -->
  <div v-cloak class="max-w-3xl mt-16">
    <div v-if="global.error_message" class="toast toast-error">
      {{ global.error_message }}
    </div>
  </div>

  <!-- Page content container -->
  <div class="max-w-3xl min-h-[calc(100vh-200px)] mx-auto transition-opacity" :class="{ 'opacity-70': global.search_focused }">
    <RouterView />
  </div>

  <Footer />
  <FeedbackModal v-if="global.feedback.open" />
</template>
