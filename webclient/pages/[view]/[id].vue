<script setup lang="ts">
import { useClipboard } from "@vueuse/core";
import type { components } from "~/api_types";
import { Tab, TabGroup, TabList, TabPanel, TabPanels } from "@headlessui/vue";
import { CalendarDaysIcon } from "@heroicons/vue/16/solid";
import { ClipboardDocumentCheckIcon, LinkIcon } from "@heroicons/vue/20/solid";
import type { DetailsFeedbackButton, DetailsInteractiveMap, DetailsRoomfinderMap } from "#components";

definePageMeta({
  validate(route) {
    return /(view|campus|site|building|room|poi)/.test(route.params.view as string);
  },
  layout: "details",
});

type DetailsResponse = components["schemas"]["DetailsResponse"];
type ImageInfo = components["schemas"]["ImageInfo"];

const { t, locale } = useI18n({ useScope: "local" });
const route = useRoute();
const router = useRouter();

const calendar = useCalendar();
const runtimeConfig = useRuntimeConfig();
const url = computed(() => `${runtimeConfig.public.apiURL}/api/get/${route.params.id}?lang=${locale.value}`);
const { data, error } = useFetch<DetailsResponse, string>(url, {
  key: "details",
  dedupe: "cancel",
  deep: false,
  retry: 120,
  retryDelay: 5000,
});

const shownImage = ref<ImageInfo | undefined>(data.value?.imgs?.length ? data.value.imgs[0] : undefined);
const slideshowOpen = ref(false);

const clipboardSource = computed(() => `https://nav.tum.de${route.fullPath}`);
const { copy, copied, isSupported: clipboardIsSupported } = useClipboard({ source: clipboardSource });

const selectedMap = computed<"interactive" | "plans">(() => {
  const map = route.query.map;
  if (!map) return "interactive";
  if (Array.isArray(map)) return map[0] === "plans" ? "plans" : "interactive";
  return map === "plans" ? "plans" : "interactive";
});
watchEffect(() => {
  if (route.params.id === "root") {
    router.replace({ path: "/" });
  }
});
watchEffect(() => {
  if (error.value) {
    router.push({
      path: "/404",
      query: { ...route.query, path: route.path },
      hash: route.hash,
    });
  }
});
watch([data, route], () => {
  if (!data.value) return;
  if (route.fullPath !== data.value.redirect_url) router.replace({ path: data.value.redirect_url });
});
watch([data], () => {
  if (!data.value) return;
  // --- Additional data ---
  slideshowOpen.value = false;
  route.query.map = data.value.maps.default;
  // --- Images ---
  shownImage.value = data.value.imgs?.length ? data.value.imgs[0] : undefined;
  tryToLoadMap();
});

const description = computed(() => {
  if (data.value === null) return "";
  const detailsFor = t("details_for");
  let description = `${detailsFor} ${data.value.type_common_name} ${data.value.name}`;
  if (data.value.props.computed) {
    description += ":";
    data.value.props.computed.forEach((prop) => {
      description += `\n- ${prop.name}: ${prop.text}`;
    });
  }
  return description;
});
const title = computed(() => data.value?.name || route.params.id + " - Navigatum");
useSeoMeta({
  title: title,
  ogTitle: title,
  description: description,
  ogDescription: description,
  ogImage: `https://nav.tum.de/api/preview/${route.params.id}`,
  twitterCard: "summary_large_image",
});

// --- Loading components ---
function tryToLoadMap() {
  /**
   * Try to load the entry map (interactive or plans). It requires the map container
   * element to be loaded in DOM.
   * @return {boolean} Whether the loading was successful
   */
  if (document.getElementById("interactive-map") !== null) {
    if (selectedMap.value === "interactive") interactiveMap.value?.loadInteractiveMap(false);
    // scrolling to the top after navigation
    window.scrollTo({ top: 0, behavior: "auto" });
    return true;
  }
  return false;
}

// following variables are bound to ref objects
const feedbackButton = ref<InstanceType<typeof DetailsFeedbackButton> | null>(null);
const interactiveMap = ref<InstanceType<typeof DetailsInteractiveMap> | null>(null);
const plansMap = ref<InstanceType<typeof DetailsRoomfinderMap> | null>(null);
onMounted(() => {
  nextTick(() => {
    // Even though 'mounted' is called there is no guarantee apparently,
    // that we can reference the map by ID in the DOM yet. For this reason we
    // try to poll now (Not the best solution probably)
    let timeoutInMs = 25;

    function pollMap() {
      if (!tryToLoadMap()) {
        console.info(`'mounted' called, but page is not mounted yet. Retrying map-load in ${timeoutInMs}ms`);
        setTimeout(pollMap, timeoutInMs);
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
      v-if="data.imgs?.length"
      type="button"
      class="focusable block lg:hidden print:!hidden"
      @click="slideshowOpen = true"
    >
      <NuxtImg
        width="256"
        height="105"
        :alt="t('image_alt')"
        :src="`${runtimeConfig.public.cdnURL}/cdn/lg/${data.imgs[0].name}`"
        sizes="1024px sm:256px md:512px"
        densities="x1 x2"
        class="block w-full"
        preload
        :placeholder="[256, 105]"
      />
    </button>

    <!-- Entry header / title -->
    <div class="px-5">
      <BreadcrumbList
        :items="data.parent_names.map((n, i) => ({ name: n, to: i > 0 ? '/view/' + data?.parents[i] : '/' }))"
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
            <button
              v-if="data.props?.calendar_url"
              type="button"
              class="focusable rounded-sm"
              :title="t('header.calendar')"
              @click="
                calendar.open = true;
                calendar.showing = [route.params.id.toString()];
              "
            >
              <CalendarDaysIcon class="text-blue-600 mt-0.5 h-4 w-4" />
            </button>
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
    <div class="grid grid-cols-1 gap-5 px-5 lg:grid-cols-3">
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
          <TabPanel id="interactiveMapPanel" :unmount="false">
            <ClientOnly>
              <DetailsInteractiveMap ref="interactiveMap" :data="data" />
            </ClientOnly>
          </TabPanel>
          <TabPanel id="plansMapPanel" :unmount="false">
            <ClientOnly>
              <LazyDetailsRoomfinderMap
                v-if="data.maps.roomfinder?.available"
                ref="plansMap"
                :available="data.maps.roomfinder.available"
                :default-map-id="data.maps.roomfinder.default"
              />
            </ClientOnly>
          </TabPanel>
        </TabPanels>
        <TabList class="bg-zinc-100 flex space-x-1 rounded-md p-1 print:!hidden">
          <Tab
            v-slot="{ selected }"
            as="template"
            @click="
              () => {
                route.query.map = 'plans';
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
            @click="route.query.map = 'plans'"
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
              {{ t("map.plans") }}
            </button>
          </Tab>
        </TabList>
      </TabGroup>
      <!-- Map container -->

      <DetailsInfoSection v-model:shown_image="shownImage" v-model:slideshow_open="slideshowOpen" :data="data" />
    </div>

    <DetailsBuildingOverviewSection :buildings="data.sections?.buildings_overview" />
    <ClientOnly>
      <LazyDetailsRoomOverviewSection :rooms="data.sections?.rooms_overview" />
    </ClientOnly>
    <DetailsSources
      :coords="data.coords"
      :sources="data.sources"
      :image="data.imgs?.length ? data.imgs[0] : undefined"
    />
  </div>
  <div v-else class="text-zinc-900 flex flex-col items-center gap-5 py-32">
    <Spinner class="h-8 w-8" />
    {{ t("Loading data...") }}
  </div>
  <ClientOnly>
    <CalendarModal v-if="calendar.open" />
  </ClientOnly>
</template>

<i18n lang="yaml">
de:
  image_alt: Header-Bild, zeigt das Gebäude
  details_for: Details für
  map:
    interactive: Interaktive Karte
    plans: Lagepläne
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
    plans: Site Plans
  no_floor_overlay: There is unfortunately no indoor map for the displayed room.
  header:
    calendar: Open calendar
    copy_link: Copy link
    favorites: Add to favorites
  Loading data...: Loading data...
</i18n>
