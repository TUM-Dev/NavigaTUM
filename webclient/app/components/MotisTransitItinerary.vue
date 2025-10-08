<script setup lang="ts">
import type { components } from "~/api_types";

type MotisRoutingResponse = components["schemas"]["MotisRoutingResponse"];
type Itinerary = NonNullable<MotisRoutingResponse["itineraries"]>[0];

interface Props {
  itinerary: Itinerary;
  itineraryIndex: number;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  selectLeg: [itineraryIndex: number, legIndex: number];
  selectItinerary: [itineraryIndex: number];
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
  <div class="mb-6 rounded-lg border">
    <!-- Itinerary Header -->
    <div
      class="bg-zinc-50 flex items-center justify-between rounded-t-lg p-4 cursor-pointer hover:bg-zinc-100 transition-colors"
      @click="emit('selectItinerary', itineraryIndex)"
      :title="`${t('select_itinerary')} ${itineraryIndex + 1}`"
    >
      <div class="flex items-center gap-4">
        <span class="text-zinc-900 font-semibold">
          {{ formatTime(itinerary.start_time) }} - {{ formatTime(itinerary.end_time) }}
        </span>
        <span class="text-zinc-600">{{ formatDuration(itinerary.duration) }}</span>
      </div>
      <div class="text-zinc-500 text-sm">
        {{ t("transfers", itinerary.transfer_count) }}
      </div>
    </div>

    <!-- Legs -->
    <div class="divide-y">
      <MotisTransitLeg
        v-for="(leg, j) in itinerary.legs"
        :key="`leg-${j}`"
        :leg="leg"
        :itinerary="itinerary"
        :leg-index="j"
        :itinerary-index="itineraryIndex"
        @select-leg="(itineraryIndex, legIndex) => emit('selectLeg', itineraryIndex, legIndex)"
      />
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  transfers: "keine Umstiege | ein Umstieg | {count} Umstiege"
  select_itinerary: Route auswählen
  minutes: "sofort | eine Minute | {count} Minuten"
  seconds: "sofort | eine Sekunde | {count} Sekunden"

en:
  transfers: "no transfers | one transfer | {count} transfers"
  select_itinerary: Select route
  minutes: "instant | one minute | {count} minutes"
  seconds: "instant | one second | {count} seconds"
</i18n>
