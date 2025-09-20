<script setup lang="ts">
import type { components } from "~/api_types";

type MotisRoutingResponse = components["schemas"]["MotisRoutingResponse"];
defineProps<{ data: MotisRoutingResponse }>();
const emit = defineEmits<{
  selectLeg: [legIndex: number];
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
const calculateDelay = (scheduledTime: string, actualTime: string) => {
  const scheduled = new Date(scheduledTime);
  const actual = new Date(actualTime);
  const diffMs = actual.getTime() - scheduled.getTime();
  const diffMinutes = Math.round(diffMs / (1000 * 60));
  return diffMinutes;
};

// Helper function to format delay
const formatDelay = (delayMinutes: number) => {
  if (delayMinutes === 0) return "";
  if (delayMinutes > 0) {
    return `+${delayMinutes}`;
  }
  return `-${delayMinutes}`;
};
</script>

<template>
  <div>
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
                        v-if="leg.scheduled_start_time"
                        :class="{
                          'text-red-600': calculateDelay(leg.scheduled_start_time, leg.start_time)! > 5,
                          'text-orange-600': calculateDelay(leg.scheduled_start_time, leg.start_time)! < 5,
                          'text-green-600': [-1, 0, 1].includes(
                            calculateDelay(leg.scheduled_start_time, leg.start_time),
                          ),
                          'text-fuchsia-pink-600': calculateDelay(leg.scheduled_start_time, leg.start_time)! < 0,
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
                </div>
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
      {{ t("no_routes_found") }}
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
</i18n>
