<script setup lang="ts">
import type { components } from "~/api_types";

type DetailsResponse = components["schemas"]["DetailsResponse"];
const runtimeConfig = useRuntimeConfig();
const url = computed(() => `${runtimeConfig.public.apiURL}/api/locations/5532.EG.002?lang=de`);
const { data } = useFetch<DetailsResponse, string>(url, {
  key: "details",
  dedupe: "cancel",
  credentials: "omit",
  retry: 120,
  retryDelay: 1000,
});
</script>

<template>
  <Suspense>
    <div v-if="data">
      <IndoorMap :type="data.type" :coords="data.coords" />
    </div>
    <Spinner v-else class="h-8 w-8" />
    <template #fallback>
      <Spinner class="h-8 w-8" />
    </template>
  </Suspense>
</template>
