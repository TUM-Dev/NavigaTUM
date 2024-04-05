<script setup lang="ts">
import { extractFacets } from "~/composables/autocomplete";
import { BuildingOffice2Icon, BuildingOfficeIcon, MagnifyingGlassIcon, MapPinIcon } from "@heroicons/vue/24/outline";
import { ChevronDownIcon } from "@heroicons/vue/16/solid";
import type { components } from "~/api_types";

type SearchResponse = components["schemas"]["SearchResponse"];

const searchBarFocused = defineModel<boolean>("searchBarFocused", { required: true });
const { t, locale } = useI18n({ useScope: "local" });
const keep_focus = ref(false);
const query = ref("");
const highlighted = ref<string | null>(null);
const router = useRouter();

const visibleElements = computed<string[]>(() => {
  const visible: string[] = [];

  sections.value.forEach((section) => {
    section.entries.forEach((entry, index: number) => {
      if (section.facet !== "sites_buildings" || section.n_visible > index || section.expanded) visible.push(entry.id);
    });
  });
  return visible;
});

function searchFocus(): void {
  searchBarFocused.value = true;
  highlighted.value = null;
}

function searchBlur(): void {
  if (keep_focus.value) {
    setTimeout(() => {
      // This is relevant if the call is delayed and focused has
      // already been disabled e.g. when clicking on an entry.
      if (searchBarFocused.value) document.getElementById("search")?.focus();
    }, 0);
    keep_focus.value = false;
  } else {
    searchBarFocused.value = false;
  }
}

function searchGo(cleanQuery: boolean): void {
  if (query.value.length === 0) return;

  router.push(`/search?q=${query.value}`);
  searchBarFocused.value = false;
  if (cleanQuery) {
    query.value = "";
  }
  document.getElementById("search")?.blur();
}

function searchGoTo(id: string, cleanQuery: boolean): void {
  // Catch is necessary because vue-router throws an error
  // if navigation is aborted for some reason (e.g. the new
  // url is the same or there is a loop in redirects)
  router.push(`/view/${id}`);
  searchBarFocused.value = false;
  if (cleanQuery) {
    query.value = "";
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
      index = visibleElements.value.indexOf(highlighted.value || "");
      if (index === -1 && visibleElements.value.length > 0) {
        highlighted.value = visibleElements.value[0];
      } else if (index >= 0 && index < visibleElements.value.length - 1) {
        highlighted.value = visibleElements.value[index + 1];
      }
      e.preventDefault();
      break;

    case "ArrowUp":
      index = visibleElements.value.indexOf(highlighted.value || "");
      if (index === 0) {
        highlighted.value = null;
      } else if (index > 0) {
        highlighted.value = visibleElements.value[index - 1];
      }
      e.preventDefault();
      break;

    case "Enter":
      if (highlighted.value !== null) searchGoTo(highlighted.value, true);
      else searchGo(false);
      break;
    default:
      break;
  }
}

const runtimeConfig = useRuntimeConfig();
const url = computed(
  () => `${runtimeConfig.public.apiURL}/api/search?q=${encodeURIComponent(query.value)}&lang=${locale.value}`,
);
const { data, error, refresh } = await useFetch<SearchResponse>(url, {});
const sections = computed(() => {
  console.log(url.value);
  if (data.value === null) return [];
  return extractFacets(data.value, t("sections.rooms"), t("sections.buildings"));
});
// a bit crude way of doing retries, but likely fine
watchEffect(() => {
  if (query.value.length && error.value !== null) setTimeout(refresh, 500);
});
</script>

<template>
  <div class="flex flex-row">
    <div
      class="bg-zinc-200 border-zinc-400 flex flex-grow flex-row rounded-s-sm border px-2.5 focus-within:outline focus-within:outline-2 focus-within:outline-offset-1 focus-within:outline-blue-600"
    >
      <MagnifyingGlassIcon class="text-zinc-800 my-auto h-4 w-4" />
      <input
        id="search"
        v-model="query"
        type="text"
        class="text-zinc-800 flex-grow bg-transparent px-3 py-2.5 font-semibold placeholder:text-zinc-800 focus-within:placeholder:text-zinc-500 placeholder:font-normal focus:outline-0"
        :placeholder="t('input.placeholder')"
        autocomplete="off"
        :aria-label="t('input.aria-searchlabel')"
        @focus="searchFocus"
        @blur="searchBlur"
        @keydown="onKeyDown"
      />
    </div>
    <button
      type="button"
      class="text-white bg-blue-500 rounded-e-sm px-3 py-1 text-xs font-semibold shadow-sm hover:bg-blue-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-blue-600"
      :aria-label="t('input.aria-actionlabel')"
      @click="searchGo(false)"
    >
      {{ t("input.action") }}
    </button>
  </div>
  <!-- Autocomplete -->
  <div
    v-if="searchBarFocused && query.length !== 0 && sections.length !== 0"
    class="shadow-4xl bg-zinc-50 border-zinc-200 absolute top-3 -ms-2 me-3 mt-16 flex max-h-[calc(100vh-75px)] max-w-xl flex-col gap-4 overflow-auto rounded border p-3.5 shadow-zinc-700/30"
  >
    <ul v-for="s in sections" v-cloak :key="s.facet" class="flex flex-col gap-2">
      <div class="flex items-center">
        <span class="text-md text-zinc-800 me-4 flex-shrink">{{ s.name }}</span>
        <div class="border-zinc-800 flex-grow border-t"></div>
      </div>
      <!--
    <li class="search-comment filter">
      Suche einschränken auf:
      <a class="bt btn-link btn-sm">Räume</a>
    </li> -->

      <template v-for="(e, i) in s.entries" :key="e.id">
        <li
          v-if="s.facet === 'rooms' || i < s.n_visible || s.expanded"
          class="bg-zinc-50 border-zinc-200 rounded-sm border hover:bg-blue-100"
        >
          <NuxtLink
            :class="{
              'bg-blue-200': e.id === highlighted,
            }"
            :to="'/view/' + e.id"
            class="flex gap-3 px-4 py-3"
            @click.exact.prevent="searchGoTo(e.id, false)"
            @mousedown="keep_focus = true"
            @mouseover="highlighted = null"
          >
            <div class="my-auto min-w-11">
              <div v-if="e.type === 'room' || e.type === 'virtual_room'" class="text-zinc-900 p-2">
                <MagnifyingGlassIcon v-if="e.parsed_id" class="h-6 w-6" />
                <MapPinIcon v-else class="h-6 w-6" />
              </div>
              <div v-else class="text-white bg-blue-500 rounded-full p-2">
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
                {{ e.subtext }}
                <template v-if="e.subtext_bold">, <b v-html="e.subtext_bold"></b></template>
              </small>
            </div>
          </NuxtLink>
          <!-- <div class="menu-badge">
              <label class="label label-primary">2</label>
            </div> -->
        </li>
      </template>
      <li class="-mt-2">
        <Btn
          v-if="s.facet === 'sites_buildings' && !s.expanded && s.n_visible < s.entries.length"
          variant="linkButton"
          size="sm"
          @mousedown="keep_focus = true"
          @click="s.expanded = true"
        >
          {{ t("show_hidden", s.entries.length - s.n_visible) }}
        </Btn>
        <span class="text-zinc-400 text-sm">
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
        <NuxtLink to="/event/">
          <div class="tile">
            <div class="tile-icon">
              <ClockIcon class="h-4 w-4" />
            </div>
            <div class="tile-content">
              <span class="tile-title">
                Advanced Practical Course Games Engineering: Building Information Modeling (IN7106)
              </span>
              <small class="tile-subtitle text-zinc"> Übung mit 4 Gruppen </small>
            </div>
          </div>
        </NuxtLink>
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
