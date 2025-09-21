<script setup lang="ts">
import type { components } from "~/api_types";

type MotisRoutingResponse = components["schemas"]["MotisRoutingResponse"];

interface Props {
  data?: MotisRoutingResponse;
  loading?: boolean;
  error?: string | null;
  pageCursor?: string;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  selectLeg: [itineraryIndex: number, legIndex: number];
  selectItinerary: [itineraryIndex: number];
  retry: [];
  loadPrevious: [cursor: string];
  loadNext: [cursor: string];
}>();

const hasResults = computed(() => {
  return (props.data?.direct?.length || 0) > 0 || (props.data?.itineraries?.length || 0) > 0;
});
</script>

<template>
  <div>
    <!-- Loading, Error, and No Results States -->
    <MotisRoutingStates :loading="loading" :error="error" :has-results="hasResults" @retry="emit('retry')" />

    <!-- Success State - Show Results -->
    <div v-if="data && !loading && !error">
      <!-- Direct connections (walking, biking, etc.) -->
      <MotisDirectConnections
        v-if="data.direct && data.direct.length > 0"
        :connections="data.direct"
        @select-leg="(itineraryIndex, legIndex) => emit('selectLeg', itineraryIndex, legIndex)"
      />

      <!-- Public Transit Itineraries -->
      <MotisTransitItineraries
        v-if="data.itineraries && data.itineraries.length > 0"
        :itineraries="data.itineraries"
        :previous-page-cursor="data.previous_page_cursor"
        :next-page-cursor="data.next_page_cursor"
        :loading="loading"
        @select-leg="(itineraryIndex, legIndex) => emit('selectLeg', itineraryIndex, legIndex)"
        @select-itinerary="emit('selectItinerary', $event)"
        @load-previous="emit('loadPrevious', $event)"
        @load-next="emit('loadNext', $event)"
      />
    </div>
  </div>
</template>
