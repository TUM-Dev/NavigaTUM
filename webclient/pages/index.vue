<script setup lang="ts">
import type { components } from "~/api_types";
import { MapPinIcon } from "@heroicons/vue/24/outline";
import { ArrowRightIcon, ChevronDownIcon, ChevronRightIcon, ChevronUpIcon } from "@heroicons/vue/24/solid";

type RootResponse = components["schemas"]["RootResponse"];

const { t, locale } = useI18n({ useScope: "local" });
const runtimeConfig = useRuntimeConfig();
const url = computed<string>(() => `${runtimeConfig.public.apiURL}/api/get/root?lang=${locale.value}`);
const { data, pending } = await useFetch<RootResponse>(url, { pick: ["sites_overview"] });

const title = computed(() => t("sites") + " - NavigaTUM");
useSeoMeta({
  title: title,
  ogTitle: title,
  description: t("description"),
  ogDescription: t("description"),
  ogImage: "https://nav.tum.de/navigatum-card.png",
  twitterCard: "summary",
});

const openPanels = ref<(boolean | undefined)[]>([]);
</script>

<template>
  <div v-if="pending" class="text-zinc-900 flex flex-col items-center gap-5 py-32">
    <Spinner class="h-8 w-8" />
    {{ t("Loading data...") }}
  </div>
  <div class="flex flex-col justify-between gap-3 pt-8">
    <div class="text-zinc-600 !text-lg font-semibold">{{ t("sites") }}</div>
    <!-- <a href="#" class="flex flex-row"><MapPinIcon class="h-4 w-4" /> {{ t("overview_map") }}</a> -->
  </div>
  <div v-if="data" class="mt-5">
    <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
      <div
        v-for="(site, siteIndex) in data.sites_overview"
        :key="site.id"
        class="border-zinc-200 flex flex-col gap-4 rounded-lg border-2 p-5"
      >
        <div>
          <NuxtLink
            v-if="site.id"
            :to="'/view/' + site.id"
            :aria-label="t('show_details_for_campus', [site.name])"
            class="focusable text-zinc-700 flex grow-0 flex-row justify-between rounded !no-underline hover:text-tumBlue-500"
          >
            <span class="text-md font-semibold">{{ site.name }}</span>
            <ArrowRightIcon v-if="site.id" class="my-auto hidden h-6 w-6 md:block" />
          </NuxtLink>
          <div v-else class="text-md text-zinc-700 font-semibold">{{ site.name }}</div>
        </div>
        <div class="flex flex-col gap-3">
          <NuxtLink
            v-for="c in site.children.slice(0, openPanels[siteIndex] ? site.children.length : site.n_visible)"
            :key="c.id"
            :to="'/view/' + c.id"
            :aria-label="t('show_details_for_building', [c.name])"
            class="focusable text-tumBlue-600 flex flex-row justify-between rounded !no-underline hover:text-tumBlue-500"
          >
            <div class="flex flex-row gap-2">
              <MapPinIcon class="h-4 w-5" />
              <span>{{ c.name }}</span>
            </div>
            <ChevronRightIcon class="my-auto hidden h-4 w-4 sm:block" />
          </NuxtLink>
          <div v-if="site.children.length > site.n_visible" class="mx-auto">
            <Btn
              v-if="openPanels[siteIndex]"
              variant="linkButton"
              :aria-label="t('less_aria')"
              @click="() => (openPanels[siteIndex] = false)"
            >
              <ChevronUpIcon class="h-4 w-4" />
              {{ t("less") }}
            </Btn>
            <Btn v-else variant="linkButton" :aria-label="t('more_aria')" @click="() => (openPanels[siteIndex] = true)">
              <ChevronDownIcon class="my-auto h-4 w-4" />
              {{ t("more") }}
            </Btn>
          </div>
        </div>
      </div>
    </div>
  </div>
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
  description: Finde Räume, Gebäude und andere Orte an der TUM mit Exzellenz. Eine moderne Alternative zum RoomFinder, entwickelt von Studierenden.
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
  description: Find rooms, buildings and other places at TUM with excellence. A modern alternative to RoomFinder, developed by students.
</i18n>
