<script setup lang="ts">
import { extractFacets } from "@/modules/autocomplete";
import { useRouter } from "vue-router";
import { useGlobalStore } from "@/stores/global";
import { useI18n } from "vue-i18n";
import { useFetch } from "@/composables/fetch";
import { computed, onMounted, reactive, ref } from "vue";
import type { SectionFacet } from "@/modules/autocomplete";
import type { components } from "@/api_types";

type SearchResponse = components["schemas"]["SearchResponse"];

import { MapPinIcon, MagnifyingGlassIcon, BuildingOfficeIcon, BuildingOffice2Icon } from "@heroicons/vue/24/outline";
import { ChevronDownIcon } from "@heroicons/vue/16/solid";
import Btn from "@/components/Btn.vue";
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
  <div
    v-if="global.search_focused && autocomplete.sections.length !== 0"
    class="absolute top-3 mt-16 max-h-[calc(100vh-75px)] max-w-xl bg-white p-3.5 shadow-2xl shadow-zinc-700/30 dark:bg-zinc-900 dark:shadow-black/60"
  >
    <ul v-for="s in autocomplete.sections" v-cloak :key="s.facet" class="mb-4 flex flex-col gap-2">
      <div class="flex items-center">
        <span class="text-md me-4 flex-shrink text-zinc-500">{{ s.name }}</span>
        <div class="flex-grow border-t border-zinc-500"></div>
      </div>
      <!--
    <li class="search-comment filter">
      Suche einschränken auf:
      <a class="bt btn-link btn-sm">Räume</a>
    </li> -->

      <template v-for="(e, i) in s.entries" :key="e.id">
        <li v-if="s.facet === 'rooms' || i < s.n_visible || s.expanded" class="rounded-sm border hover:bg-tumBlue-50">
          <RouterLink
            :class="{
              'bg-tumBlue-200': e.id === autocomplete.highlighted,
            }"
            :to="'/view/' + e.id"
            class="flex gap-3 px-4 py-3"
            @click.exact.prevent="searchGoTo(e.id, false)"
            @mousedown="keep_focus = true"
            @mouseover="autocomplete.highlighted = null"
          >
            <div class="my-auto">
              <div v-if="e.type === 'room' || e.type === 'virtual_room'" class="p-2">
                <MagnifyingGlassIcon v-if="e.parsed_id" class="h-6 w-6" />
                <MapPinIcon v-else class="h-6 w-6" />
              </div>
              <div v-else class="rounded-full bg-tumBlue-500 p-2 text-white">
                <BuildingOfficeIcon v-if="e.type === 'building'" class="h-6 w-6" />
                <BuildingOffice2Icon v-else class="h-6 w-6" />
              </div>
            </div>
            <div class="flex flex-col gap-0.5">
              <div class="flex flex-row">
                <span v-if="e.parsed_id" v-html="e.parsed_id" />
                <ChevronDownIcon v-if="e.parsed_id" class="h-4 w-4" />
                <span v-html="e.name" />
              </div>
              <small class="text-zinc-500">
                {{ e.subtext }}<template v-if="e.subtext_bold">, <b v-html="e.subtext_bold"></b></template>
              </small>
            </div>
          </RouterLink>
          <!-- <div class="menu-badge">
              <label class="label label-primary">2</label>
            </div> -->
        </li>
      </template>
      <li class="-mt-2">
        <Btn
          v-if="s.facet === 'sites_buildings' && !s.expanded && s.n_visible < s.entries.length"
          variant="link"
          size="sm"
          @mousedown="keep_focus = true"
          @click="s.expanded = true"
        >
          {{ t("show_hidden", s.entries.length - s.n_visible) }}
        </Btn>
        <span class="text-sm text-zinc-400">
          {{
            s.estimatedTotalHits > 20 ? t("approx_results", s.estimatedTotalHits) : t("results", s.estimatedTotalHits)
          }}
        </span>
      </li>

      <!--
      <li class="search-comment actions">
        <Btn size="sm"><ChevronRightIcon class="h-4 w-4" /> in Gebäude Suchen</Btn>
        <Btn size="sm"><MapPinIcon class="h-4 w-4" /> Hörsäle</Btn>
        <Btn size="sm"><MapPinIcon class="h-4 w-4" /> Seminarräume</Btn>
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
  </div>
</template>

<i18n lang="yaml">
de:
  input:
    placeholder: Suche
    aria-actionlabel: Suche nach dem im Suchfeld eingetragenen Raum
    aria-searchlabel: Suchfeld
    action: Go
  show_hidden: +{count} ausgeblendet
  sections:
    buildings: Gebäude / Standorte
    rooms: Räume
  results: 1 Ergebnis | {count} Ergebnisse
  approx_results: ca. {count} Ergebnisse
en:
  input:
    placeholder: Search
    aria-actionlabel: Search for the room-query entered in the search field
    aria-searchlabel: Search-field
    action: Go
  show_hidden: +{count} hidden
  sections:
    buildings: Buildings / Sites
    rooms: Rooms
  results: 1 result | {count} results
  approx_results: approx. {count} results
</i18n>
