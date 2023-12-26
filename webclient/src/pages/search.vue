<script setup lang="ts">
//import "@/assets/spectre-all.scss";
import { useFetch } from "@/composables/fetch";
import { computed } from "vue";
import { setDescription, setTitle } from "@/composables/common";
import type { SectionFacet } from "@/modules/autocomplete";
import { extractFacets } from "@/modules/autocomplete";
import { useGlobalStore } from "@/stores/global";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";

import type { components } from "@/api_types";
import { MapPinIcon, MagnifyingGlassIcon } from "@heroicons/vue/24/outline";
type SearchResponse = components["schemas"]["SearchResponse"];

const { t } = useI18n({ useScope: "local" });
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
  params.append("limit_rooms", "30");
  params.append("limit_all", "30");

  return `/api/search?${params.toString()}`;
});
// eslint-disable-next-line vue/no-ref-object-reactivity-loss
const { data } = useFetch<SearchResponse>(apiUrl.value, () => {
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
  <div v-if="data" id="view-search">
    <small class="search_meta">
      {{ t("runtime") }}: {{ data.time_ms }}ms –
      <button
        type="button"
        data-cy="open-feedback-search"
        class="btn btn-link"
        :aria-label="t('feedback.open')"
        @click="global.openFeedback('search')"
      >
        {{ t("feedback.give") }}
      </button>
    </small>

    <template v-for="s in sections" :key="s.type">
      <section>
        <div class="columns">
          <div class="column">
            <h2>{{ s.name }}</h2>
          </div>
        </div>
        <ul class="result-list">
          <li v-for="e in s.entries" :key="e.id">
            <RouterLink :to="'/view/' + e.id" class="tile tile-centered">
              <div class="tile-icon">
                <template v-if="e.type === 'room' || e.type === 'virtual_room'">
                  <MagnifyingGlassIcon v-if="e.parsed_id" class="h-4 w-4" />
                  <MapPinIcon v-else class="h-4 w-4" />
                </template>
                <img v-else class="avatar avatar-sm" src="@/assets/thumb-building.webp" :alt="t('thumbnail_alt')" />
              </div>
              <div class="tile-content">
                <div class="tile-title">
                  <span v-if="e.parsed_id" v-html="e.parsed_id" />
                  <i v-if="e.parsed_id" class="icon icon-caret" />
                  <span v-html="e.name" />
                </div>
                <small class="text-gray tile-subtitle">
                  {{ e.subtext }}<template v-if="e.subtext_bold">, <b v-html="e.subtext_bold"></b></template>
                </small>
              </div>
              <!-- <div class="tile-action">
              <button class="btn btn-link">
                <i class="icon icon-more-vert" />
              </button>
            </div> -->
            </RouterLink>
          </li>
        </ul>
        <p class="nb_results search-comment">
          {{ s.estimatedTotalHits > 20 ? t("approx") : "" }}
          {{ t("results", s.estimatedTotalHits) }}{{ s.estimatedTotalHits > 10 ? ", " + t("max_results") : "" }}
        </p>
      </section>
    </template>
  </div>
</template>

<style lang="scss">
@import "@/assets/variables";

#view-search {
  padding-top: 25px;

  h1 {
    font-size: 1.2rem;
    font-weight: 500;
  }

  h2 {
    font-size: 1rem;
    font-weight: 500;
  }

  section {
    margin-top: 40px;

    .search-comment {
      &.nb_results {
        color: $text-gray;
      }
    }
  }

  .divider + section {
    margin-top: 30px;
  }

  ul.result-list {
    list-style: none;
    margin-left: 0;
    margin-top: 0;

    li {
      padding: 8px 10px 6px;
      border-radius: 6px;
      box-shadow: 3px 3px 4px rgba(106, 106, 106, 1%);
      border: 0.05rem solid $search-border;
      transition: border 0.2s;

      &:hover {
        box-shadow: 3px 3px 4px rgba(106, 106, 106, 3%);
        border: 0.05rem solid $search-border-hover;
      }

      a {
        text-decoration: none;
        color: $body-font-color;
      }

      .tile-title {
        line-height: 1rem;

        i.icon-caret {
          transform: rotate(-90deg);
        }
      }

      .tile-subtitle {
        line-height: 1rem;
      }

      em {
        font-style: normal;
        font-weight: bold;
        color: $theme-accent;
      }
    }
  }

  small {
    button {
      // feedback-form, ..
      font-size: 12px;
      padding: 0;
    }

    .search_meta {
      display: block;

      // color: $text-gray;

      a {
        color: $body-font-color;
      }
    }
  }
}
</style>

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
    give: Feedback zur Suche geben
    open: Feedback-Formular für Rückmeldungen zur Suchanfrage geben
  max_results: bitte grenze die Suche weiter ein
  runtime: Laufzeit
  search_for: Suche nach
  thumbnail_alt: Vorschaubild für das besagte Gebäude
  approx: ca.
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
    give: Send feedback to search
    open: Open the feedback-form for feedback about the search
  max_results: please narrow the search further
  runtime: Runtime
  search_for: Search for
  thumbnail_alt: Thumbnail for said building
  approx: approx.
  results: 1 result | {count} results
</i18n>
