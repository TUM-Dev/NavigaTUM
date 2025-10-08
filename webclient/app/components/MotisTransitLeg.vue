<script setup lang="ts">
import { mdiTransitConnectionVariant } from "@mdi/js";
import type { components } from "~/api_types";

type MotisRoutingResponse = components["schemas"]["MotisRoutingResponse"];
type Itinerary = NonNullable<MotisRoutingResponse["itineraries"]>[0];
type Leg = Itinerary["legs"][0];

interface Props {
  leg: Leg;
  itinerary: Itinerary;
  legIndex: number;
  itineraryIndex: number;
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

// Helper function to safely get previous leg info for interline display
const getPreviousLegInfo = (currentIndex: number) => {
  if (currentIndex <= 0 || !props.itinerary.legs[currentIndex - 1]) {
    return null;
  }
  return props.itinerary.legs[currentIndex - 1];
};

const previousLeg = computed(() => getPreviousLegInfo(props.legIndex));
</script>

<template>
  <div
    class="group cursor-pointer p-4 transition-colors hover:bg-zinc-50"
    @click="emit('selectLeg', itineraryIndex, legIndex)"
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
              backgroundColor: leg.route_color,
              color: leg.route_text_color,
            }"
          >
            {{ leg.route_short_name }}
          </span>
          <span v-if="leg.headsign" class="text-zinc-600 text-sm">
            {{ leg.headsign }}
          </span>
        </div>

        <!-- Interline Information -->
        <div v-if="leg.interline_with_previous_leg" class="mb-2">
          <div
            class="flex items-start gap-2 bg-blue-50 text-blue-800 px-3 py-2 rounded-md border border-blue-200"
          >
            <MdiIcon
              :path="mdiTransitConnectionVariant"
              :size="16"
              class="text-blue-600 mt-0.5 flex-shrink-0"
            />
            <div class="flex-grow">
              <div class="font-medium text-sm">{{ t("stay_on_vehicle") }}</div>
              <div class="text-blue-700 text-xs mt-2">
                <div
                  v-if="
                    previousLeg?.route_short_name &&
                    leg.route_short_name &&
                    previousLeg.route_short_name !== leg.route_short_name
                  "
                  class="flex items-center gap-2"
                >
                  <span class="text-blue-700">{{ t("route_becomes_short") }}:</span>
                  <span class="rounded px-1.5 py-0.5 text-xs font-bold text-white bg-blue-600">
                    {{ previousLeg.route_short_name }}
                  </span>
                  <span class="text-blue-600">→</span>
                  <span
                    class="rounded px-1.5 py-0.5 text-xs font-bold text-white"
                    :style="{
                      backgroundColor: leg.route_color,
                      color: leg.route_text_color,
                    }"
                  >
                    {{ leg.route_short_name }}
                  </span>
                </div>
                <div v-else>
                  {{ t("interline_explanation") }}
                </div>
              </div>
            </div>
          </div>
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

        <!-- Departure Stop Alerts -->
        <MotisAlertList
          v-if="leg.from.alerts && leg.from.alerts.length > 0"
          :alerts="leg.from.alerts"
          :title="t('departure_stop_alerts')"
          size="sm"
          class="mt-2"
        />

        <!-- Arrival Stop Alerts -->
        <MotisAlertList
          v-if="leg.to.alerts && leg.to.alerts.length > 0"
          :alerts="leg.to.alerts"
          :title="t('arrival_stop_alerts')"
          size="sm"
          class="mt-2"
        />

        <!-- Leg Alerts -->
        <MotisAlertList v-if="leg.alerts && leg.alerts.length > 0" :alerts="leg.alerts" class="mt-2" />

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

                <!-- Stop-level alerts -->
                <MotisAlertList
                  v-if="stop.alerts && stop.alerts.length > 0"
                  :alerts="stop.alerts"
                  size="sm"
                  :show-cause-effect="false"
                  class="mt-1"
                />
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
                    <span v-if="step.toll" class="text-orange-600">{{ t("toll") }}</span>
                  </div>
                </div>
              </div>
            </div>
          </details>
        </div>
      </div>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
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
  departure_stop_alerts: Abfahrtshaltestelle
  arrival_stop_alerts: Ankunftshaltestelle
  stay_on_vehicle: Im Fahrzeug bleiben
  interline_explanation: Das Fahrzeug ändert seine Route, aber du musst nicht umsteigen
  route_becomes_short: Linie wechselt

en:
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
  departure_stop_alerts: Departure stop
  arrival_stop_alerts: Arrival stop
  stay_on_vehicle: Stay on vehicle
  interline_explanation: The vehicle changes route, but you don't need to transfer
  route_becomes_short: Line changes
</i18n>
