<script setup lang="ts">
import Footer from "@/components/AppFooter.vue";
import { useGlobalStore } from "@/stores/global";
import FeedbackModal from "@/components/AppFeedbackModal.vue";
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
  <!-- General message modal -->
  <div class="modal active" v-if="global.information_modal?.body">
    <a class="modal-overlay" :aria-label="$t('close')" @click="global.information_modal.body = null" />
    <div class="modal-container">
      <div class="modal-header">
        <button
          class="btn btn-clear float-right"
          :aria-label="$t('close')"
          @click="global.information_modal.body = null"
        />
        <div v-if="global.information_modal.header" class="modal-title h5">{{ global.information_modal.header }}</div>
      </div>
      <div class="modal-body">
        <div class="content">
          <p>{{ global.information_modal.body }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
@import "./assets/variables";

/* === Content === */
#content {
  min-height: calc(100vh - 200px);
}

// 10px + 60px for header
#content-header {
  margin-top: 70px;
}

#content.visible {
  /* For some reason (I assume because the 'visible' class is not set when vue loads),
     * this class gets removed if vue adds/removes the 'search_focus' class. For this reason
     * opacity on page navigation is set as style property in JS. It is only guaranteed that
     * this class is there on page-load. */
  transition: opacity 0.07s;
}

#content.search_focus {
  opacity: 0.7;
}
</style>
