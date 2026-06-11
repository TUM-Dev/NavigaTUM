<script setup lang="ts">
import type { ValidationFailure } from "~/composables/feedbackSubmission";

defineProps<{
  failures: readonly ValidationFailure[];
}>();

const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <div
    v-if="failures.length"
    class="bg-red-50 dark:bg-red-900 border-red-300 dark:border-red-600 rounded border px-3 py-2"
    data-cy="validation-failures"
  >
    <p class="text-red-900 dark:text-red-50 text-sm font-semibold">{{ t("title") }}</p>
    <ul class="mt-1 list-disc pl-5 text-sm text-red-900 dark:text-red-50">
      <li v-for="failure in failures" :key="failure.key">
        <code class="bg-red-100 dark:bg-red-800 rounded px-1 py-0.5 text-xs">{{ failure.key }}</code>
        - {{ failure.error }}
      </li>
    </ul>
  </div>
</template>

<i18n lang="yaml">
de:
  title: "Folgende Einträge konnten nicht angelegt werden:"
en:
  title: "These entries could not be created:"
</i18n>
