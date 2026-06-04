<script setup lang="ts">
import type { components } from "~/api_types";
import SearchSectionList from "~/components/SearchSectionList.vue";
import { firstOrDefault } from "~/composables/common";
import { useSearchFilters } from "~/composables/searchFilters";

type SearchResponse = components["schemas"]["SearchResponse"];

const { t, locale } = useI18n({ useScope: "local" });
const route = useRoute();
const runtimeConfig = useRuntimeConfig();
const feedback = useFeedback();
const filters = useSearchFilters();

const query_q = computed<string>(() => firstOrDefault(route.query.q, ""));
const query_limit_sites = computed<number>(() =>
  Number.parseInt(firstOrDefault(route.query.limit_sites, "10"), 10)
);
const query_limit_buildings = computed<number>(() =>
  Number.parseInt(firstOrDefault(route.query.limit_buildings, "10"), 10)
);
const query_limit_rooms = computed<number>(() =>
  Number.parseInt(firstOrDefault(route.query.limit_rooms, "50"), 10)
);
const query_limit_pois = computed<number>(() =>
  Number.parseInt(firstOrDefault(route.query.limit_pois, "10"), 10)
);
const query_limit_all = computed<number>(
  () =>
    query_limit_sites.value +
    query_limit_buildings.value +
    query_limit_rooms.value +
    query_limit_pois.value
);
const apiUrl = computed(() => {
  const params = new URLSearchParams();
  params.append("q", query_q.value);
  params.append("limit_sites", query_limit_sites.value.toString());
  params.append("limit_buildings", query_limit_buildings.value.toString());
  params.append("limit_rooms", query_limit_rooms.value.toString());
  params.append("limit_pois", query_limit_pois.value.toString());
  params.append("limit_all", query_limit_all.value.toString());
  params.append("lang", locale.value);
  params.append("pre_highlight", "<b class='text-blue'>");
  params.append("post_highlight", "</b>");
  filters.appendToParams(params);

  return `${runtimeConfig.public.apiURL}/api/search?${params.toString()}`;
});
const { data } = useFetch<SearchResponse>(apiUrl, {
  dedupe: "cancel",
  credentials: "omit",
  retry: 120,
  retryDelay: 1000,
});
const description = computed(() => {
  if (!data.value) return "";
  let sectionsDescr = "";
  let estimatedTotalHits = 0;
  for (const section of data.value.sections) {
    if (section.estimatedTotalHits) {
      let facetStr = t(`sections.${section.facet}`);
      if (section.estimatedTotalHits !== section.n_visible) {
        const visibleStr = t("sections.of_which_visible");
        facetStr = `(${section.n_visible} ${visibleStr}) ${facetStr}`;
      }
      if (estimatedTotalHits > 0) sectionsDescr += t("sections.and");
      sectionsDescr += `${section.estimatedTotalHits} ${facetStr}`;
    }
    estimatedTotalHits += section.estimatedTotalHits;
  }
  if (estimatedTotalHits === 0) sectionsDescr = t("sections.nothing_found");
  else sectionsDescr += t("sections.were_found");
  return sectionsDescr;
});
const title = computed(() => `${t("search_for")} "${route.query.q}"`);
useSeoMeta({
  title: title,
  ogTitle: title,
  description: description,
  ogDescription: description,
  ogImage: "https://nav.tum.de/navigatum-card.png",
  twitterCard: "summary",
});
</script>

<template>
  <div v-if="data" class="flex flex-col gap-5 pt-5">
    <small class="text-zinc-500">
      {{ t("runtime") }}: {{ data.time_ms }}ms –
      <button
        data-cy="open-feedback-search"
        type="button"
        class="focusable text-blue-600 visited:text-blue-600 hover:text-blue-500"
        :aria-label="t('feedback.open')"
        @click="
          () => {
            feedback.open = true;
            feedback.data = {
              category: 'search',
              subject: '',
              body: '',
              deletion_requested: false,
            };
          }
        "
      >
        {{ t("feedback.give") }}
      </button>
    </small>
    <div class="flex flex-wrap items-center gap-x-4 gap-y-2">
      <SearchFilterChips :filters="filters" />
      <div class="ms-auto">
        <SearchSortControl :filters="filters" />
      </div>
    </div>
    <ClientOnly>
      <SearchSectionList
        :data="data"
        :query-limit-sites="query_limit_sites"
        :query-limit-buildings="query_limit_buildings"
        :query-limit-rooms="query_limit_rooms"
        :query-limit-pois="query_limit_pois"
      />
    </ClientOnly>
  </div>
</template>

<i18n lang="yaml">
de:
  sections:
    sites: Standorte
    buildings: Gebäude
    rooms: Räume
    pois: POIs
    addresses: Adressen
    and: und
    nothing_found: Es konnten keine Ergebnisse gefunden werden.
    of_which_visible: davon sichtbar
    were_found: wurden gefunden.
  feedback:
    give: Feedback zu Sucheergebnissen geben
    open: Feedback-Formular für Rückmeldungen zur Suchanfrage geben
  runtime: Laufzeit
  search_for: Suche nach
en:
  sections:
    sites: Sites
    buildings: Buildings
    rooms: Rooms
    pois: POIs
    addresses: Addresses
    and: and
    nothing_found: No results could be found.
    of_which_visible: of them visible
    were_found: were found.
  feedback:
    give: Send feedback about search results
    open: Open the feedback-form for feedback about the search
  runtime: Runtime
  search_for: Search for
</i18n>
