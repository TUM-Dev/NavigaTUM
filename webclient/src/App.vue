<script setup lang="ts">
import Footer from "@/components/AppFooter.vue";
import { useGlobalStore } from "@/stores/global";
import FeedbackModal from "@/components/feedback/FeedbackModal.vue";
import AppSearchBar from "@/components/AppSearchBar.vue";
import AppNavHeader from "@/components/AppNavHeader.vue";
import Toast from "@/components/Toast.vue";

const global = useGlobalStore();
</script>

<template>
  <AppNavHeader>
    <AppSearchBar />
  </AppNavHeader>

  <!-- Page content container -->
  <div
    id="pageContent"
    class="mx-auto mb-16 mt-16 min-h-[calc(100vh-400px)] max-w-4xl transition-opacity"
    :class="{ 'opacity-70': global.search_focused }"
  >
    <div v-if="global.error_message" class="mx-5 pt-5">
      <Toast :msg="global.error_message" level="error" />
    </div>
    <RouterView class="mx-5" />
  </div>

  <Footer />
  <FeedbackModal v-if="global.feedback.open" />
</template>
