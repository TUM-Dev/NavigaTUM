<script setup lang="ts">
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <div>
    <div class="py-12 text-center">
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
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  no_routes_found: Keine Routen gefunden
  no_routes_description: Es konnten keine Verbindungen für Ihre Suchanfrage gefunden werden. Versuche andere Parameter oder eine andere Zeit.
en:
  no_routes_found: No routes found
  no_routes_description: No connections could be found for your search query. Try different parameters or a different time.
</i18n>
