<script setup lang="ts">
import { mdiChevronLeft, mdiTune } from "@mdi/js";
import { refDebounced } from "@vueuse/core";
import { useRouteQuery } from "@vueuse/router";
import { useTemplateRef } from "vue";
import type { components, operations } from "~/api_types";
import IndoorMap from "~/components/IndoorMap.vue";
import Toast from "~/components/Toast.vue";
import { clientOnlyRetries, firstOrDefault } from "~/composables/common";
import type { TimeSelection } from "~/types/navigation";
import { entityPath, isEntityType } from "~/utils/entityPath";
import { floorLevelForSelection } from "~/utils/motisLevels";

// Utility function to parse coordinate IDs and convert to coordinate objects
function parseCoordinateId(value: string): { lat: number; lon: number } | string {
  if (value.startsWith("coord:")) {
    const coordString = value.substring(6);
    const coords = coordString.split(",");
    if (coords.length === 2 && coords[0] && coords[1]) {
      const lat = Number.parseFloat(coords[0]);
      const lon = Number.parseFloat(coords[1]);
      if (!Number.isNaN(lat) && !Number.isNaN(lon)) {
        return { lat, lon };
      }
    }
  }
  return value;
}

definePageMeta({
  layout: "navigation",
});

const indoorMap = useTemplateRef("indoorMap");
const route = useRoute();
const router = useRouter();
const runtimeConfig = useRuntimeConfig();
const { t, locale } = useI18n({ useScope: "local" });
const { preferences } = useUserPreferences();
const coming_from = computed<string>(() => firstOrDefault(route.query.coming_from, ""));
// Only a routable type produces a canonical back-link; otherwise omit it (never a /view/{id}).
const comingFromTo = computed(() => {
  const type = firstOrDefault(route.query.coming_from_type, "");
  return coming_from.value && isEntityType(type) ? entityPath(coming_from.value, type) : undefined;
});
const selected_from = computed<string>(() => firstOrDefault(route.query.from, ""));
const selected_to = computed<string>(() => firstOrDefault(route.query.to, ""));
const mode = useRouteQuery<RequestQuery["route_costing"]>(
  "mode",
  computed(() => preferences.value.route_costing),
  {
    mode: "replace",
    route,
    router,
  }
);
type RequestQuery = operations["route_handler"]["parameters"]["query"];
type NavigationResponse =
  operations["route_handler"]["responses"][200]["content"]["application/json"];

const timeSelection = ref<TimeSelection | undefined>(undefined);
const debouncedTimeSelection = refDebounced(timeSelection, 200);
const motisPageCursor = ref<string | undefined>(undefined);
// Currently selected itinerary for map display
const selectedItineraryIndex = ref(0);

const { data, status, error } = await useFetch<NavigationResponse>(
  `${runtimeConfig.public.apiURL}/api/maps/route`,
  {
    query: computed(() => ({
      lang: locale.value,
      from: parseCoordinateId(selected_from.value),
      to: parseCoordinateId(selected_to.value),
      route_costing: mode.value,
      page_cursor: motisPageCursor.value,
      pedestrian_type: preferences.value.pedestrian_type,
      ptw_type: preferences.value.ptw_type,
      bicycle_type: preferences.value.bicycle_type,
      arrive_by: debouncedTimeSelection.value?.type === "arrive_by" ? "true" : "false",
      time: debouncedTimeSelection.value?.time.toISOString(),
    })),
    dedupe: "defer",
    credentials: "omit",
    retry: clientOnlyRetries(10),
    retryDelay: 1000,
    key: "navigation",
  }
);

// Narrow the routing response to the Motis variant once, so Motis-only UI
// (pagination, itinerary results) reads its fields without re-checking `router`.
const motisData = computed(() => (data.value?.router === "motis" ? data.value : undefined));

watch(
  [data, indoorMap],
  ([newData, newMap]) => {
    if (!newData || !newMap) return;
    if (newData.router === "valhalla") newMap.drawRoute(newData.legs[0].shape);
    if (newData.router === "motis") {
      // A door-to-door query often yields only a direct (walk) connection.
      const initialIndex = newData.itineraries.length > 0 ? 0 : -1;
      selectedItineraryIndex.value = initialIndex;
      const itinerary = initialIndex === 0 ? newData.itineraries[0] : newData.direct[0];
      if (itinerary) newMap.drawMotisItinerary(itinerary);
    }
  },
  { immediate: true }
);

type LocationDetailsResponse = components["schemas"]["LocationDetailsResponse"];

// The endpoint to centre on when only `from` or `to` is set (no route to fit).
const single_endpoint = computed<string | undefined>(() => {
  if (selected_from.value && !selected_to.value) return selected_from.value;
  if (selected_to.value && !selected_from.value) return selected_to.value;
  return undefined;
});

async function focusSingleEndpoint(value: string) {
  const map = indoorMap.value;
  if (!map) return;

  const parsed = parseCoordinateId(value);
  if (typeof parsed !== "string") {
    map.flyToCoords(parsed);
    return;
  }
  try {
    const details = await $fetch<LocationDetailsResponse>(
      `${runtimeConfig.public.apiURL}/api/locations/${encodeURIComponent(parsed)}`,
      { query: { lang: locale.value }, credentials: "omit" }
    );
    map.flyToCoords(details.coords, details.type);
  } catch {
    // Unresolvable id: keep the default view.
  }
}

watch(
  [single_endpoint, indoorMap],
  ([value]) => {
    if (value) focusSingleEndpoint(value);
  },
  { immediate: true }
);

const title = computed(() => {
  if (selected_from.value && selected_to.value)
    return t("navigate_from_to", {
      from: selected_from.value,
      to: selected_to.value,
    });
  if (selected_from.value) return t("navigate_from", { from: selected_from.value });
  if (selected_to.value) return t("navigate_to", { to: selected_to.value });
  return t("navigate");
});
const description = computed(() => {
  if (data.value?.router === "valhalla") {
    const length_meters = data.value.summary.length_meters;
    const length_kilometers = (length_meters / 1000).toFixed(1);
    const time_seconds = data.value.summary.time_seconds;
    const time_minutes = Math.ceil(data.value.summary.time_seconds / 60);
    return t(
      data.value.summary.has_highway
        ? "description_highway_time_length"
        : "description_time_length",
      {
        time: time_seconds >= 60 ? t("minutes", time_minutes) : t("seconds", time_seconds),
        length:
          length_meters >= 1000 ? t("kilometers", [length_kilometers]) : t("meters", length_meters),
      }
    );
  }
  if (data.value?.router === "motis") {
    return t("description_public_transport", {
      itinerary_count: data.value.itineraries.length,
    });
  }

  return t("description");
});
useSeoMeta({
  title: title,
  ogTitle: title,
  description: description,
  ogDescription: description,
  ogImage: "https://nav.tum.de/navigatum-card.png",
  twitterCard: "summary",
});

// The endpoints already live in the URL query, so the route below recomputes reactively as
// they change. A native GET submit would full-reload the page and discard the in-memory time
// selection, so we only dismiss the keyboard / autocomplete instead.
function onSearchSubmit() {
  if (typeof document !== "undefined") (document.activeElement as HTMLElement | null)?.blur();
}

function setBoundingBoxFromIndex(from_shape_index: number, to_shape_index: number) {
  if (data.value?.router !== "valhalla") return;

  const coords = data.value.legs[0].shape.slice(from_shape_index, to_shape_index);
  const latitudes = coords.map((c: { lat: number; lon: number }) => c.lat);
  const longitudes = coords.map((c: { lat: number; lon: number }) => c.lon);
  indoorMap.value?.fitBounds(
    [Math.min(...longitudes), Math.max(...longitudes)],
    [Math.min(...latitudes), Math.max(...latitudes)]
  );
}

function handleSelectManeuver(payload: { begin_shape_index: number; end_shape_index: number }) {
  setBoundingBoxFromIndex(payload.begin_shape_index, payload.end_shape_index);
}

// Index convention: `0..` are the transit itineraries, `-1 - i` the i-th direct connection.
function motisItineraryAt(itineraryIndex: number) {
  if (!motisData.value) return undefined;
  return itineraryIndex >= 0
    ? motisData.value.itineraries[itineraryIndex]
    : motisData.value.direct[-itineraryIndex - 1];
}

function handleSelectLeg(itineraryIndex: number, legIndex: number) {
  const itinerary = motisItineraryAt(itineraryIndex);
  if (!itinerary || !indoorMap.value) return;

  // If selecting a different itinerary, redraw the route
  if (selectedItineraryIndex.value !== itineraryIndex) {
    selectedItineraryIndex.value = itineraryIndex;
    indoorMap.value.drawMotisItinerary(itinerary);
  }

  indoorMap.value.highlightMotisLeg(legIndex);

  // Switch floors before fitting bounds so the fit's camera animation wins.
  const leg = itinerary.legs[legIndex];
  const floor = leg ? floorLevelForSelection(leg) : null;
  if (floor !== null) indoorMap.value.setFloor(floor);

  indoorMap.value.focusOnMotisLeg(legIndex, itinerary);
}

function handleSelectStep(itineraryIndex: number, legIndex: number, stepIndex: number) {
  const itinerary = motisItineraryAt(itineraryIndex);
  const leg = itinerary?.legs[legIndex];
  const step = leg?.steps?.[stepIndex];
  if (!itinerary || !leg || !step || !indoorMap.value) return;

  if (selectedItineraryIndex.value !== itineraryIndex) {
    selectedItineraryIndex.value = itineraryIndex;
    indoorMap.value.drawMotisItinerary(itinerary);
  }

  indoorMap.value.highlightMotisLeg(legIndex);

  const floor = floorLevelForSelection(leg, step);
  if (floor !== null) indoorMap.value.setFloor(floor);

  indoorMap.value.focusOnMotisStep(step, legIndex, itinerary);
}

function handleSelectItinerary(itineraryIndex: number) {
  const itinerary = motisItineraryAt(itineraryIndex);
  if (!itinerary || !indoorMap.value) return;
  selectedItineraryIndex.value = itineraryIndex;
  indoorMap.value.drawMotisItinerary(itinerary);
}
</script>

<template>
  <div class="flex h-full flex-col lg:flex-row-reverse">
    <div class="min-h-96 grow">
      <ClientOnly>
        <IndoorMap ref="indoorMap" type="room" :coords="{ lat: 0, lon: 0, source: 'navigatum' }" />
      </ClientOnly>
    </div>
    <div class="bg-zinc-100 dark:bg-zinc-800 flex min-w-96 flex-col gap-3 overflow-auto p-4 lg:max-w-96">
      <NuxtLinkLocale
        v-if="comingFromTo"
        :to="comingFromTo"
        property="item"
        class="focusable text-blue-400 dark:text-blue-500 rounded-md pb-2 hover:text-blue-500 dark:hover:text-blue-400 hover:underline"
      >
        <div class="my-auto flex flex-row gap-2">
          <MdiIcon :path="mdiChevronLeft" :size="16" />
          <span class="text-xs font-semibold uppercase">{{ t("back") }}</span>
        </div>
      </NuxtLinkLocale>
      <div class="flex flex-row items-center gap-2">
        <NavigationModeSelector v-model:mode="mode" class="grow" />
        <PreferencesPopup>
          <template #trigger="{ open }">
            <Btn variant="secondary" size="md" :title="t('preferences')" :aria-label="t('preferences')" @click="open">
              <MdiIcon :path="mdiTune" :size="24" />
            </Btn>
          </template>
        </PreferencesPopup>
      </div>
      <form
        action="/navigate"
        autocomplete="off"
        method="GET"
        role="search"
        class="flex flex-col gap-2"
        @submit.prevent="onSearchSubmit"
      >
        <NavigationSearchBar query-id="from" />
        <NavigationSearchBar query-id="to" />
        <div v-if="mode === 'public_transit'" class="mb-4">
          <div class="flex items-center justify-between mb-3">
            <NavigationTimeSelector v-model:time-selection="timeSelection" />
            <MotisPaginationControls
              v-if="motisData && (motisData.previous_page_cursor || motisData.next_page_cursor)"
              :previous-page-cursor="motisData.previous_page_cursor"
              :next-page-cursor="motisData.next_page_cursor"
              v-model:page-cursor="motisPageCursor"
              size="sm"
            />
          </div>
          <NavigationTimeInput v-model:time-selection="timeSelection" />
        </div>
      </form>
      <ValhallaNavigationRoutingResults
        v-if="status === 'success' && data?.router === 'valhalla'"
        :data="data"
        @select-maneuver="handleSelectManeuver"
      />
      <MotisNavigationRoutingResults
        v-else-if="status === 'success' && motisData"
        :data="motisData"
        v-model:page-cursor="motisPageCursor"
        @select-leg="handleSelectLeg"
        @select-step="handleSelectStep"
        @select-itinerary="handleSelectItinerary"
      />
      <div v-else-if="status === 'pending'" class="text-zinc-900 dark:text-zinc-50 flex flex-col items-center gap-5 py-32">
        <Spinner class="h-8 w-8" />
        {{ t("calculating best route") }}
      </div>
      <Toast v-else-if="status === 'error' && !!error && error.statusCode !== 404" id="nav-error" level="error">
        {{ error.message }}
      </Toast>

      <div v-if="status === 'success' && !!data" class="border-zinc-500 dark:border-zinc-400 border-t p-1" />
      <NavigationDisclaimerToast :coming-from="coming_from" :selected-from="selected_from" :selected-to="selected_to" />
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  back: zurück
  preferences: Präferenzen
  calculating best route: Berechnet optimale Route
  navigate_from_to: Navigiere von {from} nach {to}
  navigate_from: Navigiere von {from}
  navigate_to: Navigiere nach {to}
  navigate: Navigiere
  description_highway_time_length: Die Fahrt dauert {time} und erstreckt sich über {length}. Bitte beachten Sie, dass sie Autobahnfahrten beinhaltet.
  description_time_length: Die Fahrt dauert {time} und erstreckt sich über {length}.
  description_public_transport: "{itinerary_count} optionen um mit öffentlichen Verkehrsmitteln zu reisen."
  description: Beste Route wird berechnet
  minutes: "sofort | eine Minute | {count} Minuten"
  seconds: "sofort | eine Sekunde | {count} Sekunden"
  meters: "hier | einen Meter | {count} Meter"
  kilometers: "{0} Kilometer"
en:
  back: back
  preferences: Preferences
  calculating best route: Calculating best route
  navigate_from_to: Navigating from {from} to {to}
  navigate_from: Navigating from {from}
  navigate_to: Navigating to {to}
  navigate: Navigating
  description_highway_time_length: The trip will take {time} and span {length}. Note that it will include highway travel.
  description_time_length: The trip will take {time} and span {length}.
  description_public_transport: "{itinerary_count} options to travel with public transport."
  description: Calculating best route
  minutes: "instant | one minute | {count} minutes"
  seconds: "instant | one second | {count} seconds"
  meters: "here | one meter | {count} meters"
  kilometers: "{0} kilometers"
</i18n>
