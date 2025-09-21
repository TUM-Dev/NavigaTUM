<script setup lang="ts">
import type { components } from "~/api_types";

type MotisRoutingResponse = components["schemas"]["MotisRoutingResponse"];
type DirectConnection = NonNullable<MotisRoutingResponse["direct"]>[0];

interface Props {
  connections: DirectConnection[];
}

const props = defineProps<Props>();
const emit = defineEmits<{
  selectLeg: [itineraryIndex: number, legIndex: number];
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

// Helper function to format distance
const formatDistance = (meters: number) => {
  if (meters >= 1000) {
    return t("kilometers", [(meters / 1000).toFixed(1)]);
  }
  return t("meters", Math.round(meters));
};
</script>

<template>
  <div v-if="connections.length > 0" class="mb-6">
    <h3 class="text-zinc-700 mb-3 text-lg font-semibold">
      {{ t("direct_connections") }}
    </h3>
    <div v-for="(itinerary, i) in connections" :key="`direct-${i}`" class="bg-zinc-50 mb-3 rounded-lg border p-4">
      <div class="mb-2 flex items-center justify-between">
        <span class="text-zinc-900 font-medium">
          {{ formatTime(itinerary.start_time) }} - {{ formatTime(itinerary.end_time) }}
        </span>
        <span class="text-zinc-600 text-sm">
          {{ formatDuration(itinerary.duration) }}
        </span>
      </div>
      <div
        v-for="(leg, j) in itinerary.legs"
        :key="`direct-leg-${j}`"
        class="group cursor-pointer py-1"
        @click="emit('selectLeg', i, j)"
      >
        <div
          class="bg-white flex flex-row items-center gap-3 overflow-auto rounded-md border p-3 group-hover:bg-zinc-100"
        >
          <MotisTransitModeIcon :mode="leg.mode" />
          <div class="flex-grow">
            <div class="text-zinc-900 font-medium">{{ leg.from.name }} → {{ leg.to.name }}</div>
            <div class="text-zinc-600 text-sm">
              {{ formatDuration(leg.duration) }}
              <span v-if="leg.distance"> • {{ formatDistance(leg.distance) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  direct_connections: Direkte Verbindungen
  minutes: "sofort | eine Minute | {count} Minuten"
  seconds: "sofort | eine Sekunde | {count} Sekunden"
  meters: "hier | einen Meter | {count} Meter"
  kilometers: "{0} Kilometer"

en:
  direct_connections: Direct connections
  minutes: "instant | one minute | {count} minutes"
  seconds: "instant | one second | {count} seconds"
  meters: "here | one meter | {count} meters"
  kilometers: "{0} kilometers"
</i18n>
