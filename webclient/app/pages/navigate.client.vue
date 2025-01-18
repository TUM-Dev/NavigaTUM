<script setup lang="ts">
import IndoorMap from "~/components/IndoorMap.vue";
import { ChevronLeftIcon } from "@heroicons/vue/16/solid";
import { firstOrDefault } from "~/composables/common";
import { useRouteQuery } from "@vueuse/router";
import Toast from "~/components/Toast.vue";
import type { Ref } from "vue";
import { useTemplateRef } from "vue";
import type { operations } from "~/api_types";

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
const mode = useRouteQuery<"bicycle" | "transit" | "motorcycle" | "car" | "pedestrian">("mode", "car", {
  mode: "replace",
  route,
  router,
});
type RequestQuery = operations["route_handler"]["parameters"]["query"];
type NavigationResponse = operations["route_handler"]["responses"][200]["content"]["application/json"];
const { data, status, error } = await useFetch<NavigationResponse>("https://nav.tum.de/api/maps/route", {
  query: {
    lang: locale as Ref<RequestQuery["lang"]>,
    from: selected_from as Ref<RequestQuery["from"]>,
    to: selected_to as Ref<RequestQuery["to"]>,
    route_costing: mode as Ref<RequestQuery["route_costing"]>,
    pedestrian_type: undefined as RequestQuery["pedestrian_type"],
    ptw_type: undefined as RequestQuery["ptw_type"],
    bicycle_type: undefined as RequestQuery["bicycle_type"],
  },
});
effect(() => {
  if (!data.value || !indoorMap.value) return;

  indoorMap.value.drawRoute(data.value.legs[0].shape);
});

function setBoundingBoxFromIndex(from_shape_index: number, to_shape_index: number) {
  if (!data.value) return;

  const coords = data.value.legs[0].shape.slice(from_shape_index, to_shape_index);
  const latitudes = coords.map((c) => c.lat);
  const longitudes = coords.map((c) => c.lon);
  indoorMap.value?.fitBounds(
    [Math.min(...longitudes), Math.max(...longitudes)],
    [Math.min(...latitudes), Math.max(...latitudes)],
  );
}
</script>

<template>
  <div
    class="flex max-h-[calc(100vh-60px)] min-h-[calc(100vh-60px)] flex-col md:max-h-[calc(100vh-150px)] md:min-h-[calc(100vh-150px)] md:flex-row-reverse"
  >
    <div class="grow">
      <IndoorMap ref="indoorMap" type="room" :coords="{ lat: 0, lon: 0, source: 'navigatum' }" />
    </div>
    <div class="bg-zinc-100 flex min-w-96 flex-col gap-3 overflow-auto p-4 md:max-w-96">
      <NuxtLinkLocale
        v-if="coming_from"
        :to="'/view/' + coming_from"
        property="item"
        class="focusable text-blue-400 rounded-md pb-2 hover:text-blue-500 hover:underline"
      >
        <div class="my-auto flex flex-row gap-2">
          <ChevronLeftIcon class="h-4 w-4" />
          <span class="text-xs font-semibold uppercase">{{ t("back") }}</span>
        </div>
      </NuxtLinkLocale>
      <NavigationModeSelector v-model:mode="mode" />
      <form action="/navigate" autocomplete="off" method="GET" role="search" class="flex flex-col gap-2">
        <NavigationSearchBar query-id="from" />
        <NavigationSearchBar query-id="to" />
      </form>
      <NavigationRoutingResults
        v-if="status === 'success' && !!data"
        :data="data"
        @select-maneuver="
          ({ begin_shape_index, end_shape_index }) => setBoundingBoxFromIndex(begin_shape_index, end_shape_index)
        "
      />
      <div v-else-if="status === 'pending'" class="text-zinc-900 flex flex-col items-center gap-5 py-32">
        <Spinner class="h-8 w-8" />
        {{ t("calculating best route") }}
      </div>
      <Toast v-else-if="status === 'error'" id="nav-error" level="error">
        {{ error }}
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
en:
  back: back
  calculating best route: Calculating best route
</i18n>
