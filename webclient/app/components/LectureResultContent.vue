<script setup lang="ts">
import PreviewIcon from "~/components/PreviewIcon.vue";
import {
  firstUpcoming,
  formatUpcoming,
  type LectureLocale,
  type LectureResultEntry,
  lectureTitle,
} from "~/utils/lectureRow";

const props = defineProps<{
  item: LectureResultEntry;
  highlighted: boolean;
}>();

const { t, locale } = useI18n({ useScope: "local" });
const localeKey = computed<LectureLocale>(() => (locale.value === "de" ? "de" : "en"));
const title = computed(() => lectureTitle(props.item, localeKey.value));
const meta = computed(() => {
  const event = firstUpcoming(props.item);
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
    <PreviewIcon :item="item" />
    <div class="text-zinc-600 dark:text-zinc-300 flex flex-1 flex-col gap-0.5">
      <span class="text-zinc-900 dark:text-zinc-50 line-clamp-1 font-medium">{{ title }}</span>
      <small v-if="meta" class="flex flex-wrap items-center gap-x-2 gap-y-0.5">
        <time :datetime="meta.startAt">{{ meta.when }}</time>
        <span aria-hidden="true">·</span>
        <span class="line-clamp-1">{{ meta.room }}</span>
      </small>
      <small v-else>{{ t("lecture.no_upcoming") }}</small>
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
