<script setup lang="ts">
import { mdiChevronDown, mdiChevronRight, mdiDotsHorizontal, mdiMapMarker } from "@mdi/js";
import type { components } from "~/api_types/index.js";
import SearchResultItem from "~/components/SearchResultItem.vue";
import {
  formatUpcoming,
  LECTURE_EVENT_NAV_CAP,
  type LectureLocale,
  LectureNavKey,
  lectureEventPath,
  useLectureRowExpansion,
} from "~/utils/lectureRow";

type ResultEntry = components["schemas"]["ResultEntry"];

const props = defineProps<{
  item: ResultEntry;
  highlighted: boolean;
}>();

// Explicit emits suppress Vue's fallthrough binding on the <li> root so the
// parent's @click doesn't fire when the user only toggles the row open.
const emit = defineEmits<{
  (e: "click"): void;
  (e: "mouseover"): void;
}>();

const { t, locale } = useI18n({ useScope: "local" });
// Controlled when AppSearchBar provides a LectureNavController; uncontrolled on
// /search where each row owns its own toggle.
const nav = inject(LectureNavKey, null);
const row = useLectureRowExpansion();

const localeKey = computed<LectureLocale>(() => (locale.value === "de" ? "de" : "en"));
const events = computed(() => props.item.upcoming ?? []);
const isExpanded = computed(() => (nav ? nav.expanded(props.item.id) : row.expanded.value));
const visibleEvents = computed(() => {
  if (!nav) return events.value;
  if (!nav.expanded(props.item.id)) return [];
  if (nav.showAll(props.item.id)) return events.value;
  return events.value.slice(0, LECTURE_EVENT_NAV_CAP);
});
const showMoreVisible = computed(() => {
  if (!nav) return false;
  if (!nav.expanded(props.item.id)) return false;
  if (nav.showAll(props.item.id)) return false;
  return events.value.length > LECTURE_EVENT_NAV_CAP;
});
const highlightedEventIndex = computed(() => nav?.highlightedEventIndex(props.item.id) ?? null);
const showMoreHighlighted = computed(() => nav?.showMoreHighlighted(props.item.id) ?? false);

function handleHeaderActivate(): void {
  if (nav) nav.toggle(props.item.id);
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
          @click="nav?.revealMore(item.id)"
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
