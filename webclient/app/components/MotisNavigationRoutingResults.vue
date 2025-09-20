<script setup lang="ts">
import type { components } from "~/api_types";

type MotisRoutingResponse = components["schemas"]["MotisRoutingResponse"];

interface Props {
  data?: MotisRoutingResponse;
  loading?: boolean;
  error?: string | null;
}

defineProps<Props>();
const emit = defineEmits<{
  selectLeg: [legIndex: number];
  retry: [];
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
  } else if (seconds >= 60) {
    return t("minutes", Math.ceil(seconds / 60));
  } else {
    return t("seconds", seconds);
  }
};

// Helper function to format distance
const formatDistance = (meters: number) => {
  if (meters >= 1000) {
    return t("kilometers", [(meters / 1000).toFixed(1)]);
  } else {
    return t("meters", Math.round(meters));
  }
};

// Helper function to calculate delay in minutes
const calculateDelay = (scheduledTime: string | null, actualTime: string) => {
  if (!scheduledTime) return null;
  const scheduled = new Date(scheduledTime);
  const actual = new Date(actualTime);
  const diffMs = actual.getTime() - scheduled.getTime();
  const diffMinutes = Math.round(diffMs / (1000 * 60));
  return diffMinutes;
};

// Helper function to format delay
const formatDelay = (delayMinutes: number) => {
  if (delayMinutes === 0) return null;
  if (delayMinutes > 0) {
    return `+${delayMinutes}`;
  } else {
    return `${delayMinutes}`;
  }
};
</script>

<template>
  <div>
<<<<<<< HEAD
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
        <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
          />
        </svg>
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
            @click="emit('selectLeg', j)"
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
        <h3 class="text-zinc-700 mb-3 text-lg font-semibold">
          {{ t("transit_connections") }}
        </h3>
        <div v-for="(itinerary, i) in data.itineraries" :key="`itinerary-${i}`" class="mb-6 rounded-lg border">
          <!-- Itinerary Header -->
          <div class="bg-zinc-50 flex items-center justify-between rounded-t-lg p-4">
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
              @click="emit('selectLeg', j)"
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
=======
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
          @click="emit('selectLeg', j)"
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
      <h3 class="text-zinc-700 mb-3 text-lg font-semibold">
        {{ t("transit_connections") }}
      </h3>
      <div v-for="(itinerary, i) in data.itineraries" :key="`itinerary-${i}`" class="mb-6 rounded-lg border">
        <!-- Itinerary Header -->
        <div class="bg-zinc-50 flex items-center justify-between rounded-t-lg p-4">
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
            @click="emit('selectLeg', j)"
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

                <!-- Intermediate stops indicator -->
                <div
                  v-if="leg.intermediate_stops && leg.intermediate_stops.length > 0"
                  class="text-zinc-500 mt-1 text-sm"
                >
                  {{ t("intermediate_stops", leg.intermediate_stops.length) }}
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
>>>>>>> main
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- No results message -->
      <div
        v-if="(!data.itineraries || data.itineraries.length === 0) && (!data.direct || data.direct.length === 0)"
        class="text-zinc-500 py-8 text-center"
      >
        <div class="text-zinc-400 mb-4">
          <svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="1.5"
              d="M9.172 16.172a4 4 0 015.656 0M9 12h6m-6-4h6m2 5.291A7.962 7.962 0 0112 15c-2.34 0-4.29-1.007-5.824-2.696"
            />
          </svg>
        </div>
        <h3 class="text-zinc-700 mb-2 font-medium">{{ t("no_routes_found") }}</h3>
        <p class="text-zinc-500 text-sm">{{ t("try_different_search") }}</p>
      </div>
    </div>

    <!-- Fallback state (should not normally happen) -->
    <div v-else class="py-8 text-center text-zinc-500">
      {{ t("no_data") }}
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  minutes: "sofort | eine Minute | {count} Minuten"
  seconds: "sofort | eine Sekunde | {count} Sekunden"
  meters: "hier | einen Meter | {count} Meter"
  kilometers: "{0} Kilometer"
  direct_connections: "Direkte Verbindungen"
  transit_connections: "ÖPNV-Verbindungen"
  transfers: "keine Umstiege | ein Umstieg | {count} Umstiege"
  cancelled: "Ausgefallen"
  intermediate_stops: "keine Zwischenstopps | ein Zwischenstopp | {count} Zwischenstopps"
  no_routes_found: "Keine Routen gefunden"
  platform: "Gleis"
  walking_instructions: "Wegbeschreibung"
  steps: "Schritte"
  continue: "Geradeaus"
  toll: "Maut"
  show_stops: "Haltestellen anzeigen"
  hide_stops: "Haltestellen ausblenden"
  level: "Ebene"
  loading_routes: "Routen werden gesucht..."
  searching_connections: "Verbindungen werden berechnet"
  error_title: "Fehler beim Laden"
  retry_search: "Erneut versuchen"
  try_different_search: "Versuchen Sie eine andere Suche"
  no_data: "Keine Daten verfügbar"
en:
  minutes: "instant | one minute | {count} minutes"
  seconds: "instant | one second | {count} seconds"
  meters: "here | one meter | {count} meters"
  kilometers: "{0} kilometers"
  direct_connections: "Direct Connections"
  transit_connections: "Public Transit Connections"
  transfers: "no transfers | one transfer | {count} transfers"
  cancelled: "Cancelled"
  intermediate_stops: "no intermediate stops | one intermediate stop | {count} intermediate stops"
  no_routes_found: "No routes found"
  platform: "Platform"
  walking_instructions: "Walking directions"
  steps: "steps"
  continue: "Continue"
  toll: "Toll"
  show_stops: "Show stops"
  hide_stops: "Hide stops"
  level: "Level"
  loading_routes: "Finding routes..."
  searching_connections: "Calculating connections"
  error_title: "Error loading routes"
  retry_search: "Try again"
  try_different_search: "Try a different search"
  no_data: "No data available"
</i18n>
