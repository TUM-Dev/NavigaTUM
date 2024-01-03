<script setup lang="ts">
import { setTitle } from "@/composables/common";
import { useFetch } from "@/composables/fetch";
import type { components } from "@/api_types";
import { useI18n } from "vue-i18n";
import { MapPinIcon } from "@heroicons/vue/24/outline";
import { ArrowRightIcon, ChevronRightIcon, ChevronDownIcon, ChevronUpIcon } from "@heroicons/vue/24/solid";
import { ref } from "vue";
type RootResponse = components["schemas"]["RootResponse"];

const { t } = useI18n({ useScope: "local" });
const { data } = useFetch<RootResponse>(`/api/get/root`, (d) => setTitle(d.name));
const openPanels = ref<(boolean | undefined)[]>([]);
</script>

<template>
  <div class="flex flex-row justify-between pt-14">
    <div class="!text-xl font-semibold text-slate-600">{{ t("sites") }}</div>
    <!-- <a href="#" class="flex flex-row"><MapPinIcon class="h-4 w-4" /> {{ t("overview_map") }}</a> -->
  </div>
  <div v-if="data" class="mt-5">
    <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
      <div
        v-for="(site, siteIndex) in data.sites_overview"
        :key="site.id"
        class="flex flex-col gap-4 rounded-xl border-2 p-8"
      >
        <div>
          <RouterLink
            v-if="site.id"
            :to="'/view/' + site.id"
            :aria-label="t('show_details_for_campus', [site.name])"
            class="flex grow-0 flex-row justify-between !no-underline"
          >
            <span class="text-xl font-semibold text-slate-700 hover:text-tumBlue-500">{{ site.name }}</span>
            <ArrowRightIcon v-if="site.id" class="my-auto hidden h-6 w-6 md:block" />
          </RouterLink>
          <div v-else class="text-xl font-semibold">{{ site.name }}</div>
        </div>
        <div class="flex flex-col gap-3">
          <RouterLink
            v-for="c in site.children.slice(0, openPanels[siteIndex] ? site.children.length : site.n_visible)"
            :key="c.id"
            :to="'/view/' + c.id"
            :aria-label="t('show_details_for_building', [c.name])"
            class="flex flex-row justify-between text-tumBlue-600 !no-underline hover:text-tumBlue-500"
          >
            <div class="flex flex-row gap-3">
              <MapPinIcon class="my-auto h-4 w-4" />
              <span class="text-lg">{{ c.name }}</span>
            </div>
            <ChevronRightIcon class="hidden h-5 w-5 sm:block" />
          </RouterLink>
          <div v-if="site.children.length > site.n_visible" class="mt-2">
            <button
              v-if="openPanels[siteIndex]"
              type="button"
              :aria-label="t('less_aria')"
              class="flex flex-row gap-2"
              @click="() => (openPanels[siteIndex] = false)"
            >
              <ChevronUpIcon class="h-4 w-4" />
              {{ t("less") }}
            </button>
            <button
              v-else
              type="button"
              class="flex flex-row gap-2"
              :aria-label="t('more_aria')"
              @click="() => (openPanels[siteIndex] = true)"
            >
              <ChevronDownIcon class="my-auto h-4 w-4" />
              {{ t("more") }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
  <div v-else>{{ t("Loading data...") }}</div>
</template>

<i18n lang="yaml">
de:
  less: weniger
  less_aria: weniger Gebäude anzeigen
  more: mehr
  more_aria: mehr Gebäude anzeigen
  overview_map: Übersichtskarte
  sites: Standorte
  "Loading data...": Lädt daten...
  show_details_for_campus: show the details for the campus '{0}'
  show_details_for_building: show the details for the building '{0}'
en:
  less: less
  less_aria: show more buildings
  more: more
  more_aria: show more buildings
  overview_map: Overview Map
  sites: Sites
  "Loading data...": Loading data...
  show_details_for_campus: show the details for the campus '{0}'
  show_details_for_building: show the details for the building '{0}'
</i18n>
