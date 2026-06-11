<script setup lang="ts">
import { mdiOpenInNew } from "@mdi/js";
import type { components } from "~/api_types";
import { clientOnlyRetries } from "~/composables/common";

definePageMeta({
  layout: "embed",
});

type LocationDetailsResponse = components["schemas"]["LocationDetailsResponse"];

const { t, locale } = useI18n({ useScope: "local" });
const localePath = useLocalePath();
const route = useRoute();
const runtimeConfig = useRuntimeConfig();

const url = computed(
  () => `${runtimeConfig.public.apiURL}/api/locations/${route.params.id}?lang=${locale.value}`
);
const { data, error } = await useFetch<LocationDetailsResponse, string>(url, {
  dedupe: "cancel",
  credentials: "omit",
  retry: clientOnlyRetries(120),
  retryDelay: 1000,
});

if (error.value) {
  showError({
    statusCode: 404,
    statusMessage: "Location not found",
  });
}

const detailsUrl = computed(() => {
  if (!data.value) return "https://nav.tum.de";
  // Prefer the canonical URL the API gives us (e.g. /building/mi); fall back to
  // type/id when no redirect_url is set. data.value.type can be a non-routable
  // value like "joined_building", so we never construct from it directly.
  const path =
    (data.value.redirect_url as string | undefined) || `/${data.value.type}/${data.value.id}`;
  return `https://nav.tum.de${localePath(path)}`;
});

const title = computed(() => data.value?.name || `${route.params.id} - Navigatum`);
useSeoMeta({
  title: title,
  ogTitle: title,
  robots: "noindex, nofollow",
});
</script>

<template>
  <div class="flex h-full w-full flex-col">
    <div v-if="data" class="embed-map-wrapper relative min-h-0 flex-1">
      <ClientOnly>
        <DetailsInteractiveMap
          :id="data.id"
          :coords="data.coords"
          :type="data.type"
          :maps="data.maps"
          :floors="data.props.floors"
          class="h-full w-full"
        />
      </ClientOnly>
    </div>
    <div v-else class="flex flex-1 items-center justify-center">
      <Spinner class="h-8 w-8" />
    </div>
    <div class="flex shrink-0 items-center gap-3 border-t border-zinc-200 dark:border-zinc-700 bg-white dark:bg-black px-3 py-2">
      <div class="min-w-0 flex-1">
        <div class="truncate text-sm font-semibold text-zinc-800 dark:text-zinc-100">{{ data?.name }}</div>
        <div v-if="data?.type_common_name" class="truncate text-xs text-zinc-500 dark:text-zinc-400">
          {{ data.type_common_name }}
        </div>
      </div>
      <a
        :href="detailsUrl"
        target="_blank"
        rel="noopener"
        class="focusable flex shrink-0 flex-row items-center gap-1 rounded-sm bg-blue-500 dark:bg-blue-400 px-3 py-1.5 text-sm font-semibold text-blue-50 dark:text-blue-900 hover:bg-blue-600 dark:hover:bg-blue-300 hover:text-white dark:hover:text-black"
      >
        {{ t("view_in_navigatum") }}
        <MdiIcon :path="mdiOpenInNew" :size="16" class="my-auto" />
      </a>
    </div>
  </div>
</template>

<style scoped>
.embed-map-wrapper :deep(#interactive-legacy-map-container) {
  margin-bottom: 0 !important;
  height: 100% !important;
  width: 100% !important;
  aspect-ratio: auto !important;
}

.embed-map-wrapper :deep(#interactive-legacy-map-container > div) {
  padding-bottom: 0 !important;
  height: 100% !important;
}
</style>

<i18n lang="yaml">
de:
  view_in_navigatum: In NavigaTUM ansehen
en:
  view_in_navigatum: View details in NavigaTUM
</i18n>
