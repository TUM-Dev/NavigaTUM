<script setup lang="ts">
import { Tab, TabGroup, TabList } from "@headlessui/vue";
import type { OpeningHoursDraft, OpeningHoursMode } from "~/utils/openingHoursEditor";

const draft = defineModel<OpeningHoursDraft>({ required: true });

const { t } = useI18n({ useScope: "local" });

const modeOptions: { value: OpeningHoursMode; label: string }[] = [
  { value: "always", label: "mode_always" },
  { value: "semester", label: "mode_semester" },
];
</script>

<template>
  <div class="space-y-3">
    <!-- Source URL is required: OpeningHoursSchema rejects a schedule without provenance.
         Kept directly under the section label so it reads as part of this edit. -->
    <div>
      <div class="flex items-center gap-2">
        <label class="text-zinc-500 dark:text-zinc-400 text-xs font-medium shrink-0" for="opening-hours-source">
          {{ t("source_url") }} <span class="text-red-600 dark:text-red-300" aria-hidden="true">*</span>
        </label>
        <input
          id="opening-hours-source"
          v-model="draft.sourceUrl"
          type="url"
          required
          placeholder="https://"
          class="focusable input-field rounded border px-2 py-1 flex-1 text-sm"
        />
      </div>
      <p class="text-zinc-500 dark:text-zinc-400 text-xs mt-1">{{ t("source_url_help") }}</p>
    </div>

    <!-- Year-round vs. semester-dependent schedule. -->
    <TabGroup :selected-index="draft.mode === 'always' ? 0 : 1">
      <TabList class="bg-zinc-100 dark:bg-zinc-800 flex space-x-1 rounded-lg p-1">
        <Tab v-for="opt in modeOptions" :key="opt.value" as="template">
          <SegmentedTab :selected="draft.mode === opt.value" class="w-full px-3 py-1.5" @click="draft.mode = opt.value">
            {{ t(opt.label) }}
          </SegmentedTab>
        </Tab>
      </TabList>
    </TabGroup>

    <!-- Public holidays (OSM `PH`) render as an extra weekday row inside the
         schedule, so the row lines up with the weekdays in either layout. -->
    <WeekScheduleInput
      v-if="draft.mode === 'always'"
      v-model:week="draft.always"
      v-model:holiday="draft.holiday"
      :holiday-label="t('holidays')"
    />
    <SemesterScheduleInput
      v-else
      v-model:lecture="draft.lecture"
      v-model:break="draft.break"
      v-model:holiday="draft.holiday"
      :holiday-label="t('holidays')"
    />
  </div>
</template>

<i18n lang="yaml">
de:
  mode_always: Ganzjährig
  mode_semester: Nach Vorlesungszeit
  holidays: Feiertage
  source_url: Quelle
  source_url_help: Link zur offiziellen Seite mit den Öffnungszeiten (z.B. die Instituts- oder Bibliotheksseite).
en:
  mode_always: Year-round
  mode_semester: By lecture period
  holidays: Holidays
  source_url: Source
  source_url_help: Link to the official page listing the opening hours (e.g. the department or library page).
</i18n>
