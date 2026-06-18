<script setup lang="ts">
import { mdiArrowRightThin, mdiMapSearchOutline } from "@mdi/js";
import { FILTER_QUERY_PARAM, type FilterId } from "~/composables/mapLayers";

// Category shortcut (see `webclient/CONTEXT.md`): a full-width row above all search sections,
// visually distinct from typed results, landing on the Browse map with the Category pre-applied.
defineProps<{
  readonly category: FilterId;
  readonly highlighted?: boolean;
}>();
defineEmits<{
  (e: "click"): void;
  (e: "mouseover"): void;
}>();
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <li class="flex list-none">
    <NuxtLinkLocale
      :to="`/map?${FILTER_QUERY_PARAM}=${category}`"
      class="focusable border-blue-200 dark:border-blue-800 flex w-full items-center gap-3 rounded-sm border px-3 py-2.5"
      :class="highlighted ? 'bg-blue-200 dark:bg-blue-700' : 'bg-blue-50 dark:bg-blue-900/30 hover:bg-blue-100 dark:hover:bg-blue-800/40'"
      @click="$emit('click')"
      @mouseover="$emit('mouseover')"
    >
      <MdiIcon :path="mdiMapSearchOutline" :size="20" class="text-blue-600 dark:text-blue-300 shrink-0" aria-hidden="true" />
      <span class="text-zinc-800 dark:text-zinc-100 flex-grow text-sm font-medium">{{ t(`explore.${category}`) }}</span>
      <MdiIcon :path="mdiArrowRightThin" :size="20" class="text-blue-600 dark:text-blue-300 shrink-0" aria-hidden="true" />
    </NuxtLinkLocale>
  </li>
</template>

<i18n lang="yaml">
de:
  explore:
    wcs: Toiletten & Duschen auf der Karte erkunden
    events: Veranstaltungen auf der Karte erkunden
    card_validator: Validierungsautomaten auf der Karte erkunden
en:
  explore:
    wcs: Explore toilets & showers on the map
    events: Explore events on the map
    card_validator: Explore card validators on the map
</i18n>
