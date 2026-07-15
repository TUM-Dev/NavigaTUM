<script setup lang="ts">
import type { components } from "~/api_types";

type MotisRoutingResponse = components["schemas"]["MotisRoutingResponse"];
type DirectConnection = NonNullable<MotisRoutingResponse["direct"]>[0];

interface Props {
  connections: readonly DirectConnection[];
}

const props = defineProps<Props>();
const emit = defineEmits<{
  selectLeg: [itineraryIndex: number, legIndex: number];
  selectStep: [itineraryIndex: number, legIndex: number, stepIndex: number];
}>();

const { t } = useI18n({ useScope: "local" });

// Helper function to format time
const formatTime = (dateString: string) => {
  return new Date(dateString).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
};

// Helper function to format duration
const formatDuration = (seconds: number) => {
  if (seconds >= 3600) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  }
  if (seconds >= 60) {
    return t("minutes", Math.ceil(seconds / 60));
  }
  return t("seconds", seconds);
};
</script>

<template>
  <div v-if="connections.length > 0" class="mb-6">
    <div v-for="(itinerary, i) in connections" :key="`direct-${i}`" class="bg-zinc-50 dark:bg-zinc-900 mb-3 rounded-lg border p-4">
      <div class="mb-2 flex items-center justify-between">
        <span class="text-zinc-900 dark:text-zinc-50 font-medium">
          {{ formatTime(itinerary.start_time) }} - {{ formatTime(itinerary.end_time) }}
        </span>
        <span class="text-zinc-600 dark:text-zinc-300 text-sm">
          {{ formatDuration(itinerary.duration) }}
        </span>
      </div>
      <div class="divide-y">
        <MotisTransitLeg
          v-for="(leg, j) in itinerary.legs"
          :key="`direct-leg-${j}`"
          :leg="leg"
          :itinerary="itinerary"
          :leg-index="j"
          :itinerary-index="i"
          @select-leg="(itineraryIndex, legIndex) => emit('selectLeg', itineraryIndex, legIndex)"
          @select-step="(itineraryIndex, legIndex, stepIndex) => emit('selectStep', itineraryIndex, legIndex, stepIndex)"
        />
      </div>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  direct_connections: Direkte Verbindungen
  minutes: "sofort | eine Minute | {count} Minuten"
  seconds: "sofort | eine Sekunde | {count} Sekunden"

en:
  direct_connections: Direct connections
  minutes: "instant | one minute | {count} minutes"
  seconds: "instant | one second | {count} seconds"
</i18n>
