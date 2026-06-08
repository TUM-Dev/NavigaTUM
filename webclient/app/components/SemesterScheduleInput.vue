<script setup lang="ts">
import { OPENING_HOURS_DAYS, type TimeRange, type WeekSchedule } from "~/utils/openingHoursEditor";

const lecture = defineModel<WeekSchedule>("lecture", { required: true });
const breakWeek = defineModel<WeekSchedule>("break", { required: true });
// Public holidays (`PH`) are not semester-dependent, so the row spans both columns.
const holiday = defineModel<TimeRange[]>("holiday", { required: true });

defineProps<{ holidayLabel: string }>();

const { t } = useI18n({ useScope: "local" });

const dayLabels = useWeekdayLabels();
</script>

<template>
  <div class="grid grid-cols-[auto_1fr_1fr] items-start gap-x-3 gap-y-1">
    <!-- Column headers; the first cell sits above the day-label column. -->
    <div></div>
    <p class="text-zinc-500 dark:text-zinc-400 text-xs font-medium">{{ t("lecture_period") }}</p>
    <p class="text-zinc-500 dark:text-zinc-400 text-xs font-medium">{{ t("break_period") }}</p>

    <template v-for="day in OPENING_HOURS_DAYS" :key="day">
      <span class="pt-1.5 text-sm text-zinc-900 dark:text-zinc-50">{{ dayLabels[day] }}</span>
      <DayRangeInput v-model:ranges="lecture[day]" class="pt-1" />
      <DayRangeInput v-model:ranges="breakWeek[day]" class="pt-1" />
    </template>

    <span class="pt-1.5 text-sm text-zinc-900 dark:text-zinc-50">{{ holidayLabel }}</span>
    <DayRangeInput v-model:ranges="holiday" class="col-span-2 pt-1" />
  </div>
</template>

<i18n lang="yaml">
de:
  lecture_period: Vorlesungszeit
  break_period: Vorlesungsfreie Zeit
en:
  lecture_period: Lecture period
  break_period: Lecture-free period
</i18n>
