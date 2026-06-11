<script setup lang="ts">
import {
  mdiMagnify,
  mdiMapMarker,
  mdiOfficeBuilding,
  mdiOfficeBuildingOutline,
  mdiSchool,
} from "@mdi/js";
import type { components } from "~/api_types/index.js";

type LocationEntry = components["schemas"]["LocationEntry"];

// Discriminated on `kind`; within a location the closed entity `type` selects
// the building/room/POI icon. Callers holding an open type string (e.g. the
// calendar API) narrow it first and pass `type: undefined` for the generic icon.
type ResultEntryItem =
  | {
      kind: "location";
      type: LocationEntry["type"] | undefined;
      parsed_id?: LocationEntry["parsed_id"];
    }
  | { kind: "address" }
  | { kind: "lecture" };
defineProps<{ item: ResultEntryItem }>();
</script>

<template>
  <div class="my-auto min-w-9 md:min-w-11">
    <div
      v-if="
        (item.kind === 'location' &&
          (item.type === 'room' || item.type === 'virtual_room' || item.type === 'poi')) ||
        item.kind === 'address'
      "
      class="text-zinc-900 dark:text-zinc-50 p-2"
    >
      <MdiIcon
        :path="mdiMagnify"
        :size="20"
        v-if="item.kind === 'location' && item.parsed_id"
        class="md:!w-6 md:!h-6"
      />
      <MdiIcon :path="mdiMapMarker" :size="20" v-else class="md:!w-6 md:!h-6" />
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
