<script setup lang="ts">
import { ChevronRightIcon } from "@heroicons/vue/16/solid/index.js";
import type { components } from "~/api_types/index.js";
import PreviewIcon from "~/components/PreviewIcon.vue";

defineProps<{
  item: SitesBuildingsEntry | RoomEntry;
  highlighted: boolean;
}>();
type SitesBuildingsEntry = components["schemas"]["SitesBuildingsEntry"];
type RoomEntry = components["schemas"]["RoomEntry"];
</script>

<template>
  <div class="flex gap-1 px-2 py-3 md:gap-3 md:px-4" :class="{ 'bg-blue-200': highlighted }">
    <PreviewIcon :item="{ type: item.type, parsed_id: undefined }" />
    <div class="text-zinc-600 flex flex-col gap-0.5">
      <div class="flex flex-col">
        <div
          v-if="(item.type === 'room' || item.type === 'virtual_room' || item.type === 'poi') && item.parsed_id"
          class="flex flex-row items-center"
        >
          <span class="text-zinc-900 shrink-0" v-html="item.parsed_id" />
          <ChevronRightIcon class="text-zinc-400 h-4 w-4 shrink-0" />
          <span class="text-zinc-400 line-clamp-1 shrink text-sm" v-html="item.name" />
        </div>
        <span v-else class="line-clamp-1" v-html="item.name" />
      </div>
      <small>
        {{ item.subtext }}
        <template v-if="item.type === 'room' || item.type === 'virtual_room' || item.type === 'poi'"
          >, <b v-html="item.subtext_bold"
        /></template>
      </small>
    </div>
  </div>
</template>
