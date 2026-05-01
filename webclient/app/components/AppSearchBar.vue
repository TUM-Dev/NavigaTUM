<script setup lang="ts">
import { mdiMagnify, mdiMagnifyClose } from "@mdi/js";
import type { components } from "~/api_types";
import SearchResultItemLink from "~/components/SearchResultItemLink.vue";
import { useStagedSearchFilters } from "~/composables/searchFilters";

type SearchResponse = components["schemas"]["SearchResponse"];

const searchBarFocused = defineModel<boolean>("searchBarFocused", {
  required: true,
});
const { t, locale } = useI18n({ useScope: "local" });
const localePath = useLocalePath();
const route = useRoute();
const router = useRouter();
const filters = useStagedSearchFilters();
const keep_focus = ref(false);
const interacting_with_panel = ref(false);
const query = ref(Array.isArray(route.query.q) ? (route.query.q[0] ?? "") : (route.query.q ?? ""));
const highlighted = ref<number | undefined>(undefined);
// Per-facet expand state. Sites/buildings/rooms can freeze with
// `n_visible < entries.length` when a lower-priority facet appears; the
// "show hidden" button on each such section toggles its slot here.
const expandedFacets = ref<Set<string>>(new Set());

const visibleElements = computed<string[]>(() => {
  if (!data.value) return [] as string[];

  const visible: string[] = [] as string[];
  for (const section of data.value.sections) {
    const cap = expandedFacets.value.has(section.facet)
      ? Number.POSITIVE_INFINITY
      : section.n_visible;
    visible.push(...section.entries.slice(0, cap).map((e) => e.id));
  }
  return visible;
});

const hasNoResults = computed(
  () => !!data.value && data.value.sections.every((s) => s.estimatedTotalHits === 0)
);

function searchFocus(): void {
  searchBarFocused.value = true;
  highlighted.value = undefined;
}

function searchBlur(): void {
  if (interacting_with_panel.value) {
    // Mouse interaction inside the dropdown panel (filters, sort, popovers, inputs):
    // keep the dropdown open but let focus stay on whatever the user clicked.
    interacting_with_panel.value = false;
    return;
  }
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

async function searchGo(cleanQuery: boolean): Promise<void> {
  if (query.value.length === 0) return;

  const target = router.resolve({
    path: localePath("/search"),
    query: { q: query.value, ...filters.buildQueryObject() },
  });
  await navigateTo(target.fullPath);
  searchBarFocused.value = false;
  if (cleanQuery) {
    query.value = "";
  }
  document.getElementById("search")?.blur();
}

async function searchGoTo(id: string): Promise<void> {
  await navigateTo(localePath(`/view/${id}`));
  searchBarFocused.value = false;
  query.value = "";
  document.getElementById("search")?.blur();
}

function closeSearchBar(): void {
  // Force-close even if a child (filter chip, sort popover, …) flipped the keep-focus flags;
  // ESC is the user's "I'm done" signal and should always tear the panel down.
  keep_focus.value = false;
  interacting_with_panel.value = false;
  searchBarFocused.value = false;
  document.getElementById("search")?.blur();
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
      if (highlighted.value === undefined) {
        highlighted.value = 0;
        e.preventDefault();
        break;
      }

      highlighted.value = (highlighted.value + 1) % visibleElements.value.length;
      e.preventDefault();
      break;

    case "ArrowUp":
      if (visibleElements.value.length === 0) {
        highlighted.value = undefined;
        e.preventDefault();
        break;
      }
      if (highlighted.value === 0 || highlighted.value === undefined) {
        highlighted.value = visibleElements.value.length - 1;
      } else {
        highlighted.value -= 1;
      }
      e.preventDefault();
      break;

    case "Enter":
      e.preventDefault();
      if (highlighted.value !== undefined) {
        const visible = visibleElements.value[highlighted.value];
        if (visible !== undefined) {
          searchGoTo(visible);
        } else {
          searchGo(true);
        }
      } else searchGo(false);
      break;
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
  retry: 120,
  retryDelay: 1000,
});
</script>

<template>
  <form action="/search" autocomplete="off" method="GET" role="search" class="flex flex-row" @submit="searchGo(false)">
    <div
      class="bg-zinc-200 border-zinc-400 flex flex-grow flex-row rounded-s-sm border focus-within:outline focus-within:outline-2 focus-within:outline-offset-1 focus-within:outline-blue-600"
    >
      <textarea
        id="search"
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
        class="text-zinc-800 flex-grow resize-none bg-transparent py-2.5 pe-5 ps-3 font-semibold placeholder:text-zinc-800 focus-within:placeholder:text-zinc-500 placeholder:font-normal focus:outline-0"
        :placeholder="t('input.placeholder')"
        :aria-label="t('input.aria-searchlabel')"
        @focus="searchFocus"
        @blur="searchBlur"
        @keydown="onKeyDown"
      />
    </div>
    <button
      type="submit"
      class="bg-blue-500 rounded-e-sm px-3 py-1 text-xs font-semibold shadow-sm hover:bg-blue-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-blue-600"
      :aria-label="t('input.aria-actionlabel')"
      :title="t('input.action')"
    >
      <MdiIcon :path="mdiMagnify" :size="28" class="text-zinc-100 my-auto" />
    </button>
  </form>
  <!-- Autocomplete -->
  <ClientOnly>
    <div
      v-if="searchBarFocused && query.length !== 0"
      class="shadow-4xl bg-zinc-50 border-zinc-200 absolute inset-x-0 top-3 mt-16 flex max-h-[calc(100vh-80px)] flex-col gap-4 overflow-auto border p-3.5 shadow-zinc-700/30 md:inset-x-auto md:-ms-2 md:me-3 md:max-w-xl md:rounded-sm"
    >
      <div
        class="flex flex-wrap items-center gap-2"
        @mousedown.capture="interacting_with_panel = true"
      >
        <SearchFilterChips :filters="filters" />
        <div class="ms-auto">
          <SearchSortControl :filters="filters" />
        </div>
      </div>
      <Toast v-if="error" id="search-error" level="error">
        <p class="text-md font-bold">{{ t("error.header") }}</p>
        <p class="text-sm">
          {{ t("error.reason") }}:<br />
          <code
            class="text-red-900 bg-red-200 mb-1 mt-2 inline-flex max-w-full items-center space-x-2 overflow-auto rounded-md px-4 py-3 text-left font-mono text-xs dark:bg-red-50/20"
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
          <MdiIcon :path="mdiMagnifyClose" :size="32" class="text-zinc-400" />
          <p class="text-zinc-800 text-sm font-semibold">{{ t("no_results.title") }}</p>
          <p class="text-zinc-500 text-xs">
            {{ filters.hasActiveFilters.value ? t("no_results.hint_filtered") : t("no_results.hint") }}
          </p>
          <Btn
            v-if="filters.hasActiveFilters.value"
            variant="linkButton"
            size="sm"
            @mousedown="keep_focus = true"
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
        >
          <template v-if="s.estimatedTotalHits > 0">
            <div class="flex items-center">
              <span class="text-md text-zinc-800 me-4 flex-shrink">{{ t(`sections.${s.facet}`) }}</span>
              <div class="border-zinc-800 flex-grow border-t" />
            </div>

          <template v-for="(e, i) in s.entries" :key="e.id">
            <SearchResultItemLink
              v-if="expandedFacets.has(s.facet) || i < s.n_visible"
              :highlighted="e.id === visibleElements[highlighted ?? -1]"
              :item="e"
              @click="searchBarFocused = false"
              @mousedown="keep_focus = true"
              @mouseover="highlighted = undefined"
            />
          </template>
          <li class="-mt-2">
            <Btn
              v-if="!expandedFacets.has(s.facet) && s.n_visible < s.entries.length"
              variant="linkButton"
              size="sm"
              @mousedown="keep_focus = true"
              @click="expandedFacets = new Set([...expandedFacets, s.facet])"
            >
              {{ t("show_hidden", s.entries.length - s.n_visible) }}
            </Btn>
            <span class="text-zinc-400 text-sm">
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
    sites: Standorte
    buildings: Gebäude
    rooms: Räume
    pois: POIs
    addresses: Adressen
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
