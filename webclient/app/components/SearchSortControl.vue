<script setup lang="ts">
import { Popover, PopoverButton, PopoverPanel } from "@headlessui/vue";
import { mdiCheck, mdiSort, mdiSortClockAscending, mdiSortVariant } from "@mdi/js";
import type { SearchFilters } from "~/composables/searchFilters";

const props = defineProps<{
  filters: SearchFilters;
}>();

const { t } = useI18n({ useScope: "local" });

type SortBy = "relevance" | "distance";

const sortBy = computed<SortBy>(() => (props.filters.nearFilter.value ? "distance" : "relevance"));

function setSort(next: SortBy) {
  props.filters.setNear(next === "distance");
}
</script>

<template>
  <ClientOnly>
    <Popover class="relative">
      <PopoverButton
        class="focusable inline-flex items-center justify-center rounded-full border p-1.5 transition-colors"
        :class="
          sortBy === 'distance'
            ? 'bg-blue-100 border-blue-400 text-blue-800'
            : 'bg-zinc-100 border-zinc-300 text-zinc-600 hover:bg-zinc-200'
        "
        :title="t('sort_by') + ': ' + t(sortBy)"
        :aria-label="t('sort_by') + ': ' + t(sortBy)"
      >
        <MdiIcon :path="mdiSort" :size="18" />
      </PopoverButton>
      <Transition
        enter-active-class="transition duration-150 ease-out"
        enter-from-class="opacity-0 translate-y-1"
        enter-to-class="opacity-100 translate-y-0"
        leave-active-class="transition duration-100 ease-in"
        leave-from-class="opacity-100 translate-y-0"
        leave-to-class="opacity-0 translate-y-1"
      >
        <PopoverPanel
          class="ring-black/5 absolute right-0 z-20 mt-2 w-52 rounded-sm bg-white p-1 shadow-lg ring-1 dark:bg-zinc-100"
        >
          <div class="text-zinc-500 px-2 py-1 text-xs font-semibold uppercase tracking-wide">
            {{ t("sort_by") }}
          </div>
          <button
            v-for="opt in (['relevance', 'distance'] as const)"
            :key="opt"
            type="button"
            class="focusable flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-start text-sm hover:bg-zinc-100 dark:hover:bg-zinc-200"
            :class="sortBy === opt ? 'text-blue-800' : 'text-zinc-800'"
            @click="setSort(opt)"
          >
            <MdiIcon :path="opt === 'distance' ? mdiSortClockAscending : mdiSortVariant" :size="16" />
            <span class="flex-grow">{{ t(opt) }}</span>
            <MdiIcon v-if="sortBy === opt" :path="mdiCheck" :size="16" class="text-blue-600" />
          </button>
        </PopoverPanel>
      </Transition>
    </Popover>
  </ClientOnly>
</template>

<i18n lang="yaml">
de:
  sort_by: Sortieren
  relevance: Relevanz
  distance: Entfernung
en:
  sort_by: Sort by
  relevance: Relevance
  distance: Distance
</i18n>
