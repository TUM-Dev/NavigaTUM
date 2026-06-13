<script setup lang="ts">
import { mdiClose } from "@mdi/js";
import { useDebounceFn } from "@vueuse/core";
import type { components } from "~/api_types";
import { type SearchResultEntry, tagSectionEntries } from "~/utils/lectureRow";

type FacetFilter = components["schemas"]["FacetFilter"];
type SearchResponse = components["schemas"]["SearchResponse"];

const selectedId = defineModel<string>("selectedId", { required: true });

const selectedName = defineModel<string>("selectedName", { required: true });

const props = defineProps<{
  allowedTypes: readonly FacetFilter[];
  placeholder?: string;
}>();

const { t } = useI18n({ useScope: "local" });
const runtimeConfig = useRuntimeConfig();

const query = ref("");
const open = ref(false);
const highlighted = ref(-1);
const debounced = ref("");
const applyDebounced = useDebounceFn((value: string) => {
  debounced.value = value.trim();
}, 200);
watch(query, applyDebounced);

const searchUrl = computed(() => {
  if (debounced.value.length < 2) return null;
  const params = new URLSearchParams();
  params.set("q", debounced.value);
  for (const allowedType of props.allowedTypes) params.append("type", allowedType);
  params.set("limit_all", "8");
  // Disable highlight markers so we can use the response strings as plain text.
  params.set("pre_highlight", "");
  params.set("post_highlight", "");
  return `${runtimeConfig.public.apiURL}/api/search?${params.toString()}`;
});

// Manual debounced fetch - useFetch with `immediate: false` + `lazy` was firing inconsistently
// against an empty URL, leaving `searchResults` in a half-state that broke the entries computed.
const searchResults = ref<SearchResponse | null>(null);
let searchCounter = 0;
watch(searchUrl, async (url) => {
  if (!url) {
    searchResults.value = null;
    return;
  }
  const ticket = ++searchCounter;
  try {
    const res = await $fetch<SearchResponse>(url, { credentials: "omit" });
    if (ticket === searchCounter) searchResults.value = res;
  } catch {
    if (ticket === searchCounter) searchResults.value = null;
  }
});

const entries = computed<SearchResultEntry[]>(() => {
  if (!debounced.value) return [];
  const sections = searchResults.value?.sections;
  if (!sections) return [];
  return sections.flatMap((s) => tagSectionEntries(s));
});

watch(entries, () => {
  highlighted.value = entries.value.length > 0 ? 0 : -1;
});

function selectEntry(entry: SearchResultEntry) {
  selectedId.value = entry.id;
  selectedName.value = entry.name;
  query.value = "";
  open.value = false;
}

function clearSelection() {
  selectedId.value = "";
  selectedName.value = "";
  query.value = "";
  open.value = true;
  nextTick(() => document.getElementById("entry-picker-input")?.focus());
}

function onBlur() {
  // Delay so click-to-select fires before the dropdown collapses.
  setTimeout(() => {
    open.value = false;
  }, 150);
}

function onKeydown(e: KeyboardEvent) {
  if (!open.value) return;
  if (e.key === "ArrowDown") {
    e.preventDefault();
    highlighted.value = Math.min(highlighted.value + 1, entries.value.length - 1);
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    highlighted.value = Math.max(highlighted.value - 1, 0);
  } else if (e.key === "Enter") {
    const entry = entries.value[highlighted.value];
    if (entry) {
      e.preventDefault();
      selectEntry(entry);
    }
  } else if (e.key === "Escape") {
    open.value = false;
  }
}
</script>

<template>
  <div class="relative">
    <div v-if="selectedId" class="bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 flex items-center gap-2 rounded border px-2 py-1">
      <div class="flex flex-grow flex-col text-sm">
        <span class="text-zinc-900 dark:text-zinc-50">{{ selectedName || selectedId }}</span>
        <span v-if="selectedName && selectedName !== selectedId" class="text-zinc-500 dark:text-zinc-400 text-xs">{{ selectedId }}</span>
      </div>
      <button type="button" class="focusable rounded-sm" :title="t('clear')" @click="clearSelection">
        <MdiIcon :path="mdiClose" :size="16" class="text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200" />
      </button>
    </div>
    <div v-else>
      <input
        id="entry-picker-input"
        v-model="query"
        type="text"
        class="focusable input-field w-full rounded border px-2 py-1 text-sm"
        :placeholder="placeholder || t('placeholder')"
        @focus="open = true"
        @blur="onBlur"
        @keydown="onKeydown"
      />
      <div
        v-if="open && debounced.length >= 2"
        class="bg-zinc-50 dark:bg-zinc-900 border-zinc-300 dark:border-zinc-600 absolute left-0 right-0 z-50 mt-1 max-h-72 overflow-y-auto rounded border shadow-lg"
      >
        <p v-if="entries.length === 0" class="text-zinc-500 dark:text-zinc-400 px-2 py-2 text-sm">{{ t("no_results") }}</p>
        <button
          v-for="(entry, idx) in entries"
          :key="entry.id"
          type="button"
          class="block w-full cursor-pointer text-left hover:bg-blue-100 dark:hover:bg-blue-800"
          :class="{ 'bg-blue-200 dark:bg-blue-700': idx === highlighted }"
          @mousedown.prevent="selectEntry(entry)"
          @mouseenter="highlighted = idx"
        >
          <SearchResultContent :item="entry" :highlighted="idx === highlighted" />
        </button>
      </div>
      <p v-else-if="open && query.length > 0 && debounced.length < 2" class="text-zinc-500 dark:text-zinc-400 mt-1 text-xs">
        {{ t("type_more") }}
      </p>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  placeholder: Suchen…
  clear: Auswahl entfernen
  no_results: Keine Ergebnisse
  type_more: Mindestens 2 Zeichen eingeben
en:
  placeholder: Search…
  clear: Clear selection
  no_results: No results
  type_more: Type at least 2 characters
</i18n>
