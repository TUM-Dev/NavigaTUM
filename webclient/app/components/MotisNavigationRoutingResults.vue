<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, watch } from "vue";
import type { components } from "~/api_types";

type MotisRoutingResponse = components["schemas"]["MotisRoutingResponse"];
type ModeResponse = components["schemas"]["ModeResponse"];

const pageCursor = defineModel<string | undefined>("pageCursor", {
  required: true,
});
const props = defineProps<{
  data: MotisRoutingResponse;
}>();
const emit = defineEmits<{
  selectLeg: [itineraryIndex: number, legIndex: number];
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

// Check for overflow after content updates
const checkOverflow = () => {
  nextTick(() => {
    if (transitContainer.value) {
      showDirectOverflow.value = transitContainer.value.scrollWidth > transitContainer.value.clientWidth;
    }
    if (transitContainer2.value) {
      showTransitOverflow.value = transitContainer2.value.scrollWidth > transitContainer2.value.clientWidth;
    }
  });
};

// Watch for data changes to recheck overflow
watch(() => props.data, checkOverflow, { deep: true, flush: "post" });

// Setup overflow detection
onMounted(() => {
  checkOverflow();

  // Recheck on window resize
  window.addEventListener("resize", checkOverflow);

  // Cleanup on unmount
  onUnmounted(() => {
    window.removeEventListener("resize", checkOverflow);
  });
});

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

// Handle leg selection - this should show details if not already showing
const handleLegSelect = (itineraryIndex: number, legIndex: number) => {
  if (viewMode.value === "summary") {
    showItineraryDetails(itineraryIndex);
  }
  emit("selectLeg", itineraryIndex, legIndex);
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
const getTransitLegs = (itinerary: {
  legs?: readonly {
    mode?: ModeResponse;
    route_short_name?: string | null;
    route_color?: string | null;
    route_text_color?: string | null;
  }[];
}) => {
  const transitLegs: Array<{
    mode: ModeResponse;
    route_short_name?: string | null;
    route_color?: string | null;
    route_text_color?: string | null;
  }> = [];

  itinerary.legs?.forEach((leg) => {
    if (leg.mode) {
      const newLeg = {
        mode: leg.mode,
        route_short_name: ["bus", "tram", "subway", "metro", "rail", "coach"].includes(leg.mode)
          ? leg.route_short_name
          : undefined,
        route_color: ["bus", "tram", "subway", "metro", "rail", "coach"].includes(leg.mode)
          ? leg.route_color
          : undefined,
        route_text_color: ["bus", "tram", "subway", "metro", "rail", "coach"].includes(leg.mode)
          ? leg.route_text_color
          : undefined,
      };

      // Deduplicate consecutive similar legs (same mode and route)
      const lastLeg = transitLegs[transitLegs.length - 1];
      const isDuplicate =
        lastLeg && lastLeg.mode === newLeg.mode && lastLeg.route_short_name === newLeg.route_short_name;

      if (!isDuplicate) {
        transitLegs.push(newLeg);
      }
    }
  });

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
        <h3 class="text-zinc-700 mb-3 text-lg font-semibold">
          {{ t("direct_connections") }}
        </h3>
        <div class="space-y-2 w-full max-w-full">
          <div
            v-for="(connection, i) in data.direct"
            :key="`direct-${i}`"
            class="bg-zinc-50 hover:bg-zinc-100 cursor-pointer rounded-lg border p-4 transition-colors w-full max-w-full"
            @click="handleItinerarySelect(-1 - i)"
          >
            <div class="flex items-center justify-between w-full min-w-0">
              <div class="flex flex-col gap-2 min-w-0 flex-1">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-4 min-w-0">
                    <span class="text-zinc-900 font-medium truncate">
                      {{ formatTime(connection.start_time) }} - {{ formatTime(connection.end_time) }}
                    </span>
                    <span class="text-zinc-600 flex-shrink-0">
                      {{ formatDuration(connection.duration) }}
                    </span>
                  </div>
                  <div class="text-zinc-500 text-sm flex-shrink-0">
                    {{ connection.legs.length === 1 ? t("direct") : t("legs", connection.legs.length) }}
                  </div>
                </div>
                <!-- Transit summary below time -->
                <div class="relative">
                  <div ref="transitContainer" class="flex items-center gap-1 overflow-x-auto scrollbar-hide">
                    <template v-for="(leg, legIndex) in getTransitLegs(connection)" :key="`${leg.mode}-${legIndex}`">
                      <!-- Separator arrow between legs -->
                      <svg
                        v-if="legIndex > 0"
                        class="w-3 h-3 text-zinc-400 mx-1 flex-shrink-0"
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
                          backgroundColor: leg.route_color ? `#${leg.route_color}` : '#3B82F6',
                          color: leg.route_text_color ? `#${leg.route_text_color}` : '#FFFFFF',
                        }"
                      >
                        <MotisTransitModeIcon :mode="leg.mode" class="w-3 h-3" transparent />
                        {{ leg.route_short_name }}
                      </div>
                      <!-- Non-transit or transit without route info -->
                      <MotisTransitModeIcon v-else :mode="leg.mode" class="w-5 h-5 flex-shrink-0" transparent />
                    </template>
                  </div>
                  <!-- Fade-out gradient for overflow -->
                  <div
                    v-show="showDirectOverflow"
                    class="absolute right-0 top-0 bottom-0 w-8 bg-gradient-to-l from-zinc-50 to-transparent pointer-events-none z-10"
                  ></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Transit itineraries summary -->
      <div v-if="data.itineraries && data.itineraries.length > 0">
        <div class="flex items-center justify-end mb-3">
          <!-- Pagination Controls - Top -->
          <MotisPaginationControls
            :previous-page-cursor="data.previous_page_cursor"
            :next-page-cursor="data.next_page_cursor"
            :loading="false"
            size="sm"
            @load-previous="emit('loadPrevious', $event)"
            @load-next="emit('loadNext', $event)"
          />
        </div>

        <div class="space-y-2 w-full max-w-full">
          <div
            v-for="(itinerary, i) in data.itineraries"
            :key="`summary-${i}`"
            class="bg-zinc-50 hover:bg-zinc-100 cursor-pointer rounded-lg border p-4 transition-colors w-full max-w-full"
            @click="handleItinerarySelect(i)"
          >
            <div class="flex items-center justify-between w-full min-w-0">
              <div class="flex flex-col gap-2 min-w-0 flex-1">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-4 min-w-0">
                    <span class="text-zinc-900 font-medium truncate">
                      {{ formatTime(itinerary.start_time) }} - {{ formatTime(itinerary.end_time) }}
                    </span>
                    <span class="text-zinc-600 flex-shrink-0">
                      {{ formatDuration(itinerary.duration) }}
                    </span>
                  </div>
                  <div class="text-zinc-500 text-sm flex-shrink-0">
                    {{ t("transfers", itinerary.transfer_count) }}
                  </div>
                </div>
                <!-- Transit summary below time -->
                <div class="relative">
                  <div ref="transitContainer2" class="flex items-center gap-1 overflow-x-auto scrollbar-hide">
                    <template v-for="(leg, legIndex) in getTransitLegs(itinerary)" :key="`${leg.mode}-${legIndex}`">
                      <!-- Separator arrow between legs -->
                      <svg
                        v-if="legIndex > 0"
                        class="w-3 h-3 text-zinc-400 mx-1 flex-shrink-0"
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
                          backgroundColor: leg.route_color ? `#${leg.route_color}` : '#3B82F6',
                          color: leg.route_text_color ? `#${leg.route_text_color}` : '#FFFFFF',
                        }"
                      >
                        <MotisTransitModeIcon :mode="leg.mode" class="w-3 h-3" transparent />
                        {{ leg.route_short_name }}
                      </div>
                      <div class="text-black-900" v-else>
                        <!-- Non-transit or transit without route info -->
                        <MotisTransitModeIcon :mode="leg.mode" class="w-5 h-5" transparent />
                      </div>
                    </template>
                  </div>
                  <!-- Fade-out gradient for overflow -->
                  <div
                    v-show="showTransitOverflow"
                    class="absolute right-0 top-0 bottom-0 w-8 bg-gradient-to-l from-zinc-50 to-transparent pointer-events-none z-10"
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
            :loading="false"
            size="lg"
            @load-previous="emit('loadPrevious', $event)"
            @load-next="emit('loadNext', $event)"
          />
        </div>
      </div>
    </div>

    <!-- Detail View -->
    <div v-else-if="viewMode === 'details'">
      <!-- Back to summary button -->
      <div class="mb-4">
        <button @click="backToSummary" class="flex items-center gap-2 text-blue-600 hover:text-blue-700 font-medium">
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
            <MotisDirectConnections
              :connections="[data.direct[Math.abs(selectedItineraryIndex + 1)]!]"
              @select-leg="
                (itineraryIndex, legIndex) =>
                  selectedItineraryIndex !== null && emit('selectLeg', Math.abs(selectedItineraryIndex + 1), legIndex)
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
