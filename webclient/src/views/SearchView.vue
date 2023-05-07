<script setup lang="ts">
import { useFetch } from "@/utils/fetch";
import { ref } from "vue";
import { setDescription, setTitle } from "@/utils/common";
import { extractFacets } from "@/modules/autocomplete";
import { useGlobalStore } from "@/stores/global";
import { useI18n } from "vue-i18n";
import type { SectionFacet } from "@/modules/autocomplete";
import type { components } from "@/api_types";
type SearchResponse = components["schemas"]["SearchResponse"];

const { t } = useI18n({
  inheritLocale: true,
  useScope: "global",
});
const global = useGlobalStore();

const query: string = new URLSearchParams(document.location.search).get("q") || "";

const sections = ref<SectionFacet[] | null>(null);
const { data } = useFetch<SearchResponse>(getSearchAPIUrl(), (d) => {
  setTitle(`${t("view_search.search_for")} "${query}"`);
  setDescription(genDescription());
  // Currently borrowing this functionality from autocomplete.
  // In the future it is planned that this search results page
  // has a different format.
  sections.value = extractFacets(d, t);
});

function getSearchAPIUrl(): string {
  const params = new URLSearchParams();
  params.append("q", query);
  params.append("limit_buildings", "10");
  params.append("limit_rooms", "30");
  params.append("limit_all", "30");

  return `/api/search?${params.toString()}`;
}
function genDescription(): string {
  let sectionsDescr = "";
  let estimatedTotalHits = 0;
  data.value?.sections.forEach((section) => {
    if (section.estimatedTotalHits) {
      let facetStr;
      if (section.facet === "sites_buildings") {
        facetStr = t("search.sections.buildings");
        if (section.estimatedTotalHits !== section.n_visible) {
          const visibleStr = t("search.sections.of_which_visible");
          facetStr = `(${section.n_visible} ${visibleStr}) ${facetStr}`;
        }
      } else facetStr = t("search.sections.rooms");
      if (estimatedTotalHits > 0) sectionsDescr += t("search.sections.and");
      sectionsDescr += `${section.estimatedTotalHits} ${facetStr}`;
    }
    estimatedTotalHits += section.estimatedTotalHits;
  });
  if (estimatedTotalHits === 0) sectionsDescr = t("search.sections.no_buildings_rooms_found");
  else sectionsDescr += t("search.sections.were_found");
  return sectionsDescr;
}
</script>

<template>
  <div id="view-search" v-if="data">
    <small class="search_meta">
      {{ $t("view_search.runtime") }}: {{ data.time_ms }}ms –
      <button
        @click="global.openFeedback('search')"
        class="btn btn-link"
        aria-label="Open the feedback-form for search"
      >
        {{ $t("view_search.give_feedback") }}
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
            <RouterLink v-bind:to="'/view/' + e.id" class="tile tile-centered">
              <div class="tile-icon">
                <template v-if="e.type === 'room' || e.type === 'virtual_room'">
                  <i v-if="e.parsed_id" class="icon icon-search"></i>
                  <i v-else class="icon icon-location"></i>
                </template>
                <img
                  v-else
                  class="avatar avatar-sm"
                  src="../assets/thumb-building.webp"
                  alt="Thumbnail for said building"
                />
              </div>
              <div class="tile-content">
                <div class="tile-title">
                  <span v-if="e.parsed_id" v-html="e.parsed_id"></span>
                  <i v-if="e.parsed_id" class="icon icon-caret"></i>
                  <span v-html="e.name"></span>
                </div>
                <small class="tile-subtitle text-gray">
                  {{ e.subtext }}<template v-if="e.subtext_bold">, <b v-html="e.subtext_bold"></b></template>
                </small>
              </div>
              <!--<div class="tile-action">
              <button class="btn btn-link">
                <i class="icon icon-more-vert"></i>
              </button>
            </div>-->
            </RouterLink>
          </li>
        </ul>
        <p class="search-comment nb_results" v-if="s.estimatedTotalHits === 1">
          {{ s.estimatedTotalHits }} {{ $t("search.result") }}
        </p>
        <p class="search-comment nb_results" v-else>
          {{ s.estimatedTotalHits > 20 ? $t("search.approx") : "" }}
          {{ s.estimatedTotalHits }} {{ $t("search.results")
          }}{{ s.estimatedTotalHits > 10 ? ", " + $t("view_search.max_results") : "" }}
        </p>
      </section>
    </template>
  </div>
</template>

<style lang="scss">
@import "../assets/variables";

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
