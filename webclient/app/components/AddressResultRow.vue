<script setup lang="ts">
import AddressResultContent from "~/components/AddressResultContent.vue";
import type { AddressResultEntry } from "~/utils/lectureRow";

defineProps<{
  item: AddressResultEntry;
  highlighted: boolean;
}>();
const emit = defineEmits<{
  (e: "click"): void;
  (e: "mouseover"): void;
}>();
</script>

<template>
  <li
    class="bg-zinc-50 dark:bg-zinc-900 border-zinc-200 dark:border-zinc-700 rounded-sm border hover:bg-blue-100 dark:hover:bg-blue-800"
    @click="() => emit('click')"
    @mouseover="() => emit('mouseover')"
  >
    <AddressResultContent :item="item" :highlighted="highlighted" />
  </li>
</template>
