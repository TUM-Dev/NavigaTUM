<script setup lang="ts">
import LocationResultContent from "~/components/LocationResultContent.vue";
import { type EntityPath, entityPath, isRoutableEntityType } from "~/utils/entityPath";
import type { LocationResultEntry } from "~/utils/lectureRow";

const props = defineProps<{
  item: LocationResultEntry;
  highlighted: boolean;
}>();
const emit = defineEmits<{
  (e: "click"): void;
  (e: "mouseover"): void;
}>();

// Routable entities link to their canonical /{type}/{id} path. Non-routable results
// (e.g. Nominatim addresses, only surfaced on the navigate page) have no entity
// route and render as a plain, non-navigable row.
const to = computed<EntityPath | null>(() =>
  isRoutableEntityType(props.item.type) ? entityPath(props.item.id, props.item.type) : null
);
</script>

<template>
  <li
    class="bg-zinc-50 dark:bg-zinc-900 border-zinc-200 dark:border-zinc-700 rounded-sm border hover:bg-blue-100 dark:hover:bg-blue-800"
  >
    <NuxtLinkLocale
      v-if="to"
      :to="to"
      class="focusable"
      @click="() => emit('click')"
      @mouseover="() => emit('mouseover')"
    >
      <LocationResultContent :item="item" :highlighted="highlighted" />
    </NuxtLinkLocale>
    <LocationResultContent v-else :item="item" :highlighted="highlighted" />
  </li>
</template>
