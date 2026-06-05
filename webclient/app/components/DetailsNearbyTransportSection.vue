<script setup lang="ts">
import { mdiArrowRightThin, mdiChevronDown, mdiHelpCircle } from "@mdi/js";
import type { components } from "~/api_types";
import {
  boardingRestriction,
  countdownPhase,
  delayMinutes,
  isStopCancelled,
  routeBadgeStyle,
  type StopTimeEntry,
  scheduledClockLabel,
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
    <p class="text-zinc-800 text-lg font-semibold">{{ t("title") }}</p>
    <ul class="flex flex-col gap-2">
      <li
        v-for="{ station, state } in stations"
        :key="station.id"
        class="bg-zinc-100 border border-zinc-200 rounded-sm"
      >
        <button
          type="button"
          class="focusable w-full flex items-center gap-3 p-3 text-left"
          :aria-expanded="!!state"
          :aria-controls="`nearby-departures-${station.id}`"
          @click="toggleExpand(station.id)"
        >
          <div class="flex items-center gap-2 min-w-0 flex-1">
            <span class="text-zinc-800 font-medium truncate" :title="station.name">{{ station.name }}</span>
            <span class="text-zinc-500 text-sm shrink-0">{{ formatDistance(station.distance_meters) }}</span>
          </div>
          <div class="flex items-center gap-1 shrink-0">
            <MotisTransitModeIcon
              v-for="mode in station.modes"
              :key="mode"
              :mode="mode"
              transparent
              :title="modeLabel(mode)"
            />
            <MdiIcon
              v-if="!station.modes.length"
              :path="mdiHelpCircle"
              :size="18"
              class="text-zinc-400"
            />
          </div>
          <MdiIcon
            :path="mdiChevronDown"
            :size="18"
            class="text-zinc-500 transition-transform shrink-0"
            :class="{ 'rotate-180': !!state }"
          />
        </button>
        <template v-if="state">
          <div
            :id="`nearby-departures-${station.id}`"
            role="status"
            aria-live="polite"
            class="border-t border-zinc-200 p-3"
          >
            <div v-if="state.loading" class="text-zinc-500 text-sm">
              {{ t("loading") }}
            </div>
            <div v-else-if="state.error" class="text-red-700 text-sm">
              {{ t("error", { msg: state.error }) }}
            </div>
            <div v-else-if="!state.entries.length" class="text-zinc-500 text-sm">
              {{ t("no_departures") }}
            </div>
            <ul v-else class="grid grid-cols-[auto_auto_auto_minmax(0,1fr)] gap-x-3 gap-y-2 items-center text-sm">
              <template v-for="(entry, idx) in state.entries" :key="idx">
                <span
                  class="text-zinc-800 font-medium tabular-nums text-right"
                  :class="{ 'text-zinc-400 line-through': isStopCancelled(entry) }"
                >
                  {{ countdownLabel(entry.place?.departure ?? entry.place?.scheduledDeparture) }}
                </span>
                <span
                  class="text-zinc-500 text-xs tabular-nums whitespace-nowrap"
                  :class="{ 'text-zinc-400 line-through': isStopCancelled(entry) }"
                >
                  {{ scheduledClockLabel(entry) }}<span
                    v-if="delayMinutes(entry) !== null"
                    class="text-red-600 font-semibold ms-1"
                  >+{{ delayMinutes(entry) }}</span>
                </span>
                <span
                  class="justify-self-start flex items-center gap-2 min-w-0"
                  :class="{ 'text-zinc-400 line-through': isStopCancelled(entry) }"
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
                    class="text-zinc-500 text-xs whitespace-nowrap"
                  >{{ trackLabel(entry) }}</span>
                </span>
                <span
                  class="text-zinc-700 flex items-center gap-1 min-w-0"
                  :class="{ 'text-zinc-400 line-through': isStopCancelled(entry) }"
                  :title="rowTitle(entry)"
                >
                  <MdiIcon :path="mdiArrowRightThin" :size="16" class="text-zinc-400 shrink-0" />
                  <span class="truncate min-w-0">{{ entry.tripTo?.name || entry.headsign }}</span>
                </span>
                <span
                  v-if="boardingRestrictionLabel(entry)"
                  class="col-span-full text-center text-orange-700 text-xs font-semibold -mt-1 mb-1"
                >
                  ⚠ {{ boardingRestrictionLabel(entry) }}
                </span>
              </template>
            </ul>
          </div>
        </template>
      </li>
    </ul>
  </div>
</template>

<i18n lang="yaml">
de:
  title: Öffentlicher Verkehr in der Nähe
  loading: Lädt Abfahrten…
  error: 'Abfahrten konnten nicht geladen werden: {msg}'
  no_departures: Keine bevorstehenden Abfahrten.
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
  loading: Loading departures…
  error: 'Could not load departures: {msg}'
  no_departures: No upcoming departures.
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
