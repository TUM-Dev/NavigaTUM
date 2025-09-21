<script setup lang="ts">
import { mdiChevronLeft } from "@mdi/js";
import { useRouteQuery } from "@vueuse/router";
import type { Ref } from "vue";
import { useTemplateRef } from "vue";
import type { operations } from "~/api_types";
import IndoorMap from "~/components/IndoorMap.vue";
import Toast from "~/components/Toast.vue";
import { firstOrDefault } from "~/composables/common";

definePageMeta({
  layout: "navigation",
});

const indoorMap = useTemplateRef("indoorMap");
const route = useRoute();
const router = useRouter();
const { t, locale } = useI18n({ useScope: "local" });
const coming_from = computed<string>(() => firstOrDefault(route.query.coming_from, ""));
const selected_from = computed<string>(() => firstOrDefault(route.query.from, ""));
const selected_to = computed<string>(() => firstOrDefault(route.query.to, ""));
const mode = useRouteQuery<RequestQuery["route_costing"]>("mode", "pedestrian", {
  mode: "replace",
  route,
  router,
});
type RequestQuery = operations["route_handler"]["parameters"]["query"];
type NavigationResponse =
  operations["route_handler"]["responses"][200]["content"]["application/json"];

// Page cursor for Motis pagination
const pageCursor = ref<string | undefined>(undefined);
const { data, status, error, refresh } = await useFetch<NavigationResponse>("https://nav.tum.de/api/maps/route", {
  query: computed(() => ({
    lang: locale.value,
    from: selected_from.value,
    to: selected_to.value,
    route_costing: mode.value,
    page_cursor: pageCursor.value,
    pedestrian_type: undefined as RequestQuery["pedestrian_type"],
    ptw_type: undefined as RequestQuery["ptw_type"],
    bicycle_type: undefined as RequestQuery["bicycle_type"],
  })),
});

effect(() => {
  if (!data.value || !indoorMap.value) return;
  if (data.value.router === "valhalla") indoorMap.value.drawRoute(data.value.legs[0].shape);
  if (data.value?.router === "motis") {
    // Reset to first itinerary when data changes
    selectedItineraryIndex.value = 0;
    // Draw the first itinerary if available
    if (data.value.itineraries.length > 0 && indoorMap.value && data.value.itineraries[0]) {
      indoorMap.value.drawMotisItinerary(data.value.itineraries[0]);
    }
  }
});

const title = computed(() => {
  if (!!selected_from.value && !!selected_to.value)
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

// Handle Motis pagination
async function loadMotisPage(cursor?: string) {
  pageCursor.value = cursor;
  await refresh();
}

// Currently selected itinerary for map display
const selectedItineraryIndex = ref(0);

function handleSelectLeg(itineraryIndex: number, legIndex: number) {
  console.log("Selected itinerary:", itineraryIndex, "leg:", legIndex);
  if (data.value?.router === "motis" && indoorMap.value) {
    // If selecting a different itinerary, redraw the route
    if (selectedItineraryIndex.value !== itineraryIndex) {
      selectedItineraryIndex.value = itineraryIndex;
      if (data.value.itineraries[itineraryIndex]) {
        indoorMap.value.drawMotisItinerary(data.value.itineraries[itineraryIndex]);
      }
    }

    // Highlight the selected leg on the map
    indoorMap.value.highlightMotisLeg(legIndex);

    // Focus map on the selected leg
    if (data.value.itineraries[itineraryIndex]) {
      indoorMap.value.focusOnMotisLeg(legIndex, data.value.itineraries[itineraryIndex]);
    }
  }
}

function handleSelectItinerary(itineraryIndex: number) {
  console.log("Selected itinerary:", itineraryIndex);
  if (data.value?.router === "motis" && indoorMap.value && data.value.itineraries[itineraryIndex]) {
    selectedItineraryIndex.value = itineraryIndex;
    indoorMap.value.drawMotisItinerary(data.value.itineraries[itineraryIndex]);
  }
}
</script>

<template>
  <div
    class="flex max-h-[calc(100vh-60px)] min-h-[calc(100vh-60px)] flex-col lg:max-h-[calc(100vh-150px)] lg:min-h-[calc(100vh-150px)] lg:flex-row-reverse"
  >
    <div class="min-h-96 grow">
      <ClientOnly>
        <IndoorMap ref="indoorMap" type="room" :coords="{ lat: 0, lon: 0, source: 'navigatum' }" />
      </ClientOnly>
    </div>
    <div class="bg-zinc-100 flex min-w-96 flex-col gap-3 overflow-auto p-4 lg:max-w-96">
      <NuxtLinkLocale
        v-if="coming_from"
        :to="'/view/' + coming_from"
        property="item"
        class="focusable text-blue-400 rounded-md pb-2 hover:text-blue-500 hover:underline"
      >
        <div class="my-auto flex flex-row gap-2">
          <MdiIcon :path="mdiChevronLeft" :size="16" />
          <span class="text-xs font-semibold uppercase">{{ t("back") }}</span>
        </div>
      </NuxtLinkLocale>
      <NavigationModeSelector v-model:mode="mode" />
      <form action="/navigate" autocomplete="off" method="GET" role="search" class="flex flex-col gap-2">
        <NavigationSearchBar query-id="from" />
        <NavigationSearchBar query-id="to" />
      </form>
      <ValhallaNavigationRoutingResults
        v-if="status === 'success' && data?.router === 'valhalla'"
        :data="data"
        @select-maneuver="handleSelectManeuver"
      />
      <MotisNavigationRoutingResults
        v-else-if="status === 'success' && data?.router === 'motis'"
        :data="data"
        :loading="false"
        :page-cursor="pageCursor"
        @select-leg="handleSelectLeg"
        @select-itinerary="handleSelectItinerary"
        @retry="() => refresh()"
        @load-next="loadMotisPage"
        @load-previous="loadMotisPage"
      />
      <div v-else-if="status === 'pending'" class="text-zinc-900 flex flex-col items-center gap-5 py-32">
        <Spinner class="h-8 w-8" />
        {{ t("calculating best route") }}
      </div>
      <Toast v-else-if="status === 'error' && !!error && error.statusCode !== 404" id="nav-error" level="error">
        {{ error.message }}
      </Toast>

      <div v-if="status === 'success' && !!data" class="border-zinc-500 border-t p-1" />
      <NavigationDisclaimerToast :coming-from="coming_from" :selected-from="selected_from" :selected-to="selected_to" />
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  back: zurück
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
