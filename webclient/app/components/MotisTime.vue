<script setup lang="ts">
import { mdiAccessPoint, mdiClockOutline } from "@mdi/js";
import { delayMinutes, formatTime } from "~/utils/motis";

const props = defineProps<{
  // The timetable slot. Travellers plan against it, so it is the time we render.
  readonly scheduled: string | null | undefined;
  // Realtime-adjusted prediction; equal to `scheduled` while the feed reports no deviation.
  readonly actual: string | null | undefined;
  // Whether the operator feeds live data for this trip at all.
  readonly realTime?: boolean;
  readonly cancelled?: boolean;
}>();

const { t } = useI18n({ useScope: "local" });

const delay = computed(() => delayMinutes(props.scheduled, props.actual));

const clock = computed(() => {
  const iso = props.scheduled ?? props.actual;
  if (!iso || Number.isNaN(Date.parse(iso))) return "";
  return formatTime(iso);
});

// Signed, because a trip leaving early is a departure you can miss, not good news.
// A trip that is not running has no delay worth reporting.
const delayLabel = computed(() => {
  if (delay.value === null || props.cancelled) return "";
  return delay.value > 0 ? `+${delay.value}` : `${delay.value}`;
});

// Doubles as the hover title and the accessible name: the glyph alone cannot say
// which of live/timetable it means to a screen reader.
const label = computed(() => {
  if (!clock.value) return "";
  if (props.cancelled) return t("cancelled", { scheduled: clock.value });
  if (delay.value === null)
    return props.realTime
      ? t("on_time", { scheduled: clock.value })
      : t("timetable_only", { scheduled: clock.value });
  return t(delay.value > 0 ? "late" : "early", {
    scheduled: clock.value,
    expected: props.actual ? formatTime(props.actual) : "",
    delay: Math.abs(delay.value),
  });
});

const timeClass = computed(() => {
  if (props.cancelled) return "text-zinc-400 dark:text-zinc-500 line-through";
  if (delay.value !== null) return "text-red-600 dark:text-red-400";
  return "text-zinc-500 dark:text-zinc-400";
});

const iconClass = computed(() => {
  if (props.cancelled || !props.realTime) return "text-zinc-400 dark:text-zinc-500";
  return "text-zinc-500 dark:text-zinc-400";
});
</script>

<template>
  <span v-if="clock" class="inline-flex items-center gap-1 tabular-nums whitespace-nowrap" :title="label">
    <MdiIcon
      :path="realTime ? mdiAccessPoint : mdiClockOutline"
      :size="14"
      class="shrink-0"
      :class="iconClass"
    />
    <span aria-hidden="true" :class="timeClass">{{ clock }}</span>
    <span v-if="delayLabel" aria-hidden="true" class="font-semibold" :class="timeClass">{{ delayLabel }}</span>
    <span class="sr-only">{{ label }}</span>
  </span>
</template>

<i18n lang="yaml">
de:
  cancelled: "Planmäßig {scheduled}, fällt aus"
  on_time: "Planmäßig {scheduled}, laut Echtzeitdaten pünktlich"
  timetable_only: "Planmäßig {scheduled}, keine Echtzeitdaten"
  late: "Planmäßig {scheduled}, erwartet {expected} ({delay} min später)"
  early: "Planmäßig {scheduled}, erwartet {expected} ({delay} min früher)"
en:
  cancelled: "Scheduled {scheduled}, cancelled"
  on_time: "Scheduled {scheduled}, on time according to live data"
  timetable_only: "Scheduled {scheduled}, no live data"
  late: "Scheduled {scheduled}, expected {expected} ({delay} min late)"
  early: "Scheduled {scheduled}, expected {expected} ({delay} min early)"
</i18n>
