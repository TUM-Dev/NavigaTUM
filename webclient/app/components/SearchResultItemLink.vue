<script setup lang="ts">
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
    <NuxtLinkLocale
      :class="{ 'bg-blue-200': highlighted }"
      :to="'/view/' + item.id"
      class="focusable"
      @click="() => emit('click')"
      @mousedown="() => emit('mousedown')"
      @mouseover="() => emit('mouseover')"
    >
      <SearchResultItem :item="item" />
    </NuxtLinkLocale>
    <!-- <div class="menu-badge"><label class="label label-primary">2</label></div> -->
  </li>
</template>
