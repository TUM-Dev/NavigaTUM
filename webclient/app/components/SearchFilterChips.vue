<script setup lang="ts">
import { Popover, PopoverButton, PopoverPanel } from "@headlessui/vue";
import {
  mdiCheck,
  mdiChevronDown,
  mdiClose,
  mdiFilterVariant,
  mdiMagnify,
  mdiMapMarker,
  mdiTagOutline,
} from "@mdi/js";
import { useTemplateRef } from "vue";
import type { components } from "~/api_types";
import { useKnownUsages } from "~/composables/knownUsages";
import { FACET_OPTIONS, type SearchFilters } from "~/composables/searchFilters";

type SearchResponse = components["schemas"]["SearchResponse"];

interface LocationSuggestion {
  id: string;
  name: string;
  subtext: string;
}

const props = defineProps<{
  filters: SearchFilters;
}>();

const { t } = useI18n({ useScope: "local" });
const runtimeConfig = useRuntimeConfig();

const knownUsages = useKnownUsages();

// --- Location panel state ----------------------------------------------------
const locationOpen = ref(false);
const locationSearch = ref("");
const locationInput = useTemplateRef<HTMLInputElement>("locationInput");

const {
  data: locationData,
  status: locationStatus,
  refresh: refreshLocation,
} = useFetch<SearchResponse>(
  () => {
    const params = new URLSearchParams();
    params.append("q", locationSearch.value);
    params.append("limit_all", "8");
    params.append("limit_sites", "8");
    params.append("limit_buildings", "8");
    params.append("limit_rooms", "0");
    params.append("limit_pois", "0");
    params.append("pre_highlight", "<b class='text-blue'>");
    params.append("post_highlight", "</b>");
    return `${runtimeConfig.public.apiURL}/api/search?${params.toString()}`;
  },
  {
    dedupe: "cancel",
    lazy: true,
    immediate: false,
    watch: false,
  }
);

watch(locationSearch, (q) => {
  if (q.length >= 2) refreshLocation();
});

const locationSuggestions = computed<LocationSuggestion[]>(() => {
  if (locationSearch.value.length < 2) return [];
  const sections = locationData.value?.sections ?? [];
  return sections
    .filter((s) => s.facet === "sites" || s.facet === "buildings")
    .flatMap((s) => s.entries.map((e) => ({ id: e.id, name: e.name, subtext: e.subtext })));
});

const locationLoading = computed(
  () => locationSearch.value.length >= 2 && locationStatus.value === "pending"
);

function toggleLocationPanel() {
  locationOpen.value = !locationOpen.value;
  if (locationOpen.value) {
    closeUsage();
    nextTick(() => locationInput.value?.focus());
  } else {
    locationSearch.value = "";
  }
}

function selectLocation(id: string) {
  props.filters.addInFilter(id);
  locationSearch.value = "";
  locationInput.value?.focus();
}

// --- Usage panel state -------------------------------------------------------
const usageOpen = ref(false);
const usageSearch = ref("");
const usageInput = useTemplateRef<HTMLInputElement>("usageInput");

const filteredUsages = computed(() => knownUsages.filter(usageSearch.value));

function toggleUsagePanel() {
  usageOpen.value = !usageOpen.value;
  if (usageOpen.value) {
    closeLocation();
    nextTick(() => usageInput.value?.focus());
  } else {
    usageSearch.value = "";
  }
}

function closeUsage() {
  usageOpen.value = false;
  usageSearch.value = "";
}

function closeLocation() {
  locationOpen.value = false;
  locationSearch.value = "";
}
</script>

<template>
  <span class="text-zinc-500 dark:text-zinc-400 hidden items-center sm:inline-flex" :title="t('filter')" :aria-label="t('filter')">
    <MdiIcon :path="mdiFilterVariant" :size="18"/>
  </span>

  <!-- Type chip with popover -->
  <Popover class="relative">
    <PopoverButton
      class="focusable inline-flex items-center gap-1 rounded-full border px-3 py-1 text-sm transition-colors"
      :class="
        props.filters.typeFilter.value.length
          ? 'bg-blue-100 dark:bg-blue-800 border-blue-400 dark:border-blue-500 text-blue-800 dark:text-blue-100'
          : 'bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-600 text-zinc-700 dark:text-zinc-200 hover:bg-zinc-200 dark:hover:bg-zinc-700'
      "
    >
      {{ t("type") }}
      <template v-if="props.filters.typeFilter.value.length">
        ({{ props.filters.typeFilter.value.length }})
      </template>
      <MdiIcon :path="mdiChevronDown" :size="16"/>
    </PopoverButton>
    <Transition
      enter-active-class="transition duration-150 ease-out"
      enter-from-class="opacity-0 translate-y-1"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition duration-100 ease-in"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 translate-y-1"
    >
      <PopoverPanel
        class="ring-black/5 dark:ring-white/5 absolute left-0 z-20 mt-2 w-48 rounded-sm bg-white p-2 shadow-lg ring-1 dark:bg-zinc-800">
        <label
          v-for="opt in FACET_OPTIONS"
          :key="opt"
          class="flex cursor-pointer items-center gap-2 rounded-sm px-2 py-1.5 text-sm hover:bg-zinc-100 dark:hover:bg-zinc-100 text-zinc-800 dark:text-zinc-100"
        >
          <span
            class="flex h-4 w-4 items-center justify-center rounded-sm border"
            :class="
              props.filters.typeFilter.value.includes(opt)
                ? 'bg-blue-500 dark:bg-blue-400 border-blue-500 dark:border-blue-400 text-white dark:text-black'
                : 'border-zinc-400 dark:border-zinc-500'
            "
          >
            <MdiIcon v-if="props.filters.typeFilter.value.includes(opt)" :path="mdiCheck" :size="12"/>
          </span>
          <input
            type="checkbox"
            class="sr-only"
            :checked="props.filters.typeFilter.value.includes(opt)"
            @change="props.filters.toggleFilterValue('type', opt)"
          />
          {{ t(`types.${opt}`) }}
        </label>
      </PopoverPanel>
    </Transition>
  </Popover>

  <!-- Usage toggle: opens the inline panel below -->
  <button
    type="button"
    class="focusable inline-flex items-center gap-1 rounded-full border px-3 py-1 text-sm transition-colors"
    :class="
      props.filters.usageFilter.value.length || usageOpen
        ? 'bg-blue-100 dark:bg-blue-800 border-blue-400 dark:border-blue-500 text-blue-800 dark:text-blue-100'
        : 'bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-600 text-zinc-700 dark:text-zinc-200 hover:bg-zinc-200 dark:hover:bg-zinc-700'
    "
    :aria-expanded="usageOpen"
    aria-controls="usage-filter-panel"
    @click="toggleUsagePanel"
  >
    {{ t("usage") }}
    <template v-if="props.filters.usageFilter.value.length">
      ({{ props.filters.usageFilter.value.length }})
    </template>
    <MdiIcon :path="mdiChevronDown" :size="16"/>
  </button>

  <!-- Location toggle: opens the inline panel below -->
  <button
    type="button"
    class="focusable inline-flex items-center gap-1 rounded-full border px-3 py-1 text-sm transition-colors"
    :class="
      props.filters.inFilter.value.length || locationOpen
        ? 'bg-blue-100 dark:bg-blue-800 border-blue-400 dark:border-blue-500 text-blue-800 dark:text-blue-100'
        : 'bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-600 text-zinc-700 dark:text-zinc-200 hover:bg-zinc-200 dark:hover:bg-zinc-700'
    "
    :aria-expanded="locationOpen"
    aria-controls="location-filter-panel"
    @click="toggleLocationPanel"
  >
    {{ t("location") }}
    <template v-if="props.filters.inFilter.value.length">
      ({{ props.filters.inFilter.value.length }})
    </template>
    <MdiIcon :path="mdiChevronDown" :size="16"/>
  </button>

  <button
    v-if="props.filters.hasActiveFilters.value"
    type="button"
    class="focusable text-xs text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200 hover:underline"
    @click="props.filters.clearAll()"
  >
    {{ t("clear_all") }}
  </button>

  <!-- Inline usage panel -->
  <div v-if="usageOpen" id="usage-filter-panel" class="order-1 basis-full">
    <div class="border-zinc-200 dark:border-zinc-700 bg-white rounded-sm border shadow-sm dark:bg-zinc-800">
      <div class="border-zinc-200 dark:border-zinc-700 flex items-center justify-between border-b px-3 py-2">
        <span class="inline-flex items-center gap-1.5 text-sm font-medium text-zinc-700 dark:text-zinc-200">
          <MdiIcon :path="mdiTagOutline" :size="16"/>
          {{ t("usage_panel_title") }}
        </span>
        <button
          type="button"
          class="focusable text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200 rounded-sm p-1"
          :aria-label="t('close')"
          @click="closeUsage"
        >
          <MdiIcon :path="mdiClose" :size="16"/>
        </button>
      </div>

      <div
        v-if="props.filters.usageFilter.value.length"
        class="flex flex-wrap gap-1.5 border-b border-zinc-200 dark:border-zinc-700 px-3 py-2"
      >
        <span
          v-for="v in props.filters.usageFilter.value"
          :key="v"
          class="inline-flex items-center gap-1 rounded-full bg-blue-50 dark:bg-blue-900 px-2 py-0.5 text-xs text-blue-700 dark:text-blue-200"
        >
          {{ knownUsages.labelFor(v) }}
          <button
            type="button"
            class="focusable rounded-full hover:bg-blue-200 dark:hover:bg-blue-700"
            :aria-label="t('remove_usage')"
            @click="props.filters.removeFilter('usage', v)"
          >
            <MdiIcon :path="mdiClose" :size="12"/>
          </button>
        </span>
      </div>

      <div class="p-3">
        <div
          class="border-zinc-300 dark:border-zinc-600 flex items-center gap-2 rounded-sm border bg-white dark:bg-black px-2 py-1.5 focus-within:border-blue-500 dark:focus-within:border-blue-400 focus-within:ring-1 focus-within:ring-blue-500 dark:focus-within:ring-blue-400">
          <MdiIcon :path="mdiMagnify" :size="16" class="text-zinc-400 dark:text-zinc-500"/>
          <input
            ref="usageInput"
            v-model="usageSearch"
            type="text"
            class="flex-grow bg-transparent text-sm text-zinc-800 dark:text-zinc-100 outline-0 placeholder:text-zinc-400 dark:placeholder:text-zinc-500"
            :placeholder="t('usage_placeholder')"
            autocomplete="off"
            spellcheck="false"
          />
        </div>

        <p v-if="knownUsages.pending.value && !knownUsages.options.value.length"
           class="mt-2 px-2 py-1 text-xs text-zinc-500 dark:text-zinc-400">
          {{ t("loading") }}
        </p>
        <ul v-else-if="filteredUsages.length" class="mt-2 max-h-64 overflow-y-auto">
          <li v-for="opt in filteredUsages" :key="opt.slug">
            <label
              class="flex cursor-pointer items-center gap-2 rounded-sm px-2 py-1.5 text-sm hover:bg-zinc-100 dark:hover:bg-zinc-100"
            >
              <span
                class="flex h-4 w-4 flex-shrink-0 items-center justify-center rounded-sm border"
                :class="
                  props.filters.usageFilter.value.includes(opt.slug)
                    ? 'bg-blue-500 dark:bg-blue-400 border-blue-500 dark:border-blue-400 text-white dark:text-black'
                    : 'border-zinc-400 dark:border-zinc-500'
                "
              >
                <MdiIcon
                  v-if="props.filters.usageFilter.value.includes(opt.slug)"
                  :path="mdiCheck"
                  :size="12"
                />
              </span>
              <input
                type="checkbox"
                class="sr-only"
                :checked="props.filters.usageFilter.value.includes(opt.slug)"
                @change="props.filters.toggleFilterValue('usage', opt.slug)"
              />
              <UsageOptionContent :usage="opt" />
            </label>
          </li>
        </ul>
        <p v-else class="mt-2 px-2 py-1 text-xs text-zinc-500 dark:text-zinc-400">
          {{ t("no_results") }}
        </p>
      </div>
    </div>
  </div>

  <!-- Inline location panel -->
  <div v-if="locationOpen" id="location-filter-panel" class="order-1 basis-full">
    <div class="border-zinc-200 dark:border-zinc-700 bg-white rounded-sm border shadow-sm dark:bg-zinc-800">
      <div class="border-zinc-200 dark:border-zinc-700 flex items-center justify-between border-b px-3 py-2">
        <span class="inline-flex items-center gap-1.5 text-sm font-medium text-zinc-700 dark:text-zinc-200">
          <MdiIcon :path="mdiMapMarker" :size="16"/>
          {{ t("location_panel_title") }}
        </span>
        <button
          type="button"
          class="focusable text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200 rounded-sm p-1"
          :aria-label="t('close')"
          @click="closeLocation"
        >
          <MdiIcon :path="mdiClose" :size="16"/>
        </button>
      </div>

      <div
        v-if="props.filters.inFilter.value.length"
        class="flex flex-wrap gap-1.5 border-b border-zinc-200 dark:border-zinc-700 px-3 py-2"
      >
        <span
          v-for="v in props.filters.inFilter.value"
          :key="v"
          class="inline-flex items-center gap-1 rounded-full bg-blue-50 dark:bg-blue-900 px-2 py-0.5 text-xs text-blue-700 dark:text-blue-200"
        >
          {{ v }}
          <button
            type="button"
            class="focusable rounded-full hover:bg-blue-200 dark:hover:bg-blue-700"
            :aria-label="t('remove_location', { id: v })"
            @click="props.filters.removeFilter('in', v)"
          >
            <MdiIcon :path="mdiClose" :size="12"/>
          </button>
        </span>
      </div>

      <div class="p-3">
        <div
          class="border-zinc-300 dark:border-zinc-600 flex items-center gap-2 rounded-sm border bg-white dark:bg-black px-2 py-1.5 focus-within:border-blue-500 dark:focus-within:border-blue-400 focus-within:ring-1 focus-within:ring-blue-500 dark:focus-within:ring-blue-400">
          <MdiIcon :path="mdiMagnify" :size="16" class="text-zinc-400 dark:text-zinc-500"/>
          <input
            ref="locationInput"
            v-model="locationSearch"
            type="text"
            class="flex-grow bg-transparent text-sm text-zinc-800 dark:text-zinc-100 outline-0 placeholder:text-zinc-400 dark:placeholder:text-zinc-500"
            :placeholder="t('location_placeholder')"
            autocomplete="off"
            spellcheck="false"
          />
        </div>

        <ul v-if="locationSuggestions.length" class="mt-2 max-h-56 overflow-y-auto">
          <li
            v-for="suggestion in locationSuggestions"
            :key="suggestion.id"
            class="cursor-pointer rounded-sm px-2 py-2 text-sm hover:bg-zinc-100 dark:hover:bg-zinc-100"
            @click="selectLocation(suggestion.id)"
          >
            <div class="text-zinc-800 dark:text-zinc-100" v-html="suggestion.name"/>
            <div v-if="suggestion.subtext" class="text-xs text-zinc-500 dark:text-zinc-400">
              {{ suggestion.subtext }}
            </div>
          </li>
        </ul>
        <p
          v-else-if="locationSearch.length >= 2 && locationLoading"
          class="mt-2 px-2 py-1 text-xs text-zinc-500 dark:text-zinc-400"
        >
          {{ t("loading") }}
        </p>
        <p
          v-else-if="locationSearch.length >= 2"
          class="mt-2 px-2 py-1 text-xs text-zinc-500 dark:text-zinc-400"
        >
          {{ t("no_results") }}
        </p>
        <p v-else class="mt-2 px-2 py-1 text-xs text-zinc-400 dark:text-zinc-500">
          {{ t("location_hint") }}
        </p>
      </div>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  filter: Filter
  type: Typ
  usage: Nutzung
  usage_panel_title: Nach Nutzung filtern
  usage_placeholder: Nutzungsart suchen...
  remove_usage: Nutzung entfernen
  location: Standort
  location_panel_title: Standort einschränken
  location_placeholder: Gebäude oder Standort suchen...
  location_hint: Beginne zu tippen, um Standorte zu suchen
  loading: Laden...
  no_results: Keine Ergebnisse
  remove_location: "Standort {id} entfernen"
  close: Schließen
  clear_all: Leeren
  types:
    room: Raum
    building: Gebäude
    site: Gelände / Campus
    poi: Ort (POI)
    lecture: Vorlesung
en:
  filter: Filter
  type: Type
  usage: Usage
  usage_panel_title: Filter by usage
  usage_placeholder: Search usage type...
  remove_usage: Remove usage
  location: Location
  location_panel_title: Restrict to location
  location_placeholder: Search building or site...
  location_hint: Start typing to search locations
  loading: Loading...
  no_results: No results
  remove_location: "Remove location {id}"
  close: Close
  clear_all: Clear
  types:
    room: Room
    building: Building
    site: Site / Campus
    poi: Point of Interest
    lecture: Lecture
</i18n>
