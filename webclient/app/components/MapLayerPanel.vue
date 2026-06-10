<script setup lang="ts">
import { mdiChevronDown, mdiChevronUp, mdiLayersOutline } from "@mdi/js";
import type { LayerDef } from "~/composables/mapLayers";

const props = defineProps<{
  layers: readonly LayerDef[];
  // Reassigned wholesale by the parent so identity changes drive reactivity.
  enabled: Set<string>;
  collapsed: boolean;
  zoom: number;
}>();
const emit = defineEmits<{
  toggle: [id: string];
  "update:collapsed": [value: boolean];
}>();

const { t } = useI18n({ useScope: "local" });

function showHint(layer: LayerDef): boolean {
  return props.enabled.has(layer.id) && props.zoom < layer.hintBelowZoom;
}
</script>

<template>
  <section
    class="border-zinc-200 dark:border-zinc-700 bg-white/95 dark:bg-zinc-800/95 pointer-events-auto absolute z-10 flex flex-col overflow-hidden border shadow-lg backdrop-blur-sm bottom-0 inset-x-0 rounded-t-xl border-b-0 md:bottom-auto md:inset-x-auto md:left-4 md:top-4 md:w-72 md:rounded-xl md:border-b"
    :aria-label="t('panel_title')"
  >
    <button
      type="button"
      class="focusable text-zinc-700 dark:text-zinc-200 flex w-full items-center justify-between gap-2 px-4 py-3 font-semibold"
      :aria-expanded="!collapsed"
      @click="emit('update:collapsed', !collapsed)"
    >
      <span class="flex items-center gap-2">
        <MdiIcon :path="mdiLayersOutline" :size="20" />
        {{ t("panel_title") }}
      </span>
      <MdiIcon :path="collapsed ? mdiChevronUp : mdiChevronDown" :size="20" />
    </button>

    <ul v-show="!collapsed" class="flex flex-col gap-1 px-2 pb-3">
      <li v-for="layer in layers" :key="layer.id">
        <label
          class="hover:bg-zinc-100 dark:hover:bg-zinc-700/60 flex cursor-pointer items-center justify-between gap-3 rounded-lg px-2 py-2"
        >
          <span class="text-zinc-700 dark:text-zinc-200 flex items-center gap-2">
            <MdiIcon :path="layer.icon" :size="22" />
            {{ t(layer.labelKey) }}
          </span>
          <input
            type="checkbox"
            class="h-4 w-4 accent-blue-600 dark:accent-blue-400"
            :checked="enabled.has(layer.id)"
            :aria-label="t(layer.labelKey)"
            @change="emit('toggle', layer.id)"
          />
        </label>
        <p v-if="showHint(layer)" class="text-zinc-500 dark:text-zinc-400 px-2 pb-1 text-xs">
          {{ t("zoom_hint", { name: t(layer.labelKey) }) }}
        </p>
      </li>
    </ul>
  </section>
</template>

<i18n lang="yaml">
de:
  panel_title: Ebenen
  zoom_hint: Hineinzoomen, um {name} zu sehen
  layers:
    wcs: Toiletten & Duschen
en:
  panel_title: Layers
  zoom_hint: Zoom in to see {name}
  layers:
    wcs: Toilets & showers
</i18n>
