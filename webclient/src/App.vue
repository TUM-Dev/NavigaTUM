<script setup lang="ts">
import Footer from "@/components/AppFooter.vue";
import { useGlobalStore } from "@/stores/global";
import FeedbackModal from "@/components/feedback/FeedbackModal.vue";
import AppSearchBar from "@/components/AppSearchBar.vue";
import AppNavHeader from "@/components/AppNavHeader.vue";
import Toast from "@/components/Toast.vue";
import Modal from "@/components/Modal.vue";
import { ref } from "vue";

const global = useGlobalStore();
const modelOpen = ref(false);
</script>

<template>
  <AppNavHeader>
    <AppSearchBar />
  </AppNavHeader>

  <!-- Page content container -->
  <div
    class="mx-auto mt-16 min-h-[calc(100vh-200px)] max-w-4xl transition-opacity"
    :class="{ 'opacity-70': global.search_focused }"
  >
    <div id="errorToasts" class="mx-5 grid gap-2">
      <Toast v-if="global.error_message" :msg="global.error_message" level="error" />
    </div>
    <RouterView class="mx-5" />
  </div>
  <Modal v-model:open="modelOpen" title="awsome title goes here"> less awsome body </Modal>

  <Footer />
  <FeedbackModal v-if="global.feedback.open" />
</template>
