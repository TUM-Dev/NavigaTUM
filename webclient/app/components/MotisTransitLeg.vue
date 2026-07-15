<script setup lang="ts">
import { mdiAlert, mdiTransitConnectionVariant } from "@mdi/js";
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
  selectStep: [itineraryIndex: number, legIndex: number, stepIndex: number];
}>();

const { t } = useI18n({ useScope: "local" });

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

// Helper function to safely get previous leg info for interline display
const getPreviousLegInfo = (currentIndex: number) => {
  if (currentIndex <= 0 || !props.itinerary.legs[currentIndex - 1]) {
    return null;
  }
  return props.itinerary.legs[currentIndex - 1];
};

const previousLeg = computed(() => getPreviousLegInfo(props.legIndex));

const hasRestrictedStep = computed(
  () => props.leg.steps?.some((step) => step.access_restriction) ?? false
);
</script>

<template>
  <div
    class="group cursor-pointer p-4 transition-colors hover:bg-zinc-50 dark:hover:bg-zinc-900"
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
            class="rounded px-2 py-1 text-sm font-bold"
            :style="{
              backgroundColor: leg.route_color,
              color: leg.route_text_color,
            }"
          >
            {{ leg.route_short_name }}
          </span>
          <span v-if="leg.headsign" class="text-zinc-600 dark:text-zinc-300 text-sm">
            {{ leg.headsign }}
          </span>
        </div>

        <!-- Interline Information -->
        <div v-if="leg.interline_with_previous_leg" class="mb-2">
          <div
            class="flex items-start gap-2 bg-blue-50 dark:bg-blue-900 text-blue-800 dark:text-blue-100 px-3 py-2 rounded-md border border-blue-200 dark:border-blue-700"
          >
            <MdiIcon
              :path="mdiTransitConnectionVariant"
              :size="16"
              class="text-blue-600 dark:text-blue-300 mt-0.5 flex-shrink-0"
            />
            <div class="flex-grow">
              <div class="font-medium text-sm">{{ t("stay_on_vehicle") }}</div>
              <div class="text-blue-700 dark:text-blue-200 text-xs mt-2">
                <div
                  v-if="
                    previousLeg?.route_short_name &&
                    leg.route_short_name &&
                    previousLeg.route_short_name !== leg.route_short_name
                  "
                  class="flex items-center gap-2"
                >
                  <span class="text-blue-700 dark:text-blue-200">{{ t("route_becomes_short") }}:</span>
                  <span class="rounded px-1.5 py-0.5 text-xs font-bold text-white dark:text-black bg-blue-600 dark:bg-blue-300">
                    {{ previousLeg.route_short_name }}
                  </span>
                  <span class="text-blue-600 dark:text-blue-300">→</span>
                  <span
                    class="rounded px-1.5 py-0.5 text-xs font-bold text-white dark:text-black"
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
              <span class="text-zinc-900 dark:text-zinc-50 font-medium">{{ leg.from.name }}</span>
              <span v-if="leg.from.track" class="text-zinc-600 dark:text-zinc-300 ml-2 text-sm">
                {{ t("platform") }} {{ leg.from.track }}
              </span>
            </div>
            <MotisTime
              class="text-sm"
              :scheduled="leg.scheduled_start_time"
              :actual="leg.start_time"
              :real-time="leg.real_time"
              :cancelled="leg.cancelled ?? false"
            />
          </div>
          <div class="flex items-center justify-between">
            <div class="flex-grow">
              <span class="text-zinc-900 dark:text-zinc-50 font-medium">{{ leg.to.name }}</span>
              <span v-if="leg.to.track" class="text-zinc-600 dark:text-zinc-300 ml-2 text-sm">
                {{ t("platform") }} {{ leg.to.track }}
              </span>
            </div>
            <MotisTime
              class="text-sm"
              :scheduled="leg.scheduled_end_time"
              :actual="leg.end_time"
              :real-time="leg.real_time"
              :cancelled="leg.cancelled ?? false"
            />
          </div>
        </div>

        <!-- Duration and Distance -->
        <div class="text-zinc-600 dark:text-zinc-300 mt-1 text-sm">
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
        <div v-if="leg.cancelled" class="text-red-600 dark:text-red-300 mt-1 text-sm font-medium">
          {{ t("cancelled") }}
        </div>

        <!-- Intermediate stops -->
        <div v-if="leg.intermediate_stops && leg.intermediate_stops.length > 0" class="mt-2">
          <details class="group">
            <summary class="text-zinc-500 dark:text-zinc-400 cursor-pointer text-sm hover:text-zinc-700 dark:hover:text-zinc-200 flex items-center gap-2">
              {{ t("intermediate_stops", leg.intermediate_stops.length) }}
              <span class="text-zinc-400 dark:text-zinc-500 text-xs group-open:hidden">{{ t("show_stops") }}</span>
              <span class="text-zinc-400 dark:text-zinc-500 text-xs hidden group-open:inline">{{ t("hide_stops") }}</span>
            </summary>
            <div class="mt-2 space-y-1 pl-4 border-l-2 border-zinc-200 dark:border-zinc-700">
              <div
                v-for="(stop, k) in leg.intermediate_stops"
                :key="`stop-${k}`"
                class="relative flex items-center justify-between py-2 text-sm group hover:bg-zinc-50 dark:hover:bg-zinc-900 rounded px-2 -mx-2"
              >
                <!-- Stop indicator dot -->
                <div
                  class="absolute -left-[17px] w-2 h-2 bg-zinc-300 dark:bg-zinc-600 rounded-full group-hover:bg-zinc-400 dark:group-hover:bg-zinc-500"
                ></div>

                <div class="flex-grow">
                  <div class="text-zinc-800 dark:text-zinc-100 font-medium">{{ stop.name }}</div>
                  <div class="text-zinc-500 dark:text-zinc-400 text-xs mt-0.5 flex items-center gap-2">
                    <span v-if="stop.track">{{ t("platform") }} {{ stop.track }}</span>
                    <span v-if="stop.level && stop.level !== 0" class="text-zinc-400 dark:text-zinc-500">
                      {{ t("level") }} {{ stop.level }}
                    </span>
                  </div>
                </div>

                <!-- Liveness is a property of the trip's feed, so stops inherit it from their leg. -->
                <div class="flex flex-col items-end gap-0.5">
                  <MotisTime
                    v-if="stop.departure"
                    class="text-xs"
                    :scheduled="stop.scheduled_departure"
                    :actual="stop.departure"
                    :real-time="leg.real_time"
                    :cancelled="stop.cancelled ?? false"
                  />
                  <MotisTime
                    v-else-if="stop.arrival"
                    class="text-xs"
                    :scheduled="stop.scheduled_arrival"
                    :actual="stop.arrival"
                    :real-time="leg.real_time"
                    :cancelled="stop.cancelled ?? false"
                  />
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
            <!-- `.stop`: opening the list must not re-select the leg. -->
            <summary
              class="text-zinc-600 dark:text-zinc-300 cursor-pointer text-sm font-medium hover:text-zinc-800 dark:hover:text-zinc-100 flex items-center gap-2"
              @click.stop
            >
              {{ t("walking_instructions") }}
              <span class="text-zinc-400 dark:text-zinc-500 text-xs">({{ leg.steps.length }} {{ t("steps") }})</span>
              <span v-if="hasRestrictedStep" class="text-amber-700 dark:text-amber-300 flex items-center gap-1 text-xs">
                <MdiIcon :path="mdiAlert" :size="14" />
                {{ t("access_restriction_hint") }}
              </span>
            </summary>
            <div class="mt-2 space-y-2 pl-4">
              <div
                v-for="(step, k) in leg.steps"
                :key="`step-${k}`"
                class="flex items-start gap-3 rounded px-2 py-1 -mx-2 hover:bg-zinc-100 dark:hover:bg-zinc-800"
                @click.stop="emit('selectStep', itineraryIndex, legIndex, k)"
              >
                <WalkingDirectionIcon :direction="step.relative_direction" />
                <div class="flex-grow text-sm">
                  <div class="text-zinc-900 dark:text-zinc-50">
                    <span v-if="step.street_name">{{ step.street_name }}</span>
                    <span v-else>{{ t("continue") }}</span>
                  </div>
                  <div class="text-zinc-600 dark:text-zinc-300 text-xs flex items-center gap-2">
                    {{ formatDistance(step.distance) }}
                    <span v-if="step.from_level !== step.to_level" class="text-zinc-500 dark:text-zinc-400">
                      {{ t("level") }} {{ step.from_level }} → {{ step.to_level }}
                    </span>
                    <span v-if="step.elevation_up || step.elevation_down" class="text-zinc-500 dark:text-zinc-400">
                      <span v-if="step.elevation_up">↗ {{ step.elevation_up }}m</span>
                      <span v-if="step.elevation_down">↘ {{ step.elevation_down }}m</span>
                    </span>
                    <span v-if="step.toll" class="text-orange-600 dark:text-orange-300">{{ t("toll") }}</span>
                  </div>
                  <div
                    v-if="step.access_restriction"
                    class="text-amber-700 dark:text-amber-300 mt-0.5 flex items-start gap-1 text-xs"
                  >
                    <MdiIcon :path="mdiAlert" :size="14" class="mt-0.5 flex-shrink-0" />
                    <span>{{ t("access_restriction") }}: {{ step.access_restriction }}</span>
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
  access_restriction: Eingeschränkter Zugang
  access_restriction_hint: enthält Zugangsbeschränkungen
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
  access_restriction: Restricted access
  access_restriction_hint: contains access restrictions
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
