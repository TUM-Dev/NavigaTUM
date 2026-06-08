<script setup lang="ts">
import { mdiPlus, mdiTrashCanOutline } from "@mdi/js";
import {
  isValidTimeRange,
  OPENING_HOURS_DAYS,
  type OpeningHoursDay,
  type WeekSchedule,
} from "~/utils/openingHours";

const week = defineModel<WeekSchedule>("week", { required: true });
const sourceUrl = defineModel<string>("sourceUrl", { required: true });

const { t } = useI18n({ useScope: "local" });

function addRange(day: OpeningHoursDay) {
  week.value[day].push({ from: "08:00", to: "18:00" });
}
function removeRange(day: OpeningHoursDay, index: number) {
  week.value[day].splice(index, 1);
}

const dayLabels = computed<Record<OpeningHoursDay, string>>(() => ({
  Mo: t("days.Mo"),
  Tu: t("days.Tu"),
  We: t("days.We"),
  Th: t("days.Th"),
  Fr: t("days.Fr"),
  Sa: t("days.Sa"),
  Su: t("days.Su"),
}));
</script>

<template>
  <div class="space-y-1">
    <div v-for="day in OPENING_HOURS_DAYS" :key="day" class="flex items-start gap-2">
      <span class="w-24 shrink-0 pt-1.5 text-sm text-zinc-900 dark:text-zinc-50">{{ dayLabels[day] }}</span>
      <div class="flex flex-1 flex-wrap items-center gap-x-2 gap-y-1 pt-1">
        <span v-if="!week[day].length" class="text-zinc-400 dark:text-zinc-500 text-xs">{{ t("closed") }}</span>
        <div v-for="(range, index) in week[day]" :key="index" class="flex items-center gap-1">
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
            @click="removeRange(day, index)"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24"><path :d="mdiTrashCanOutline" fill="currentColor" /></svg>
          </button>
          <span v-if="!isValidTimeRange(range)" class="text-red-600 dark:text-red-300 text-xs">{{ t("invalid_range") }}</span>
        </div>
      </div>
      <button
        type="button"
        class="focusable text-blue-700 dark:text-blue-300 hover:text-blue-900 dark:hover:text-blue-100 shrink-0 rounded-sm pt-1"
        :aria-label="t('add_range')"
        :title="t('add_range')"
        @click="addRange(day)"
      >
        <svg class="h-5 w-5" viewBox="0 0 24 24"><path :d="mdiPlus" fill="currentColor" /></svg>
      </button>
    </div>

    <!-- Source URL is required: OpeningHoursSchema rejects a schedule without provenance. -->
    <div class="pt-2">
      <label class="text-zinc-500 dark:text-zinc-400 text-xs font-medium block mb-1" for="opening-hours-source">
        {{ t("source_url") }} <span class="text-red-600 dark:text-red-300" aria-hidden="true">*</span>
      </label>
      <input
        id="opening-hours-source"
        v-model="sourceUrl"
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
  add_range: Zeitraum hinzufügen
  closed: Geschlossen
  from: Von
  to: Bis
  invalid_range: Ungültig
  remove_range: Zeitraum entfernen
  source_url: Quelle (URL)
  source_url_help: Link zur offiziellen Seite mit den Öffnungszeiten (z.B. die Instituts- oder Bibliotheksseite).
  days:
    Mo: Montag
    Tu: Dienstag
    We: Mittwoch
    Th: Donnerstag
    Fr: Freitag
    Sa: Samstag
    Su: Sonntag
en:
  add_range: Add time range
  closed: Closed
  from: From
  to: To
  invalid_range: Invalid
  remove_range: Remove time range
  source_url: Source (URL)
  source_url_help: Link to the official page listing the opening hours (e.g. the department or library page).
  days:
    Mo: Monday
    Tu: Tuesday
    We: Wednesday
    Th: Thursday
    Fr: Friday
    Sa: Saturday
    Su: Sunday
</i18n>
