<script setup lang="ts">
import { BuildingOffice2Icon, BuildingOfficeIcon, MagnifyingGlassIcon, MapPinIcon } from "@heroicons/vue/24/outline";
import { ChevronDownIcon } from "@heroicons/vue/16/solid";
type SearchResponse = components["schemas"]["SearchResponse"];
import { extractFacets, type SectionFacet } from "~/composables/autocomplete";
import type { components } from "~/api_types";

const route = useRoute();
const props = defineProps<{
  data: SearchResponse;
  query_limit_buildings: number;
  query_limit_rooms: number;
}>();

const { t } = useI18n({ useScope: "local" });
const sections = computed<SectionFacet[]>(() => {
  // Currently borrowing this functionality from autocomplete.
  // In the future it is planned that this search results page
  // has a different format.
  return extractFacets(props.data, t("sections.rooms"), t("sections.buildings"));
});
</script>

<template>
  <div v-for="s in sections" :key="s.facet">
    <section class="flex flex-col gap-2">
      <h2 class="text-md text-zinc-500 font-semibold">{{ s.name }}</h2>
      <ul class="flex flex-col gap-3">
        <li v-for="e in s.entries" :key="e.id" class="bg-zinc-50 border-zinc-200 rounded-sm border hover:bg-blue-100">
          <NuxtLink :to="'/view/' + e.id" class="focusable flex gap-3 p-4">
            <div class="my-auto min-w-11">
              <div v-if="e.type === 'room' || e.type === 'virtual_room'" class="text-zinc-900 p-2">
                <MagnifyingGlassIcon v-if="e.parsed_id" class="h-6 w-6" />
                <MapPinIcon v-else class="h-6 w-6" />
              </div>
              <div v-else class="text-white bg-blue-500 rounded-full p-2">
                <BuildingOfficeIcon v-if="e.type === 'building'" class="mx-auto h-6 w-6" />
                <BuildingOffice2Icon v-else class="mx-auto h-6 w-6" />
              </div>
            </div>
            <div class="text-zinc-600 flex flex-col gap-0.5">
              <div class="flex flex-row">
                <span v-if="e.parsed_id" v-html="e.parsed_id" />
                <ChevronDownIcon v-if="e.parsed_id" class="h-4 w-4" />
                <span class="line-clamp-1" v-html="e.name" />
              </div>
              <small>
                {{ e.subtext }}
                <template v-if="e.subtext_bold">, <b v-html="e.subtext_bold"></b></template>
              </small>
            </div>
            <!-- <div class="tile-action">
          <button class="btn btn-link">
            <EllipsisVerticalIcon class="h-4 w-4"
          </button>
        </div> -->
          </NuxtLink>
        </li>
      </ul>
      <p v-if="s.estimatedTotalHits > 10" class="text-zinc-500 text-sm">
        {{ t("approx_results", s.estimatedTotalHits) }}
        <NuxtLink
          :to="
            s.facet === 'rooms'
              ? `/search?q=${route.query.q}&limit_buildings=${query_limit_buildings}&limit_rooms=${query_limit_rooms + 50}`
              : `/search?q=${route.query.q}&limit_buildings=${query_limit_buildings + 20}&limit_rooms=${query_limit_rooms}`
          "
          class="focusable text-blue-500 rounded-sm visited:text-blue-500 hover:text-blue-600 hover:underline"
          >{{ t("view_more") }}</NuxtLink
        >
      </p>
      <p v-else class="text-zinc-500 text-sm">
        {{ t("results", s.estimatedTotalHits) }}
      </p>
    </section>
  </div>
</template>

<i18n lang="yaml">
de:
  sections:
    buildings: Gebäude / Standorte
    rooms: Räume
  view_more: mehr anzeigen
  approx_results: ca. {count} Ergebnisse, bitte grenze die Suche weiter ein
  results: 1 Ergebnis | {count} Ergebnisse
en:
  sections:
    buildings: Buildings / Sites
    rooms: Rooms
  view_more: view more
  approx_results: approx. {count} results, please narrow the search further
  results: 1 result | {count} results
</i18n>
