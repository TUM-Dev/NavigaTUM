<script setup lang="ts">
import { mdiPlus, mdiTrashCanOutline } from "@mdi/js";
import {
  buildOsmOpeningHours,
  emptyWeekSchedule,
  isValidTimeRange,
  OPENING_HOURS_DAYS,
  type OpeningHoursDay,
  type WeekSchedule,
} from "~/utils/openingHours";

type Emits = (e: "confirm", payload: { opening_hours: string; source_url: string }) => void;
const modalOpen = defineModel<boolean>("open", { required: true });
const emit = defineEmits<Emits>();

const HTTP_URL_RE = /^https?:\/\//;

const { t } = useI18n({ useScope: "local" });

const week = ref<WeekSchedule>(emptyWeekSchedule());
const sourceUrl = ref("");

// Each open of the modal starts from a blank schedule so a previous draft can't
// leak into an unrelated entry.
watch(modalOpen, (isOpen) => {
  if (isOpen) {
    week.value = emptyWeekSchedule();
    sourceUrl.value = "";
  }
});

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

const osmPreview = computed(() => buildOsmOpeningHours(week.value));

const hasInvalidRange = computed(() =>
  OPENING_HOURS_DAYS.some((day) => week.value[day].some((range) => !isValidTimeRange(range)))
);

const isValidSourceUrl = computed(
  () => HTTP_URL_RE.test(sourceUrl.value) && URL.canParse(sourceUrl.value)
);

const canSubmit = computed(
  () => Boolean(osmPreview.value) && !hasInvalidRange.value && isValidSourceUrl.value
);

function confirm() {
  if (!canSubmit.value) return;
  emit("confirm", { opening_hours: osmPreview.value, source_url: sourceUrl.value });
  modalOpen.value = false;
}
</script>

<template>
  <Modal v-model="modalOpen" :title="t('title')">
    <p class="text-zinc-500 dark:text-zinc-400 text-sm mb-3">{{ t("intro") }}</p>

    <div class="space-y-2">
      <div
        v-for="day in OPENING_HOURS_DAYS"
        :key="day"
        class="bg-zinc-100 dark:bg-zinc-800 rounded p-2"
      >
        <div class="flex items-center justify-between">
          <span class="font-medium text-sm text-zinc-900 dark:text-zinc-50">{{ dayLabels[day] }}</span>
          <button
            type="button"
            class="focusable text-blue-700 dark:text-blue-300 hover:text-blue-900 dark:hover:text-blue-100 inline-flex items-center gap-1 rounded-sm text-xs"
            @click="addRange(day)"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24"><path :d="mdiPlus" fill="currentColor" /></svg>
            {{ t("add_range") }}
          </button>
        </div>

        <p v-if="!week[day].length" class="text-zinc-400 dark:text-zinc-500 text-xs mt-1">{{ t("closed") }}</p>

        <div v-for="(range, index) in week[day]" :key="index" class="mt-2 flex items-center gap-2">
          <input
            v-model="range.from"
            type="time"
            :aria-label="t('from')"
            class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 rounded border px-2 py-1 text-sm"
          />
          <span class="text-zinc-500 dark:text-zinc-400 text-sm">-</span>
          <input
            v-model="range.to"
            type="time"
            :aria-label="t('to')"
            class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 rounded border px-2 py-1 text-sm"
          />
          <span v-if="!isValidTimeRange(range)" class="text-red-600 dark:text-red-300 text-xs">{{ t("invalid_range") }}</span>
          <button
            type="button"
            class="focusable text-red-600 dark:text-red-300 hover:text-red-800 dark:hover:text-red-100 ms-auto rounded-sm"
            :aria-label="t('remove_range')"
            @click="removeRange(day, index)"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24"><path :d="mdiTrashCanOutline" fill="currentColor" /></svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Source URL is required: OpeningHoursSchema rejects a schedule without provenance. -->
    <div class="mt-4">
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

    <div class="flex justify-end pt-4">
      <Btn variant="primary" size="md" :disabled="!canSubmit" @click="confirm">{{ t("save") }}</Btn>
    </div>
  </Modal>
</template>

<i18n lang="yaml">
de:
  title: Öffnungszeiten korrigieren
  intro: Trage die regulären Öffnungszeiten pro Wochentag ein. Tage ohne Zeiten gelten als geschlossen.
  add_range: Zeitraum hinzufügen
  closed: Geschlossen
  from: Von
  to: Bis
  invalid_range: Ungültig
  remove_range: Zeitraum entfernen
  source_url: Quelle (URL)
  source_url_help: Link zur offiziellen Seite mit den Öffnungszeiten (z.B. die Instituts- oder Bibliotheksseite).
  save: Speichern
  days:
    Mo: Montag
    Tu: Dienstag
    We: Mittwoch
    Th: Donnerstag
    Fr: Freitag
    Sa: Samstag
    Su: Sonntag
en:
  title: Correct opening hours
  intro: Enter the regular opening hours per weekday. Days without any hours are treated as closed.
  add_range: Add time range
  closed: Closed
  from: From
  to: To
  invalid_range: Invalid
  remove_range: Remove time range
  source_url: Source (URL)
  source_url_help: Link to the official page listing the opening hours (e.g. the department or library page).
  save: Save
  days:
    Mo: Monday
    Tu: Tuesday
    We: Wednesday
    Th: Thursday
    Fr: Friday
    Sa: Saturday
    Su: Sunday
</i18n>
