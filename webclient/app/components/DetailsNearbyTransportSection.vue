<script setup lang="ts">
import { mdiChevronDown, mdiHelpCircle } from "@mdi/js";
import { useIntervalFn } from "@vueuse/core";
import type { components } from "~/api_types";
import { formatDistance } from "~/utils/motis";

type NearbyLocationsResponse = components["schemas"]["NearbyLocationsResponse"];
type TransportationResponse = components["schemas"]["TransportationResponse"];
type ModeResponse = components["schemas"]["ModeResponse"];

const props = defineProps<{
  readonly id: string;
}>();

const { t } = useI18n({ useScope: "local" });
const runtimeConfig = useRuntimeConfig();

const { data } = await useFetch<NearbyLocationsResponse, string>(
  () => `${runtimeConfig.public.apiURL}/api/locations/${props.id}/nearby`,
  { dedupe: "cancel", credentials: "omit" }
);

// Endorsed by the motis maintainers: CORS is `*` and the endpoint is cheap enough for per-pageview hits.
const STOPTIMES_URL = "https://api.transitous.org/api/v4/stoptimes";
const REFRESH_INTERVAL_MS = 180_000;
const TICK_INTERVAL_MS = 1_000;
const N_DEPARTURES = 3;

type StopTimeEntry = {
  readonly mode?: ModeResponse;
  readonly headsign?: string | null;
  readonly cancelled?: boolean;
  readonly displayName?: string | null;
  readonly routeShortName?: string | null;
  readonly routeColor?: string | null;
  readonly routeTextColor?: string | null;
  readonly tripTo?: { readonly name?: string | null } | null;
  readonly place?: {
    readonly departure?: string | null;
    readonly scheduledDeparture?: string | null;
    readonly cancelled?: boolean;
  } | null;
};
type StopTimesResponse = { readonly stopTimes?: readonly StopTimeEntry[] };

type DepartureState = {
  loading: boolean;
  error: string | null;
  entries: readonly StopTimeEntry[];
};
const stationState = reactive(new Map<string, DepartureState>());

const now = ref(Date.now());

const sortedStations = computed<readonly TransportationResponse[]>(() => {
  const list = data.value?.public_transport ?? [];
  return [...list].sort((a, b) => a.distance_meters - b.distance_meters);
});

// `immediate` + watching `sortedStations` covers `useFetch` resolving after mount.
watch(
  sortedStations,
  (stations) => {
    const closest = stations[0];
    if (closest && !stationState.has(closest.id)) {
      void fetchDepartures(closest.id);
    }
  },
  { immediate: true }
);

async function fetchDepartures(stationId: string): Promise<void> {
  const existing = stationState.get(stationId);
  if (existing) {
    existing.loading = true;
    existing.error = null;
  } else {
    stationState.set(stationId, { loading: true, error: null, entries: [] });
  }
  const params = new URLSearchParams({ stopId: stationId, n: String(N_DEPARTURES) });
  try {
    const res = await fetch(`${STOPTIMES_URL}?${params.toString()}`, { credentials: "omit" });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const body = (await res.json()) as StopTimesResponse;
    const entry = stationState.get(stationId);
    if (entry) {
      entry.entries = (body.stopTimes ?? []).slice(0, N_DEPARTURES);
      entry.loading = false;
    }
  } catch (e) {
    const entry = stationState.get(stationId);
    if (entry) {
      entry.error = e instanceof Error ? e.message : String(e);
      entry.loading = false;
    }
  }
}

function toggleExpand(stationId: string): void {
  if (stationState.has(stationId)) {
    stationState.delete(stationId);
    return;
  }
  void fetchDepartures(stationId);
}

function modeLabel(mode: ModeResponse | undefined | null): string {
  if (!mode) return "";
  return t(`mode.${mode}`, mode);
}

function countdownLabel(iso: string | null | undefined): string {
  if (!iso) return "";
  const departure = Date.parse(iso);
  if (Number.isNaN(departure)) return "";
  const diffMs = departure - now.value;
  if (diffMs < -30_000) return t("departed");
  if (diffMs < 30_000) return t("now");
  const minutes = Math.round(diffMs / 60_000);
  return t("in_minutes", { count: minutes });
}

function scheduledClockLabel(entry: StopTimeEntry): string {
  const iso = entry.place?.departure ?? entry.place?.scheduledDeparture;
  if (!iso) return "";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "";
  return d.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
}

function delayMinutes(entry: StopTimeEntry): number | null {
  const actualIso = entry.place?.departure;
  const scheduledIso = entry.place?.scheduledDeparture;
  if (!actualIso || !scheduledIso) return null;
  const diff = Date.parse(actualIso) - Date.parse(scheduledIso);
  if (Number.isNaN(diff) || Math.abs(diff) < 30_000) return null;
  return Math.round(diff / 60_000);
}

function routeBadgeStyle(entry: StopTimeEntry): Record<string, string> {
  const bg = entry.routeColor ? `#${entry.routeColor}` : "#3f3f46";
  const fg = entry.routeTextColor ? `#${entry.routeTextColor}` : "#ffffff";
  return { backgroundColor: bg, color: fg };
}

useIntervalFn(() => {
  now.value = Date.now();
}, TICK_INTERVAL_MS);

useIntervalFn(() => {
  for (const id of stationState.keys()) {
    void fetchDepartures(id);
  }
}, REFRESH_INTERVAL_MS);
</script>

<template>
  <div v-if="sortedStations.length" class="flex flex-col gap-3 print:!hidden">
    <p class="text-zinc-800 text-lg font-semibold">{{ t("title") }}</p>
    <ul class="flex flex-col gap-2">
      <li
        v-for="station in sortedStations"
        :key="station.id"
        class="bg-zinc-100 border border-zinc-200 rounded-md"
      >
        <button
          type="button"
          class="focusable w-full flex items-center justify-between gap-3 p-3 text-left"
          :aria-expanded="stationState.has(station.id)"
          @click="toggleExpand(station.id)"
        >
          <div class="flex items-center gap-2 min-w-0">
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
            <span class="text-zinc-800 font-medium truncate">{{ station.name }}</span>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <span class="text-zinc-500 text-sm">{{ formatDistance(station.distance_meters) }}</span>
            <MdiIcon
              :path="mdiChevronDown"
              :size="18"
              class="text-zinc-500 transition-transform"
              :class="{ 'rotate-180': stationState.has(station.id) }"
            />
          </div>
        </button>
        <template v-if="stationState.get(station.id) as DepartureState | undefined">
          <div class="border-t border-zinc-200 p-3">
            <div v-if="stationState.get(station.id)!.loading" class="text-zinc-500 text-sm">
              {{ t("loading") }}
            </div>
            <div v-else-if="stationState.get(station.id)!.error" class="text-red-700 text-sm">
              {{ t("error", { msg: stationState.get(station.id)!.error }) }}
            </div>
            <div v-else-if="!stationState.get(station.id)!.entries.length" class="text-zinc-500 text-sm">
              {{ t("no_departures") }}
            </div>
            <ul v-else class="flex flex-col gap-2">
              <li
                v-for="(entry, idx) in stationState.get(station.id)!.entries"
                :key="idx"
                class="flex items-center gap-2 text-sm"
                :class="{ 'text-zinc-400 line-through': entry.cancelled || entry.place?.cancelled }"
              >
                <span
                  class="rounded px-2 py-0.5 text-xs font-semibold shrink-0"
                  :style="routeBadgeStyle(entry)"
                >
                  {{ entry.displayName || entry.routeShortName || modeLabel(entry.mode) }}
                </span>
                <span class="text-zinc-700 truncate flex-1">{{ entry.tripTo?.name || entry.headsign }}</span>
                <span class="text-zinc-500 text-xs shrink-0">{{ scheduledClockLabel(entry) }}</span>
                <span
                  v-if="delayMinutes(entry) !== null"
                  class="text-red-600 text-xs font-semibold shrink-0"
                >
                  +{{ delayMinutes(entry) }}
                </span>
                <span class="text-zinc-800 font-medium shrink-0">{{ countdownLabel(entry.place?.departure ?? entry.place?.scheduledDeparture) }}</span>
              </li>
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
