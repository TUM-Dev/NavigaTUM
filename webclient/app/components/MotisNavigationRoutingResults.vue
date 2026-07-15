<script setup lang="ts">
import { useResizeObserver } from "@vueuse/core";
import type { components } from "~/api_types";

type MotisRoutingResponse = components["schemas"]["MotisRoutingResponse"];
type MotisLegResponse = components["schemas"]["MotisLegResponse"];

const pageCursor = defineModel<string | undefined>("pageCursor", {
  required: true,
});
const props = defineProps<{
  data: MotisRoutingResponse;
}>();
const emit = defineEmits<{
  selectLeg: [itineraryIndex: number, legIndex: number];
  selectStep: [itineraryIndex: number, legIndex: number, stepIndex: number];
  selectItinerary: [itineraryIndex: number];
  loadPrevious: [cursor: string];
  loadNext: [cursor: string];
}>();

const { t } = useI18n({ useScope: "local" });

// View state management
const viewMode = ref<"summary" | "details">("summary");
const selectedItineraryIndex = ref<number | null>(null);

// Refs for overflow detection
const transitContainer = ref<HTMLElement | null>(null);
const transitContainer2 = ref<HTMLElement | null>(null);
const showDirectOverflow = ref(false);
const showTransitOverflow = ref(false);

const checkOverflow = () => {
  if (transitContainer.value) {
    showDirectOverflow.value =
      transitContainer.value.scrollWidth > transitContainer.value.clientWidth;
  }
  if (transitContainer2.value) {
    showTransitOverflow.value =
      transitContainer2.value.scrollWidth > transitContainer2.value.clientWidth;
  }
};

// Both needed: data changes can grow scrollWidth without the box (clientWidth) changing.
watch(
  () => props.data,
  () => nextTick(checkOverflow),
  {
    deep: true,
    flush: "post",
    immediate: true,
  }
);
useResizeObserver(transitContainer, checkOverflow);
useResizeObserver(transitContainer2, checkOverflow);

const hasResults = computed(() => {
  return (props.data?.direct?.length || 0) > 0 || (props.data?.itineraries?.length || 0) > 0;
});

// Show itinerary details
const showItineraryDetails = (index: number) => {
  selectedItineraryIndex.value = index;
  viewMode.value = "details";
};

// Go back to summary view
const backToSummary = () => {
  viewMode.value = "summary";
  selectedItineraryIndex.value = null;
};

// Handle itinerary selection
const handleItinerarySelect = (itineraryIndex: number) => {
  showItineraryDetails(itineraryIndex);
  emit("selectItinerary", itineraryIndex);
};

// Helper functions
const formatTime = (dateString: string) => {
  return new Date(dateString).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
};

const formatDuration = (seconds: number) => {
  if (seconds >= 3600) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}min`;
  }
  if (seconds >= 60) {
    return `${Math.ceil(seconds / 60)}min`;
  }
  return `${seconds}s`;
};

// Get transit legs for an itinerary summary
const getTransitLegs = (legs: readonly MotisLegResponse[]) => {
  type TransitLeg = Pick<
    MotisLegResponse,
    "mode" | "route_short_name" | "route_color" | "route_text_color"
  >;
  let transitLegs: Array<TransitLeg> = [];

  for (const leg of legs) {
    const newLeg = {
      mode: leg.mode,
      route_short_name: leg.route_short_name,
      route_color: leg.route_color,
      route_text_color: leg.route_text_color,
    };

    // Deduplicate consecutive similar legs (same mode and route)
    const lastLeg = transitLegs[transitLegs.length - 1];
    const isDuplicate =
      lastLeg &&
      lastLeg.mode === newLeg.mode &&
      lastLeg.route_short_name === newLeg.route_short_name;

    if (!isDuplicate) {
      transitLegs.push(newLeg);
    }
  }

  return transitLegs;
};
</script>

<template>
  <div class="w-full max-w-full">
    <!-- No Results States -->
    <MotisNoRoutesFound v-if="!hasResults" />

    <!-- Summary View -->
    <div v-else-if="viewMode === 'summary'">
      <!-- Direct connections summary -->
      <div v-if="data.direct && data.direct.length > 0" class="mb-6">
        <h3 class="text-zinc-700 dark:text-zinc-200 mb-3 text-lg font-semibold">
          {{ t("direct_connections") }}
        </h3>
        <div class="space-y-2 w-full max-w-full">
          <div
            v-for="(connection, i) in data.direct"
            :key="`direct-${i}`"
            class="bg-zinc-50 dark:bg-zinc-900 hover:bg-zinc-100 dark:hover:bg-zinc-800 cursor-pointer rounded-lg border p-4 transition-colors w-full max-w-full"
            @click="handleItinerarySelect(-1 - i)"
          >
            <div class="flex items-center justify-between w-full min-w-0">
              <div class="flex flex-col gap-2 min-w-0 flex-1">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-4 min-w-0">
                    <span class="text-zinc-900 dark:text-zinc-50 font-medium truncate">
                      {{ formatTime(connection.start_time) }} - {{ formatTime(connection.end_time) }}
                    </span>
                    <span class="text-zinc-600 dark:text-zinc-300 flex-shrink-0">
                      {{ formatDuration(connection.duration) }}
                    </span>
                  </div>
                  <div class="text-zinc-500 dark:text-zinc-400 text-sm flex-shrink-0">
                    {{ connection.legs.length === 1 ? t("direct") : t("legs", connection.legs.length) }}
                  </div>
                </div>
                <!-- Transit summary below time -->
                <div class="relative">
                  <div ref="transitContainer" class="flex items-center gap-1 overflow-x-auto scrollbar-hide">
                    <template
                      v-for="(leg, legIndex) in getTransitLegs(connection.legs)"
                      :key="`${leg.mode}-${legIndex}`"
                    >
                      <!-- Separator arrow between legs -->
                      <svg
                        v-if="legIndex > 0"
                        class="w-3 h-3 text-zinc-400 dark:text-zinc-500 mx-1 flex-shrink-0"
                        fill="currentColor"
                        viewBox="0 0 20 20"
                      >
                        <path
                          fill-rule="evenodd"
                          d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z"
                          clip-rule="evenodd"
                        />
                      </svg>

                      <!-- Transit with route info -->
                      <div
                        v-if="leg.route_short_name"
                        class="flex items-center gap-1 rounded-md px-2 py-1 text-xs font-bold min-w-[2rem] h-5 shadow-sm flex-shrink-0"
                        :style="{
                          backgroundColor: leg.route_color,
                          color: leg.route_text_color,
                        }"
                      >
                        <MotisTransitModeIcon :mode="leg.mode" class="w-3 h-3" variant="inherit" />
                        {{ leg.route_short_name }}
                      </div>
                      <!-- Non-transit or transit without route info -->
                      <MotisTransitModeIcon v-else :mode="leg.mode" class="w-5 h-5" variant="inherit" />
                    </template>
                  </div>
                  <!-- Fade-out gradient for overflow -->
                  <div
                    v-show="showDirectOverflow"
                    class="absolute right-0 top-0 bottom-0 w-8 bg-gradient-to-l from-zinc-50 dark:from-zinc-900 to-transparent pointer-events-none z-10"
                  ></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Transit itineraries summary -->
      <div v-if="data.itineraries && data.itineraries.length > 0">
        <div class="space-y-2 w-full max-w-full">
          <div
            v-for="(itinerary, i) in data.itineraries"
            :key="`summary-${i}`"
            class="bg-zinc-50 dark:bg-zinc-900 hover:bg-zinc-100 dark:hover:bg-zinc-800 cursor-pointer rounded-lg border p-4 transition-colors w-full max-w-full"
            @click="handleItinerarySelect(i)"
          >
            <div class="flex items-center justify-between w-full min-w-0">
              <div class="flex flex-col gap-2 min-w-0 flex-1">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-4 min-w-0">
                    <span class="text-zinc-900 dark:text-zinc-50 font-medium truncate">
                      {{ formatTime(itinerary.start_time) }} - {{ formatTime(itinerary.end_time) }}
                    </span>
                    <span class="text-zinc-600 dark:text-zinc-300 flex-shrink-0">
                      {{ formatDuration(itinerary.duration) }}
                    </span>
                  </div>
                  <div class="text-zinc-500 dark:text-zinc-400 text-sm flex-shrink-0">
                    {{ t("transfers", itinerary.transfer_count) }}
                  </div>
                </div>
                <!-- Transit summary below time -->
                <div class="relative">
                  <div ref="transitContainer2" class="flex items-center gap-1 overflow-x-auto scrollbar-hide">
                    <template
                      v-for="(leg, legIndex) in getTransitLegs(itinerary.legs)"
                      :key="`${leg.mode}-${legIndex}`"
                    >
                      <!-- Separator arrow between legs -->
                      <svg
                        v-if="legIndex > 0"
                        class="w-3 h-3 text-zinc-400 dark:text-zinc-500 mx-1 flex-shrink-0"
                        fill="currentColor"
                        viewBox="0 0 20 20"
                      >
                        <path
                          fill-rule="evenodd"
                          d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z"
                          clip-rule="evenodd"
                        />
                      </svg>

                      <!-- Transit with route info -->
                      <div
                        v-if="leg.route_short_name"
                        class="flex items-center gap-1 rounded-md px-2 py-1 text-xs font-bold min-w-[2rem] h-5 shadow-sm flex-shrink-0"
                        :style="{
                          backgroundColor: leg.route_color,
                          color: leg.route_text_color,
                        }"
                      >
                        <MotisTransitModeIcon :mode="leg.mode" class="w-3 h-3" variant="inherit" />
                        {{ leg.route_short_name }}
                      </div>
                      <!-- Non-transit or transit without route info -->
                      <MotisTransitModeIcon v-else :mode="leg.mode" class="w-5 h-5" variant="inherit" />
                    </template>
                  </div>
                  <!-- Fade-out gradient for overflow -->
                  <div
                    v-show="showTransitOverflow"
                    class="absolute right-0 top-0 bottom-0 w-8 bg-gradient-to-l from-zinc-50 dark:from-zinc-900 to-transparent pointer-events-none z-10"
                  ></div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Pagination Controls - Bottom -->
        <div class="mt-4">
          <MotisPaginationControls
            :previous-page-cursor="data.previous_page_cursor"
            :next-page-cursor="data.next_page_cursor"
            size="lg"
            v-model:page-cursor="pageCursor"
          />
        </div>
      </div>
    </div>

    <!-- Detail View -->
    <div v-else-if="viewMode === 'details'">
      <!-- Back to summary button -->
      <div class="mb-4">
        <button @click="backToSummary" class="flex items-center gap-2 text-blue-600 dark:text-blue-300 hover:text-blue-700 dark:hover:text-blue-200 font-medium">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
          {{ t("back_to_routes") }}
        </button>
      </div>

      <!-- Show selected itinerary details -->
      <div v-if="selectedItineraryIndex !== null">
        <!-- Direct connection details -->
        <div v-if="selectedItineraryIndex < 0">
          <template v-if="data.direct[Math.abs(selectedItineraryIndex + 1)]">
            <!-- The negative `-1 - i` index tells the page this is a direct connection. -->
            <MotisDirectConnections
              :connections="[data.direct[Math.abs(selectedItineraryIndex + 1)]!]"
              @select-leg="
                (_, legIndex) => selectedItineraryIndex !== null && emit('selectLeg', selectedItineraryIndex, legIndex)
              "
              @select-step="
                (_, legIndex, stepIndex) =>
                  selectedItineraryIndex !== null && emit('selectStep', selectedItineraryIndex, legIndex, stepIndex)
              "
            />
          </template>
        </div>

        <!-- Transit itinerary details -->
        <div v-else-if="selectedItineraryIndex >= 0">
          <template v-if="data.itineraries[selectedItineraryIndex]">
            <MotisTransitItinerary
              :itinerary="data.itineraries[selectedItineraryIndex]!"
              :itinerary-index="selectedItineraryIndex"
              @select-leg="(itineraryIndex, legIndex) => emit('selectLeg', itineraryIndex, legIndex)"
              @select-step="(itineraryIndex, legIndex, stepIndex) => emit('selectStep', itineraryIndex, legIndex, stepIndex)"
              @select-itinerary="emit('selectItinerary', $event)"
            />
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
.scrollbar-hide::-webkit-scrollbar {
  display: none;
}
</style>

<i18n lang="yaml">
de:
  direct_connections: Direkte Verbindungen
  back_to_routes: Zurück zur Übersicht
  direct: Direkt
  legs: "{count} Abschnitte"
  transfers: "keine Umstiege | ein Umstieg | {count} Umstiege"

en:
  direct_connections: Direct connections
  back_to_routes: Back to routes overview
  direct: Direct
  legs: "{count} segments"
  transfers: "no transfers | one transfer | {count} transfers"
</i18n>
