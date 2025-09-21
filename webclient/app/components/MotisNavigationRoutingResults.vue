<script setup lang="ts">
import { mdiChevronLeft, mdiChevronRight, mdiRefresh } from "@mdi/js";
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

// Helper function to calculate delay in minutes
const calculateDelay = (scheduledTime: string | null, actualTime: string) => {
  if (!scheduledTime) return null;
  const scheduled = new Date(scheduledTime);
  const actual = new Date(actualTime);
  return Math.round((actual.getTime() - scheduled.getTime()) / (1000 * 60));
};

// Helper function to format delay
const formatDelay = (delayMinutes: number) => {
  if (delayMinutes === 0) return null;
  if (delayMinutes > 0) {
    return `+${delayMinutes}`;
  }
  return `${delayMinutes}`;
};

// Check if pagination controls should be shown
const showPagination = computed(() => {
  return props.data && (props.data.next_page_cursor || props.data.previous_page_cursor);
});

// Handle pagination
const handlePreviousPage = () => {
  if (props.data?.previous_page_cursor) {
    emit("loadPrevious", props.data.previous_page_cursor);
  }
};

const handleNextPage = () => {
  if (props.data?.next_page_cursor) {
    emit("loadNext", props.data.next_page_cursor);
  }
};
</script>

<template>
  <div>
    <!-- Loading State -->
    <div v-if="loading" class="py-12 text-center">
      <div class="inline-flex items-center justify-center">
        <Spinner class="h-8 w-8 text-blue-600" />
      </div>
      <p class="text-zinc-600 mt-4 text-lg">{{ t("loading_routes") }}</p>
      <p class="text-zinc-500 mt-2 text-sm">{{ t("searching_connections") }}</p>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="py-12 text-center">
      <div class="text-red-500 mb-4">
        <svg class="mx-auto h-16 w-16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="1.5"
            d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z"
          />
        </svg>
      </div>
      <h3 class="text-zinc-900 mb-2 text-lg font-semibold">{{ t("error_title") }}</h3>
      <p class="text-zinc-600 mb-6 text-sm max-w-md mx-auto">{{ error }}</p>
      <button
        @click="emit('retry')"
        class="inline-flex items-center px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-lg hover:bg-blue-700 transition-colors"
      >
        <MdiIcon :path="mdiRefresh" :size="16" class="mr-2" />
        {{ t("retry_search") }}
      </button>
    </div>

    <!-- Success State - Show Results -->
    <div v-else-if="data">
      <!-- Direct connections (walking, biking, etc.) -->
      <div v-if="data.direct && data.direct.length > 0" class="mb-6">
        <h3 class="text-zinc-700 mb-3 text-lg font-semibold">
          {{ t("direct_connections") }}
        </h3>
        <div v-for="(itinerary, i) in data.direct" :key="`direct-${i}`" class="bg-zinc-50 mb-3 rounded-lg border p-4">
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

      <!-- Public Transit Itineraries -->
      <div v-if="data.itineraries && data.itineraries.length > 0">
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-zinc-700 text-lg font-semibold">
            {{ t("transit_connections") }}
          </h3>
          <!-- Pagination Controls - Top -->
          <div v-if="showPagination" class="flex items-center gap-2">
            <button
              :disabled="!data.previous_page_cursor || loading"
              @click="handlePreviousPage"
              class="inline-flex items-center px-2 py-1 text-xs font-medium text-zinc-600 bg-zinc-100 rounded hover:bg-zinc-200 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              <MdiIcon :path="mdiChevronLeft" :size="14" class="mr-1" />
              {{ t("earlier") }}
            </button>
            <button
              :disabled="!data.next_page_cursor || loading"
              @click="handleNextPage"
              class="inline-flex items-center px-2 py-1 text-xs font-medium text-zinc-600 bg-zinc-100 rounded hover:bg-zinc-200 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {{ t("later") }}
              <MdiIcon :path="mdiChevronRight" :size="14" class="ml-1" />
            </button>
          </div>
        </div>

        <div v-for="(itinerary, i) in data.itineraries" :key="`itinerary-${i}`" class="mb-6 rounded-lg border">
          <!-- Itinerary Header -->
          <div
            class="bg-zinc-50 flex items-center justify-between rounded-t-lg p-4 cursor-pointer hover:bg-zinc-100 transition-colors"
            @click="emit('selectItinerary', i)"
            :title="`${t('select_itinerary')} ${i + 1}`"
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
            <div
              v-for="(leg, j) in itinerary.legs"
              :key="`leg-${j}`"
              class="group cursor-pointer p-4 transition-colors hover:bg-zinc-50"
              @click="emit('selectLeg', i, j)"
            >
              <div class="flex items-start gap-3">
                <!-- Mode Icon -->
                <div class="mt-1">
                  <MotisTransitModeIcon :mode="leg.mode" />
                </div>

                <!-- Leg Details -->
                <div class="flex-grow">
                  <!-- Route Info -->
                  <div v-if="leg.route_short_name" class="mb-1 flex items-center gap-2">
                    <span
                      class="rounded px-2 py-1 text-sm font-bold text-white"
                      :style="{
                        backgroundColor: leg.route_color || '#3b82f6',
                        color: leg.route_text_color || 'white',
                      }"
                    >
                      {{ leg.route_short_name }}
                    </span>
                    <span v-if="leg.headsign" class="text-zinc-600 text-sm">
                      {{ leg.headsign }}
                    </span>
                  </div>

                  <!-- From/To -->
                  <div class="space-y-1">
                    <div class="flex items-center justify-between">
                      <div class="flex-grow">
                        <span class="text-zinc-900 font-medium">{{ leg.from.name }}</span>
                        <span v-if="leg.from.track" class="text-zinc-600 ml-2 text-sm">
                          {{ t("platform") }} {{ leg.from.track }}
                        </span>
                      </div>
                      <div class="text-zinc-500 text-sm flex items-center gap-1">
                        {{ formatTime(leg.start_time) }}
                        <span
                          v-if="calculateDelay(leg.scheduled_start_time, leg.start_time)"
                          :class="{
                            'text-red-600': calculateDelay(leg.scheduled_start_time, leg.start_time)! > 0,
                            'text-green-600': calculateDelay(leg.scheduled_start_time, leg.start_time)! < 0,
                          }"
                          class="text-xs font-medium"
                        >
                          {{ formatDelay(calculateDelay(leg.scheduled_start_time, leg.start_time)!) }}
                        </span>
                        <span v-if="leg.real_time" class="text-green-600 ml-1">●</span>
                      </div>
                    </div>
                    <div class="flex items-center justify-between">
                      <div class="flex-grow">
                        <span class="text-zinc-900 font-medium">{{ leg.to.name }}</span>
                        <span v-if="leg.to.track" class="text-zinc-600 ml-2 text-sm">
                          {{ t("platform") }} {{ leg.to.track }}
                        </span>
                      </div>
                      <div class="text-zinc-500 text-sm flex items-center gap-1">
                        {{ formatTime(leg.end_time) }}
                        <span
                          v-if="calculateDelay(leg.scheduled_end_time, leg.end_time)"
                          :class="{
                            'text-red-600': calculateDelay(leg.scheduled_end_time, leg.end_time)! > 0,
                            'text-green-600': calculateDelay(leg.scheduled_end_time, leg.end_time)! < 0,
                          }"
                          class="text-xs font-medium"
                        >
                          {{ formatDelay(calculateDelay(leg.scheduled_end_time, leg.end_time)!) }}
                        </span>
                        <span v-if="leg.real_time" class="text-green-600 ml-1">●</span>
                      </div>
                    </div>
                  </div>

                  <!-- Duration and Distance -->
                  <div class="text-zinc-600 mt-1 text-sm">
                    {{ formatDuration(leg.duration) }}
                    <span v-if="leg.distance"> • {{ formatDistance(leg.distance) }}</span>
                  </div>

                  <!-- Alerts -->
                  <div v-if="leg.alerts && leg.alerts.length > 0" class="mt-2">
                    <div
                      v-for="(alert, k) in leg.alerts"
                      :key="`alert-${k}`"
                      class="bg-amber-50 text-amber-800 rounded p-2 text-sm"
                    >
                      {{ alert.header_text }}
                    </div>
                  </div>

                  <!-- Cancelled indicator -->
                  <div v-if="leg.cancelled" class="text-red-600 mt-1 text-sm font-medium">
                    {{ t("cancelled") }}
                  </div>

                  <!-- Intermediate stops -->
                  <div v-if="leg.intermediate_stops && leg.intermediate_stops.length > 0" class="mt-2">
                    <details class="group">
                      <summary class="text-zinc-500 cursor-pointer text-sm hover:text-zinc-700 flex items-center gap-2">
                        {{ t("intermediate_stops", leg.intermediate_stops.length) }}
                        <span class="text-zinc-400 text-xs group-open:hidden">{{ t("show_stops") }}</span>
                        <span class="text-zinc-400 text-xs hidden group-open:inline">{{ t("hide_stops") }}</span>
                      </summary>
                      <div class="mt-2 space-y-1 pl-4 border-l-2 border-zinc-200">
                        <div
                          v-for="(stop, k) in leg.intermediate_stops"
                          :key="`stop-${k}`"
                          class="relative flex items-center justify-between py-2 text-sm group hover:bg-zinc-50 rounded px-2 -mx-2"
                        >
                          <!-- Stop indicator dot -->
                          <div
                            class="absolute -left-[17px] w-2 h-2 bg-zinc-300 rounded-full group-hover:bg-zinc-400"
                          ></div>

                          <div class="flex-grow">
                            <div class="text-zinc-800 font-medium">{{ stop.name }}</div>
                            <div class="text-zinc-500 text-xs mt-0.5 flex items-center gap-2">
                              <span v-if="stop.track">{{ t("platform") }} {{ stop.track }}</span>
                              <span v-if="stop.level && stop.level !== 0" class="text-zinc-400">
                                {{ t("level") }} {{ stop.level }}
                              </span>
                            </div>
                          </div>

                          <div class="text-zinc-500 text-xs flex flex-col items-end gap-0.5">
                            <!-- Departure time with delay -->
                            <div v-if="stop.departure" class="flex items-center gap-1">
                              <span>{{ formatTime(stop.departure) }}</span>
                              <span
                                v-if="
                                  stop.scheduled_departure && calculateDelay(stop.scheduled_departure, stop.departure)
                                "
                                :class="{
                                  'text-red-600': calculateDelay(stop.scheduled_departure, stop.departure)! > 0,
                                  'text-green-600': calculateDelay(stop.scheduled_departure, stop.departure)! < 0,
                                }"
                                class="text-xs font-medium"
                              >
                                {{ formatDelay(calculateDelay(stop.scheduled_departure, stop.departure)!) }}
                              </span>
                            </div>

                            <!-- Arrival time with delay (if no departure) -->
                            <div v-else-if="stop.arrival" class="flex items-center gap-1">
                              <span>{{ formatTime(stop.arrival) }}</span>
                              <span
                                v-if="stop.scheduled_arrival && calculateDelay(stop.scheduled_arrival, stop.arrival)"
                                :class="{
                                  'text-red-600': calculateDelay(stop.scheduled_arrival, stop.arrival)! > 0,
                                  'text-green-600': calculateDelay(stop.scheduled_arrival, stop.arrival)! < 0,
                                }"
                                class="text-xs font-medium"
                              >
                                {{ formatDelay(calculateDelay(stop.scheduled_arrival, stop.arrival)!) }}
                              </span>
                            </div>

                            <!-- Cancelled indicator -->
                            <span v-if="stop.cancelled" class="text-red-600 text-xs font-medium bg-red-50 px-1 rounded">
                              {{ t("cancelled") }}
                            </span>
                          </div>
                        </div>
                      </div>
                    </details>
                  </div>

                  <!-- Walking Instructions -->
                  <div v-if="leg.steps && leg.steps.length > 0" class="mt-3">
                    <details class="group">
                      <summary
                        class="text-zinc-600 cursor-pointer text-sm font-medium hover:text-zinc-800 flex items-center gap-2"
                      >
                        {{ t("walking_instructions") }}
                        <span class="text-zinc-400 text-xs">({{ leg.steps.length }} {{ t("steps") }})</span>
                      </summary>
                      <div class="mt-2 space-y-2 pl-4">
                        <div v-for="(step, k) in leg.steps" :key="`step-${k}`" class="flex items-start gap-3 py-1">
                          <WalkingDirectionIcon :direction="step.relative_direction" />
                          <div class="flex-grow text-sm">
                            <div class="text-zinc-900">
                              <span v-if="step.street_name">{{ step.street_name }}</span>
                              <span v-else>{{ t("continue") }}</span>
                            </div>
                            <div class="text-zinc-600 text-xs flex items-center gap-2">
                              {{ formatDistance(step.distance) }}
                              <span v-if="step.elevation_up || step.elevation_down" class="text-zinc-500">
                                <span v-if="step.elevation_up">↗ {{ step.elevation_up }}m</span>
                                <span v-if="step.elevation_down">↘ {{ step.elevation_down }}m</span>
                              </span>
                              <span v-if="step.toll" class="text-amber-600">{{ t("toll") }}</span>
                            </div>
                          </div>
                        </div>
                      </div>
                    </details>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Pagination Controls - Bottom -->
        <div v-if="showPagination" class="flex justify-center gap-4 mt-6 pt-4 border-t border-zinc-200">
          <button
            :disabled="!data.previous_page_cursor || loading"
            @click="handlePreviousPage"
            class="inline-flex items-center px-4 py-2 text-sm font-medium text-zinc-700 bg-white border border-zinc-300 rounded-lg hover:bg-zinc-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            <MdiIcon :path="mdiChevronLeft" :size="16" class="mr-2" />
            {{ t("load_earlier_connections") }}
          </button>
          <button
            :disabled="!data.next_page_cursor || loading"
            @click="handleNextPage"
            class="inline-flex items-center px-4 py-2 text-sm font-medium text-zinc-700 bg-white border border-zinc-300 rounded-lg hover:bg-zinc-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {{ t("load_later_connections") }}
            <MdiIcon :path="mdiChevronRight" :size="16" class="ml-2" />
          </button>
        </div>
      </div>

      <!-- No results -->
      <div v-else-if="!data.direct?.length && !data.itineraries?.length" class="py-12 text-center">
        <div class="text-zinc-400 mb-4">
          <svg class="mx-auto h-16 w-16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="1.5"
              d="M9.75 9.75l4.5 4.5m0-4.5l-4.5 4.5M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        </div>
        <h3 class="text-zinc-700 mb-2 text-lg font-semibold">{{ t("no_routes_found") }}</h3>
        <p class="text-zinc-500 mb-6 text-sm max-w-md mx-auto">{{ t("no_routes_description") }}</p>
        <button
          @click="emit('retry')"
          class="inline-flex items-center px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-lg hover:bg-blue-700 transition-colors"
        >
          <MdiIcon :path="mdiRefresh" :size="16" class="mr-2" />
          {{ t("try_again") }}
        </button>
      </div>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  loading_routes: Routen werden geladen
  searching_connections: Suche nach Verbindungen...
  error_title: Fehler beim Laden
  retry_search: Erneut versuchen
  direct_connections: Direkte Verbindungen
  transit_connections: Öffentliche Verkehrsmittel
  transfers: "keine Umstiege | ein Umstieg | {count} Umstiege"
  platform: Gleis
  level: Ebene
  cancelled: Ausfall
  intermediate_stops: "keine Zwischenstopps | ein Zwischenstopp | {count} Zwischenstopps"
  show_stops: anzeigen
  hide_stops: ausblenden
  walking_instructions: Wegbeschreibung
  steps: Schritte
  continue: Weiter
  toll: Maut
  minutes: "sofort | eine Minute | {count} Minuten"
  seconds: "sofort | eine Sekunde | {count} Sekunden"
  meters: "hier | einen Meter | {count} Meter"
  kilometers: "{0} Kilometer"
  earlier: Früher
  later: Später
  load_earlier_connections: Frühere Verbindungen laden
  load_later_connections: Spätere Verbindungen laden
  no_routes_found: Keine Routen gefunden
  no_routes_description: Es konnten keine Verbindungen für Ihre Suchanfrage gefunden werden. Versuchen Sie andere Parameter oder eine andere Zeit.
  try_again: Erneut versuchen
  select_itinerary: Route auswählen

en:
  loading_routes: Loading routes
  searching_connections: Searching for connections...
  error_title: Error loading routes
  retry_search: Try again
  direct_connections: Direct connections
  transit_connections: Public transport
  transfers: "no transfers | one transfer | {count} transfers"
  platform: Platform
  level: Level
  cancelled: Cancelled
  intermediate_stops: "no intermediate stops | one intermediate stop | {count} intermediate stops"
  show_stops: show
  hide_stops: hide
  walking_instructions: Walking directions
  steps: steps
  continue: Continue
  toll: Toll
  minutes: "instant | one minute | {count} minutes"
  seconds: "instant | one second | {count} seconds"
  meters: "here | one meter | {count} meters"
  kilometers: "{0} kilometers"
  earlier: Earlier
  later: Later
  load_earlier_connections: Load earlier connections
  load_later_connections: Load later connections
  no_routes_found: No routes found
  no_routes_description: No connections could be found for your search query. Try different parameters or a different time.
  try_again: Try again
  select_itinerary: Select route
</i18n>
