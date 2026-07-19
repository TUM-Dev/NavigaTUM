<script setup lang="ts">
import { mdiChevronDown, mdiTransitConnectionVariant } from "@mdi/js";
import type { components } from "~/api_types";
import { useHighlightRows, useMotisItineraryIndex } from "~/composables/useRouteHighlight";
import { formatDuration } from "~/utils/motis";

type MotisLegResponse = components["schemas"]["MotisLegResponse"];

const props = defineProps<{
  leg: MotisLegResponse;
  legIndex: number;
  previousLeg: MotisLegResponse | null;
  open: boolean;
}>();
const emit = defineEmits<{ toggle: [] }>();

const { t } = useI18n({ useScope: "local" });

const itineraryIndex = useMotisItineraryIndex();
const { registerRow, isEmphasised, hover } = useHighlightRows();
const legTarget = computed(
  () =>
    ({
      router: "motis",
      itineraryIndex: itineraryIndex.value,
      legIndex: props.legIndex,
      stepIndex: null,
    }) as const
);

const intermediateStops = computed(() => props.leg.intermediate_stops ?? []);
const hasCollapsibleDetail = computed(
  () => intermediateStops.value.length > 0 || (props.leg.alerts?.length ?? 0) > 0
);
const showRouteChange = computed(() => {
  const previousName = props.previousLeg?.route_short_name;
  const currentName = props.leg.route_short_name;
  return (
    props.leg.interline_with_previous_leg === true &&
    previousName != null &&
    currentName != null &&
    previousName !== currentName
  );
});
</script>

<template>
  <div class="flex items-stretch gap-3">
    <div class="flex w-4 flex-shrink-0 justify-center">
      <span class="my-0.5 w-1.5 rounded-full" :style="{ backgroundColor: leg.route_color }" />
    </div>

    <div class="min-w-0 flex-grow py-1">
      <button
        :ref="(el) => registerRow(legTarget, el)"
        type="button"
        class="focusable flex w-full items-center gap-2 rounded-sm text-left"
        :class="{ 'ring-1 ring-blue-400 dark:ring-blue-500': isEmphasised(legTarget) }"
        :aria-expanded="open"
        @click="emit('toggle')"
        @mouseenter="hover(legTarget)"
        @mouseleave="hover(null)"
      >
        <span
          class="flex-shrink-0 rounded px-2 py-0.5 text-sm font-bold"
          :style="{ backgroundColor: leg.route_color, color: leg.route_text_color }"
        >
          {{ leg.route_short_name ?? t("transit") }}
        </span>
        <MotisTransitModeIcon
          :mode="leg.mode"
          variant="inherit"
          class="h-4 w-4 flex-shrink-0 text-zinc-500 dark:text-zinc-400"
        />
        <span v-if="leg.headsign" class="text-zinc-600 dark:text-zinc-300 min-w-0 truncate text-sm">
          {{ leg.headsign }}
        </span>
        <span class="text-zinc-400 dark:text-zinc-500 ml-auto flex-shrink-0 text-xs">
          <template v-if="intermediateStops.length">{{ t("stops", intermediateStops.length) }} · </template>{{ formatDuration(leg.duration) }}
        </span>
        <MdiIcon
          v-if="hasCollapsibleDetail"
          :path="mdiChevronDown"
          :size="14"
          class="flex-shrink-0 text-zinc-400 dark:text-zinc-500 transition-transform"
          :class="{ 'rotate-180': open }"
        />
      </button>

      <div v-if="leg.cancelled" class="text-red-600 dark:text-red-300 mt-1 text-sm font-medium">
        {{ t("cancelled") }}
      </div>

      <div
        v-if="leg.interline_with_previous_leg"
        class="mt-2 flex items-start gap-2 rounded-md border border-blue-200 dark:border-blue-700 bg-blue-50 dark:bg-blue-900 px-3 py-2 text-blue-800 dark:text-blue-100"
      >
        <MdiIcon :path="mdiTransitConnectionVariant" :size="16" class="mt-0.5 flex-shrink-0 text-blue-600 dark:text-blue-300" />
        <div class="min-w-0">
          <div class="text-sm font-medium">{{ t("stay_on_vehicle") }}</div>
          <div v-if="showRouteChange" class="mt-1 flex items-center gap-2 text-xs">
            <span class="rounded px-1.5 py-0.5 font-bold text-white dark:text-black bg-blue-600 dark:bg-blue-300">
              {{ previousLeg?.route_short_name }}
            </span>
            <span class="text-blue-600 dark:text-blue-300">→</span>
            <span
              class="rounded px-1.5 py-0.5 font-bold"
              :style="{ backgroundColor: leg.route_color, color: leg.route_text_color }"
            >
              {{ leg.route_short_name }}
            </span>
          </div>
          <div v-else class="text-blue-700 dark:text-blue-200 mt-1 text-xs">{{ t("interline_explanation") }}</div>
        </div>
      </div>

      <Collapsible :open="open">
        <ul v-if="intermediateStops.length" class="mt-2 space-y-1.5">
          <li
            v-for="(stop, s) in intermediateStops"
            :key="`stop-${s}`"
            class="flex items-baseline justify-between gap-2"
          >
            <span class="flex min-w-0 items-baseline gap-2 text-xs text-zinc-500 dark:text-zinc-400">
              <span class="h-1.5 w-1.5 flex-shrink-0 self-center rounded-full bg-zinc-300 dark:bg-zinc-600" />
              <span class="truncate">{{ stop.name }}</span>
              <span v-if="stop.track" class="flex-shrink-0">{{ t("platform") }} {{ stop.track }}</span>
            </span>
            <!-- Stops inherit their leg's liveness: it is a property of the trip's feed. -->
            <MotisTime
              v-if="stop.departure || stop.arrival"
              class="flex-shrink-0 text-xs"
              :scheduled="stop.departure ? stop.scheduled_departure : stop.scheduled_arrival"
              :actual="stop.departure ?? stop.arrival"
              :real-time="leg.real_time"
              :cancelled="stop.cancelled ?? false"
            />
          </li>
        </ul>

        <MotisAlertList v-if="leg.alerts && leg.alerts.length > 0" :alerts="leg.alerts" size="sm" class="mt-2" />
      </Collapsible>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  transit: ÖPNV
  stops: "keine Halte | ein Halt | {count} Halte"
  platform: Gleis
  cancelled: Ausfall
  stay_on_vehicle: Im Fahrzeug bleiben
  interline_explanation: Das Fahrzeug ändert seine Linie, aber du musst nicht umsteigen
en:
  transit: Transit
  stops: "no stops | one stop | {count} stops"
  platform: Platform
  cancelled: Cancelled
  stay_on_vehicle: Stay on vehicle
  interline_explanation: The vehicle changes line, but you don't need to transfer
</i18n>
