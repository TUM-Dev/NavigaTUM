<script setup lang="ts">
import type { components } from "~/api_types";
import SearchResultItem from "~/components/SearchResultItem.vue";

type SearchResponse = components["schemas"]["SearchResponse"];

defineProps<{
  data: SearchResponse;
  queryLimitBuildings: number;
  queryLimitRooms: number;
}>();
const route = useRoute();
const router = useRouter();
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <div v-for="s in data.sections" :key="s.facet">
    <section class="flex flex-col gap-2">
      <h2 class="text-md text-zinc-500 font-semibold">{{ t(`sections.${s.facet}`) }}</h2>
      <ul class="flex flex-col gap-3">
        <template v-for="(e, i) in s.entries" :key="e.id">
          <SearchResultItem
            v-if="i < s.n_visible"
            :highlighted="false"
            :item="e"
            @click="router.push(`/view/${e.id}`)"
          />
        </template>
      </ul>
      <p v-if="s.estimatedTotalHits > 10" class="text-zinc-500 text-sm">
        {{ t("approx_results", s.estimatedTotalHits) }}
        <NuxtLink
          :to="
            s.facet === 'rooms'
              ? `/search?q=${route.query.q}&limit_buildings=${queryLimitBuildings}&limit_rooms=${queryLimitRooms + 50}`
              : `/search?q=${route.query.q}&limit_buildings=${queryLimitBuildings + 20}&limit_rooms=${queryLimitRooms}`
          "
          class="focusable text-blue-500 rounded-sm visited:text-blue-500 hover:text-blue-600 hover:underline"
          >{{ t("view_more") }}
        </NuxtLink>
      </p>
      <p v-else class="text-zinc-500 text-sm">
        {{ t("results", s.estimatedTotalHits) }}
      </p>
    </section>
  </div>
</template>

<i18n lang="yaml">
de:
  sections:
    sites_buildings: Gebäude / Standorte
    rooms: Räume
  view_more: mehr anzeigen
  approx_results: ca. {count} Ergebnisse, bitte grenze die Suche weiter ein
  results: 1 Ergebnis | {count} Ergebnisse
en:
  sections:
    sites_buildings: Buildings / Sites
    rooms: Rooms
  view_more: view more
  approx_results: approx. {count} results, please narrow the search further
  results: 1 result | {count} results
</i18n>
