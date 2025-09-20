<script setup lang="ts">
import { mdiMagnify, mdiMapMarker, mdiOfficeBuilding, mdiOfficeBuildingOutline } from "@mdi/js";
import type { components } from "~/api_types/index.js";

type ResultEntry = components["schemas"]["ResultEntry"];
type ResultEntryItem = {
  type: ResultEntry["type"];
  parsed_id: ResultEntry["parsed_id"];
};
defineProps<{ item: ResultEntryItem }>();
</script>

<template>
  <div class="my-auto min-w-9 md:min-w-11">
    <div v-if="item.type === 'room' || item.type === 'virtual_room' || item.type === 'poi'" class="text-zinc-900 p-2">
      <MdiIcon :path="mdiMagnify" :size="20" v-if="item.parsed_id" class="md:!w-6 md:!h-6" />
      <MdiIcon :path="mdiMapMarker" :size="20" v-else class="md:!w-6 md:!h-6" />
    </div>
    <div v-else class="text-white bg-blue-500 rounded-full p-2">
      <MdiIcon
        :path="mdiOfficeBuildingOutline"
        :size="20"
        v-if="item.type === 'building'"
        class="mx-auto md:!w-6 md:!h-6"
      />
      <MdiIcon :path="mdiOfficeBuilding" :size="20" v-else class="mx-auto md:!w-6 md:!h-6" />
    </div>
  </div>
</template>
