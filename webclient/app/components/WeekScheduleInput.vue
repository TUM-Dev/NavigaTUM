<script setup lang="ts">
import { OPENING_HOURS_DAYS, type TimeRange, type WeekSchedule } from "~/utils/openingHoursEditor";

const week = defineModel<WeekSchedule>("week", { required: true });
// Public holidays (`PH`) render as an extra row, in line with the weekdays.
const holiday = defineModel<TimeRange[]>("holiday", { required: true });

defineProps<{ holidayLabel: string }>();

const dayLabels = useWeekdayLabels();
</script>

<template>
  <div class="space-y-1">
    <div v-for="day in OPENING_HOURS_DAYS" :key="day" class="flex items-start gap-2">
      <span class="w-24 shrink-0 pt-1.5 text-sm text-zinc-900 dark:text-zinc-50">{{ dayLabels[day] }}</span>
      <DayRangeInput v-model:ranges="week[day]" class="flex-1 pt-1" />
    </div>
    <div class="flex items-start gap-2">
      <span class="w-24 shrink-0 pt-1.5 text-sm text-zinc-900 dark:text-zinc-50">{{ holidayLabel }}</span>
      <DayRangeInput v-model:ranges="holiday" class="flex-1 pt-1" />
    </div>
  </div>
</template>
