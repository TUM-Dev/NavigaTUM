<script setup lang="ts">
import { mdiMagnify, mdiMagnifyClose } from "@mdi/js";
import type { components } from "~/api_types";
import SearchResultRow from "~/components/SearchResultRow.vue";
import { clientOnlyRetries } from "~/composables/common";
import { categoriesForQuery, FILTER_QUERY_PARAM, type FilterId } from "~/composables/mapLayers";
import { useSearchDropdownNav } from "~/composables/searchDropdownNav";
import { useStagedSearchFilters } from "~/composables/searchFilters";
import { entityPath } from "~/utils/entityPath";
import { LectureNavKey, type SearchResultEntry, tagSectionEntries } from "~/utils/lectureRow";

type SearchResponse = components["schemas"]["SearchResponse"];
type UpcomingEvent = components["schemas"]["UpcomingEvent"];

const searchBarFocused = defineModel<boolean>("searchBarFocused", {
  required: true,
});
const { t, locale } = useI18n({ useScope: "local" });
const localePath = useLocalePath();
const route = useRoute();
const router = useRouter();
const filters = useStagedSearchFilters();
const query = ref(Array.isArray(route.query.q) ? (route.query.q[0] ?? "") : (route.query.q ?? ""));
const searchWrapper = ref<HTMLElement | null>(null);
const searchInput = useTemplateRef<HTMLTextAreaElement>("searchInput");
const { focused: wrapperFocused } = useFocusWithin(searchWrapper);

const sections = computed(() => data.value?.sections);
const shortcutCategories = computed(() => categoriesForQuery(query.value));
const nav = useSearchDropdownNav(sections, shortcutCategories);
const { expandedFacets, highlighted, highlightedEntry, lectureNav } = nav;
provide(LectureNavKey, lectureNav);

watch(wrapperFocused, (isFocused) => {
  searchBarFocused.value = isFocused;
  if (isFocused) highlighted.value = undefined;
});
watch(query, () => nav.resetAll());
watch(searchBarFocused, (focused) => {
  if (!focused) nav.clearLectureExpansion();
});

function resultHighlighted(entry: SearchResultEntry): boolean {
  const current = highlightedEntry.value;
  return current?.kind === "result" && current.entry.id === entry.id;
}

function shortcutHighlighted(category: FilterId): boolean {
  const current = highlightedEntry.value;
  return current?.kind === "category_shortcut" && current.category === category;
}

const hasNoResults = computed(
  () => data.value?.sections.every((s) => s.estimatedTotalHits === 0) ?? false
);

async function searchGo(cleanQuery: boolean): Promise<void> {
  if (query.value.length === 0) return;

  const target = router.resolve({
    path: localePath("/search"),
    query: { q: query.value, ...filters.buildQueryObject() },
  });
  await navigateTo(target.fullPath);
  if (cleanQuery) {
    query.value = "";
  }
  searchInput.value?.blur();
}

async function searchGoTo(entry: SearchResultEntry): Promise<void> {
  // Lectures have no entity page; jump to the next occurrence's room instead.
  if (entry.kind === "lecture") {
    const room = entry.upcoming[0]?.room_code;
    if (!room) {
      await searchGo(false);
      return;
    }
    await navigateTo(localePath(entityPath(room, "room")));
    query.value = "";
    searchInput.value?.blur();
    return;
  }
  // Addresses have no entity page to jump to.
  if (entry.kind === "address") return;
  await navigateTo(localePath(entityPath(entry.id, entry.type)));
  query.value = "";
  searchInput.value?.blur();
}

async function goToEvent(event: UpcomingEvent): Promise<void> {
  await navigateTo(localePath(entityPath(event.room_code, "room")));
  query.value = "";
  searchInput.value?.blur();
}

async function goToCategory(category: FilterId): Promise<void> {
  await navigateTo({ path: localePath("/map"), query: { [FILTER_QUERY_PARAM]: category } });
  query.value = "";
  searchInput.value?.blur();
}

// The shortcut's own link handles mouse navigation; only reset the bar here.
function onShortcutFollowed(): void {
  query.value = "";
  searchInput.value?.blur();
}

function closeSearchBar(): void {
  // Blur whichever descendant holds focus so useFocusWithin flips shut.
  const active = document.activeElement;
  if (active instanceof HTMLElement && searchWrapper.value?.contains(active)) {
    active.blur();
  }
}

onKeyStroke("Escape", () => {
  if (searchBarFocused.value) closeSearchBar();
});

function onKeyDown(e: KeyboardEvent): void {
  switch (e.key) {
    case "Escape":
      closeSearchBar();
      break;
    case "ArrowDown":
      e.preventDefault();
      nav.arrowDown();
      break;
    case "ArrowUp":
      e.preventDefault();
      nav.arrowUp();
      break;
    case "Enter": {
      e.preventDefault();
      const entry = highlightedEntry.value;
      if (!entry) {
        searchGo(false);
      } else if (entry.kind === "category_shortcut") {
        goToCategory(entry.category);
      } else if (entry.kind === "result") {
        searchGoTo(entry.entry);
      } else if (entry.kind === "event") {
        goToEvent(entry.event);
      } else {
        nav.revealMoreEvents(entry.lectureId);
      }
      break;
    }
    default:
      break;
  }
}

const runtimeConfig = useRuntimeConfig();
const url = computed(() => {
  const params = new URLSearchParams();
  params.append("q", query.value);
  params.append("lang", locale.value);
  params.append("pre_highlight", "<b class='text-blue'>");
  params.append("post_highlight", "</b>");
  filters.appendToParams(params);

  return `${runtimeConfig.public.apiURL}/api/search?${params.toString()}`;
});
const { data, error } = useFetch<SearchResponse>(url, {
  lazy: true,
  dedupe: "cancel",
  credentials: "omit",
  retry: clientOnlyRetries(120),
  retryDelay: 1000,
});
</script>

<template>
  <div ref="searchWrapper" class="contents">
    <form action="/search" autocomplete="off" method="GET" role="search" class="flex flex-row" @submit="searchGo(false)">
      <div
        class="bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 flex flex-grow flex-row rounded-s-sm border focus-within:outline focus-within:outline-2 focus-within:outline-offset-1 focus-within:outline-blue-600 dark:focus-within:outline-blue-300"
      >
        <textarea
          id="search"
          ref="searchInput"
          v-model="query"
          cols="1"
          rows="1"
          :title="t('input.aria-searchlabel')"
          aria-autocomplete="both"
          aria-haspopup="false"
          autocapitalize="off"
          autocomplete="off"
          spellcheck="false"
          maxlength="2048"
          name="q"
          type="text"
          class="text-zinc-800 dark:text-zinc-100 flex-grow resize-none bg-transparent py-2.5 pe-5 ps-3 font-semibold placeholder:text-zinc-800 dark:placeholder:text-zinc-100 focus-within:placeholder:text-zinc-500 dark:focus-within:placeholder:text-zinc-400 placeholder:font-normal focus:outline-0"
          :placeholder="t('input.placeholder')"
          :aria-label="t('input.aria-searchlabel')"
          @keydown="onKeyDown"
        />
      </div>
      <button
        type="submit"
        class="bg-blue-500 dark:bg-blue-400 rounded-e-sm px-3 py-1 text-xs font-semibold shadow-sm hover:bg-blue-600 dark:hover:bg-blue-300 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-blue-600 dark:focus-visible:outline-blue-300"
        :aria-label="t('input.aria-actionlabel')"
        :title="t('input.action')"
      >
        <MdiIcon :path="mdiMagnify" :size="28" class="text-zinc-100 dark:text-zinc-800 my-auto" />
      </button>
    </form>
    <!-- Autocomplete -->
    <ClientOnly>
      <div
        v-if="searchBarFocused && query.length !== 0"
        class="shadow-4xl bg-zinc-50 dark:bg-zinc-900 border-zinc-200 dark:border-zinc-700 absolute inset-x-0 top-3 mt-16 flex max-h-[calc(100vh-80px)] flex-col gap-4 overflow-auto border p-3.5 shadow-zinc-700/30 dark:shadow-zinc-200/30 md:inset-x-auto md:-ms-2 md:me-3 md:max-w-xl md:rounded-sm"
      >
        <div class="flex flex-wrap items-center gap-2">
          <SearchFilterChips :filters="filters" />
          <div class="ms-auto">
            <SearchSortControl :filters="filters" />
          </div>
        </div>
        <!-- Keep the bar focused on tap so iOS Safari doesn't swallow the click (#3324). -->
        <ul v-if="shortcutCategories.length" class="flex flex-col gap-2" @mousedown.prevent>
          <SearchCategoryShortcut
            v-for="category in shortcutCategories"
            :key="category"
            :category="category"
            :highlighted="shortcutHighlighted(category)"
            @click="onShortcutFollowed"
            @mouseover="highlighted = undefined"
          />
        </ul>
        <Toast v-if="error" id="search-error" level="error">
          <p class="text-md font-bold">{{ t("error.header") }}</p>
          <p class="text-sm">
            {{ t("error.reason") }}:<br />
            <code
              class="text-red-900 dark:text-red-50 bg-red-200 mb-1 mt-2 inline-flex max-w-full items-center space-x-2 overflow-auto rounded-md px-4 py-3 text-left font-mono text-xs dark:bg-red-900/20"
            >
              {{ error }}
            </code>
          </p>
          <p class="text-sm">{{ t("error.call_to_action") }}</p>
        </Toast>
        <template v-if="data">
          <div
            v-if="hasNoResults"
            role="status"
            class="flex flex-col items-center gap-1 px-2 py-6 text-center"
          >
            <MdiIcon :path="mdiMagnifyClose" :size="32" class="text-zinc-400 dark:text-zinc-500" />
            <p class="text-zinc-800 dark:text-zinc-100 text-sm font-semibold">{{ t("no_results.title") }}</p>
            <p class="text-zinc-500 dark:text-zinc-400 text-xs">
              {{ filters.hasActiveFilters.value ? t("no_results.hint_filtered") : t("no_results.hint") }}
            </p>
            <Btn
              v-if="filters.hasActiveFilters.value"
              variant="linkButton"
              size="sm"
              @click="filters.clearAll()"
            >
              {{ t("no_results.clear_filters") }}
            </Btn>
          </div>
          <ul
            v-for="s in data.sections"
            v-else
            v-cloak
            :key="s.facet"
            class="flex flex-col gap-2"
            @mousedown.prevent
          >
            <template v-if="s.estimatedTotalHits > 0">
              <div class="flex items-center">
                <span class="text-md text-zinc-800 dark:text-zinc-100 me-4 flex-shrink">{{ t(`sections.${s.facet}`) }}</span>
                <div class="border-zinc-800 dark:border-zinc-100 flex-grow border-t" />
              </div>

              <template v-for="(e, i) in tagSectionEntries(s)" :key="e.id">
                <SearchResultRow
                  v-if="expandedFacets.has(s.facet) || i < s.n_visible"
                  :highlighted="resultHighlighted(e)"
                  :item="e"
                  @mouseover="highlighted = undefined"
                />
              </template>
              <li class="-mt-2">
                <Btn
                  v-if="!expandedFacets.has(s.facet) && s.n_visible < s.entries.length"
                  variant="linkButton"
                  size="sm"
                  @click="nav.expandFacet(s.facet)"
                >
                  {{ t("show_hidden", s.entries.length - s.n_visible) }}
                </Btn>
                <span class="text-zinc-400 dark:text-zinc-500 text-sm">
                  {{
                    s.estimatedTotalHits > 20 ? t("approx_results", s.estimatedTotalHits) : t("results", s.estimatedTotalHits)
                  }}
                </span>
              </li>
            </template>
          </ul>
        </template>
      </div>
    </ClientOnly>
  </div>
</template>

<i18n lang="yaml">
de:
  input:
    placeholder: Suche
    aria-actionlabel: Suche nach dem im Suchfeld eingetragenen Raum
    aria-searchlabel: Suchfeld
    action: Los
  show_hidden: +{count} ausgeblendet
  sections:
    sites: Standorte
    buildings: Gebäude
    rooms: Räume
    pois: POIs
    addresses: Adressen
    lectures: Vorlesungen
  results: 1 Ergebnis | {count} Ergebnisse
  approx_results: ca. {count} Ergebnisse
  no_results:
    title: Keine Ergebnisse gefunden
    hint: Versuche es mit anderen Suchbegriffen.
    hint_filtered: Keine Treffer mit den aktiven Filtern.
    clear_filters: Filter entfernen
  error:
    header: Bei der Suche ist ein Fehler aufgetreten
    reason: Der Grund für diesen Fehler ist
    call_to_action: Wenn dieses Problem weiterhin besteht, kontaktiere uns bitte über das Feedback-Formular.
en:
  input:
    placeholder: Search
    aria-actionlabel: Search for the room-query entered in the search field
    aria-searchlabel: Search-field
    action: Go
  show_hidden: +{count} hidden
  sections:
    sites: Sites
    buildings: Buildings
    rooms: Rooms
    pois: POIs
    addresses: Addresses
    lectures: Lectures
  results: 1 result | {count} results
  approx_results: approx. {count} results
  no_results:
    title: No results found
    hint: Try different keywords.
    hint_filtered: No matches with the active filters.
    clear_filters: Clear filters
  error:
    header: Something went wrong while searching
    reason: Reason for this error is
    call_to_action: If this issue persists, please contact us via the feedback form.
</i18n>
