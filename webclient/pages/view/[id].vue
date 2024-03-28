<script setup lang="ts">
import ShareButton from "../../components/ShareButton.vue";
import DetailsInteractiveMap from "../../components/DetailsInteractiveMap.vue";
import DetailsRoomOverviewSection from "../../components/DetailsRoomOverviewSection.vue";
import DetailsBuildingOverviewSection from "../../components/DetailsBuildingOverviewSection.vue";
import DetailsInfoSection from "../../components/DetailsInfoSection.vue";
import DetailsSources from "../../components/DetailsSources.vue";
import DetailsFeedbackButton from "../../components/DetailsFeedbackButton.vue";
import DetailsRoomfinderMap from "../../components/DetailsRoomfinderMap.vue";
import { useI18n } from "vue-i18n";
import { setDescription, setTitle } from "../../composables/common";
import { useClipboard } from "@vueuse/core";
import { computed, nextTick, onMounted, ref, shallowRef, watchEffect } from "vue";
import { useFetch } from "../../composables/fetch";
import { useRoute, useRouter } from "vue-router";
import type { components } from "../../api_types";
import Toast from "../../components/Toast.vue";
import { Tab, TabGroup, TabList, TabPanel, TabPanels } from "@headlessui/vue";
import { CalendarDaysIcon, ClipboardDocumentCheckIcon, LinkIcon } from "@heroicons/vue/24/outline";
import BreadcrumbList from "../../components/BreadcrumbList.vue";
import Spinner from "../../components/Spinner.vue";

type DetailsResponse = components["schemas"]["DetailsResponse"];
type ImageInfo = components["schemas"]["ImageInfo"];

const { t, locale } = useI18n({ useScope: "local" });
const route = useRoute();
const router = useRouter();
const data = shallowRef<DetailsResponse | null>(null);
const shownImage = ref<ImageInfo | undefined>(undefined);
const slideshowOpen = ref(false);

const clipboardSource = computed(() => `https://nav.tum.de${route.fullPath}`);
const { copy, copied, isSupported: clipboardIsSupported } = useClipboard({ source: clipboardSource });
const appURL = import.meta.env.VITE_APP_URL;
const selectedMap = ref<"interactive" | "roomfinder">("interactive");

function loadData(d: DetailsResponse) {
  if (route.fullPath !== d.redirect_url) router.replace({ path: d.redirect_url });
  data.value = d;
  // --- Additional data ---
  slideshowOpen.value = false;
  setTitle(d.name);
  setDescription(genDescription(d));
  selectedMap.value = d.maps.default;
  // --- Images ---
  if (d.imgs && d.imgs.length > 0) {
    shownImage.value = d.imgs[0];
  } else {
    shownImage.value = undefined;
  }
  tryToLoadMap();
}

watchEffect(() => {
  if (route.params.id === "root") {
    router.replace({ path: "/" });
    return;
  }
  useFetch<DetailsResponse>(`/api/get/${route.params.id}?lang=${locale.value}`, loadData, () => {
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
    const previoslyOnInteractiveMap = false
    if ( === "interactive") interactiveMap.value?.loadInteractiveMap(false,previoslyOnInteractiveMap);
    // scrolling to the top after navigation
    window.scrollTo({ top: 0, behavior: "auto" });
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
        console.info(`'mounted' called, but page is not mounted yet. Retrying map-load in ${timeoutInMs}ms`);
        window.setTimeout(pollMap, timeoutInMs);
        timeoutInMs *= 1.5;
      }
    }

    pollMap();
  });
});
</script>

<template>
  <div v-if="data" class="flex flex-col gap-5">
    <!-- Header image (on mobile) -->
    <button
      v-if="shownImage"
      type="button"
      class="focusable !-mx-5 block lg:hidden print:!hidden"
      @click="slideshowOpen = !!data.imgs"
    >
      <img :alt="t('image_alt')" :src="`${appURL}/cdn/header/${shownImage.name}`" class="block w-full" />
    </button>

    <!-- Entry header / title -->
    <div>
      <BreadcrumbList
        :items="data.parent_names.map((n, i) => ({ name: n, to: '/view/' + data?.parents[i] }))"
        class="pb-3 pt-6"
      />
      <div class="group flex flex-row gap-2">
        <button
          v-if="clipboardIsSupported"
          :title="t('header.copy_link')"
          type="button"
          tabindex="1"
          class="-ms-8 hidden px-1 text-transparent transition-colors focus:text-zinc-800 group-hover:text-zinc-800 lg:block"
          @click="copy(`https://nav.tum.de${route.fullPath}`)"
        >
          <ClipboardDocumentCheckIcon v-if="copied" class="h-4 w-4" />
          <LinkIcon v-else class="h-4 w-4" />
        </button>
        <h1 class="text-zinc-700 text-xl font-bold">{{ data.name }}</h1>
      </div>
      <div>
        <div class="flex grow place-items-center justify-between">
          <span class="text-zinc-500 mt-0.5 text-sm">{{ data.type_common_name }}</span>
          <div class="flex flex-row place-items-center gap-3">
            <a
              v-if="data.props?.calendar_url"
              :href="data.props.calendar_url"
              target="_blank"
              class="focusable rounded-sm"
              :title="t('header.calendar')"
            >
              <CalendarDaysIcon class="text-tumBlue-600 mt-0.5 h-4 w-4" />
            </a>
            <ShareButton :coords="data.coords" :name="data.name" />
            <DetailsFeedbackButton ref="feedbackButton" />
            <!-- <button class="btn btn-link btn-action btn-sm"
                  :title="t('header.favorites')">
            <BookmarkIcon class="w-4 h-4" v-if="bookmarked" />
            <BookmarkSquareIcon class="w-4 h-4" v-else />
          </button> -->
          </div>
        </div>
      </div>
    </div>

    <!-- First info section (map + infocard) -->
    <div class="grid grid-cols-1 gap-5 lg:grid-cols-3">
      <TabGroup class="col-span-1 lg:col-span-2" as="div" manual>
        <div class="mb-3 grid gap-2 lg:hidden">
          <Toast
            v-if="data.type === 'room' && data.maps?.overlays?.default === null"
            level="warning"
            :msg="t('no_floor_overlay')"
          />
          <Toast v-if="data.props.comment" :msg="data.props.comment" />
        </div>
        <TabPanels>
          <TabPanel :unmount="false">
            <DetailsInteractiveMap ref="interactiveMap" :data="data" />
          </TabPanel>
          <TabPanel :unmount="false">
            <DetailsRoomfinderMap
              v-if="data.maps.roomfinder?.available"
              ref="roomfinderMap"
              :available="data.maps.roomfinder.available"
              :default-map-id="data.maps.roomfinder.default"
            />
          </TabPanel>
        </TabPanels>
        <TabList class="bg-zinc-100 flex space-x-1 rounded-md p-1 print:!hidden">
          <Tab
            v-slot="{ selected }"
            as="template"
            @click="
              () => {
                selectedMap = 'roomfinder';
                interactiveMap?.loadInteractiveMap(true);
              }
            "
          >
            <button
              type="button"
              class="focusable w-full rounded-md py-2.5 text-sm font-medium leading-5"
              :class="[
                selected
                  ? 'text-zinc-900 bg-zinc-300 shadow'
                  : 'text-zinc-800 bg-zinc-300/5 hover:text-zinc-900 hover:bg-zinc-500/20',
              ]"
            >
              {{ t("map.interactive") }}
            </button>
          </Tab>
          <Tab
            v-slot="{ selected }"
            as="template"
            :disabled="!data.maps.roomfinder?.available"
            @click="selectedMap = 'roomfinder'"
          >
            <button
              type="button"
              class="focusable w-full rounded-md py-2.5 text-sm font-medium leading-5"
              :class="{
                'text-zinc-900 bg-zinc-300 shadow': selected,
                'text-zinc-800 bg-zinc-300/5': !selected,
                'hover:text-zinc-900 hover:bg-zinc-500/20': data.maps.roomfinder?.available,
                '!text-zinc-400 cursor-not-allowed': !data.maps.roomfinder?.available,
              }"
            >
              {{ t("map.roomfinder") }}
            </button>
          </Tab>
        </TabList>
      </TabGroup>
      <!-- Map container -->

      <DetailsInfoSection v-model:shown_image="shownImage" v-model:slideshow_open="slideshowOpen" :data="data" />
    </div>

    <DetailsBuildingOverviewSection :buildings="data.sections?.buildings_overview" />
    <DetailsRoomOverviewSection :rooms="data.sections?.rooms_overview" />
    <DetailsSources :coords="data.coords" :sources="data.sources" :shown_image="shownImage" />
  </div>
  <div v-else class="text-zinc-900 flex flex-col items-center gap-5 py-32">
    <Spinner class="h-8 w-8" />
    {{ t("Loading data...") }}
  </div>
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
