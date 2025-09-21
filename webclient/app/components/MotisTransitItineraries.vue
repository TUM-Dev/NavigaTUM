<script setup lang="ts">
import type { components } from "~/api_types";

type MotisRoutingResponse = components["schemas"]["MotisRoutingResponse"];
type Itinerary = NonNullable<MotisRoutingResponse["itineraries"]>[0];

interface Props {
  itineraries: readonly Itinerary[];
  previousPageCursor?: string | null;
  nextPageCursor?: string | null;
  loading?: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  selectLeg: [itineraryIndex: number, legIndex: number];
  selectItinerary: [itineraryIndex: number];
  loadPrevious: [cursor: string];
  loadNext: [cursor: string];
}>();

const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <div v-if="itineraries.length > 0">
    <div class="flex items-center justify-end mb-3">
      <!-- Pagination Controls - Top -->
      <MotisPaginationControls
        :previous-page-cursor="previousPageCursor"
        :next-page-cursor="nextPageCursor"
        :loading="loading"
        size="sm"
        @load-previous="emit('loadPrevious', $event)"
        @load-next="emit('loadNext', $event)"
      />
    </div>

    <MotisTransitItinerary
      v-for="(itinerary, i) in itineraries"
      :key="`itinerary-${i}`"
      :itinerary="itinerary"
      :itinerary-index="i"
      @select-leg="(itineraryIndex, legIndex) => emit('selectLeg', itineraryIndex, legIndex)"
      @select-itinerary="emit('selectItinerary', $event)"
    />

    <!-- Pagination Controls - Bottom -->
    <MotisPaginationControls
      :previous-page-cursor="previousPageCursor"
      :next-page-cursor="nextPageCursor"
      :loading="loading"
      size="lg"
      @load-previous="emit('loadPrevious', $event)"
      @load-next="emit('loadNext', $event)"
    />
  </div>
</template>
