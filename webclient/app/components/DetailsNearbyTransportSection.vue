<script setup lang="ts">
import {
  mdiArrowRightThin,
  mdiChevronDown,
  mdiChevronUp,
  mdiHelpCircle,
  mdiOpenInNew,
} from "@mdi/js";
import { useToggle } from "@vueuse/core";
import type { components } from "~/api_types";
import {
  boardingRestriction,
  countdownPhase,
  isStopCancelled,
  routeBadgeStyle,
  type StopTimeEntry,
  trackOf,
  useNearbyDepartures,
} from "~/composables/nearbyDepartures";
import { formatDistance } from "~/utils/motis";

type ModeResponse = components["schemas"]["ModeResponse"];

const props = defineProps<{
  readonly id: string;
}>();

const { t } = useI18n({ useScope: "local" });
const { stations, toggleExpand, now } = await useNearbyDepartures(() => props.id);

const INITIAL_VISIBLE = 5;
const [showAll, toggleShowAll] = useToggle(false);
const visibleStations = computed(() =>
  showAll.value ? stations.value : stations.value.slice(0, INITIAL_VISIBLE)
);
const hiddenCount = computed(() => Math.max(0, stations.value.length - INITIAL_VISIBLE));

function modeLabel(mode: ModeResponse | undefined | null): string {
  if (!mode) return "";
  return t(`mode.${mode}`, mode);
}

function countdownLabel(iso: string | null | undefined): string {
  const phase = countdownPhase(iso, now.value);
  switch (phase.kind) {
    case "empty":
      return "";
    case "departed":
      return t("departed");
    case "now":
      return t("now");
    case "minutes":
      return t("in_minutes", { count: phase.count });
    case "hours":
      return t("in_hours", { count: phase.hours });
    case "hoursMinutes":
      return t("in_hours_minutes", { h: phase.hours, m: phase.minutes });
  }
}

function trackLabel(entry: StopTimeEntry): string {
  const track = trackOf(entry);
  return track ? t("track", { track }) : "";
}

function boardingRestrictionLabel(entry: StopTimeEntry): string {
  const restriction = boardingRestriction(entry);
  return restriction === "none" ? "" : t(restriction);
}

// Aggregated context for hover: who runs the line, any boarding restriction,
// and any cancellation note. Destination already shown inline.
function rowTitle(entry: StopTimeEntry): string {
  const parts: string[] = [];
  const dest = entry.tripTo?.name || entry.headsign;
  if (dest) parts.push(`→ ${dest}`);
  if (entry.agencyName) parts.push(entry.agencyName);
  const restriction = boardingRestrictionLabel(entry);
  if (restriction) parts.push(restriction);
  if (entry.tripCancelled) parts.push(t("trip_cancelled"));
  else if (isStopCancelled(entry)) parts.push(t("stop_cancelled"));
  return parts.join(" · ");
}
</script>

<template>
  <div v-if="stations.length" class="flex flex-col gap-3 print:!hidden">
    <div class="flex flex-row items-baseline justify-between gap-2">
      <p class="text-zinc-800 dark:text-zinc-100 text-lg font-semibold">{{ t("title") }}</p>
      <Btn to="https://transitous.org/" variant="link" size="text-xs gap-1 rounded">
        {{ t("source") }}
        <MdiIcon :path="mdiOpenInNew" :size="14" class="my-auto" aria-hidden="true" />
      </Btn>
    </div>
    <ul class="flex flex-col gap-2">
      <li
        v-for="{ station, state } in visibleStations"
        :key="station.id"
        class="bg-zinc-100 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-sm"
      >
        <button
          type="button"
          class="focusable w-full flex items-center gap-3 p-3 text-left"
          :aria-expanded="!!state"
          :aria-controls="`nearby-departures-${station.id}`"
          @click="toggleExpand(station.id)"
        >
          <div class="flex items-center gap-2 min-w-0 flex-1">
            <span class="text-zinc-800 dark:text-zinc-100 font-medium truncate" :title="station.name">{{ station.name }}</span>
            <span class="text-zinc-500 dark:text-zinc-400 text-sm shrink-0">{{ formatDistance(station.distance_meters) }}</span>
          </div>
          <div class="flex items-center gap-1 shrink-0">
            <MotisTransitModeIcon
              v-for="mode in station.modes"
              :key="mode"
              :mode="mode"
              variant="mode-colored"
              :title="modeLabel(mode)"
            />
            <MdiIcon
              v-if="!station.modes.length"
              :path="mdiHelpCircle"
              :size="18"
              class="text-zinc-400 dark:text-zinc-500"
            />
          </div>
          <MdiIcon
            :path="mdiChevronDown"
            :size="18"
            class="text-zinc-500 dark:text-zinc-400 transition-transform shrink-0"
            :class="{ 'rotate-180': !!state }"
          />
        </button>
        <template v-if="state">
          <div
            :id="`nearby-departures-${station.id}`"
            role="status"
            aria-live="polite"
            class="border-t border-zinc-200 dark:border-zinc-700 p-3"
          >
            <div v-if="state.loading" class="text-zinc-500 dark:text-zinc-400 text-sm">
              {{ t("loading") }}
            </div>
            <div v-else-if="state.error" class="text-red-700 dark:text-red-300 text-sm">
              {{ t("error", { msg: state.error }) }}
            </div>
            <div v-else-if="!state.entries.length" class="text-zinc-500 dark:text-zinc-400 text-sm">
              {{ t("no_departures") }}
            </div>
            <ul v-else class="grid grid-cols-[auto_auto_auto_minmax(0,1fr)] gap-x-3 gap-y-2 items-center text-sm">
              <template v-for="(entry, idx) in state.entries" :key="idx">
                <span
                  class="text-zinc-800 dark:text-zinc-100 font-medium tabular-nums text-right"
                  :class="{ 'text-zinc-400 dark:text-zinc-500 line-through': isStopCancelled(entry) }"
                >
                  {{ countdownLabel(entry.place?.departure ?? entry.place?.scheduledDeparture) }}
                </span>
                <MotisTime
                  class="text-xs"
                  :scheduled="entry.place?.scheduledDeparture"
                  :actual="entry.place?.departure"
                  :real-time="entry.realTime"
                  :cancelled="isStopCancelled(entry)"
                />
                <span
                  class="justify-self-start flex items-center gap-2 min-w-0"
                  :class="{ 'text-zinc-400 dark:text-zinc-500 line-through': isStopCancelled(entry) }"
                >
                  <span
                    class="rounded px-2 py-0.5 text-xs font-semibold truncate max-w-28"
                    :style="routeBadgeStyle(entry)"
                    :title="entry.routeShortName || entry.displayName || modeLabel(entry.mode)"
                  >
                    {{ entry.routeShortName || entry.displayName || modeLabel(entry.mode) }}
                  </span>
                  <span
                    v-if="trackLabel(entry)"
                    class="text-zinc-500 dark:text-zinc-400 text-xs whitespace-nowrap"
                  >{{ trackLabel(entry) }}</span>
                </span>
                <span
                  class="text-zinc-700 dark:text-zinc-200 flex items-center gap-1 min-w-0"
                  :class="{ 'text-zinc-400 dark:text-zinc-500 line-through': isStopCancelled(entry) }"
                  :title="rowTitle(entry)"
                >
                  <MdiIcon :path="mdiArrowRightThin" :size="16" class="text-zinc-400 dark:text-zinc-500 shrink-0" />
                  <span class="truncate min-w-0">{{ entry.tripTo?.name || entry.headsign }}</span>
                </span>
                <span
                  v-if="boardingRestrictionLabel(entry)"
                  class="col-span-full text-center text-orange-700 dark:text-orange-300 text-xs font-semibold -mt-1 mb-1"
                >
                  ⚠ {{ boardingRestrictionLabel(entry) }}
                </span>
              </template>
            </ul>
          </div>
        </template>
      </li>
    </ul>
    <button
      v-if="hiddenCount > 0"
      type="button"
      class="focusable self-start flex items-center gap-1 rounded-sm px-2 py-1 text-sm font-medium text-blue-700 dark:text-blue-300 hover:text-blue-900 dark:hover:text-blue-100"
      :aria-expanded="showAll"
      @click="toggleShowAll()"
    >
      <MdiIcon :path="showAll ? mdiChevronUp : mdiChevronDown" :size="16" />
      {{ showAll ? t("show_less") : t("show_more", { count: hiddenCount }) }}
    </button>
  </div>
</template>

<i18n lang="yaml">
de:
  title: Öffentlicher Verkehr in der Nähe
  source: via Transitous
  loading: Lädt Abfahrten…
  error: 'Abfahrten konnten nicht geladen werden: {msg}'
  no_departures: Keine bevorstehenden Abfahrten.
  show_more: "{count} weitere anzeigen"
  show_less: Weniger anzeigen
  now: jetzt
  departed: abgefahren
  in_minutes: "in {count} min"
  in_hours: "in {count} h"
  in_hours_minutes: "in {h} h {m} min"
  track: "Gl. {track}"
  no_boarding_alighting: "Kein Ein-/Ausstieg"
  alighting_only: "nur Ausstieg"
  boarding_only: "nur Einstieg"
  trip_cancelled: "Fahrt fällt aus"
  stop_cancelled: "Halt entfällt"
  mode:
    walk: zu Fuß
    bike: Fahrrad
    rental: Leihfahrzeug
    car: Auto
    car_parking: Parken
    car_dropoff: Absetzen
    odm: On-Demand
    flex: Bedarfsverkehr
    transit: Öffis
    tram: Tram
    subway: U-Bahn
    suburban: S-Bahn
    metro: Metro
    bus: Bus
    coach: Reisebus
    rail: Zug
    highspeed_rail: ICE
    long_distance: Fernverkehr
    night_rail: Nachtzug
    regional_fast_rail: RE
    regional_rail: Regio
    cable_car: Seilbahn
    funicular: Standseilbahn
    areal_lift: Liftsystem
    ferry: Fähre
    airplane: Flug
    ride_sharing: Mitfahrgelegenheit
    other: Sonstiges
en:
  title: Nearby public transport
  source: via Transitous
  loading: Loading departures…
  error: 'Could not load departures: {msg}'
  no_departures: No upcoming departures.
  show_more: "Show {count} more"
  show_less: Show less
  now: now
  departed: departed
  in_minutes: "in {count} min"
  in_hours: "in {count}h"
  in_hours_minutes: "in {h}h {m}min"
  track: "Pl. {track}"
  no_boarding_alighting: "No boarding/alighting"
  alighting_only: "Alighting only"
  boarding_only: "Boarding only"
  trip_cancelled: "Trip cancelled"
  stop_cancelled: "Stop cancelled"
  mode:
    walk: Walk
    bike: Bike
    rental: Rental
    car: Car
    car_parking: Parking
    car_dropoff: Drop-off
    odm: On-demand
    flex: Flex
    transit: Transit
    tram: Tram
    subway: Subway
    suburban: Suburban
    metro: Metro
    bus: Bus
    coach: Coach
    rail: Train
    highspeed_rail: High-speed rail
    long_distance: Long-distance
    night_rail: Night train
    regional_fast_rail: Regional express
    regional_rail: Regional
    cable_car: Cable car
    funicular: Funicular
    areal_lift: Aerial lift
    ferry: Ferry
    airplane: Flight
    ride_sharing: Ride share
    other: Other
</i18n>
