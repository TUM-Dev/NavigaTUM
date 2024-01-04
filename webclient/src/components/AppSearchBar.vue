<script setup lang="ts">
import { extractFacets } from "@/modules/autocomplete";
import { useRouter } from "vue-router";
import { useGlobalStore } from "@/stores/global";
import { useI18n } from "vue-i18n";
import { useFetch } from "@/composables/fetch";
import { computed, onMounted, reactive, ref } from "vue";
import type { SectionFacet } from "@/modules/autocomplete";
import type { components } from "@/api_types";

import { MapPinIcon, MagnifyingGlassIcon } from "@heroicons/vue/24/outline";
type SearchResponse = components["schemas"]["SearchResponse"];

const { t } = useI18n({ useScope: "local" });
const global = useGlobalStore();
const keep_focus = ref(false);
const query = ref("");
const autocomplete = reactive({ sections: [] as SectionFacet[], highlighted: null as string | null });
// As a simple measure against out-of-order responses
// to the autocompletion, we count queries and make sure
// that late results will not overwrite the currently
// visible results.
const queryCounter = ref(0);
const latestUsedQueryId = ref(-1);
const router = useRouter();

const visibleElements = computed<string[]>(() => {
  const visible: string[] = [];

  autocomplete.sections.forEach((section) => {
    section.entries.forEach((entry, index: number) => {
      if (section.facet !== "sites_buildings" || section.n_visible > index || section.expanded) visible.push(entry.id);
    });
  });
  return visible;
});

function searchFocus(): void {
  global.focusSearchBar();
  autocomplete.highlighted = null;
}

function searchBlur(): void {
  if (keep_focus.value) {
    window.setTimeout(() => {
      // This is relevant if the call is delayed and focused has
      // already been disabled e.g. when clicking on an entry.
      if (global.search_focused) document.getElementById("search")?.focus();
    }, 0);
    keep_focus.value = false;
  } else {
    global.unfocusSearchBar();
  }
}

function searchGo(cleanQuery: boolean): void {
  if (query.value.length === 0) return;

  router.push(`/search?q=${query.value}`);
  global.unfocusSearchBar();
  if (cleanQuery) {
    query.value = "";
    autocomplete.sections = [];
  }
  document.getElementById("search")?.blur();
}

function searchGoTo(id: string, cleanQuery: boolean): void {
  // Catch is necessary because vue-router throws an error
  // if navigation is aborted for some reason (e.g. the new
  // url is the same or there is a loop in redirects)
  router.push(`/view/${id}`);
  global.unfocusSearchBar();
  if (cleanQuery) {
    query.value = "";
    autocomplete.sections = [];
  }
  document.getElementById("search")?.blur();
}

function onKeyDown(e: KeyboardEvent): void {
  let index;
  switch (e.key) {
    case "Escape":
      document.getElementById("search")?.blur();
      break;

    case "ArrowDown":
      index = visibleElements.value.indexOf(autocomplete.highlighted || "");
      if (index === -1 && visibleElements.value.length > 0) {
        autocomplete.highlighted = visibleElements.value[0];
      } else if (index >= 0 && index < visibleElements.value.length - 1) {
        autocomplete.highlighted = visibleElements.value[index + 1];
      }
      e.preventDefault();
      break;

    case "ArrowUp":
      index = visibleElements.value.indexOf(autocomplete.highlighted || "");
      if (index === 0) {
        autocomplete.highlighted = null;
      } else if (index > 0) {
        autocomplete.highlighted = visibleElements.value[index - 1];
      }
      e.preventDefault();
      break;

    case "Enter":
      if (autocomplete.highlighted !== null) searchGoTo(autocomplete.highlighted, true);
      else searchGo(false);
      break;
    default:
      break;
  }
}

function onInput() {
  autocomplete.highlighted = null;

  if (query.value.length === 0) {
    autocomplete.sections = [];
  } else {
    const queryId = queryCounter.value;
    queryCounter.value += 1;
    useFetch<SearchResponse>(`/api/search?q=${encodeURIComponent(query.value)}`, (d) => {
      // Data will be cached anyway in case the user hits backspace,
      // but we need to discard the data here if it arrived out of order.
      if (queryId > latestUsedQueryId.value) {
        latestUsedQueryId.value = queryId;
        autocomplete.sections = extractFacets(d, t("sections.rooms"), t("sections.buildings"));
      }
    });
  }
}

onMounted(() => {
  window.addEventListener("keydown", (e) => {
    if (
      (e.key === "s" || e.key === "/") &&
      document.activeElement?.tagName !== "INPUT" &&
      document.activeElement?.tagName !== "TEXTAREA"
    ) {
      e.preventDefault();
      document.getElementById("search")?.focus();
    }
  });
});
</script>

<template>
  <div class="flex flex-row">
    <div
      class="flex flex-grow flex-row rounded-s-sm border px-2.5 focus-within:outline focus-within:outline-2 focus-within:outline-offset-1 focus-within:outline-tumBlue-600 dark:border-gray-200"
    >
      <MagnifyingGlassIcon class="my-auto h-4 w-4" />
      <input
        id="search"
        v-model="query"
        type="text"
        class="flex-grow bg-transparent px-3 py-2.5 focus:outline-0"
        :placeholder="t('input.placeholder')"
        autocomplete="off"
        :aria-label="t('input.aria-searchlabel')"
        @input="onInput"
        @focus="searchFocus"
        @blur="searchBlur"
        @keydown="onKeyDown"
      />
    </div>
    <button
      type="button"
      class="rounded-e-sm bg-tumBlue-500 px-3 py-1 text-xs font-semibold text-white shadow-sm hover:bg-tumBlue-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-tumBlue-600"
      :aria-label="t('input.aria-actionlabel')"
      @click="searchGo(false)"
    >
      {{ t("input.action") }}
    </button>
  </div>
  <!-- Autocomplete -->
  <ul
    v-cloak
    class="absolute mt-16 top-0 list-none bg-white p-3.5 shadow-2xl shadow-zinc-700/30 dark:bg-zinc-900 dark:shadow-black/60"
    :class="{
      hidden: !global.search_focused || autocomplete.sections.length === 0,
    }"
  >
    <!--
    <li class="search-comment filter">
      Suche einschränken auf:
      <a class="bt btn-link btn-sm">Räume</a>
    </li> -->

    <template v-for="s in autocomplete.sections" :key="s.facet">
      <li class="divider" :data-content="s.name" />
      <template v-for="(e, i) in s.entries" :key="e.id">
        <li v-if="s.facet === 'rooms' || i < s.n_visible || s.expanded" class="menu-item">
          <a
            :class="{
              active: e.id === autocomplete.highlighted,
            }"
            :href="'/view/' + e.id"
            @click.exact.prevent="searchGoTo(e.id, true)"
            @mousedown="keep_focus = true"
            @mouseover="autocomplete.highlighted = null"
          >
            <div class="tile">
              <div class="tile-icon">
                <template v-if="e.type === 'room' || e.type === 'virtual_room'">
                  <MagnifyingGlassIcon v-if="e.parsed_id" class="h-4 w-4" />
                  <MapPinIcon v-else class="h-4 w-4" />
                </template>
                <img v-else src="@/assets/thumb-building.webp" class="avatar avatar-sm" />
              </div>
              <div class="tile-content">
                <span class="tile-title">
                  <span v-if="e.parsed_id" v-html="e.parsed_id" />
                  <i v-if="e.parsed_id" class="icon icon-caret" />
                  <span :style="{ opacity: e.parsed_id ? 0.5 : 1 }" v-html="e.name" />
                </span>
                <small class="text-gray tile-subtitle">
                  {{ e.subtext }}
                  <template v-if="e.subtext_bold">, <b v-html="e.subtext_bold"></b></template>
                </small>
              </div>
            </div>
          </a>
          <!-- <div class="menu-badge">
              <label class="label label-primary">2</label>
            </div> -->
        </li>
      </template>
      <li class="nb_results search-comment">
        <a
          v-if="s.facet === 'sites_buildings' && !s.expanded && s.n_visible < s.entries.length"
          class="cursor-pointer"
          @mousedown="keep_focus = true"
          @click="s.expanded = true"
        >
          +{{ s.entries.length - s.n_visible }} {{ t("hidden") }},
        </a>
        {{ s.estimatedTotalHits > 20 ? t("approx") : "" }}{{ t("results", s.estimatedTotalHits) }}
      </li>
    </template>

    <!--
      <li class="search-comment actions">
        <Button size="sm"><ChevronRightIcon class="h-4 w-4" /> in Gebäude Suchen</Button>
        <Button size="sm"><MapPinIcon class="h-4 w-4" /> Hörsäle</Button>
        <Button size="sm"><MapPinIcon class="h-4 w-4" /> Seminarräume</Button>
      </li>

      <li class="divider" data-content="Veranstaltungen" />
      <li class="menu-item">
        <RouterLink to="/event/">
          <div class="tile">
            <div class="tile-icon">
              <ClockIcon class="h-4 w-4" />
            </div>
            <div class="tile-content">
              <span class="tile-title">
                Advanced Practical Course Games Engineering: Building Information Modeling (IN7106)
              </span>
              <small class="tile-subtitle text-gray"> Übung mit 4 Gruppen </small>
            </div>
          </div>
        </RouterLink>
        <div class="menu-badge" style="display: none">
          <label class="label label-primary">frei</label>
        </div>
      </li> -->
  </ul>
</template>

<style lang="scss">
@import "@/assets/variables";
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

.search-comment {
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
</style>

<i18n lang="yaml">
de:
  input:
    placeholder: Suche
    aria-actionlabel: Suche nach dem im Suchfeld eingetragenen Raum
    aria-searchlabel: Suchfeld
    action: Go
  approx: ca.
  hidden: ausgeblendet
  sections:
    buildings: Gebäude / Standorte
    rooms: Räume
  results: 1 Ergebnis | {count} Ergebnisse
en:
  input:
    placeholder: Search
    aria-actionlabel: Search for the room-query entered in the search field
    aria-searchlabel: Search-field
    action: Go
  approx: approx.
  hidden: hidden
  sections:
    buildings: Buildings / Sites
    rooms: Rooms
  results: 1 result | {count} results
</i18n>
