<script setup lang="ts">
import { mdiDoorClosedLock, mdiPlus, mdiTrashCanOutline } from "@mdi/js";
import { isValidTimeRange, type TimeRange } from "~/utils/openingHoursEditor";

const ranges = defineModel<TimeRange[]>("ranges", { required: true });

const { t } = useI18n({ useScope: "local" });

function addRange() {
  ranges.value.push({ from: "08:00", to: "18:00" });
}
function removeRange(index: number) {
  ranges.value.splice(index, 1);
}
</script>

<template>
  <div class="flex flex-wrap items-center gap-x-2 gap-y-1">
    <button
      type="button"
      class="focusable text-blue-700 dark:text-blue-300 hover:text-blue-900 dark:hover:text-blue-100 shrink-0 rounded-sm"
      :aria-label="t('add_range')"
      :title="t('add_range')"
      @click="addRange"
    >
      <MdiIcon :path="mdiPlus" :size="20" />
    </button>
    <span v-if="!ranges.length" class="text-zinc-400 dark:text-zinc-500 inline-flex items-center gap-1 text-xs">
      <MdiIcon :path="mdiDoorClosedLock" :size="14" />
      {{ t("closed") }}
    </span>
    <div v-for="(range, index) in ranges" :key="index" class="flex items-center gap-1">
      <input
        v-model="range.from"
        type="time"
        :aria-label="t('from')"
        class="focusable input-field rounded border px-1.5 py-0.5 text-sm"
      />
      <span class="text-zinc-500 dark:text-zinc-400 text-sm">-</span>
      <input
        v-model="range.to"
        type="time"
        :aria-label="t('to')"
        class="focusable input-field rounded border px-1.5 py-0.5 text-sm"
      />
      <button
        type="button"
        class="focusable text-red-600 dark:text-red-300 hover:text-red-800 dark:hover:text-red-100 rounded-sm"
        :aria-label="t('remove_range')"
        @click="removeRange(index)"
      >
        <MdiIcon :path="mdiTrashCanOutline" :size="16" />
      </button>
      <span v-if="!isValidTimeRange(range)" class="text-red-600 dark:text-red-300 text-xs">{{ t("invalid_range") }}</span>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  add_range: Zeitraum hinzufügen
  closed: Geschlossen
  from: Von
  to: Bis
  invalid_range: Ungültig
  remove_range: Zeitraum entfernen
en:
  add_range: Add time range
  closed: Closed
  from: From
  to: To
  invalid_range: Invalid
  remove_range: Remove time range
</i18n>
