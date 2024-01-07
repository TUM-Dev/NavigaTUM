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
import { selectedMap, useDetailsStore } from "@/stores/details";
import { computed, nextTick, onMounted, ref, watchEffect } from "vue";
import { useFetch } from "@/composables/fetch";
import { useRoute, useRouter } from "vue-router";
import type { components } from "@/api_types";
import Toast from "@/components/Toast.vue";
import { CalendarDaysIcon, CheckIcon, ClipboardDocumentCheckIcon } from "@heroicons/vue/24/outline";
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
    if (state.map.selected === selectedMap.interactive) interactiveMap.value?.loadInteractiveMap();
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
  window.addEventListener("resize", () => {
    if (state.map.selected === selectedMap.roomfinder) {
      roomfinderMap.value?.loadRoomfinderMap(state.map.roomfinder.selected_index);
    }
  });

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
      @click="state.showImageSlideshow(state.image.shown_image_id || 0)"
    >
      <img :alt="t('image_alt')" :src="`${appURL}/cdn/header/${state.image.shown_image.name}`" class="block w-full" />
    </button>

    <BreadcrumbList
      :items="state.data.parent_names.map((n, i) => ({ name: n, to: '/view/' + state.data?.parents[i] }))"
    />

    <!-- Entry header / title -->
    <div>
      <div>
        <div v-if="clipboardIsSupported" class="hidden lg:block">
          <button
            type="button"
            class="btn btn-action btn-link btn-sm"
            :title="t('header.copy_link')"
            @click="copy(`https://nav.tum.de${route.fullPath}`)"
          >
            <CheckIcon v-if="copied" class="w-4 h-4" />
            <ClipboardDocumentCheckIcon v-else class="w-4 h-4" />
          </button>
        </div>
        <h1>
          {{ state.data?.name }}
          <!-- <small class="label">Exaktes Ergebnis</small> -->
        </h1>
      </div>
      <div class="subtitle">
        <div class="flex grow place-items-center justify-between">
          <span class="text-neutral-400">{{ state.data?.type_common_name }}</span>
          <div class="flex flex-row place-items-center gap-3">
            <template v-if="state.data?.props?.calendar_url">
              <a :href="state.data.props.calendar_url" target="_blank" :title="t('header.calendar')">
                <CalendarDaysIcon class="mt-0.5 h-4 w-4" />
              </a>
            </template>
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
    <div class="columns">
      <!-- Map container -->
      <div id="map-container" class="col-7 col-md-12 column">
        <div class="mb-3 grid gap-2 md:hidden">
          <Toast
            v-if="state.data?.type === 'room' && state.data?.maps?.overlays?.default === null"
            level="warning"
            :msg="t('no_floor_overlay')"
          />
          <Toast v-if="state.data?.props?.comment" :msg="state.data.props.comment" />
        </div>

        <DetailsInteractiveMap ref="interactiveMap" />
        <DetailsRoomfinderMap ref="roomfinderMap" />
        <div class="btn-group btn-group-block">
          <button
            type="button"
            class="btn btn-sm"
            :class="{
              active: state.map.selected === selectedMap.interactive,
            }"
            @click="interactiveMap?.loadInteractiveMap(true)"
          >
            {{ t("map.interactive") }}
          </button>
          <button
            type="button"
            class="btn btn-sm"
            :class="{
              active: state.map.selected === selectedMap.roomfinder,
            }"
            :disabled="!state.data.maps.roomfinder?.available"
            @click="roomfinderMap?.loadRoomfinderMap(state.map.roomfinder.selected_index, true)"
          >
            {{ t("map.roomfinder") }}
          </button>
        </div>
        <div class="divider mt-2" />
      </div>

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
