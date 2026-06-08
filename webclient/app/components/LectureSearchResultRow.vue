<script setup lang="ts">
import { mdiChevronDown, mdiChevronRight, mdiDotsHorizontal, mdiMapMarker } from "@mdi/js";
import type { components } from "~/api_types/index.js";
import SearchResultItem from "~/components/SearchResultItem.vue";
import {
  formatUpcoming,
  type LectureLocale,
  lectureEventPath,
  useLectureRowExpansion,
} from "~/utils/lectureRow";

type ResultEntry = components["schemas"]["ResultEntry"];

// Controlled props (expanded, visibleEventCount, …) drive the row from the
// dropdown's keyboard nav. When omitted the row falls back to its own toggle.
const props = withDefaults(
  defineProps<{
    item: ResultEntry;
    highlighted: boolean;
    expanded?: boolean | null;
    visibleEventCount?: number | null;
    highlightedEventIndex?: number | null;
    showMoreVisible?: boolean;
    showMoreHighlighted?: boolean;
  }>(),
  {
    expanded: null,
    visibleEventCount: null,
    highlightedEventIndex: null,
    showMoreVisible: false,
    showMoreHighlighted: false,
  }
);

// Explicit emits suppress Vue's fallthrough binding on the <li> root so the
// parent's @click doesn't fire when the user only toggles the row open.
const emit = defineEmits<{
  (e: "click"): void;
  (e: "mousedown"): void;
  (e: "mouseover"): void;
  (e: "showMore"): void;
  (e: "toggle"): void;
}>();

const { t, locale } = useI18n({ useScope: "local" });
const row = useLectureRowExpansion();

const localeKey = computed<LectureLocale>(() => (locale.value === "de" ? "de" : "en"));
const events = computed(() => props.item.upcoming ?? []);
// Controlled when the parent passes a boolean `expanded` (AppSearchBar's
// dropdown owns the truth), uncontrolled when `expanded` is null (the
// /search page, where each row drives its own toggle).
const isControlled = computed(() => props.expanded !== null);
const isExpanded = computed(() =>
  isControlled.value ? Boolean(props.expanded) : row.expanded.value
);
const visibleEvents = computed(() => {
  const cap = props.visibleEventCount;
  if (cap === null || cap === undefined) return events.value;
  return events.value.slice(0, Math.max(0, cap));
});

function handleHeaderActivate(): void {
  if (isControlled.value) emit("toggle");
  else row.toggle();
}

function onHeaderKeydown(e: KeyboardEvent) {
  if (e.key === "Enter" || e.key === " ") {
    e.preventDefault();
    handleHeaderActivate();
  }
}
</script>

<template>
  <li
    class="bg-zinc-50 dark:bg-zinc-900 border-zinc-200 dark:border-zinc-700 rounded-sm border"
    :class="{ 'hover:bg-blue-100 dark:hover:bg-blue-800': !isExpanded }"
    @mouseover="emit('mouseover')"
  >
    <button
      type="button"
      class="focusable flex w-full items-center gap-1 text-left md:gap-3"
      :class="{ 'bg-blue-200 dark:bg-blue-700': highlighted }"
      :aria-expanded="isExpanded"
      :aria-label="isExpanded ? t('collapse') : t('expand')"
      @click="handleHeaderActivate"
      @mousedown="emit('mousedown')"
      @keydown="onHeaderKeydown"
    >
      <!-- Highlight bg lives on the button so it spans the chevron column. -->
      <SearchResultItem :item="item" :highlighted="false" />
      <MdiIcon
        :path="isExpanded ? mdiChevronDown : mdiChevronRight"
        :size="20"
        class="text-zinc-500 dark:text-zinc-400 me-2 ms-auto shrink-0 md:me-4"
      />
    </button>
    <ul
      v-if="isExpanded && (visibleEvents.length || showMoreVisible)"
      class="border-zinc-200 dark:border-zinc-700 flex flex-col gap-1 border-t px-2 py-2 md:px-4"
    >
      <li v-for="(event, idx) in visibleEvents" :key="`${event.start_at}-${event.room_code}-${idx}`">
        <NuxtLinkLocale
          :to="lectureEventPath(event)"
          class="focusable text-zinc-700 dark:text-zinc-200 flex items-center gap-2 rounded-sm px-1 py-1 hover:bg-blue-100 dark:hover:bg-blue-800"
          :class="{ 'bg-blue-200 dark:bg-blue-700': highlightedEventIndex === idx }"
          @click="emit('click')"
          @mousedown="emit('mousedown')"
        >
          <MdiIcon :path="mdiMapMarker" :size="16" class="text-zinc-400 dark:text-zinc-500 shrink-0" />
          <time :datetime="event.start_at" class="text-sm">{{ formatUpcoming(event, localeKey) }}</time>
          <span aria-hidden="true">·</span>
          <span class="line-clamp-1 text-sm">{{ event.room_name }}</span>
        </NuxtLinkLocale>
      </li>
      <li v-if="showMoreVisible">
        <button
          type="button"
          class="focusable text-zinc-600 dark:text-zinc-300 flex w-full items-center gap-2 rounded-sm px-1 py-1 text-sm font-medium hover:bg-blue-100 dark:hover:bg-blue-800"
          :class="{ 'bg-blue-200 dark:bg-blue-700': showMoreHighlighted }"
          @click="emit('showMore')"
          @mousedown="emit('mousedown')"
        >
          <MdiIcon :path="mdiDotsHorizontal" :size="16" class="text-zinc-400 dark:text-zinc-500 shrink-0" />
          <span>{{ t("show_more_events") }}</span>
        </button>
      </li>
    </ul>
  </li>
</template>

<i18n lang="yaml">
de:
  expand: Anstehende Termine anzeigen
  collapse: Anstehende Termine ausblenden
  show_more_events: Weitere Termine anzeigen
en:
  expand: Show upcoming events
  collapse: Hide upcoming events
  show_more_events: Show more events
</i18n>
