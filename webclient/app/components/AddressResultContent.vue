<script setup lang="ts">
import { mdiMapMarker } from "@mdi/js";
import type { AddressResultEntry } from "~/utils/lectureRow";

defineProps<{
  item: AddressResultEntry;
  highlighted: boolean;
}>();
</script>

<template>
  <div class="flex gap-1 px-2 py-3 md:gap-3 md:px-4" :class="{ 'bg-blue-200 dark:bg-blue-700': highlighted }">
    <div class="my-auto min-w-9 md:min-w-11">
      <div class="text-zinc-900 dark:text-zinc-50 p-2">
        <MdiIcon :path="mdiMapMarker" :size="20" class="md:!w-6 md:!h-6" />
      </div>
    </div>
    <div class="text-zinc-600 dark:text-zinc-300 flex flex-1 flex-col gap-0.5">
      <span class="line-clamp-1" v-html="item.name" />
      <small v-if="item.subtext">{{ item.subtext }}</small>
    </div>
  </div>
</template>
