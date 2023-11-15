<script setup lang="ts">
import { useGlobalStore } from "@/stores/global";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";

const route = useRoute();
const global = useGlobalStore();
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <button
    type="button"
    class="btn btn-action btn-link btn-sm"
    :title="t('feedback')"
    data-cy="open-feedback-details"
    @click="global.openFeedback('entry', `[${route.params.id}]: `)"
  >
    <i class="icon icon-flag" />
  </button>
</template>

<i18n lang="yaml">
de:
  feedback: Problem melden oder Änderung vorschlagen
en:
  feedback: Report issue or suggest changes
</i18n>
