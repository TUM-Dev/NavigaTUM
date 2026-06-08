<script setup lang="ts">
import type { components } from "~/api_types/index.js";
import LectureSearchResultRow from "~/components/LectureSearchResultRow.vue";
import { type EntityPath, entityPath, isRoutableEntityType } from "~/utils/entityPath";

type ResultEntry = components["schemas"]["ResultEntry"];

const props = withDefaults(
  defineProps<{
    item: ResultEntry;
    highlighted: boolean;
    lectureExpanded?: boolean | null;
    lectureVisibleEventCount?: number | null;
    lectureHighlightedEventIndex?: number | null;
    lectureShowMoreVisible?: boolean;
    lectureShowMoreHighlighted?: boolean;
  }>(),
  {
    lectureExpanded: null,
    lectureVisibleEventCount: null,
    lectureHighlightedEventIndex: null,
    lectureShowMoreVisible: false,
    lectureShowMoreHighlighted: false,
  }
);
const emit = defineEmits(["click", "mousedown", "mouseover", "showMore", "toggle"]);

const isLecture = computed(() => props.item.type === "lecture");

// Entity results link to their canonical /{type}/{id} path. Non-routable results
// (e.g. Nominatim addresses, only surfaced on the navigate page) have no entity
// route and render as a plain, non-navigable row.
const to = computed<EntityPath | null>(() =>
  isRoutableEntityType(props.item.type) ? entityPath(props.item.id, props.item.type) : null
);
</script>

<template>
  <LectureSearchResultRow
    v-if="isLecture"
    :item="item"
    :highlighted="highlighted"
    :expanded="lectureExpanded"
    :visible-event-count="lectureVisibleEventCount"
    :highlighted-event-index="lectureHighlightedEventIndex"
    :show-more-visible="lectureShowMoreVisible"
    :show-more-highlighted="lectureShowMoreHighlighted"
    @click="() => emit('click')"
    @mousedown="() => emit('mousedown')"
    @mouseover="() => emit('mouseover')"
    @show-more="() => emit('showMore')"
    @toggle="() => emit('toggle')"
  />
  <li
    v-else
    class="bg-zinc-50 dark:bg-zinc-900 border-zinc-200 dark:border-zinc-700 rounded-sm border hover:bg-blue-100 dark:hover:bg-blue-800"
  >
    <NuxtLinkLocale
      v-if="to"
      :to="to"
      class="focusable"
      @click="() => emit('click')"
      @mousedown="() => emit('mousedown')"
      @mouseover="() => emit('mouseover')"
    >
      <SearchResultItem :item="item" :highlighted="highlighted" />
    </NuxtLinkLocale>
    <SearchResultItem v-else :item="item" :highlighted="highlighted" />
  </li>
</template>
