<script setup lang="ts">
import { mdiChevronDown, mdiChevronRight } from "@mdi/js";
import type { components } from "~/api_types";
import { formatDuration, formatTime, isSelfNavigatedLeg } from "~/utils/motis";

type ItineraryResponse = components["schemas"]["ItineraryResponse"];
type MotisLegResponse = components["schemas"]["MotisLegResponse"];

const props = defineProps<{
  itinerary: ItineraryResponse;
  expanded: boolean;
}>();
const emit = defineEmits<{
  toggle: [];
  selectLeg: [legIndex: number];
  selectStep: [legIndex: number, stepIndex: number];
}>();

const { t } = useI18n({ useScope: "local" });

const isDirect = computed(() => props.itinerary.legs.every(isSelfNavigatedLeg));

const modeStrip = computed(() => {
  type StripLeg = Pick<
    MotisLegResponse,
    "mode" | "route_short_name" | "route_color" | "route_text_color"
  >;
  const strip: StripLeg[] = [];
  for (const leg of props.itinerary.legs) {
    const last = strip[strip.length - 1];
    if (last && last.mode === leg.mode && last.route_short_name === leg.route_short_name) continue;
    strip.push({
      mode: leg.mode,
      route_short_name: leg.route_short_name,
      route_color: leg.route_color,
      route_text_color: leg.route_text_color,
    });
  }
  return strip;
});
</script>

<template>
  <div
    class="bg-zinc-50 dark:bg-zinc-900 overflow-hidden rounded-lg border transition-colors"
    :class="expanded ? 'border-blue-500 dark:border-blue-400 ring-1 ring-blue-500 dark:ring-blue-400' : 'border-zinc-300 dark:border-zinc-600'"
  >
    <button
      type="button"
      class="focusable flex w-full flex-col gap-2 p-4 text-left hover:bg-zinc-100 dark:hover:bg-zinc-800"
      :aria-expanded="expanded"
      @click="emit('toggle')"
    >
      <div class="flex items-center justify-between gap-2">
        <div class="flex items-baseline gap-3">
          <span class="text-zinc-900 dark:text-zinc-50 font-medium tabular-nums">
            {{ formatTime(itinerary.start_time) }} - {{ formatTime(itinerary.end_time) }}
          </span>
          <span class="text-zinc-600 dark:text-zinc-300 text-sm">{{ formatDuration(itinerary.duration) }}</span>
        </div>
        <div class="flex flex-shrink-0 items-center gap-2">
          <span class="text-zinc-500 dark:text-zinc-400 text-sm">
            {{ isDirect ? t("direct") : t("transfers", itinerary.transfer_count) }}
          </span>
          <MdiIcon
            :path="mdiChevronDown"
            :size="16"
            class="text-zinc-400 dark:text-zinc-500 transition-transform"
            :class="{ 'rotate-180': expanded }"
          />
        </div>
      </div>

      <div class="flex flex-wrap items-center gap-1">
        <template v-for="(leg, i) in modeStrip" :key="`strip-${i}`">
          <MdiIcon v-if="i > 0" :path="mdiChevronRight" :size="12" class="text-zinc-400 dark:text-zinc-500" />
          <span
            v-if="leg.route_short_name"
            class="flex h-5 items-center gap-1 rounded-md px-2 text-xs font-bold shadow-sm"
            :style="{ backgroundColor: leg.route_color, color: leg.route_text_color }"
          >
            <MotisTransitModeIcon :mode="leg.mode" variant="inherit" class="h-3 w-3" />
            {{ leg.route_short_name }}
          </span>
          <MotisTransitModeIcon v-else :mode="leg.mode" variant="inherit" class="h-5 w-5 text-zinc-500 dark:text-zinc-400" />
        </template>
      </div>
    </button>

    <Collapsible :open="expanded">
      <div class="px-4 pb-4">
        <MotisConnectionTimeline
          :itinerary="itinerary"
          @select-leg="(legIndex) => emit('selectLeg', legIndex)"
          @select-step="(legIndex, stepIndex) => emit('selectStep', legIndex, stepIndex)"
        />
      </div>
    </Collapsible>
  </div>
</template>

<i18n lang="yaml">
de:
  direct: Direkt
  transfers: "keine Umstiege | ein Umstieg | {count} Umstiege"
en:
  direct: Direct
  transfers: "no transfers | one transfer | {count} transfers"
</i18n>
