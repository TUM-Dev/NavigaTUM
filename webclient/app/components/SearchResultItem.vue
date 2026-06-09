<script setup lang="ts">
import { mdiChevronRight } from "@mdi/js";
import type { components } from "~/api_types/index.js";
import PreviewIcon from "~/components/PreviewIcon.vue";
import {
  firstUpcoming,
  formatUpcoming,
  type LectureLocale,
  lectureTitle,
} from "~/utils/lectureRow";

type ResultEntry = components["schemas"]["ResultEntry"];

const props = defineProps<{
  item: ResultEntry;
  highlighted: boolean;
}>();

const { t, locale } = useI18n({ useScope: "local" });
const localeKey = computed<LectureLocale>(() => (locale.value === "de" ? "de" : "en"));
const lectureTitleText = computed(() =>
  props.item.kind === "lecture" ? lectureTitle(props.item, localeKey.value) : ""
);
const lectureFirst = computed(() =>
  props.item.kind === "lecture" ? firstUpcoming(props.item) : null
);
const lectureMeta = computed(() => {
  const event = lectureFirst.value;
  if (!event) return null;
  return {
    when: formatUpcoming(event, localeKey.value),
    room: event.room_name,
    startAt: event.start_at,
  };
});
</script>

<template>
  <div class="flex gap-1 px-2 py-3 md:gap-3 md:px-4" :class="{ 'bg-blue-200 dark:bg-blue-700': highlighted }">
    <PreviewIcon :item="{ type: item.type, parsed_id: undefined }" />
    <div class="text-zinc-600 dark:text-zinc-300 flex flex-1 flex-col gap-0.5">
      <template v-if="item.kind === 'lecture'">
        <span class="text-zinc-900 dark:text-zinc-50 line-clamp-1 font-medium">{{ lectureTitleText }}</span>
        <small v-if="lectureMeta" class="flex flex-wrap items-center gap-x-2 gap-y-0.5">
          <time :datetime="lectureMeta.startAt">{{ lectureMeta.when }}</time>
          <span aria-hidden="true">·</span>
          <span class="line-clamp-1">{{ lectureMeta.room }}</span>
        </small>
        <small v-else>{{ t("lecture.no_upcoming") }}</small>
      </template>
      <template v-else>
        <div class="flex flex-col">
          <div
            v-if="(item.type === 'room' || item.type === 'virtual_room' || item.type === 'poi') && item.parsed_id"
            class="flex flex-row items-center"
          >
            <span class="text-zinc-900 dark:text-zinc-50 shrink-0" v-html="item.parsed_id" />
            <MdiIcon :path="mdiChevronRight" :size="16" class="text-zinc-400 dark:text-zinc-500 shrink-0" />
            <span class="text-zinc-400 dark:text-zinc-500 line-clamp-1 shrink text-sm" v-html="item.name" />
          </div>
          <span v-else class="line-clamp-1" v-html="item.name" />
        </div>
        <small
          v-if="
            item.subtext ||
            ((item.type === 'room' || item.type === 'virtual_room' || item.type === 'poi') && item.subtext_bold)
          "
        >
          {{ item.subtext }}
          <template
            v-if="(item.type === 'room' || item.type === 'virtual_room' || item.type === 'poi') && item.subtext_bold"
            ><template v-if="item.subtext">, </template><b v-html="item.subtext_bold"
          /></template>
        </small>
      </template>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  lecture:
    no_upcoming: Keine anstehenden Termine
en:
  lecture:
    no_upcoming: No upcoming events
</i18n>
