<script setup lang="ts">
import {
  OPENING_HOURS_DAYS,
  type OpeningHoursDay,
  type TimeRange,
  type WeekSchedule,
} from "~/utils/openingHours";

const week = defineModel<WeekSchedule>("week", { required: true });
// Public holidays (`PH`) render as an extra row, in line with the weekdays.
const holiday = defineModel<TimeRange[]>("holiday", { required: true });

defineProps<{ holidayLabel: string }>();

const { t } = useI18n({ useScope: "local" });

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
      <DayRangeInput v-model:ranges="week[day]" class="flex-1 pt-1" />
    </div>
    <div class="flex items-start gap-2">
      <span class="w-24 shrink-0 pt-1.5 text-sm text-zinc-900 dark:text-zinc-50">{{ holidayLabel }}</span>
      <DayRangeInput v-model:ranges="holiday" class="flex-1 pt-1" />
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  days:
    Mo: Montag
    Tu: Dienstag
    We: Mittwoch
    Th: Donnerstag
    Fr: Freitag
    Sa: Samstag
    Su: Sonntag
en:
  days:
    Mo: Monday
    Tu: Tuesday
    We: Wednesday
    Th: Thursday
    Fr: Friday
    Sa: Saturday
    Su: Sunday
</i18n>
