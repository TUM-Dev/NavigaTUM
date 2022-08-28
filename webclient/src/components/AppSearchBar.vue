<script lang="ts">
import { onInput, onKeyDown } from "@/modules/autocomplete";
import router from "@/router";
import { useSearchBarStore } from "@/stores/search_focus";

export default {
  data() {
    return {
      store: useSearchBarStore(),
      keep_focus: false,
      query: "",
      autocomplete: {
        sections: [],
        highlighted: null,
      },
    };
  },
  methods: {
    searchFocus() {
      this.store.focus();
      this.autocomplete.highlighted = null;
    },
    searchBlur() {
      if (this.keep_focus) {
        window.setTimeout(() => {
          // This is relevant if the call is delayed and focused has
          // already been disabled e.g. when clicking on an entry.
          if (this.store.focused) document.getElementById("search")?.focus();
        }, 0);
        this.keep_focus = false;
      } else {
        this.store.unfocus();
      }
    },
    searchInput(e) {
      onInput(e.srcElement.value);
    },
    searchKeydown: function (e) {
      onKeyDown(e);
    },
    searchExpand(s) {
      s.expanded = true;
    },
    searchGo(cleanQuery: boolean) {
      if (this.query.length === 0) return;

      router.push(`/search?q=${this.query}`);
      this.store.unfocus();
      if (cleanQuery) {
        this.query = "";
        this.autocomplete.sections = [];
      }
      document.getElementById("search")?.blur();
    },
    searchGoTo(id: string, cleanQuery: boolean) {
      // Catch is necessary because vue-router throws an error
      // if navigation is aborted for some reason (e.g. the new
      // url is the same or there is a loop in redirects)
      router.push(`/view/${id}`);
      this.store.unfocus();
      if (cleanQuery) {
        this.query = "";
        this.autocomplete.sections = [];
      }
      document.getElementById("search")?.blur();
    },
  },
};
</script>

<style scoped lang="scss">
@import "../assets/variables";

.form-autocomplete {
  .menu {
    box-shadow: $autocomplete-box-shadow;

    .menu-item {
      & > a {
        cursor: pointer;

        &.active {
          color: #fff;
          background-color: $theme-accent;
        }

        em {
          color: $theme-accent;
          font-style: normal;
          font-weight: bold;
        }

        &:focus em,
        &:hover em,
        &.active em {
          color: #fff;
        }
      }
    }
  }

  .tile-content {
    max-width: 100%;
    margin-bottom: -5px;
    line-height: 100%;
    padding-bottom: 0.2rem;
  }

  .tile-title {
    margin-right: 3px;

    i.icon-caret {
      transform: rotate(-90deg);
    }
  }

  .tile-subtitle {
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
    padding-right: 16px;
    display: inline-block;
    overflow: hidden;
    vertical-align: middle;
    margin-top: -5px;

    // Correction for Chrome
    padding-top: 2px;
    height: 1.2rem;
  }

  .menu .search-comment {
    margin: -8px -8px 0;
    padding: 6px 16px;
    font-size: 14px;
    color: $autocomplete-comment-color;

    &.filter {
      color: $autocomplete-filter-text;
      background-color: $autocomplete-filter-bg;
      border-bottom: 1px solid $border-light;

      > a {
        display: inline;
      }
    }

    &.nb_results {
      margin: -4px 0;
      padding: 4px 8px;

      > a {
        cursor: pointer;
      }
    }

    &.actions {
      margin: -4px 0 -4px 32px;
      padding: 4px 8px;
      overflow-x: auto;
      white-space: nowrap;

      div {
        display: inline-block;
        margin-right: 8px;
      }

      button {
        margin-top: 6px;
        margin-bottom: 3px;
      }
    }
  }
}
</style>

<template>
  <div class="form-autocomplete">
    <div class="input-group has-icon-left">
      <input
        id="search"
        type="text"
        class="form-input input-lg"
        v-bind:placeholder="$t('search.placeholder')"
        v-model="query"
        @input="searchInput"
        @focus="searchFocus"
        @blur="searchBlur"
        @keydown="searchKeydown"
        autocomplete="off"
        v-bind:aria-label="$t('search.aria-searchlabel')"
      />
      <i class="form-icon icon icon-search"></i>
      <button
        class="btn btn-primary input-group-btn btn-lg"
        @click="searchGo(false)"
        v-bind:aria-label="$t('search.aria-actionlabel')"
      >
        {{ $t("search.action") }}
      </button>
    </div>
    <!-- Autocomplete -->
    <ul
      class="menu"
      v-bind:class="{
        'd-none': !store.focused || autocomplete.sections.length === 0,
      }"
      v-cloak
    >
      <!--<li class="search-comment filter">
                    Suche einschränken auf:
                    <a class="bt btn-link btn-sm">Räume</a>
                  </li>-->

      <template v-for="s in autocomplete.sections">
        <li
          class="divider"
          DISABLED-v-if="autocomplete.sections.length > 1"
          v-bind:data-content="s.name"
        ></li>
        <li
          v-for="(e, i) in s.entries"
          v-if="s.n_visible === undefined || i < s.n_visible || s.expanded"
          class="menu-item"
        >
          <a
            v-bind:class="{
              active: e.id === autocomplete.highlighted,
            }"
            @click="searchGoTo(e.id, true)"
            @mousedown="keep_focus = true"
            @mouseover="autocomplete.highlighted = null"
          >
            <div class="tile">
              <div class="tile-icon">
                <template v-if="e.type === 'room' || e.type === 'virtual_room'">
                  <i v-if="e.parsed_id" class="icon icon-search"></i>
                  <i v-else class="icon icon-location"></i>
                </template>
                <img
                  v-else
                  src="@/assets/thumb-building.webp"
                  class="avatar avatar-sm"
                />
              </div>
              <div class="tile-content">
                <span class="tile-title">
                  <span v-if="e.parsed_id" v-html="e.parsed_id"></span>
                  <i v-if="e.parsed_id" class="icon icon-caret"></i>
                  <span
                    v-html="e.name"
                    v-bind:style="{ opacity: e.parsed_id ? 0.5 : 1 }"
                  ></span>
                </span>
                <small class="tile-subtitle text-gray">
                  {{ e.subtext }}
                  <template v-if="e.subtext_bold"
                    >, <b v-html="e.subtext_bold"></b
                  ></template>
                </small>
              </div>
            </div>
          </a>
          <!--<div class="menu-badge">
                        <label class="label label-primary">2</label>
                      </div>-->
        </li>

        <li class="search-comment nb_results">
          <a
            v-if="!s.expanded && s.n_visible < s.entries.length"
            @mousedown="keep_focus = true"
            @click="searchExpand(s)"
          >
            +{{ s.entries.length - s.n_visible }} {{ $t("search.hidden") }},
          </a>
          <template>
            {{ s.estimatedTotalHits > 20 ? $t("approx") : ""
            }}{{ s.estimatedTotalHits }}
            {{
              s.estimatedTotalHits === 1
                ? $t("search.result")
                : $t("search.results")
            }}
          </template>
        </li>
      </template>

      <!--<li class="search-comment actions">
                    <div>
                      <button class="btn btn-sm">
                        <i class="icon icon-arrow-right"></i> in Gebäude Suchen
                      </button>
                    </div>
                    <div>
                      <button class="btn btn-sm">
                        <i class="icon icon-location"></i> Hörsäle
                      </button>
                    </div>
                    <div>
                      <button class="btn btn-sm">
                        <i class="icon icon-location"></i> Seminarräume
                      </button>
                    </div>
                  </li>-->

      <!--<li class="divider" data-content="Veranstaltungen"></li>
                  <li class="menu-item">
                    <a href="#">
                      <div class="tile">
                        <div class="tile-icon">
                          <i class="icon icon-time"></i>
                        </div>
                        <div class="tile-content">
                          <span class="tile-title">
                            Advanced Practical Course Games Engineering: Building Information Modeling (IN7106)
                          </span>
                          <small class="tile-subtitle text-gray">
                            Übung mit 4 Gruppen
                          </small>
                        </div>
                      </div>
                    </a>
                    <div class="menu-badge" style="display: none;">
                      <label class="label label-primary">frei</label>
                    </div>
                  </li>-->
    </ul>
  </div>
</template>
