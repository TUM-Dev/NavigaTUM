<script setup lang="ts">
import type { components } from "~/api_types";
import { buildItineraryTimeline } from "~/utils/motis";

type ItineraryResponse = components["schemas"]["ItineraryResponse"];

const props = defineProps<{ itinerary: ItineraryResponse }>();
const emit = defineEmits<{
  selectLeg: [legIndex: number];
  selectStep: [legIndex: number, stepIndex: number];
}>();

const { t } = useI18n({ useScope: "local" });

const timeline = computed(() => buildItineraryTimeline(props.itinerary));

const openLegs = ref<Set<number>>(new Set());
watch(
  () => props.itinerary,
  () => {
    openLegs.value = new Set();
  }
);

// Tapping an edge both reveals its detail and focuses that leg on the map.
function toggleLeg(legIndex: number) {
  const next = new Set(openLegs.value);
  if (next.has(legIndex)) next.delete(legIndex);
  else next.add(legIndex);
  openLegs.value = next;
  emit("selectLeg", legIndex);
}

function previousLeg(legIndex: number) {
  return props.itinerary.legs[legIndex - 1] ?? null;
}
</script>

<template>
  <div class="flex flex-col">
    <template v-for="(node, n) in timeline.nodes" :key="`node-${n}`">
      <div class="flex items-start gap-3">
        <div class="flex w-4 flex-shrink-0 justify-center pt-1">
          <span class="h-3 w-3 rounded-full border-2 border-zinc-500 bg-white dark:border-zinc-300 dark:bg-zinc-900" />
        </div>
        <div class="min-w-0 flex-grow pb-1">
          <div class="flex items-baseline justify-between gap-2">
            <div class="min-w-0">
              <span class="text-zinc-900 dark:text-zinc-50 font-medium">{{ node.name }}</span>
              <span v-if="node.track" class="text-zinc-500 dark:text-zinc-400 ml-2 whitespace-nowrap text-xs">{{ t("platform") }} {{ node.track }}</span>
              <span v-if="node.level !== 0" class="text-zinc-400 dark:text-zinc-500 ml-2 whitespace-nowrap text-xs">{{ t("level") }} {{ node.level }}</span>
            </div>
            <MotisTime
              class="flex-shrink-0 text-sm"
              :scheduled="node.time.scheduled"
              :actual="node.time.actual"
              :real-time="node.time.realTime"
              :cancelled="node.time.cancelled"
            />
          </div>
          <MotisAlertList v-if="node.alerts.length > 0" :alerts="node.alerts" size="sm" class="mt-1" />
        </div>
      </div>

      <template v-if="n < timeline.edges.length">
        <MotisTimelineWalkEdge
          v-if="timeline.edges[n]!.selfNavigated"
          :leg="timeline.edges[n]!.leg"
          :leg-index="n"
          :open="openLegs.has(n)"
          @toggle="toggleLeg(n)"
          @select-step="(stepIndex) => emit('selectStep', n, stepIndex)"
        />
        <MotisTimelineRideEdge
          v-else
          :leg="timeline.edges[n]!.leg"
          :leg-index="n"
          :previous-leg="previousLeg(n)"
          :open="openLegs.has(n)"
          @toggle="toggleLeg(n)"
        />
      </template>
    </template>
  </div>
</template>

<i18n lang="yaml">
de:
  platform: Gleis
  level: Ebene
en:
  platform: Platform
  level: Level
</i18n>
