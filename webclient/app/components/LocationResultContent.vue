<script setup lang="ts">
import { mdiChevronRight } from "@mdi/js";
import PreviewIcon from "~/components/PreviewIcon.vue";
import type { LocationResultEntry } from "~/utils/lectureRow";

const props = defineProps<{
  item: LocationResultEntry;
  highlighted: boolean;
}>();

// Only room | virtual_room | poi carry a `parsed_id` chevron and a bold subtext.
const ROOM_LIKE_TYPES: ReadonlySet<LocationResultEntry["type"]> = new Set([
  "room",
  "virtual_room",
  "poi",
]);
const isRoomLike = computed(() => ROOM_LIKE_TYPES.has(props.item.type));
const parsedId = computed(() => (isRoomLike.value ? props.item.parsed_id : null) || null);
const subtextBold = computed(() => (isRoomLike.value ? props.item.subtext_bold : null) || null);
</script>

<template>
  <div class="flex gap-1 px-2 py-3 md:gap-3 md:px-4" :class="{ 'bg-blue-200 dark:bg-blue-700': highlighted }">
    <PreviewIcon :item="item" />
    <div class="text-zinc-600 dark:text-zinc-300 flex flex-1 flex-col gap-0.5">
      <div class="flex flex-col">
        <div v-if="parsedId" class="flex flex-row items-center">
          <span class="text-zinc-900 dark:text-zinc-50 shrink-0" v-html="parsedId" />
          <MdiIcon :path="mdiChevronRight" :size="16" class="text-zinc-400 dark:text-zinc-500 shrink-0" />
          <span class="text-zinc-400 dark:text-zinc-500 line-clamp-1 shrink text-sm" v-html="item.name" />
        </div>
        <span v-else class="line-clamp-1" v-html="item.name" />
      </div>
      <small v-if="item.subtext || subtextBold">
        {{ item.subtext }}
        <template v-if="subtextBold"><template v-if="item.subtext">, </template><b v-html="subtextBold" /></template>
      </small>
    </div>
  </div>
</template>
