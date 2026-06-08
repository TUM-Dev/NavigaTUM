<script setup lang="ts">
import { Tab, TabGroup, TabList } from "@headlessui/vue";
import { mdiPlus, mdiTrashCanOutline } from "@mdi/js";
import { isValidTimeRange, type OpeningHoursDraft, type OpeningHoursMode } from "~/utils/openingHours";

const draft = defineModel<OpeningHoursDraft>({ required: true });

const { t } = useI18n({ useScope: "local" });

const modeOptions: { value: OpeningHoursMode; label: string }[] = [
  { value: "always", label: "mode_always" },
  { value: "semester", label: "mode_semester" },
];

function addHolidayRange() {
  draft.value.holiday.ranges.push({ from: "10:00", to: "14:00" });
}
function removeHolidayRange(index: number) {
  draft.value.holiday.ranges.splice(index, 1);
}
</script>

<template>
  <div class="space-y-3">
    <!-- Year-round vs. semester-dependent schedule. -->
    <TabGroup :selected-index="draft.mode === 'always' ? 0 : 1">
      <TabList class="bg-zinc-100 dark:bg-zinc-800 flex space-x-1 rounded-lg p-1">
        <Tab v-for="opt in modeOptions" :key="opt.value" as="template">
          <button
            type="button"
            :class="[
              'w-full rounded-md px-3 py-1.5 text-sm font-medium leading-5 transition-all',
              'ring-white/60 dark:ring-black/60 ring-offset-2 ring-offset-blue-400 dark:ring-offset-blue-500 focus:outline-none focus:ring-2',
              draft.mode === opt.value
                ? 'bg-white dark:bg-black text-zinc-700 dark:text-zinc-200 shadow'
                : 'text-zinc-500 dark:text-zinc-400 hover:bg-white/[0.12] dark:hover:bg-black/[0.12] hover:text-zinc-700 dark:hover:text-zinc-200',
            ]"
            @click="draft.mode = opt.value"
          >
            {{ t(opt.label) }}
          </button>
        </Tab>
      </TabList>
    </TabGroup>

    <WeekScheduleInput v-if="draft.mode === 'always'" v-model:week="draft.always" />
    <SemesterScheduleInput v-else v-model:lecture="draft.lecture" v-model:break="draft.break" />

    <!-- Public holidays (OSM `PH`). -->
    <div class="border-t border-zinc-200 dark:border-zinc-700 pt-3">
      <label class="text-zinc-500 dark:text-zinc-400 text-xs font-medium block mb-1" for="opening-hours-holiday">{{ t("holidays") }}</label>
      <select
        id="opening-hours-holiday"
        v-model="draft.holiday.mode"
        class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 rounded border px-2 py-1 w-full text-sm"
      >
        <option value="unspecified">{{ t("holiday_unspecified") }}</option>
        <option value="closed">{{ t("holiday_closed") }}</option>
        <option value="open">{{ t("holiday_open") }}</option>
      </select>

      <div v-if="draft.holiday.mode === 'open'" class="mt-2 flex flex-wrap items-center gap-2">
        <div v-for="(range, index) in draft.holiday.ranges" :key="index" class="flex items-center gap-1">
          <input
            v-model="range.from"
            type="time"
            :aria-label="t('from')"
            class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 rounded border px-1.5 py-0.5 text-sm"
          />
          <span class="text-zinc-500 dark:text-zinc-400 text-sm">-</span>
          <input
            v-model="range.to"
            type="time"
            :aria-label="t('to')"
            class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 rounded border px-1.5 py-0.5 text-sm"
          />
          <button
            type="button"
            class="focusable text-red-600 dark:text-red-300 hover:text-red-800 dark:hover:text-red-100 rounded-sm"
            :aria-label="t('remove_range')"
            @click="removeHolidayRange(index)"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24"><path :d="mdiTrashCanOutline" fill="currentColor" /></svg>
          </button>
          <span v-if="!isValidTimeRange(range)" class="text-red-600 dark:text-red-300 text-xs">{{ t("invalid_range") }}</span>
        </div>
        <button
          type="button"
          class="focusable text-blue-700 dark:text-blue-300 hover:text-blue-900 dark:hover:text-blue-100 inline-flex items-center gap-1 rounded-sm text-xs"
          @click="addHolidayRange"
        >
          <svg class="h-4 w-4" viewBox="0 0 24 24"><path :d="mdiPlus" fill="currentColor" /></svg>
          {{ t("add_range") }}
        </button>
      </div>
    </div>

    <!-- Source URL is required: OpeningHoursSchema rejects a schedule without provenance. -->
    <div class="border-t border-zinc-200 dark:border-zinc-700 pt-3">
      <label class="text-zinc-500 dark:text-zinc-400 text-xs font-medium block mb-1" for="opening-hours-source">
        {{ t("source_url") }} <span class="text-red-600 dark:text-red-300" aria-hidden="true">*</span>
      </label>
      <input
        id="opening-hours-source"
        v-model="draft.sourceUrl"
        type="url"
        required
        placeholder="https://"
        class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 rounded border px-2 py-1 w-full text-sm"
      />
      <p class="text-zinc-500 dark:text-zinc-400 text-xs mt-1">{{ t("source_url_help") }}</p>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  mode_always: Ganzjährig
  mode_semester: Nach Vorlesungszeit
  holidays: An Feiertagen
  holiday_unspecified: Keine Angabe
  holiday_closed: Geschlossen
  holiday_open: Geöffnet
  add_range: Zeitraum hinzufügen
  from: Von
  to: Bis
  invalid_range: Ungültig
  remove_range: Zeitraum entfernen
  source_url: Quelle (URL)
  source_url_help: Link zur offiziellen Seite mit den Öffnungszeiten (z.B. die Instituts- oder Bibliotheksseite).
en:
  mode_always: Year-round
  mode_semester: By lecture period
  holidays: On public holidays
  holiday_unspecified: Not specified
  holiday_closed: Closed
  holiday_open: Open
  add_range: Add time range
  from: From
  to: To
  invalid_range: Invalid
  remove_range: Remove time range
  source_url: Source (URL)
  source_url_help: Link to the official page listing the opening hours (e.g. the department or library page).
</i18n>
