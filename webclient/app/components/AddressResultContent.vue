<script setup lang="ts">
import PreviewIcon from "~/components/PreviewIcon.vue";
import type { AddressResultEntry } from "~/utils/lectureRow";

defineProps<{
  item: AddressResultEntry;
  highlighted: boolean;
}>();
</script>

<template>
  <div class="flex gap-1 px-2 py-3 md:gap-3 md:px-4" :class="{ 'bg-blue-200 dark:bg-blue-700': highlighted }">
    <PreviewIcon :item="{ kind: 'address' }" />
    <div class="text-zinc-600 dark:text-zinc-300 flex flex-1 flex-col gap-0.5">
      <span class="line-clamp-1" v-html="item.name" />
      <small v-if="item.subtext">{{ item.subtext }}</small>
    </div>
  </div>
</template>
