<script setup lang="ts">
import { mdiMagnifyClose } from "@mdi/js";
import type { components } from "~/api_types";
import SearchResultItemLink from "~/components/SearchResultItemLink.vue";
import { useSearchFilters } from "~/composables/searchFilters";

type SearchResponse = components["schemas"]["SearchResponse"];

const props = defineProps<{
  data: SearchResponse;
  queryLimitSites: number;
  queryLimitBuildings: number;
  queryLimitRooms: number;
  queryLimitPois: number;
}>();
const { t } = useI18n({ useScope: "local" });
const route = useRoute();
const filters = useSearchFilters();

const SITE_BUMP = 20;
const BUILDING_BUMP = 20;
const ROOM_BUMP = 50;
const POI_BUMP = 20;

const hasNoResults = computed(() => props.data.sections.every((s) => s.estimatedTotalHits === 0));

function viewMoreQuery(facet: string) {
  // Bump only the facet whose "view more" was clicked; keep the other limits
  // at their current values so unrelated sections don't grow.
  return {
    q: route.query.q,
    limit_sites: props.queryLimitSites + (facet === "sites" ? SITE_BUMP : 0),
    limit_buildings: props.queryLimitBuildings + (facet === "buildings" ? BUILDING_BUMP : 0),
    limit_rooms: props.queryLimitRooms + (facet === "rooms" ? ROOM_BUMP : 0),
    limit_pois: props.queryLimitPois + (facet === "pois" ? POI_BUMP : 0),
    ...filters.buildQueryObject(),
  };
}
</script>

<template>
  <div
    v-if="hasNoResults"
    role="status"
    class="bg-zinc-50 dark:bg-zinc-900 border-zinc-200 dark:border-zinc-700 flex flex-col items-center gap-2 rounded-sm border px-4 py-10 text-center"
  >
    <MdiIcon :path="mdiMagnifyClose" :size="40" class="text-zinc-400 dark:text-zinc-500" />
    <p class="text-zinc-800 dark:text-zinc-100 text-md font-semibold">{{ t("no_results.title") }}</p>
    <p class="text-zinc-500 dark:text-zinc-400 text-sm">
      {{ filters.hasActiveFilters.value ? t("no_results.hint_filtered") : t("no_results.hint") }}
    </p>
    <Btn v-if="filters.hasActiveFilters.value" variant="linkButton" size="sm" @click="filters.clearAll()">
      {{ t("no_results.clear_filters") }}
    </Btn>
  </div>
  <template v-else>
    <div v-for="s in data.sections" :key="s.facet">
      <section v-if="s.entries.length" class="flex flex-col gap-2">
        <h2 class="text-md text-zinc-500 dark:text-zinc-400 font-semibold">{{ t(`sections.${s.facet}`) }}</h2>
        <ul v-for="(e, i) in s.entries" :key="e.id" class="flex flex-col gap-3">
          <SearchResultItemLink v-if="i < s.n_visible" :highlighted="false" :item="e" />
        </ul>
        <p v-if="s.estimatedTotalHits > 10" class="text-zinc-500 dark:text-zinc-400 text-sm">
          {{ t("approx_results", s.estimatedTotalHits) }}
          <NuxtLinkLocale
            :to="{ path: '/search', query: viewMoreQuery(s.facet) }"
            class="focusable text-blue-500 dark:text-blue-400 rounded-sm visited:text-blue-500 dark:visited:text-blue-400 hover:text-blue-600 dark:hover:text-blue-300 hover:underline"
            >{{ t("view_more") }}
          </NuxtLinkLocale>
        </p>
        <p v-else class="text-zinc-500 dark:text-zinc-400 text-sm">
          {{ t("results", s.estimatedTotalHits) }}
        </p>
      </section>
    </div>
  </template>
</template>

<i18n lang="yaml">
de:
  sections:
    sites: Standorte
    buildings: Gebäude
    rooms: Räume
    pois: POIs
  view_more: mehr anzeigen
  approx_results: ca. {count} Ergebnisse, bitte grenze die Suche weiter ein
  results: 1 Ergebnis | {count} Ergebnisse
  no_results:
    title: Keine Ergebnisse gefunden
    hint: Versuche es mit anderen Suchbegriffen.
    hint_filtered: Keine Treffer für diese Suche mit den aktiven Filtern. Versuche, Filter zu entfernen oder andere Suchbegriffe zu verwenden.
    clear_filters: Alle Filter entfernen
en:
  sections:
    sites: Sites
    buildings: Buildings
    rooms: Rooms
    pois: POIs
  view_more: view more
  approx_results: approx. {count} results, please narrow the search further
  results: 1 result | {count} results
  no_results:
    title: No results found
    hint: Try different keywords.
    hint_filtered: No matches for this query with the active filters. Try removing filters or using different keywords.
    clear_filters: Clear all filters
</i18n>
