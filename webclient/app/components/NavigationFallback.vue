<script setup lang="ts">
import type { components, operations } from "~/api_types";
import { googleMapsDirectionsUrl, googleMapsEndpoint } from "~/utils/navigationFallback";

const props = defineProps<{
  from: string;
  to: string;
  fromLabel: string;
  toLabel: string;
  mode: operations["route_handler"]["parameters"]["query"]["route_costing"];
}>();
const { t, locale } = useI18n({ useScope: "local" });
const runtimeConfig = useRuntimeConfig();
const { data: googleMapsUrl } = useAsyncData(
  computed(() => `navigation-fallback:${JSON.stringify([props, locale.value])}`),
  async () => {
    const loadCoords = async (id: string) => {
      const details = await $fetch<components["schemas"]["LocationDetailsResponse"]>(
        `${runtimeConfig.public.apiURL}/api/locations/${encodeURIComponent(id)}`,
        { query: { lang: locale.value }, credentials: "omit" }
      );
      return details.coords;
    };
    const [from, to] = await Promise.all([
      googleMapsEndpoint(props.from, props.fromLabel, loadCoords),
      googleMapsEndpoint(props.to, props.toLabel, loadCoords),
    ]);
    return googleMapsDirectionsUrl(from, to, props.mode) ?? null;
  }
);
</script>

<template>
  <MotisNoRoutesFound>
    <template v-if="googleMapsUrl">
      <Btn :to="googleMapsUrl" class="justify-center">
        {{ t("google_maps") }}
      </Btn>
      <p class="mt-3 text-sm text-zinc-500 dark:text-zinc-400">{{ t("building_only") }}</p>
    </template>
  </MotisNoRoutesFound>
</template>

<i18n lang="yaml">
de:
  google_maps: Route in Google Maps öffnen
  building_only: Google Maps navigiert zum Gebäude, nicht zum Raum. Prüfe dort den Startpunkt.
en:
  google_maps: Open route in Google Maps
  building_only: Google Maps navigates to the building, not the room. Check the starting point there.
</i18n>
