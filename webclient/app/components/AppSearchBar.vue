<script setup lang="ts">
import { mdiMagnify, mdiMagnifyClose } from "@mdi/js";
import type { components } from "~/api_types";
import SearchResultItemLink from "~/components/SearchResultItemLink.vue";
import { useStagedSearchFilters } from "~/composables/searchFilters";
import { entityPath, isRoutableEntityType } from "~/utils/entityPath";
import {
  buildVisibleSearchEntries,
  collapsedHighlightTarget,
  collapsedUpwardHighlightTarget,
  LECTURE_EVENT_NAV_CAP,
  type VisibleSearchEntry,
} from "~/utils/lectureRow";

type SearchResponse = components["schemas"]["SearchResponse"];
type ResultEntry = components["schemas"]["ResultEntry"];
type UpcomingEvent = components["schemas"]["UpcomingEvent"];

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
// Sticky for the dropdown session so ArrowUp back into a lecture doesn't
// recollapse it and shift every downstream highlight index.
const expandedLectures = ref<Set<string>>(new Set());
const lectureShowAll = ref<Set<string>>(new Set());

const visibleElements = computed<VisibleSearchEntry[]>(() => {
  if (!data.value) return [];
  return buildVisibleSearchEntries(data.value.sections, {
    expandedFacets: expandedFacets.value,
    expandedLectures: expandedLectures.value,
    lectureShowAll: lectureShowAll.value,
  });
});

function resultHighlighted(entry: ResultEntry): boolean {
  if (highlighted.value === undefined) return false;
  const current = visibleElements.value[highlighted.value];
  return current?.kind === "result" && current.entry.id === entry.id;
}

function lectureHighlightedEventIndex(entry: ResultEntry): number | null {
  if (highlighted.value === undefined) return null;
  const current = visibleElements.value[highlighted.value];
  if (current?.kind === "event" && current.lectureId === entry.id) {
    return current.eventIndex;
  }
  return null;
}

function lectureShowMoreHighlighted(entry: ResultEntry): boolean {
  if (highlighted.value === undefined) return false;
  const current = visibleElements.value[highlighted.value];
  return current?.kind === "show_more_events" && current.lectureId === entry.id;
}

function lectureVisibleEventCount(entry: ResultEntry): number | null {
  if (entry.type !== "lecture") return null;
  // Uncontrolled mode (mouse click on the header): let the row render its
  // full list rather than the keyboard-nav cap.
  if (!expandedLectures.value.has(entry.id)) return null;
  const total = entry.upcoming?.length ?? 0;
  if (lectureShowAll.value.has(entry.id)) return total;
  return Math.min(LECTURE_EVENT_NAV_CAP, total);
}

function lectureShowMoreVisible(entry: ResultEntry): boolean {
  if (entry.type !== "lecture") return false;
  if (!expandedLectures.value.has(entry.id)) return false;
  if (lectureShowAll.value.has(entry.id)) return false;
  const total = entry.upcoming?.length ?? 0;
  return total > LECTURE_EVENT_NAV_CAP;
}

function expandHighlightedLecture(): void {
  if (highlighted.value === undefined) return;
  const current = visibleElements.value[highlighted.value];
  if (current?.kind !== "result") return;
  if (current.entry.type !== "lecture") return;
  if (expandedLectures.value.has(current.entry.id)) return;
  expandedLectures.value = new Set([...expandedLectures.value, current.entry.id]);
}

function revealMoreEvents(lectureId: string): void {
  if (lectureShowAll.value.has(lectureId)) return;
  lectureShowAll.value = new Set([...lectureShowAll.value, lectureId]);
}

function collapseLecturePastShowMore(lectureId: string): void {
  if (highlighted.value === undefined) return;
  const target = collapsedHighlightTarget(visibleElements.value, highlighted.value, lectureId);
  if (expandedLectures.value.has(lectureId)) {
    const next = new Set(expandedLectures.value);
    next.delete(lectureId);
    expandedLectures.value = next;
  }
  highlighted.value = target;
}

function collapseLectureOverTheTop(lectureId: string): void {
  if (highlighted.value === undefined) return;
  const oldIdx = highlighted.value;
  if (expandedLectures.value.has(lectureId)) {
    const next = new Set(expandedLectures.value);
    next.delete(lectureId);
    expandedLectures.value = next;
  }
  // Recompute against the post-collapse list so a wrap lands on the new tail.
  highlighted.value = collapsedUpwardHighlightTarget(oldIdx, visibleElements.value.length);
}

watch(query, () => {
  expandedLectures.value = new Set();
  lectureShowAll.value = new Set();
  highlighted.value = undefined;
});
watch(searchBarFocused, (focused) => {
  if (focused) return;
  expandedLectures.value = new Set();
  lectureShowAll.value = new Set();
});

const hasNoResults = computed(
  () => data.value?.sections.every((s) => s.estimatedTotalHits === 0) ?? false
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

async function searchGoTo(entry: ResultEntry): Promise<void> {
  // Lectures have no entity page; jump to the next occurrence's room instead.
  if (entry.type === "lecture") {
    const room = entry.upcoming?.[0]?.room_code;
    if (!room) {
      await searchGo(false);
      return;
    }
    await navigateTo(localePath(entityPath(room, "room")));
    searchBarFocused.value = false;
    query.value = "";
    document.getElementById("search")?.blur();
    return;
  }
  if (!isRoutableEntityType(entry.type)) return;
  await navigateTo(localePath(entityPath(entry.id, entry.type)));
  searchBarFocused.value = false;
  query.value = "";
  document.getElementById("search")?.blur();
}

async function goToEvent(event: UpcomingEvent): Promise<void> {
  await navigateTo(localePath(entityPath(event.room_code, "room")));
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

    case "ArrowDown": {
      e.preventDefault();
      if (visibleElements.value.length === 0) break;
      const current =
        highlighted.value === undefined ? undefined : visibleElements.value[highlighted.value];
      if (current?.kind === "show_more_events") {
        const collapsedId = current.lectureId;
        collapseLecturePastShowMore(collapsedId);
        // Don't re-expand if wrap landed back on the same lecture's header.
        const newCurrent =
          highlighted.value === undefined ? undefined : visibleElements.value[highlighted.value];
        const sameLecture =
          newCurrent?.kind === "result" &&
          newCurrent.entry.type === "lecture" &&
          newCurrent.entry.id === collapsedId;
        if (!sameLecture) expandHighlightedLecture();
        break;
      }
      highlighted.value =
        highlighted.value === undefined
          ? 0
          : (highlighted.value + 1) % visibleElements.value.length;
      expandHighlightedLecture();
      break;
    }

    case "ArrowUp": {
      e.preventDefault();
      if (visibleElements.value.length === 0) {
        highlighted.value = undefined;
        break;
      }
      const current =
        highlighted.value === undefined ? undefined : visibleElements.value[highlighted.value];
      if (
        current?.kind === "result" &&
        current.entry.type === "lecture" &&
        expandedLectures.value.has(current.entry.id)
      ) {
        const collapsedId = current.entry.id;
        collapseLectureOverTheTop(collapsedId);
        const newCurrent =
          highlighted.value === undefined ? undefined : visibleElements.value[highlighted.value];
        const sameLecture =
          newCurrent?.kind === "result" &&
          newCurrent.entry.type === "lecture" &&
          newCurrent.entry.id === collapsedId;
        if (!sameLecture) expandHighlightedLecture();
        break;
      }
      if (highlighted.value === 0 || highlighted.value === undefined) {
        highlighted.value = visibleElements.value.length - 1;
      } else {
        highlighted.value -= 1;
      }
      expandHighlightedLecture();
      break;
    }

    case "Enter":
      e.preventDefault();
      if (highlighted.value === undefined) {
        searchGo(false);
        break;
      }
      {
        const entry = visibleElements.value[highlighted.value];
        if (entry === undefined) {
          searchGo(true);
        } else if (entry.kind === "result") {
          searchGoTo(entry.entry);
        } else if (entry.kind === "event") {
          goToEvent(entry.event);
        } else {
          revealMoreEvents(entry.lectureId);
        }
      }
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
      class="bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 flex flex-grow flex-row rounded-s-sm border focus-within:outline focus-within:outline-2 focus-within:outline-offset-1 focus-within:outline-blue-600 dark:focus-within:outline-blue-300"
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
        class="text-zinc-800 dark:text-zinc-100 flex-grow resize-none bg-transparent py-2.5 pe-5 ps-3 font-semibold placeholder:text-zinc-800 dark:placeholder:text-zinc-100 focus-within:placeholder:text-zinc-500 dark:focus-within:placeholder:text-zinc-400 placeholder:font-normal focus:outline-0"
        :placeholder="t('input.placeholder')"
        :aria-label="t('input.aria-searchlabel')"
        @focus="searchFocus"
        @blur="searchBlur"
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
              <span class="text-md text-zinc-800 dark:text-zinc-100 me-4 flex-shrink">{{ t(`sections.${s.facet}`) }}</span>
              <div class="border-zinc-800 dark:border-zinc-100 flex-grow border-t" />
            </div>

          <template v-for="(e, i) in s.entries" :key="e.id">
            <SearchResultItemLink
              v-if="expandedFacets.has(s.facet) || i < s.n_visible"
              :highlighted="resultHighlighted(e)"
              :item="e"
              :lecture-expanded="e.type === 'lecture' ? expandedLectures.has(e.id) : null"
              :lecture-visible-event-count="lectureVisibleEventCount(e)"
              :lecture-highlighted-event-index="lectureHighlightedEventIndex(e)"
              :lecture-show-more-visible="lectureShowMoreVisible(e)"
              :lecture-show-more-highlighted="lectureShowMoreHighlighted(e)"
              @click="searchBarFocused = false"
              @mousedown="keep_focus = true"
              @mouseover="highlighted = undefined"
              @show-more="revealMoreEvents(e.id)"
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
