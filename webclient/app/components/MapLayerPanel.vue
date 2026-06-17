<script setup lang="ts">
import { mdiChevronDown, mdiChevronUp, mdiFilterVariant } from "@mdi/js";
import type { FilterDef } from "~/composables/mapLayers";

const props = defineProps<{
  filters: readonly FilterDef[];
  // Reassigned wholesale by the parent so identity changes drive reactivity.
  active: Set<string>;
  collapsed: boolean;
  zoom: number;
}>();
const emit = defineEmits<{
  toggle: [id: string];
  "update:collapsed": [value: boolean];
}>();

const { t } = useI18n({ useScope: "local" });

function showHint(filter: FilterDef): boolean {
  return props.active.has(filter.id) && props.zoom < filter.hintBelowZoom;
}
</script>

<template>
  <section
    class="border-zinc-200 dark:border-zinc-700 bg-white/95 dark:bg-zinc-800/95 pointer-events-auto absolute z-10 flex flex-col overflow-hidden border shadow-lg backdrop-blur-sm bottom-0 inset-x-0 rounded-t-md border-b-0 md:bottom-auto md:inset-x-auto md:left-4 md:top-4 md:w-72 md:rounded-md md:border-b"
    :aria-label="t('panel_title')"
  >
    <button
      type="button"
      class="focusable text-zinc-700 dark:text-zinc-200 flex w-full items-center justify-between gap-2 px-4 py-3 font-semibold"
      :aria-expanded="!collapsed"
      @click="emit('update:collapsed', !collapsed)"
    >
      <span class="flex items-center gap-2">
        <MdiIcon :path="mdiFilterVariant" :size="20" />
        {{ t("panel_title") }}
      </span>
      <MdiIcon :path="collapsed ? mdiChevronUp : mdiChevronDown" :size="20" />
    </button>

    <ul v-show="!collapsed" class="flex flex-col gap-1 px-2 pb-3">
      <li v-for="filter in filters" :key="filter.id">
        <label
          class="hover:bg-zinc-100 dark:hover:bg-zinc-700/60 flex cursor-pointer items-center justify-between gap-3 rounded-md px-2 py-2"
        >
          <span class="text-zinc-700 dark:text-zinc-200 flex items-center gap-2">
            <MdiIcon :path="filter.icon" :size="22" />
            {{ t(filter.labelKey) }}
          </span>
          <input
            type="checkbox"
            class="h-4 w-4 accent-blue-600 dark:accent-blue-400"
            :checked="active.has(filter.id)"
            :aria-label="t(filter.labelKey)"
            @change="emit('toggle', filter.id)"
          />
        </label>
        <p v-if="showHint(filter)" class="text-zinc-500 dark:text-zinc-400 px-2 pb-1 text-xs">
          {{ t("zoom_hint", { name: t(filter.labelKey) }) }}
        </p>
        <!-- Per-filter sub-options (e.g. the events time window), owned by the page. -->
        <slot v-if="active.has(filter.id)" :name="`filter-${filter.id}`" />
      </li>
    </ul>
  </section>
</template>

<i18n lang="yaml">
de:
  panel_title: Filter
  zoom_hint: Hineinzoomen, um {name} zu sehen
  filters:
    wcs: Toiletten & Duschen
    events: Veranstaltungen
    card_validators: Validierungsautomaten
en:
  panel_title: Filter
  zoom_hint: Zoom in to see {name}
  filters:
    wcs: Toilets & showers
    events: Events
    card_validators: Card validators
</i18n>
