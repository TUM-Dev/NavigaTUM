<script setup lang="ts">
import LocationResultContent from "~/components/LocationResultContent.vue";
import { type EntityPath, entityPath } from "~/utils/entityPath";
import type { LocationResultEntry } from "~/utils/lectureRow";

const props = defineProps<{
  item: LocationResultEntry;
  highlighted: boolean;
}>();
const emit = defineEmits<{
  (e: "click"): void;
  (e: "mouseover"): void;
}>();

// Every entity type is routable, so a location row always links to its
// canonical /{type}/{id} path; non-routable Nominatim addresses are a separate
// `kind` rendered by AddressResultRow.
const to = computed<EntityPath>(() => entityPath(props.item.id, props.item.type));
</script>

<template>
  <li
    class="bg-zinc-50 dark:bg-zinc-900 border-zinc-200 dark:border-zinc-700 rounded-sm border hover:bg-blue-100 dark:hover:bg-blue-800"
  >
    <NuxtLinkLocale
      :to="to"
      class="focusable"
      @click="() => emit('click')"
      @mouseover="() => emit('mouseover')"
    >
      <LocationResultContent :item="item" :highlighted="highlighted" />
    </NuxtLinkLocale>
  </li>
</template>
