<script setup lang="ts">
import { useGlobalStore } from "@/stores/global";
import { useRoute } from "vue-router";

const route = useRoute();
const global = useGlobalStore();
</script>

<template>
  <div class="toast toast-error">{{ $t("core_js.error.404") }}</div>
  <h5 style="margin-top: 25px">{{ $t("view_404.header") }}</h5>
  <p>{{ $t("view_404.description") }}</p>
  <button
    @click="global.openFeedback('bug', `404 on ${route.fullPath}`, $t('view_404.got_here'))"
    class="btn btn-link p-0"
    :aria-label="$t('view_404.open_feedback')"
  >
    {{ $t("view_404.call_to_action") }}
  </button>
</template>
