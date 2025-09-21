<script setup lang="ts">
import { mdiRefresh } from "@mdi/js";

interface Props {
  loading?: boolean;
  error?: string | null;
  hasResults?: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  retry: [];
}>();

const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <div>
    <!-- Loading State -->
    <div v-if="loading" class="py-12 text-center">
      <div class="inline-flex items-center justify-center">
        <Spinner class="h-8 w-8 text-blue-600" />
      </div>
      <p class="text-zinc-600 mt-4 text-lg">{{ t("loading_routes") }}</p>
      <p class="text-zinc-500 mt-2 text-sm">{{ t("searching_connections") }}</p>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="py-12 text-center">
      <div class="text-red-500 mb-4">
        <svg class="mx-auto h-16 w-16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="1.5"
            d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z"
          />
        </svg>
      </div>
      <h3 class="text-zinc-900 mb-2 text-lg font-semibold">{{ t("error_title") }}</h3>
      <p class="text-zinc-600 mb-6 text-sm max-w-md mx-auto">{{ error }}</p>
      <button
        @click="emit('retry')"
        class="inline-flex items-center px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-lg hover:bg-blue-700 transition-colors"
      >
        <MdiIcon :path="mdiRefresh" :size="16" class="mr-2" />
        {{ t("retry_search") }}
      </button>
    </div>

    <!-- No results -->
    <div v-else-if="!hasResults" class="py-12 text-center">
      <div class="text-zinc-400 mb-4">
        <svg class="mx-auto h-16 w-16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="1.5"
            d="M9.75 9.75l4.5 4.5m0-4.5l-4.5 4.5M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
      </div>
      <h3 class="text-zinc-700 mb-2 text-lg font-semibold">{{ t("no_routes_found") }}</h3>
      <p class="text-zinc-500 mb-6 text-sm max-w-md mx-auto">{{ t("no_routes_description") }}</p>
      <button
        @click="emit('retry')"
        class="inline-flex items-center px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-lg hover:bg-blue-700 transition-colors"
      >
        <MdiIcon :path="mdiRefresh" :size="16" class="mr-2" />
        {{ t("try_again") }}
      </button>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  loading_routes: Routen werden geladen
  searching_connections: Suche nach Verbindungen...
  error_title: Fehler beim Laden
  retry_search: Erneut versuchen
  no_routes_found: Keine Routen gefunden
  no_routes_description: Es konnten keine Verbindungen für Ihre Suchanfrage gefunden werden. Versuche andere Parameter oder eine andere Zeit.
  try_again: Erneut versuchen

en:
  loading_routes: Loading routes
  searching_connections: Searching for connections...
  error_title: Error loading routes
  retry_search: Try again
  no_routes_found: No routes found
  no_routes_description: No connections could be found for your search query. Try different parameters or a different time.
  try_again: Try again
</i18n>
