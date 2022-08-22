<script setup lang="ts">

function searchNavigateTo(to, from, next, component) {
  navigatum.beforeNavigate(to, from);

  const params = new URLSearchParams();
  params.append("q", to.query.q);
  params.append("limit_buildings", "10");
  params.append("limit_rooms", "30");
  params.append("limit_all", "30");

  /* global cachedFetch */
  cachedFetch
    .fetch(`${navigatum.apiBase}search?${params.toString()}`, {
      cache: "no-cache",
    })
    .then((resp) => {
      if (component) {
        next();
        navigatum.afterNavigate(to, from);
        component.loadSearchData(to.query.q, resp);
      } else {
        next((vm) => {
          navigatum.afterNavigate(to, from);
          vm.loadSearchData(to.query.q, resp);
        });
      }
    });
}

const _searchDefaultState = {};

export default {
  name: "view-search",
  template: { gulp_inject: "view-search.inc" },
  data: function () {
    return {
      search_data: null,
      sections: null,
      query: null,
      // State is preserved when navigating in history.
      // May only contain serializable objects!
      state: structuredClone(_searchDefaultState),
    };
  },
  beforeRouteEnter: function (to, from, next) {
    searchNavigateTo(to, from, next, null);
  },
  beforeRouteUpdate: function (to, from, next) {
    searchNavigateTo(to, from, next, this);
  },
  methods: {
    genDescription: function (data) {
      let sectionsDescr = "";
      let estimatedTotalHits = 0;
      data.sections.forEach((section) => {
        if (section.estimatedTotalHits) {
          let facetStr;
          if (section.facet === "sites_buildings") {
            facetStr = {{ $t("search.sections.buildings ") }};
            if (section.estimatedTotalHits !== section.n_visible) {
              const visibleStr = {{ $t("search.sections.of_which_visible ") }};
              facetStr = `(${section.n_visible} ${visibleStr}) ${facetStr}`;
            }
          } else facetStr = {{ $t("search.sections.rooms ") }};
          if (estimatedTotalHits > 0)
            sectionsDescr += " {{ $t("search.sections.and ") }} ";
          sectionsDescr += `${section.estimatedTotalHits} ${facetStr}`;
        }
        estimatedTotalHits += section.estimatedTotalHits;
      });
      if (estimatedTotalHits === 0)
        sectionsDescr = "{{ $t("search.sections.no_buildings_rooms_found ") }}";
      else sectionsDescr += " {{ $t("search.sections.were_found ") }}";
      return sectionsDescr;
    },
    loadSearchData: function (query, data) {
      this.search_data = data;
      this.query = query;
      navigatum.app.search.query = query;
      navigatum.setTitle(`\{{ $t("view_search.search_for ") }} "${query}"`);
      navigatum.setDescription(this.genDescription(data));
      // Currently borrowing this functionality from autocomplete.
      // In the future it is planned that this search results page
      // has a different format.
      const _this = this;
      navigatum.getModule("autocomplete").then((c) => {
        _this.sections = c.extractFacets(data);
      });
    },
  },
};
</script>

<style lang="scss">
@import "src/assets/variables";

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
            border: .05rem solid $search-border;
            transition: border .2s;

            &:hover {
                box-shadow: 3px 3px 4px rgba(106, 106, 106, 3%);
                border: .05rem solid $search-border-hover;
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
        button { // feedback-form, ..
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

<template>
  <div id="view-search" v-if="search_data">
  <small class="search_meta">
    {{ $t("view_search.runtime ") }}: {{ search_data.time_ms }}ms –
    <button
      onclick="openFeedback('search')"
      class="btn btn-link"
      aria-label="Open the feedback-form for search"
    >
      {{ $t("view_search.give_feedback") }}
    </button>
  </small>

  <template v-for="s in sections">
    <section>
      <div class="columns">
        <div class="column">
          <h2>{{ s.name }}</h2>
        </div>
      </div>
      <ul class="result-list">
        <li v-for="e in s.entries">
          <router-link v-bind:to="'/view/' + e.id" class="tile tile-centered">
            <div class="tile-icon">
              <template v-if="e.type == 'room' || e.type == 'virtual_room'">
                <i v-if="e.parsed_id" class="icon icon-search"></i>
                <i v-else class="icon icon-location"></i>
              </template>
              <img
                v-else
                class="avatar avatar-sm"
                src="<!-- @echo app_prefix -->assets/thumb-building.webp"
              />
            </div>
            <div class="tile-content">
              <div class="tile-title">
                <span v-if="e.parsed_id" v-html="e.parsed_id"></span>
                <i v-if="e.parsed_id" class="icon icon-caret"></i>
                <span v-html="e.name"></span>
              </div>
              <small class="tile-subtitle text-gray">
                {{ e.subtext }}<template v-if="e.subtext_bold"
                  >, <b v-html="e.subtext_bold"></b
                ></template>
              </small>
            </div>
            <!--<div class="tile-action">
              <button class="btn btn-link">
                <i class="icon icon-more-vert"></i>
              </button>
            </div>-->
          </router-link>
        </li>
      </ul>
      <p class="search-comment nb_results" v-if="s.estimatedTotalHits === 1">
        {{ s.estimatedTotalHits }} {{ $t("search.result") }}
      </p>
      <p class="search-comment nb_results" v-else>
        {{ s.estimatedTotalHits > 20 ? {{ $t("search.approx") }} : "" }}
        {{ s.estimatedTotalHits }} {{ $t("search.results") }}{{ s.estimatedTotalHits > 10 ? ", "+{{ $t("view_search.max_results ") }} : ""}}
      </p>
      </section>
    </template>
  </div>
</template>
