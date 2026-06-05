<script setup lang="ts">
import { mdiFlag } from "@mdi/js";
import { useFeedback } from "~/composables/feedback";

const route = useRoute();
const feedback = useFeedback();
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <button
    type="button"
    class="focusable rounded-sm"
    :title="t('feedback')"
    data-cy="open-feedback-details"
    @click="
      () => {
        feedback.open = true;
        feedback.data = {
          category: 'entry',
          subject: `[${route.params.id}]: `,
          body: '',
          deletion_requested: false,
        };
      }
    "
  >
    <MdiIcon :path="mdiFlag" :size="28" class="text-blue-600 dark:text-blue-300 hover:text-blue-900 dark:hover:text-blue-50" />
  </button>
</template>

<i18n lang="yaml">
de:
  feedback: Problem melden oder Änderung vorschlagen
en:
  feedback: Report issue or suggest changes
</i18n>
