<script setup lang="ts">
import { useGlobalStore } from "@/stores/global";

const global = useGlobalStore();
const href = window.location.href;
</script>

<template>
  <div class="toast toast-error">{{ $t("core_js.error.404") }}</div>
  <h5 style="margin-top: 25px">{{ $t("view_404.header") }}</h5>
  <p>{{ $t("view_404.description") }}</p>
  <button
    @click="global.openFeedback('bug', `404 on ${href}`, $t('view_404.got_here'))"
    class="btn btn-link p-0"
    aria-label="Open the feedback-form"
  >
    {{ $t("view_404.call_to_action") }}
  </button>
</template>
