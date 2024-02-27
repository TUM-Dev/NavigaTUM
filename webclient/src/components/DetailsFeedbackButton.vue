<script setup lang="ts">
import { useGlobalStore } from "@/stores/global";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";
import { FlagIcon } from "@heroicons/vue/24/outline";

const route = useRoute();
const global = useGlobalStore();
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <button
    type="button"
    class="focusable rounded-sm"
    :title="t('feedback')"
    data-cy="open-feedback-details"
    @click="global.openFeedback('entry', `[${route.params.id}]: `)"
  >
    <FlagIcon class="text-blue-600 h-4 w-4" />
  </button>
</template>

<i18n lang="yaml">
de:
  feedback: Problem melden oder Änderung vorschlagen
en:
  feedback: Report issue or suggest changes
</i18n>
