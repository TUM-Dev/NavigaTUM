<script setup lang="ts">
import { useFetch } from "@/composables/fetch";
import { computed } from "vue";
import { setDescription, setTitle } from "@/composables/common";
import type { SectionFacet } from "@/modules/autocomplete";
import { extractFacets } from "@/modules/autocomplete";
import { useGlobalStore } from "@/stores/global";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";

import type { components } from "@/api_types";
import { ChevronDownIcon } from "@heroicons/vue/16/solid";
import { MapPinIcon, MagnifyingGlassIcon, BuildingOfficeIcon, BuildingOffice2Icon } from "@heroicons/vue/24/outline";
type SearchResponse = components["schemas"]["SearchResponse"];

const { t, locale } = useI18n({ useScope: "local" });
const global = useGlobalStore();
const route = useRoute();

const sections = computed<SectionFacet[] | null>(() => {
  if (data.value === null) return null;
  // Currently borrowing this functionality from autocomplete.
  // In the future it is planned that this search results page
  // has a different format.
  return extractFacets(data.value, t("sections.rooms"), t("sections.buildings"));
});
const apiUrl = computed(() => {
  const q = route.query.q;
  const params = new URLSearchParams();
  if (typeof q === "string") {
    params.append("q", q);
  }
  params.append("limit_buildings", "10");
  params.append("limit_rooms", "50");
  params.append("limit_all", "60");
  params.append("lang", locale.value);

  return `/api/search?${params.toString()}`;
});
const { data } = useFetch<SearchResponse>(apiUrl, () => {
  setTitle(`${t("search_for")} "${route.query.q}"`);
  setDescription(genDescription());
});

function genDescription(): string {
  let sectionsDescr = "";
  let estimatedTotalHits = 0;
  data.value?.sections.forEach((section) => {
    if (section.estimatedTotalHits) {
      let facetStr;
      if (section.facet === "sites_buildings") {
        facetStr = t("sections.buildings");
        if (section.estimatedTotalHits !== section.n_visible) {
          const visibleStr = t("sections.of_which_visible");
          facetStr = `(${section.n_visible} ${visibleStr}) ${facetStr}`;
        }
      } else facetStr = t("sections.rooms");
      if (estimatedTotalHits > 0) sectionsDescr += t("sections.and");
      sectionsDescr += `${section.estimatedTotalHits} ${facetStr}`;
    }
    estimatedTotalHits += section.estimatedTotalHits;
  });
  if (estimatedTotalHits === 0) sectionsDescr = t("sections.no_buildings_rooms_found");
  else sectionsDescr += t("sections.were_found");
  return sectionsDescr;
}
</script>

<template>
  <div v-if="data" class="flex flex-col gap-5 pt-5">
    <small class="text-zinc-500">
      {{ t("runtime") }}: {{ data.time_ms }}ms –
      <button
        data-cy="open-feedback-search"
        type="button"
        class="focusable text-tumBlue-600 visited:text-tumBlue-600 hover:text-tumBlue-500"
        :aria-label="t('feedback.open')"
        @click="global.openFeedback('search')"
      >
        {{ t("feedback.give") }}
      </button>
    </small>

    <template v-for="s in sections" :key="s.type">
      <section class="flex flex-col gap-2">
        <h2 class="text-md text-zinc-500 font-semibold">{{ s.name }}</h2>
        <ul class="flex flex-col gap-3">
          <li v-for="e in s.entries" :key="e.id" class="focusable rounded-sm border hover:bg-tumBlue-50">
            <RouterLink :to="'/view/' + e.id" class="flex gap-3 p-4">
              <div class="my-auto min-w-11">
                <div v-if="e.type === 'room' || e.type === 'virtual_room'" class="text-zinc-900 p-2">
                  <MagnifyingGlassIcon v-if="e.parsed_id" class="h-6 w-6" />
                  <MapPinIcon v-else class="h-6 w-6" />
                </div>
                <div v-else class="text-white bg-tumBlue-500 rounded-full p-2">
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
                  {{ e.subtext }}<template v-if="e.subtext_bold">, <b v-html="e.subtext_bold"></b></template>
                </small>
              </div>
              <!-- <div class="tile-action">
              <button class="btn btn-link">
                <EllipsisVerticalIcon class="h-4 w-4"
              </button>
            </div> -->
            </RouterLink>
          </li>
        </ul>
        <p v-if="s.estimatedTotalHits > 20" class="text-zinc-500 text-sm">
          {{ t("approx_results", s.estimatedTotalHits) }}
        </p>
        <p v-else class="text-zinc-500 text-sm">
          {{ t("results", s.estimatedTotalHits) }}
        </p>
      </section>
    </template>
  </div>
</template>

<i18n lang="yaml">
de:
  sections:
    buildings: Gebäude / Standorte
    rooms: Räume
    and: und
    no_buildings_rooms_found: Keine Gebäude / Standorte oder Räume konnten gefunden werden.
    of_which_visible: davon sichtbar
    were_found: wurden gefunden.
  feedback:
    give: Feedback zu Sucheergebnissen geben
    open: Feedback-Formular für Rückmeldungen zur Suchanfrage geben
  runtime: Laufzeit
  search_for: Suche nach
  thumbnail_alt: Vorschaubild für das besagte Gebäude
  approx_results: ca. {count} Ergebnisse, bitte grenze die Suche weiter ein
  results: 1 Ergebnis | {count} Ergebnisse
en:
  sections:
    buildings: Buildings / Sites
    rooms: Rooms
    and: and
    no_buildings_rooms_found: No buildings / locations or rooms could be found.
    of_which_visible: of them visible
    were_found: were found.
  feedback:
    give: Send feedback about search results
    open: Open the feedback-form for feedback about the search
  runtime: Runtime
  search_for: Search for
  thumbnail_alt: Thumbnail for said building
  approx_results: approx. {count} results, please narrow the search further
  results: 1 result | {count} results
</i18n>
