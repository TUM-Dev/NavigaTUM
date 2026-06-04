<script setup lang="ts">
import { mdiChevronLeft, mdiChevronRight } from "@mdi/js";

interface Props {
  previousPageCursor?: string | null;
  nextPageCursor?: string | null;
  size?: "sm" | "lg";
}

const props = withDefaults(defineProps<Props>(), {
  size: "sm",
});
const pageCursor = defineModel<string | undefined>("pageCursor", {
  required: true,
});

const { t } = useI18n({ useScope: "local" });

const showPagination = computed(() => {
  return props.previousPageCursor || props.nextPageCursor;
});

const handlePreviousPage = () => {
  if (props.previousPageCursor) {
    pageCursor.value = props.previousPageCursor;
  }
};

const handleNextPage = () => {
  if (props.nextPageCursor) {
    pageCursor.value = props.nextPageCursor;
  }
};
</script>

<template>
  <div v-if="showPagination">
    <!-- Small pagination controls (for top of list) -->
    <div v-if="size === 'sm'" class="flex items-center gap-2">
      <button
        :disabled="!previousPageCursor"
        @click="handlePreviousPage"
        class="inline-flex items-center px-2 py-1 text-xs font-medium text-zinc-600 dark:text-zinc-300 bg-zinc-100 dark:bg-zinc-800 rounded hover:bg-zinc-200 dark:hover:bg-zinc-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        <MdiIcon :path="mdiChevronLeft" :size="14" class="mr-1" />
        {{ t("earlier") }}
      </button>
      <button
        :disabled="!nextPageCursor"
        @click="handleNextPage"
        class="inline-flex items-center px-2 py-1 text-xs font-medium text-zinc-600 dark:text-zinc-300 bg-zinc-100 dark:bg-zinc-800 rounded hover:bg-zinc-200 dark:hover:bg-zinc-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        {{ t("later") }}
        <MdiIcon :path="mdiChevronRight" :size="14" class="ml-1" />
      </button>
    </div>

    <!-- Large pagination controls (for bottom of list) -->
    <div v-else class="flex justify-center gap-4 mt-6 pt-4 border-t border-zinc-200 dark:border-zinc-700">
      <button
        :disabled="!previousPageCursor"
        @click="handlePreviousPage"
        class="inline-flex items-center px-4 py-2 text-sm font-medium text-zinc-700 dark:text-zinc-200 bg-white dark:bg-black border border-zinc-300 dark:border-zinc-600 rounded-lg hover:bg-zinc-50 dark:hover:bg-zinc-900 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        <MdiIcon :path="mdiChevronLeft" :size="16" class="mr-2" />
        {{ t("load_earlier_connections") }}
      </button>
      <button
        :disabled="!nextPageCursor"
        @click="handleNextPage"
        class="inline-flex items-center px-4 py-2 text-sm font-medium text-zinc-700 dark:text-zinc-200 bg-white dark:bg-black border border-zinc-300 dark:border-zinc-600 rounded-lg hover:bg-zinc-50 dark:hover:bg-zinc-900 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        {{ t("load_later_connections") }}
        <MdiIcon :path="mdiChevronRight" :size="16" class="ml-2" />
      </button>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  earlier: Früher
  later: Später
  load_earlier_connections: Frühere Verbindungen
  load_later_connections: Spätere Verbindungen

en:
  earlier: Earlier
  later: Later
  load_earlier_connections: Earlier connections
  load_later_connections: Later connections
</i18n>
