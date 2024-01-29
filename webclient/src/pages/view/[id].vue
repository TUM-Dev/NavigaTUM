<script setup lang="ts">
import ShareButton from "@/components/ShareButton.vue";
import DetailsInteractiveMap from "@/components/DetailsInteractiveMap.vue";
import DetailsRoomOverviewSection from "@/components/DetailsRoomOverviewSection.vue";
import DetailsBuildingOverviewSection from "@/components/DetailsBuildingOverviewSection.vue";
import DetailsInfoSection from "@/components/DetailsInfoSection.vue";
import DetailsSources from "@/components/DetailsSources.vue";
import DetailsFeedbackButton from "@/components/DetailsFeedbackButton.vue";
import DetailsRoomfinderMap from "@/components/DetailsRoomfinderMap.vue";
import { useI18n } from "vue-i18n";
import { setDescription, setTitle } from "@/composables/common";
import { useClipboard } from "@vueuse/core";
import { MapSelections, useDetailsStore } from "@/stores/details";
import { computed, nextTick, onMounted, ref, watchEffect } from "vue";
import { useFetch } from "@/composables/fetch";
import { useRoute, useRouter } from "vue-router";
import type { components } from "@/api_types";
import Toast from "@/components/Toast.vue";
import { Tab, TabGroup, TabList, TabPanel, TabPanels } from "@headlessui/vue";
import { CalendarDaysIcon, ClipboardDocumentCheckIcon, LinkIcon } from "@heroicons/vue/24/outline";
import BreadcrumbList from "@/components/BreadcrumbList.vue";

type DetailsResponse = components["schemas"]["DetailsResponse"];

const { t } = useI18n({ useScope: "local" });
const route = useRoute();
const router = useRouter();
const state = useDetailsStore();
const clipboardSource = computed(() => `https://nav.tum.de${route.fullPath}`);
const { copy, copied, isSupported: clipboardIsSupported } = useClipboard({ source: clipboardSource });
const appURL = import.meta.env.VITE_APP_URL;

function loadData(data: DetailsResponse) {
  if (route.fullPath !== data.redirect_url) router.replace({ path: data.redirect_url });
  // --- Additional data ---
  setTitle(data.name);
  setDescription(genDescription(data));
  state.$reset();
  state.loadData(data);
  tryToLoadMap();
}

watchEffect(() => {
  if (route.params.id === "root") {
    router.replace({ path: "/" });
    return;
  }
  useFetch<DetailsResponse>(`/api/get/${route.params.id}`, loadData, () => {
    router.push({
      name: "404",
      // preserve current path and remove the first char to avoid the target URL starting with `//`
      params: { catchAll: route.path.substring(1).split("/") },
      query: route.query,
      hash: route.hash,
    });
  });
});

function genDescription(d: DetailsResponse) {
  const detailsFor = t("details_for");
  let description = `${detailsFor} ${d.type_common_name} ${d.name}`;
  if (d.props.computed) {
    description += ":";
    d.props.computed.forEach((prop) => {
      description += `\n- ${prop.name}: ${prop.text}`;
    });
  }
  return description;
}

// --- Loading components ---
function tryToLoadMap() {
  /**
   * Try to load the entry map (interactive or roomfinder). It requires the map container
   * element to be loaded in DOM.
   * @return {boolean} Whether the loading was successful
   */
  if (document.getElementById("interactive-map") !== null) {
    if (state.map.selected === MapSelections.interactive) interactiveMap.value?.loadInteractiveMap();
    else roomfinderMap.value?.loadRoomfinderMap(state.map.roomfinder.selected_index);
    return true;
  }
  return false;
}

// following variables are bound to ref objects
const feedbackButton = ref<InstanceType<typeof DetailsFeedbackButton> | null>(null);
const interactiveMap = ref<InstanceType<typeof DetailsInteractiveMap> | null>(null);
const roomfinderMap = ref<InstanceType<typeof DetailsRoomfinderMap> | null>(null);
onMounted(() => {
  nextTick(() => {
    // Even though 'mounted' is called there is no guarantee apparently,
    // that we can reference the map by ID in the DOM yet. For this reason we
    // try to poll now (Not the best solution probably)
    let timeoutInMs = 25;

    function pollMap() {
      if (!tryToLoadMap()) {
        console.warn(
          `'mounted' called, but page doesn't appear to be mounted yet. Retrying to load the map in ${timeoutInMs}ms`,
        );
        window.setTimeout(pollMap, timeoutInMs);
        timeoutInMs *= 1.5;
      }
    }

    pollMap();
  });
});
</script>

<template>
  <div v-if="state.data">
    <!-- Header image (on mobile) -->
    <button
      v-if="state.image.shown_image"
      type="button"
      class="block cursor-pointer lg:hidden"
      @click="state.showImageSlideshow(true)"
    >
      <img :alt="t('image_alt')" :src="`${appURL}/cdn/header/${state.image.shown_image.name}`" class="block w-full" />
    </button>

    <!-- Entry header / title -->
    <div>
      <BreadcrumbList
        :items="state.data.parent_names.map((n, i) => ({ name: n, to: '/view/' + state.data?.parents[i] }))"
        class="pb-3 pt-6"
      />
      <div class="group flex flex-row gap-2">
        <button
          v-if="clipboardIsSupported"
          :title="t('header.copy_link')"
          type="button"
          class="hidden -ms-6 text-transparent group-hover:text-tumBlue-500 lg:block"
          @click="copy(`https://nav.tum.de${route.fullPath}`)"
        >
          <ClipboardDocumentCheckIcon v-if="copied" class="h-4 w-4" />
          <LinkIcon v-else class="h-4 w-4" />
        </button>
        <h1 class="text-xl font-bold text-zinc-700">{{ state.data?.name }}</h1>
      </div>
      <div>
        <div class="flex grow place-items-center justify-between">
          <span class="mt-0.5 text-sm text-neutral-400">{{ state.data?.type_common_name }}</span>
          <div class="flex flex-row place-items-center gap-3">
            <a
              v-if="state.data?.props?.calendar_url"
              :href="state.data.props.calendar_url"
              target="_blank"
              class="focusable rounded-sm"
              :title="t('header.calendar')"
            >
              <CalendarDaysIcon class="mt-0.5 h-4 w-4 text-tumBlue-600" />
            </a>
            <ShareButton :coords="state.data.coords" :name="state.data.name" />
            <DetailsFeedbackButton ref="feedbackButton" />
            <!-- <button class="btn btn-link btn-action btn-sm"
                  :title="t('header.favorites')">
            <BookmarkIcon class="w-4 h-4" v-if="bookmarked" />
            <BookmarkSquareIcon class="w-4 h-4" v-else />
          </button> -->
          </div>
        </div>
      </div>
      <div class="divider mb-5" />
    </div>

    <!-- First info section (map + infocard) -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-5">
      <TabGroup class="col-span-1 md:col-span-2" as="div" manual>
        <div class="mb-3 grid gap-2 md:hidden">
          <Toast
            v-if="state.data?.type === 'room' && state.data?.maps?.overlays?.default === null"
            level="warning"
            :msg="t('no_floor_overlay')"
          />
          <Toast v-if="state.data.props.comment" :msg="state.data.props.comment" />
        </div>
        <TabPanels>
          <TabPanel :unmount="false">
            <DetailsInteractiveMap ref="interactiveMap" />
          </TabPanel>
          <TabPanel :unmount="false">
            <DetailsRoomfinderMap ref="roomfinderMap" />
          </TabPanel>
        </TabPanels>
        <TabList class="flex space-x-1 rounded-md bg-tumBlue-900/20 p-1">
          <Tab v-slot="{ selected }" as="template" @click="interactiveMap?.loadInteractiveMap(true)">
            <button
              type="button"
              class="focusable w-full rounded-md py-2.5 text-sm font-medium leading-5"
              :class="[
                selected
                  ? 'bg-white text-tumBlue-700 shadow'
                  : 'text-tumBlue-100 hover:bg-white/[0.12] hover:text-white',
              ]"
            >
              {{ t("map.interactive") }}
            </button>
          </Tab>
          <Tab v-slot="{ selected }" as="template" :disabled="!state.data.maps.roomfinder?.available">
            <button
              type="button"
              class="focusable w-full rounded-md py-2.5 text-sm font-medium leading-5"
              :class="[
                selected ? 'bg-white text-tumBlue-700 shadow' : 'text-tumBlue-50 hover:bg-white/[0.1] hover:text-white',
              ]"
            >
              {{ t("map.roomfinder") }}
            </button>
          </Tab>
        </TabList>
      </TabGroup>
      <!-- Map container -->

      <DetailsInfoSection />
    </div>

    <DetailsBuildingOverviewSection :buildings="state.data?.sections?.buildings_overview" />
    <DetailsRoomOverviewSection :rooms="state.data?.sections?.rooms_overview" />
    <DetailsSources />
  </div>
  <div v-else>{{ t("Loading data...") }}</div>
</template>

<i18n lang="yaml">
de:
  image_alt: Header-Bild, zeigt das Gebäude
  details_for: Details für
  map:
    interactive: Interaktive Karte
    roomfinder: Lagepläne
  no_floor_overlay: Für den angezeigten Raum gibt es leider keine Indoor Karte.
  header:
    calendar: Kalender öffnen
    copy_link: Link kopieren
    favorites: Zu Favoriten hinzufügen
  Loading data...: Lädt daten...
en:
  image_alt: Header image, showing the building
  details_for: Details for
  map:
    interactive: Interactive Map
    roomfinder: Site Plans
  no_floor_overlay: There is unfortunately no indoor map for the displayed room.
  header:
    calendar: Open calendar
    copy_link: Copy link
    favorites: Add to favorites
  Loading data...: Loading data...
</i18n>
