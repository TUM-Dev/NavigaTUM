<script setup lang="ts">
import { mdiMapMarker, mdiOfficeBuilding, mdiOfficeBuildingOutline, mdiSchool } from "@mdi/js";
import type { LectureResultEntry, LocationResultEntry } from "~/utils/lectureRow";

// Only what the icon depends on. `type` widens with undefined for callers
// holding an open string (the calendar API) that did not narrow to an entity type.
type ResultEntryItem =
  | (Pick<LocationResultEntry, "kind"> & { type: LocationResultEntry["type"] | undefined })
  | Pick<LectureResultEntry, "kind">;
defineProps<{ item: ResultEntryItem }>();
</script>

<template>
  <div class="my-auto min-w-9 md:min-w-11">
    <div
      v-if="
        item.kind === 'location' &&
        (item.type === 'room' || item.type === 'virtual_room' || item.type === 'poi')
      "
      class="text-zinc-900 dark:text-zinc-50 p-2"
    >
      <MdiIcon :path="mdiMapMarker" :size="20" class="md:!w-6 md:!h-6" />
    </div>
    <div v-else class="text-white dark:text-black bg-blue-500 dark:bg-blue-400 rounded-full p-2">
      <MdiIcon :path="mdiSchool" :size="20" v-if="item.kind === 'lecture'" class="mx-auto md:!w-6 md:!h-6" />
      <MdiIcon
        :path="mdiOfficeBuildingOutline"
        :size="20"
        v-else-if="item.kind === 'location' && item.type === 'building'"
        class="mx-auto md:!w-6 md:!h-6"
      />
      <MdiIcon :path="mdiOfficeBuilding" :size="20" v-else class="mx-auto md:!w-6 md:!h-6" />
    </div>
  </div>
</template>
