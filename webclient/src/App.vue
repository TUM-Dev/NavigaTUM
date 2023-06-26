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
  <div id="content-header" class="container grid-lg" v-cloak>
    <div class="columns">
      <div class="column col-lg-11 col-mx-auto">
        <div class="toast toast-error" v-if="global.error_message">
          {{ global.error_message }}
        </div>
      </div>
    </div>
  </div>

  <!-- Page content container -->
  <div id="content" class="container grid-lg visible" :class="{ search_focus: global.search_focused }">
    <div class="columns">
      <div class="column col-lg-11 col-mx-auto">
        <RouterView />
      </div>
    </div>
  </div>
  <!-- Loading indicator -->
  <div id="loading-page" v-cloak>
    <div class="loading loading-lg" />
  </div>

  <Footer />
  <FeedbackModal v-if="global.feedback.open" />
</template>

<style lang="scss">
@import "@/assets/variables";

// 10px + 60px for header
#content-header {
  margin-top: 70px;
}

#content {
  min-height: calc(100vh - 200px);
  &.visible {
    /* For some reason (I assume because the 'visible' class is not set when vue loads),
     * this class gets removed if vue adds/removes the 'search_focus' class. For this reason
     * opacity on page navigation is set as style property in JS. It is only guaranteed that
     * this class is there on page-load. */
    transition: opacity 0.07s;
  }

  &.search_focus {
    opacity: 0.7;
  }
}
</style>
