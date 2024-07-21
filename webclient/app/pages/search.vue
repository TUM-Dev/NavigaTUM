<script setup lang="ts">
import type { components } from "~/api_types";
import SearchSectionList from "~/components/SearchSectionList.vue";
import type { LocationQueryValue } from "vue-router";

type SearchResponse = components["schemas"]["SearchResponse"];

const { t, locale } = useI18n({ useScope: "local" });
const route = useRoute();
const runtimeConfig = useRuntimeConfig();
const feedback = useFeedback();

function firstOrDefault(value: LocationQueryValue | LocationQueryValue[] | undefined, defaultValue: string): string {
  if (Array.isArray(value)) return value[0] ?? defaultValue;
  return value ?? defaultValue;
}

const query_q = computed<string>(() => firstOrDefault(route.query.q, ""));
const query_limit_buildings = computed<number>(() => parseInt(firstOrDefault(route.query.limit_buildings, "10")));
const query_limit_rooms = computed<number>(() => parseInt(firstOrDefault(route.query.limit_rooms, "50")));
const query_limit_all = computed<number>(() => query_limit_rooms.value + query_limit_rooms.value);
const apiUrl = computed(() => {
  const params = new URLSearchParams();
  params.append("q", query_q.value);
  params.append("limit_buildings", query_limit_buildings.value.toString());
  params.append("limit_rooms", query_limit_rooms.value.toString());
  params.append("limit_all", query_limit_all.value.toString());
  params.append("lang", locale.value);
  params.append("pre_highlight", "<b class='text-blue'>");
  params.append("post_highlight", "</b>");

  return `${runtimeConfig.public.apiURL}/api/search?${params.toString()}`;
});
const { data } = useFetch<SearchResponse>(apiUrl, {
  key: "search",
  dedupe: "cancel",
  credentials: "omit",
  retry: 120,
  retryDelay: 1000,
});
const description = computed(() => {
  if (!data.value) return "";
  let sectionsDescr = "";
  let estimatedTotalHits = 0;
  data.value.sections.forEach((section) => {
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
    <ClientOnly>
      <SearchSectionList
        :data="data"
        :query-limit-buildings="query_limit_buildings"
        :query-limit-rooms="query_limit_rooms"
      />
    </ClientOnly>
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
</i18n>
