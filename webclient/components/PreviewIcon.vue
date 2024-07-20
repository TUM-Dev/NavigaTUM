<script setup lang="ts">
import {
  BuildingOffice2Icon,
  BuildingOfficeIcon,
  MagnifyingGlassIcon,
  MapPinIcon,
} from "@heroicons/vue/24/outline/index.js";
import type { components } from "~/api_types/index.js";

type SitesBuildingsEntry = components["schemas"]["SitesBuildingsEntry"];
type SitesBuildingsEntryItem = {
  type: SitesBuildingsEntry["type"];
};
type RoomEntry = components["schemas"]["RoomEntry"];
type RoomEntryItem = {
  type: RoomEntry["type"];
  parsed_id: RoomEntry["parsed_id"];
};
defineProps<{ item: SitesBuildingsEntryItem | RoomEntryItem }>();
</script>

<template>
  <div class="my-auto min-w-9 md:min-w-11">
    <div v-if="item.type === 'room' || item.type === 'virtual_room' || item.type === 'poi'" class="text-zinc-900 p-2">
      <MagnifyingGlassIcon v-if="item.parsed_id" class="h-5 w-5 md:h-6 md:w-6" />
      <MapPinIcon v-else class="h-5 w-5 md:h-6 md:w-6" />
    </div>
    <div v-else class="text-white bg-blue-500 rounded-full p-2">
      <BuildingOfficeIcon v-if="item.type === 'building'" class="mx-auto h-5 w-5 md:h-6 md:w-6" />
      <BuildingOffice2Icon v-else class="mx-auto h-5 w-5 md:h-6 md:w-6" />
    </div>
  </div>
</template>
