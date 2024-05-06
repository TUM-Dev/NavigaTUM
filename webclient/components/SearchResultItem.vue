<script setup lang="ts">
import {
  BuildingOffice2Icon,
  BuildingOfficeIcon,
  MagnifyingGlassIcon,
  MapPinIcon,
} from "@heroicons/vue/24/outline/index.js";
import { ChevronDownIcon } from "@heroicons/vue/16/solid/index.js";
import type { components } from "~/api_types/index.js";

defineProps<{
  item: SitesBuildingsEntry | RoomEntry;
  highlighted: boolean;
}>();
const emit = defineEmits(["click", "mousedown", "mouseover"]);
type SitesBuildingsEntry = components["schemas"]["SitesBuildingsEntry"];
type RoomEntry = components["schemas"]["RoomEntry"];
</script>

<template>
  <li class="bg-zinc-50 border-zinc-200 rounded-sm border hover:bg-blue-100">
    <NuxtLink
      :class="{ 'bg-blue-200': highlighted }"
      :to="'/view/' + item.id"
      class="focusable flex gap-3 px-4 py-3"
      @click="() => emit('click')"
      @mousedown="() => emit('mousedown')"
      @mouseover="() => emit('mouseover')"
    >
      <div class="my-auto min-w-11">
        <div
          v-if="item.type === 'room' || item.type === 'virtual_room' || item.type === 'poi'"
          class="text-zinc-900 p-2"
        >
          <MagnifyingGlassIcon v-if="item.parsed_id" class="h-6 w-6" />
          <MapPinIcon v-else class="h-6 w-6" />
        </div>
        <div v-else class="text-white bg-blue-500 rounded-full p-2">
          <BuildingOfficeIcon v-if="item.type === 'building'" class="mx-auto h-6 w-6" />
          <BuildingOffice2Icon v-else class="mx-auto h-6 w-6" />
        </div>
      </div>
      <div class="text-zinc-600 flex flex-col gap-0.5">
        <div class="flex flex-col">
          <div
            v-if="(item.type === 'room' || item.type === 'virtual_room' || item.type === 'poi') && item.parsed_id"
            class="flex flex-row"
          >
            <ChevronDownIcon class="mt-0.5 h-4 w-4" />
            <span v-html="item.parsed_id" />
          </div>
          <span class="line-clamp-1" v-html="item.name" />
        </div>
        <small>
          {{ item.subtext }}
          <template v-if="item.type === 'room' || item.type === 'virtual_room' || item.type === 'poi'"
            >, <b v-html="item.subtext_bold"
          /></template>
        </small>
      </div>
    </NuxtLink>
    <!-- <div class="menu-badge"><label class="label label-primary">2</label></div> -->
  </li>
</template>
