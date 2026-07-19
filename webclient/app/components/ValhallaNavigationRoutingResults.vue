<script setup lang="ts">
import type { components } from "~/api_types";
import { useHighlightRows } from "~/composables/useRouteHighlight";

type ValhallaRoutingResponse = components["schemas"]["ValhallaRoutingResponse"];
defineProps<{ data: ValhallaRoutingResponse }>();
const emit = defineEmits<{
  selectManeuver: [maneuverIndex: number];
}>();
const { t } = useI18n({ useScope: "local" });
const { registerRow, isEmphasised, hover } = useHighlightRows();

// The map only draws the first leg, so maneuvers are addressed by their index within it.
const target = (maneuverIndex: number) => ({ router: "valhalla", maneuverIndex }) as const;
</script>

<template>
  <div>
    <div v-for="(l, i) in data.legs" :key="i" class="gap-1">
      <p class="text-zinc-500 dark:text-zinc-400 mt-3 flex items-center gap-5 pb-4 font-semibold">
        <span>{{
          l.summary.length_meters >= 1000
            ? t("kilometers", [(l.summary.length_meters / 1000).toFixed(1)])
            : t("meters", l.summary.length_meters)
        }}</span>
        <span class="border-zinc-500 dark:border-zinc-400 flex-grow border-t" />
        <span>{{
          l.summary.time_seconds >= 60
            ? t("minutes", Math.ceil(l.summary.time_seconds / 60))
            : t("seconds", l.summary.time_seconds)
        }}</span>
      </p>
      <div
        v-for="(m, j) in l.maneuvers"
        :key="j"
        :ref="(el) => registerRow(target(j), el)"
        class="group cursor-pointer py-1"
        @click="emit('selectManeuver', j)"
        @mouseenter="hover(target(j))"
        @mouseleave="hover(null)"
      >
        <div
          class="bg-zinc-200 dark:bg-zinc-700 flex flex-row items-center gap-3 overflow-auto rounded-md p-2 py-1 group-hover:bg-zinc-300 dark:group-hover:bg-zinc-600"
          :class="{ 'ring-2 ring-blue-400 dark:ring-blue-500': isEmphasised(target(j)) }"
          :aria-label="m.verbal_transition_alert_instruction ?? undefined"
        >
          <NavigationRoutingManeuverIcon :type="m.type" />
          <div class="text-zinc-900 dark:text-zinc-50">{{ m.instruction }}</div>
        </div>
        <small
          v-if="m.length_meters"
          class="text-zinc-500 dark:text-zinc-400"
          :aria-label="m.verbal_post_transition_instruction ?? undefined"
          >{{ t("meters", m.length_meters) }}</small
        >
      </div>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  minutes: "sofort | eine Minute | {count} Minuten"
  seconds: "sofort | eine Sekunde | {count} Sekunden"
  meters: "hier | einen Meter | {count} Meter"
  kilometers: "{0} Kilometer"
en:
  minutes: "instant | one minute | {count} minutes"
  seconds: "instant | one second | {count} seconds"
  meters: "here | one meter | {count} meters"
  kilometers: "{0} kilometers"
</i18n>
