<script setup lang="ts">
import { useDebounceFn } from "@vueuse/core";
import type { components } from "~/api_types";
import { formatEventDateRange } from "~/utils/datetime";

type EventEntry = components["schemas"]["EventEntry"];
type SearchResponse = components["schemas"]["SearchResponse"];

const emit = defineEmits<{ pick: [entry: EventEntry] }>();

const { t, locale } = useI18n({ useScope: "local" });
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
  // `type=event` implies enabling the default-off events facet.
  params.append("type", "event");
  params.set("limit_events", "8");
  // Disable highlight markers so the response strings render as plain text.
  params.set("pre_highlight", "");
  params.set("post_highlight", "");
  return `${runtimeConfig.public.apiURL}/api/search?${params.toString()}`;
});

const entries = ref<readonly EventEntry[]>([]);
let searchCounter = 0;
watch(searchUrl, async (url) => {
  if (!url) {
    entries.value = [];
    return;
  }
  const ticket = ++searchCounter;
  try {
    const res = await $fetch<SearchResponse>(url, { credentials: "omit" });
    if (ticket !== searchCounter) return;
    entries.value = res.sections.flatMap((s) => (s.facet === "events" ? s.entries : []));
  } catch {
    if (ticket === searchCounter) entries.value = [];
  }
});

watch(entries, () => {
  highlighted.value = entries.value.length > 0 ? 0 : -1;
});

function lastHeld(entry: EventEntry): string {
  return formatEventDateRange(entry.starts_at, entry.ends_at, locale.value === "de" ? "de" : "en");
}

function selectEntry(entry: EventEntry) {
  query.value = "";
  open.value = false;
  emit("pick", entry);
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
  <div>
    <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium" for="add-event-search">
      {{ t("label") }}
    </label>
    <div class="relative">
      <input
        id="add-event-search"
        v-model="query"
        type="text"
        class="focusable input-field w-full rounded border px-2 py-1 text-sm"
        :placeholder="t('placeholder')"
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
          class="flex w-full cursor-pointer items-center gap-2 px-2 py-1.5 text-left hover:bg-blue-100 dark:hover:bg-blue-800"
          :class="{ 'bg-blue-200 dark:bg-blue-700': idx === highlighted }"
          @mousedown.prevent="selectEntry(entry)"
          @mouseenter="highlighted = idx"
        >
          <img
            :src="`${runtimeConfig.public.cdnURL}${entry.image}`"
            alt=""
            class="h-10 w-10 flex-shrink-0 rounded object-cover"
            loading="lazy"
          />
          <span class="flex flex-col">
            <span class="text-zinc-900 dark:text-zinc-50 text-sm">{{ entry.name }}</span>
            <span class="text-zinc-500 dark:text-zinc-400 text-xs">{{ t("last_held", [lastHeld(entry)]) }}</span>
          </span>
        </button>
      </div>
      <p v-else-if="open && query.length > 0 && debounced.length < 2" class="text-zinc-500 dark:text-zinc-400 mt-1 text-xs">
        {{ t("type_more") }}
      </p>
    </div>
    <p class="text-zinc-500 dark:text-zinc-400 mt-1 text-xs">{{ t("help") }}</p>
  </div>
</template>

<i18n lang="yaml">
de:
  label: Auf einer bestehenden Veranstaltung aufbauen (optional)
  placeholder: Bestehende Veranstaltung suchen…
  help: Für wiederkehrende Veranstaltungen oder Korrekturen - die Felder werden vorausgefüllt.
  no_results: Keine passende Veranstaltung gefunden
  type_more: Mindestens 2 Zeichen eingeben
  last_held: "Zuletzt: {0}"
en:
  label: Base it on an existing event (optional)
  placeholder: Search for an existing event…
  help: For recurring events or corrections - the fields are pre-filled.
  no_results: No matching event found
  type_more: Type at least 2 characters
  last_held: "Last held: {0}"
</i18n>
