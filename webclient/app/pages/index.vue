<script setup lang="ts">
import {
  mdiArrowRight,
  mdiChevronDown,
  mdiChevronRight,
  mdiChevronUp,
  mdiMapMarker,
  mdiMapSearchOutline,
} from "@mdi/js";
import { entityPath, type RoutableEntityType } from "~/utils/entityPath";

/** A linkable card: its `type` + `id` resolve to a canonical `/{type}/{id}` path via {@link entityPath}. */
interface OverviewLink {
  readonly id: string;
  readonly name: string;
  readonly type: RoutableEntityType;
}

/**
 * @description This is a list of all sites, that are available in the system.
 * It is sorted by the number of rooms in the site, descending.
 * The first entry is the site with the most importance
 */
interface SitesOverview {
  readonly id: string;
  readonly name: string;
  /** Absent only for the synthetic "others" grouping, which has no entity page of its own. */
  readonly type?: RoutableEntityType;
  /**
   * @description A recommendation how many of the entries should be displayed by default.
   * The number is usually from 0-5.
   * More results might be displayed when clicking "expand".
   */
  readonly n_visible: number;
  /**
   * @description A select list of buildings, that are in this site.
   * Derived from the areatree.
   */
  readonly children: readonly OverviewLink[];
}

const { t } = useI18n({ useScope: "local" });
const localePath = useLocalePath();
// Types mirror the server's canonical `redirect_url` mapping so the helper emits
// the final `/{type}/{id}` directly (no `/view/{id}` redirect round-trip). The
// three StudiTUM/RiWa ids use their canonical building ids (`studitum-garching`,
// `s1`, `riwa1`) rather than the legacy aliases `5532`/`0201`/`2910`, which would
// 301-redirect to exactly these on a direct hit.
const sites_overview: readonly SitesOverview[] = [
  {
    id: "garching",
    name: "Garching Forschungszentrum",
    type: "campus",
    n_visible: 4,
    children: [
      { id: "mi", name: "Mathematik / Informatik", type: "joined_building" },
      { id: "mw", name: "Maschinenwesen", type: "joined_building" },
      { id: "physik", name: "Physik", type: "area" },
      { id: "chemie", name: "Chemie", type: "joined_building" },
      { id: "garching-interims", name: "Interimshörsäle", type: "area" },
      { id: "studitum-garching", name: "StudiTUM Garching", type: "building" },
    ],
  },
  {
    id: "stammgelaende",
    name: "Stammgelände",
    type: "campus",
    n_visible: 3,
    children: [
      { id: "zentralgelaende", name: "Zentralgelände", type: "area" },
      { id: "nordgelaende", name: "Nordgelände", type: "area" },
      { id: "suedgelaende", name: "Südgelände", type: "area" },
      { id: "suedwestgelaende", name: "Südwestgelände", type: "area" },
      { id: "s1", name: "StudiTUM Innenstadt", type: "building" },
      { id: "riwa1", name: "RiWa 1 (HfP, Governance)", type: "building" },
    ],
  },
  {
    id: "wzw",
    name: "Weihenstephan (Freising)",
    type: "campus",
    n_visible: 3,
    children: [
      { id: "wzw-berg", name: "Gebiet 4100 Berg", type: "area" },
      { id: "wzw-mitte", name: "Gebiet 4200 Mitte", type: "area" },
      { id: "wzw-nord", name: "Gebiet 4300 Nord", type: "area" },
      { id: "duernast", name: "Dürnast (Versuchsstation)", type: "area" },
      { id: "roggenstein", name: "Roggenstein (Versuchsstation)", type: "site" },
      { id: "thalhausen", name: "Thalhausen (Versuchsstation)", type: "site" },
      { id: "veitshof", name: "Veitshof (Stallungen)", type: "site" },
      { id: "viehhausen", name: "Viehhausen (Versuchsstation)", type: "site" },
    ],
  },
  {
    id: "others",
    name: t("sites_overview.others"),
    n_visible: 5,
    children: [
      { id: "mri", name: "MRI Klinikum rechts der Isar", type: "site" },
      { id: "olympiapark", name: "Campus im Olympiapark", type: "campus" },
      { id: "cs", name: "Campus Straubing", type: "campus" },
      { id: "heilbronn", name: "Heilbronn", type: "campus" },
      { id: "taufkirchen-ottobrunn", name: "Taufkirchen / Ottobrunn", type: "campus" },
      { id: "garching-hochbrueck", name: "Garching Hochbrück", type: "site" },
    ],
  },
];

const title = computed(() => `${t("sites")} - NavigaTUM`);
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
  <div class="flex flex-col justify-between gap-3 pt-6">
    <AppToasts />
    <h1 class="text-zinc-600 dark:text-zinc-300 !text-lg font-semibold">{{ t("sites") }}</h1>
    <!-- <NuxtLink :to="localePath('#')" class="flex flex-row"><MdiIcon :path="mdiMapMarker" :size="16" /> {{ t("overview_map") }}</NuxtLink> -->
  </div>
  <div class="mt-5">
    <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
      <div
        v-for="(site, siteIndex) in sites_overview"
        :key="site.id"
        class="border-zinc-200 dark:border-zinc-700 flex flex-col gap-4 rounded-lg border-2 p-5"
      >
        <div>
          <NuxtLink
            v-if="site.type"
            :to="localePath(entityPath(site.id, site.type))"
            :aria-label="t('show_details_for_campus', [site.name])"
            class="focusable text-zinc-700 dark:text-zinc-200 flex grow-0 flex-row justify-between rounded !no-underline hover:text-blue-500 dark:hover:text-blue-400"
          >
            <h2 class="text-md font-semibold">{{ site.name }}</h2>
            <MdiIcon :path="mdiArrowRight" :size="24" v-if="site.id" class="my-auto hidden h-6 w-6 md:block" />
          </NuxtLink>
          <h2 v-else class="text-md text-zinc-700 dark:text-zinc-200 font-semibold">{{ site.name }}</h2>
        </div>
        <div class="flex flex-col gap-3">
          <NuxtLink
            v-for="c in site.children.slice(0, openPanels[siteIndex] ? site.children.length : site.n_visible)"
            :key="c.id"
            :to="localePath(entityPath(c.id, c.type))"
            :aria-label="t('show_details_for_building', [c.name])"
            class="focusable text-blue-600 dark:text-blue-300 flex flex-row justify-between rounded !no-underline hover:text-blue-500 dark:hover:text-blue-400"
          >
            <div class="flex flex-row gap-2">
              <MdiIcon :path="mdiMapMarker" :size="24" class="my-auto" />
              <span>{{ c.name }}</span>
            </div>
            <MdiIcon :path="mdiChevronRight" :size="16" class="my-auto hidden sm:block" />
          </NuxtLink>
          <div v-if="site.children.length > site.n_visible" class="mx-auto">
            <Btn
              v-if="openPanels[siteIndex]"
              variant="linkButton"
              :aria-label="t('less_aria')"
              @click="() => (openPanels[siteIndex] = false)"
            >
              <MdiIcon :path="mdiChevronUp" :size="16" />
              {{ t("less") }}
            </Btn>
            <Btn v-else variant="linkButton" :aria-label="t('more_aria')" @click="() => (openPanels[siteIndex] = true)">
              <MdiIcon :path="mdiChevronDown" :size="16" class="my-auto" />
              {{ t("more") }}
            </Btn>
          </div>
        </div>
      </div>
    </div>
  </div>
  <NuxtLink
    :to="localePath('/map')"
    :aria-label="t('explore_map_aria')"
    class="focusable border-zinc-200 dark:border-zinc-700 text-zinc-700 dark:text-zinc-200 mt-4 flex flex-row items-center justify-between gap-4 rounded-lg border-2 p-5 !no-underline hover:text-blue-500 dark:hover:text-blue-400"
  >
    <div class="flex flex-row items-center gap-3">
      <MdiIcon :path="mdiMapSearchOutline" :size="28" class="shrink-0" />
      <div class="flex flex-col">
        <span class="text-md font-semibold">{{ t("explore_map") }}</span>
        <span class="text-zinc-500 dark:text-zinc-400 text-sm">{{ t("explore_map_subtitle") }}</span>
      </div>
    </div>
    <MdiIcon :path="mdiArrowRight" :size="24" class="my-auto hidden h-6 w-6 shrink-0 md:block" />
  </NuxtLink>
</template>

<i18n lang="yaml">
de:
  less: weniger
  less_aria: weniger Gebäude anzeigen
  more: mehr
  more_aria: mehr Gebäude anzeigen
  overview_map: Übersichtskarte
  sites: Standorte
  show_details_for_campus: Details für den Campus '{0}' anzeigen
  show_details_for_building: Details für das Gebäude '{0}' anzeigen
  description: Finde Räume, Gebäude und andere Orte an der TUM mit Exzellenz. Eine moderne Alternative zum RoomFinder, entwickelt von Studierenden.
  explore_map: Karte erkunden
  explore_map_subtitle: Stöbere auf der Karte und blende Ebenen wie Toiletten und Duschen ein.
  explore_map_aria: Die interaktive Karte erkunden
  sites_overview:
    others: Sonstige
en:
  less: less
  less_aria: show less buildings
  more: more
  more_aria: show more buildings
  overview_map: Overview Map
  sites: Sites
  show_details_for_campus: show the details for the campus '{0}'
  show_details_for_building: show the details for the building '{0}'
  description: Find rooms, buildings and other places at TUM with excellence. A modern alternative to RoomFinder, developed by students.
  explore_map: Explore the map
  explore_map_subtitle: Browse the map and toggle layers such as toilets and showers.
  explore_map_aria: Explore the interactive map
  sites_overview:
    others: Others
</i18n>
