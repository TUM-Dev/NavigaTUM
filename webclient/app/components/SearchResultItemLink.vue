<script setup lang="ts">
import type { components } from "~/api_types/index.js";

type ResultEntry = components["schemas"]["ResultEntry"];

defineProps<{
  item: ResultEntry;
  highlighted: boolean;
}>();
const emit = defineEmits(["click", "mousedown", "mouseover"]);
</script>

<template>
  <li class="bg-zinc-50 dark:bg-zinc-900 border-zinc-200 dark:border-zinc-700 rounded-sm border hover:bg-blue-100 dark:hover:bg-blue-800">
    <NuxtLinkLocale
      :to="'/view/' + item.id"
      class="focusable"
      @click="() => emit('click')"
      @mousedown="() => emit('mousedown')"
      @mouseover="() => emit('mouseover')"
    >
      <SearchResultItem :item="item" :highlighted="highlighted" />
    </NuxtLinkLocale>
    <!-- <div class="menu-badge"><label class="label label-primary">2</label></div> -->
  </li>
</template>
