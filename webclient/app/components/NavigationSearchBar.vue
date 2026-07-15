<script setup lang="ts">
import { mdiCrosshairsGps, mdiMagnify } from "@mdi/js";
import { useRouteQuery } from "@vueuse/router";
import type { operations } from "~/api_types";
import { clientOnlyRetries } from "~/composables/common";
import { useSharedGeolocation } from "~/composables/geolocation";
import { type ResultsSectionFacet, tagSectionEntries } from "~/utils/lectureRow";

type SearchResponse = operations["search_handler"]["responses"][200]["content"]["application/json"];

const props = defineProps<{
  queryId: string;
}>();
const { t, locale } = useI18n({ useScope: "local" });
const route = useRoute();
const router = useRouter();
const currently_actively_picking = ref(false);
const isFocused = ref(false);
const searchWrapper = ref<HTMLElement | null>(null);
const { focused: wrapperFocused } = useFocusWithin(searchWrapper);
watch(wrapperFocused, (focused) => {
  if (!focused) currently_actively_picking.value = false;
});

const geolocationState = useSharedGeolocation();

const isSearchingLocation = ref(false);

watch(
  () => geolocationState.value.userLocation,
  (location) => {
    if (location && geolocationState.value.triggeringSearchBarId === props.queryId) {
      query.value = t("gps.my_location");
      selected.value = `${location.lat},${location.lon}`;
      currently_actively_picking.value = false;
      isSearchingLocation.value = false;
      geolocationState.value.triggeringSearchBarId = null;
    }
  }
);

watch(
  () => geolocationState.value.triggeringSearchBarId,
  (triggeringId) => {
    if (triggeringId !== props.queryId && isSearchingLocation.value) {
      isSearchingLocation.value = false;
    }
  }
);

const isGeolocationSupported = computed(() => {
  return typeof navigator !== "undefined" && "geolocation" in navigator;
});

const query = useRouteQuery<string>(`q_${props.queryId}`, "", {
  mode: "replace",
  route,
  router,
});
const selected = useRouteQuery<string>(props.queryId, "", {
  mode: "replace",
  route,
  router,
});
const highlighted = ref<number>(0);
const expandedFacets = ref<Set<ResultsSectionFacet>>(new Set());

const visibleElements = computed<string[]>(() => {
  if (!data.value) return [];

  const visible: string[] = [];
  for (const section of data.value.sections) {
    const cap = expandedFacets.value.has(section.facet)
      ? Number.POSITIVE_INFINITY
      : section.n_visible;
    visible.push(...section.entries.slice(0, cap).map((e) => e.id));
  }
  return visible;
});

function select(id: string) {
  currently_actively_picking.value = false;
  selected.value = id;
  for (const section of data.value?.sections ?? []) {
    for (const entry of section.entries) {
      if (entry.id === id) {
        query.value = entry.name
          .replaceAll("<b class='text-blue'>", "")
          .replaceAll("</b>", "")
          .trim();
      }
    }
  }
}

function useCurrentLocation() {
  query.value = t("gps.searching_location");
  currently_actively_picking.value = false;
  isSearchingLocation.value = true;

  geolocationState.value.triggeringSearchBarId = props.queryId;
  geolocationState.value.shouldTriggerMapGeolocation = true;
}

function onKeyDown(e: KeyboardEvent): void {
  switch (e.key) {
    case "Escape":
      document.getElementById("search")?.blur();
      break;

    case "ArrowDown":
      if (visibleElements.value.length === 0) {
        e.preventDefault();
        break;
      }

      highlighted.value = (highlighted.value + 1) % visibleElements.value.length;
      e.preventDefault();
      break;

    case "ArrowUp":
      if (visibleElements.value.length === 0) {
        e.preventDefault();
        break;
      }

      if (highlighted.value <= 0) {
        highlighted.value = visibleElements.value.length - 1;
      } else {
        highlighted.value -= 1;
      }
      e.preventDefault();
      break;

    case "Enter":
      if (highlighted.value === undefined) {
        query.value = "";
        selected.value = "";
      } else {
        e.preventDefault();
        const visible = visibleElements.value[highlighted.value];
        if (visible) {
          select(visible);
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
  params.append("search_addresses", "true");

  return `${runtimeConfig.public.apiURL}/api/search?${params.toString()}`;
});
const { data, error } = await useFetch<SearchResponse>(url, {
  dedupe: "cancel",
  credentials: "omit",
  retry: clientOnlyRetries(120),
  retryDelay: 1000,
});
</script>

<template>
  <div ref="searchWrapper" class="relative flex flex-grow flex-col">
    <div
      class="bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 flex flex-row rounded-s-sm border focus-within:outline focus-within:outline-2 focus-within:outline-offset-1 focus-within:outline-blue-600 dark:focus-within:outline-blue-300"
    >
      <textarea
        :id="queryId"
        v-model="query"
        cols="1"
        rows="2"
        :title="t('input.aria-searchlabel')"
        aria-autocomplete="both"
        aria-haspopup="false"
        autocapitalize="off"
        autocomplete="off"
        spellcheck="false"
        maxlength="2048"
        :name="queryId"
        type="text"
        class="text-zinc-800 dark:text-zinc-100 flex-grow resize-none bg-transparent py-2.5 ps-3 pe-2 text-sm font-semibold placeholder:text-zinc-800 dark:placeholder:text-zinc-100 focus-within:placeholder:text-zinc-500 dark:focus-within:placeholder:text-zinc-400 placeholder:font-normal focus:outline-0"
        :placeholder="t('input.placeholder-' + queryId)"
        :aria-label="t('input.aria-searchlabel')"
        @focus="
          isFocused = true;
          currently_actively_picking = true;
          highlighted = 0;
        "
        @blur="isFocused = false"
        @keydown="onKeyDown"
      />
      <button
        v-if="selected && isFocused"
        type="submit"
        class="focusable text-zinc-600 dark:text-zinc-300 hover:text-blue-600 dark:hover:text-blue-300 hover:bg-blue-50 dark:hover:bg-blue-900 flex items-center justify-center px-3 py-2.5 transition-all duration-200 rounded-sm"
        :title="t('search_route')"
        :aria-label="t('search_route')"
        @mousedown.prevent
        @click="currently_actively_picking = false"
      >
        <MdiIcon :path="mdiMagnify" :size="16" />
      </button>
      <ClientOnly>
        <button
          v-if="isGeolocationSupported && !geolocationState.mapGeolocationActive"
          type="button"
          class="focusable text-zinc-600 dark:text-zinc-300 hover:text-blue-600 dark:hover:text-blue-300 hover:bg-blue-50 dark:hover:bg-blue-900 flex items-center justify-center px-3 py-2.5 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-transparent rounded-sm text-xs font-medium whitespace-nowrap"
          :title="t('gps.use_current_location')"
          :aria-label="t('gps.use_current_location')"
          @click="useCurrentLocation"
        >
          <MdiIcon
            :path="mdiCrosshairsGps"
            :size="16"
            :class="[
              'mr-1',
              {
                'text-blue-600 dark:text-blue-300 animate-pulse': isSearchingLocation,
                'text-zinc-600 dark:text-zinc-300': !isSearchingLocation,
              },
            ]"
          />
        </button>
      </ClientOnly>
    </div>
    <ClientOnly>
      <div
        v-if="currently_actively_picking && data && query.length !== 0"
        class="shadow-4xl bg-zinc-50 dark:bg-zinc-900 border-zinc-200 dark:border-zinc-700 absolute inset-x-0 top-full z-30 mt-1 flex max-h-[calc(100vh-80px)] flex-col gap-4 overflow-auto rounded border p-3.5 shadow-zinc-700/30 dark:shadow-zinc-200/30"
      >
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
        <ul v-for="s in data.sections" v-cloak :key="s.facet" class="flex flex-col gap-2">
          <div class="flex items-center">
            <span class="text-md text-zinc-800 dark:text-zinc-100 me-4 flex-shrink">{{ t(`sections.${s.facet}`) }}</span>
            <div class="border-zinc-800 dark:border-zinc-100 flex-grow border-t" />
          </div>

          <template v-for="(e, i) in tagSectionEntries(s)" :key="e.id">
            <SearchResultContent
              v-if="expandedFacets.has(s.facet) || i < s.n_visible"
              :highlighted="e.id === visibleElements[highlighted ?? -1]"
              :item="e"
              @mousedown.prevent
              @click="select(e.id)"
              @mouseover="highlighted = i"
            />
          </template>
          <li class="-mt-2">
            <Btn
              v-if="!expandedFacets.has(s.facet) && s.n_visible < s.entries.length"
              variant="linkButton"
              size="sm"
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
        </ul>
      </div>
    </ClientOnly>
  </div>
</template>

<i18n lang="yaml">
de:
  input:
    placeholder-from: Von
    placeholder-to: Nach
    aria-actionlabel: Suche nach dem im Suchfeld eingetragenen Raum
    aria-searchlabel: Suchfeld
    action: Go
  search_route: Route suchen
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
  error:
    header: Bei der Suche ist ein Fehler aufgetreten
    reason: Der Grund für diesen Fehler ist
    call_to_action: Wenn dieses Problem weiterhin besteht, kontaktiere uns bitte über das Feedback-Formular.
  gps:
    use_current_location: Aktuellen Standort verwenden (GPS)
    my_location: Mein Standort
    searching_location: Standort wird gesucht...
    error:
      permission_denied: Standortzugriff wurde verweigert. Bitte erlaube den Zugriff auf deinen Standort in den Browser-Einstellungen.
      position_unavailable: Standort konnte nicht ermittelt werden. Bitte versuche es später erneut.
      timeout: Standortermittlung dauerte zu lange. Bitte versuche es erneut.
      general: Fehler beim Ermitteln des Standorts. Bitte versuche es erneut.
      not_supported: Geolokation wird von diesem Browser nicht unterstützt.
      https_required: Geolokation erfordert eine sichere Verbindung (HTTPS).
en:
  input:
    placeholder-from: From
    placeholder-to: To
    aria-actionlabel: Search for the room-query entered in the search field
    aria-searchlabel: Search-field
    action: Go
  search_route: Search route
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
  error:
    header: Something went wrong while searching
    reason: Reason for this error is
    call_to_action: If this issue persists, please contact us via the feedback form.
  gps:
    use_current_location: Use current location (GPS)
    my_location: My location
    searching_location: Searching for location...
    error:
      permission_denied: Location access was denied. Please allow location access in your browser settings.
      position_unavailable: Location could not be determined. Please try again later.
      timeout: Location request timed out. Please try again.
      general: Error getting location. Please try again.
      not_supported: Geolocation is not supported by this browser.
      https_required: Geolocation requires a secure connection (HTTPS).
</i18n>
