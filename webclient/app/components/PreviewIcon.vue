<script setup lang="ts">
import {
  mdiMagnify,
  mdiMapMarker,
  mdiOfficeBuilding,
  mdiOfficeBuildingOutline,
  mdiSchool,
} from "@mdi/js";

// The icon keys off the opaque `type` string and whether a `parsed_id` matched;
// it is fed a small projection rather than a full search result, so it stays
// decoupled from the result union's variants.
interface ResultEntryItem {
  type: string;
  parsed_id?: string | null;
}
defineProps<{ item: ResultEntryItem }>();
</script>

<template>
  <div class="my-auto min-w-9 md:min-w-11">
    <div v-if="item.type === 'room' || item.type === 'virtual_room' || item.type === 'poi'" class="text-zinc-900 dark:text-zinc-50 p-2">
      <MdiIcon :path="mdiMagnify" :size="20" v-if="item.parsed_id" class="md:!w-6 md:!h-6" />
      <MdiIcon :path="mdiMapMarker" :size="20" v-else class="md:!w-6 md:!h-6" />
    </div>
    <div v-else class="text-white dark:text-black bg-blue-500 dark:bg-blue-400 rounded-full p-2">
      <MdiIcon
        :path="mdiOfficeBuildingOutline"
        :size="20"
        v-if="item.type === 'building'"
        class="mx-auto md:!w-6 md:!h-6"
      />
      <MdiIcon
        :path="mdiSchool"
        :size="20"
        v-else-if="item.type === 'lecture'"
        class="mx-auto md:!w-6 md:!h-6"
      />
      <MdiIcon :path="mdiOfficeBuilding" :size="20" v-else class="mx-auto md:!w-6 md:!h-6" />
    </div>
  </div>
</template>
