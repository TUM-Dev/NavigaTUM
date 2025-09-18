<script setup lang="ts">
import { Tab, TabGroup, TabList, TabPanel, TabPanels } from "@headlessui/vue";
import { ClipboardDocumentCheckIcon, LinkIcon } from "@heroicons/vue/20/solid";
import { CalendarDaysIcon, PlusIcon } from "@heroicons/vue/24/outline";
import { useClipboard } from "@vueuse/core";
import { useRouteQuery } from "@vueuse/router";
import type { DetailsFeedbackButton, DetailsInteractiveMap } from "#components";
import type { components } from "~/api_types";
import { useEditProposal } from "~/composables/editProposal";

definePageMeta({
  validate(route) {
    return /(view|campus|site|building|room|poi)/.test(route.params.view as string);
  },
  layout: "details",
});

type LocationDetailsResponse = components["schemas"]["LocationDetailsResponse"];
type ImageInfoResponse = components["schemas"]["ImageInfoResponse"];

const { t, locale } = useI18n({ useScope: "local" });
const localePath = useLocalePath();
const route = useRoute();
const router = useRouter();

const calendar = useCalendar();
const runtimeConfig = useRuntimeConfig();
const url = computed(
  () => `${runtimeConfig.public.apiURL}/api/locations/${route.params.id}?lang=${locale.value}`
);
const { data, error } = useFetch<LocationDetailsResponse, string>(url, {
  dedupe: "cancel",
  credentials: "omit",
  retry: 120,
  retryDelay: 1000,
});

const editProposal = useEditProposal();

const shownImage = ref<ImageInfoResponse | undefined>(
  data.value?.imgs?.length ? data.value.imgs[0] : undefined
);
const slideshowOpen = ref(false);

const clipboardSource = computed(() => `https://nav.tum.de${route.fullPath}`);
const {
  copy,
  copied,
  isSupported: clipboardIsSupported,
} = useClipboard({ source: clipboardSource });

const suggestImage = () => {
  if (!data.value) return;

  editProposal.value.selected = {
    id: data.value.id,
    name: data.value.name,
  };
  if (!editProposal.value.data.additional_context) {
    editProposal.value.data.additional_context = `I would like to suggest a new image for ${data.value.name} (${data.value.id}).`;
  }
  editProposal.value.locationPicker = {
    lat: data.value.coords.lat,
    lon: data.value.coords.lon,
    open: false,
  };
  editProposal.value.open = true;
  editProposal.value.imageUpload.open = true;
};

const selectedMap = useRouteQuery<"interactive" | "plans">("map", "interactive", {
  mode: "replace",
  route,
  router,
  transform: (val) => (val === "plans" ? "plans" : "interactive"),
});

watchEffect(async () => {
  if (route.params.id === "root") {
    await navigateTo({ path: localePath("/"), replace: true });
  }
});
watchEffect(async () => {
  if (error.value) {
    await navigateTo({
      path: "/404",
      query: { ...route.query, path: route.path },
      hash: route.hash,
      replace: true,
    });
  }
});
watch([data, route], async () => {
  if (!data.value) return;
  const redirectPath = localePath(data.value.redirect_url as string);
  if (route.path !== redirectPath) {
    await navigateTo({ path: redirectPath, query: route.query, replace: true });
  }
});
watch([data], () => {
  if (!data.value) return;
  // --- Additional data ---
  slideshowOpen.value = false;
  // --- Images ---
  shownImage.value = data.value.imgs?.length ? data.value.imgs[0] : undefined;
});

const description = computed(() => {
  if (data.value === undefined || data.value === null) return "";

  let description = t("details_for");
  if (data.value.name.includes(data.value.type_common_name)) {
    description += ` ${data.value.name}`;
  } else {
    description += ` ${data.value.type_common_name} ${data.value.name}`;
  }
  if (data.value.props.computed) {
    description += ":";
    for (const prop of data.value.props.computed) {
      description += `\n- ${prop.name}: ${prop.text}`;
    }
  }
  return description;
});
const title = computed(() => data.value?.name || `${route.params.id} - Navigatum`);
useSeoMeta({
  title: title,
  ogTitle: title,
  description: description,
  ogDescription: description,
  ogImage: `https://nav.tum.de/api/locations/${route.params.id}/preview`,
  twitterCard: "summary_large_image",
});
</script>

<template>
  <div v-if="data" class="flex flex-col gap-5">
    <!-- Header image (on mobile) -->
    <div v-if="data.imgs?.length && data.imgs[0]" class="relative block lg:hidden print:!hidden">
      <button type="button" class="focusable block w-full" @click="slideshowOpen = true">
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
    </div>
    <!-- No header image placeholder (on mobile) -->
    <div
      v-else-if="!data.imgs?.length"
      class="relative group hover:bg-zinc-200 hover:border-zinc-400 m-1 mt-2 block lg:hidden print:!hidden bg-zinc-100 border-2 border-dashed border-zinc-300 rounded-lg"
    >
      <button
        type="button"
        class="w-full h-20 flex flex-col items-center justify-center text-zinc-500 group-hover:text-zinc-700 group-hover:border-zinc-400 transition-colors"
        @click="suggestImage"
      >
        <PlusIcon class="h-8 w-8 mb-2" />
        <span class="text-sm font-medium">{{ t("add_first_image") }}</span>
      </button>
    </div>

    <!-- Entry header / title -->
    <div class="px-5">
      <BreadcrumbList
        :items="
          data.parent_names.map((n, i) => ({
            name: n,
            to: i > 0 ? '/view/' + data?.parents[i] : '/',
          }))
        "
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
              @click="calendar = [...new Set([...calendar, route.params.id?.toString() ?? '404'])]"
            >
              <CalendarDaysIcon class="text-blue-600 mt-0.5 h-7 w-7 hover:text-blue-900" />
            </button>
            <ShareButton :coords="data.coords" :name="data.name" />
            <DetailsFeedbackButton />
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
      <TabGroup
        class="col-span-1 lg:col-span-2"
        as="div"
        :selected-index="selectedMap === 'interactive' ? 0 : 1"
        :default-index="selectedMap === 'interactive' ? 0 : 1"
        @change="(index) => (selectedMap = index === 0 ? 'interactive' : 'plans')"
      >
        <div class="mb-3 grid gap-2 lg:hidden">
          <Toast v-if="data.type === 'room' && data.maps?.overlays?.default === null" level="warning" :msg="t('no_floor_overlay')" />
          <Toast v-if="data.props.comment" :msg="data.props.comment" />
        </div>
        <TabPanels>
          <TabPanel id="interactiveMapPanel" :tab-index="0" :unmount="false">
            <ClientOnly>
              <DetailsInteractiveMap :id="data.id" :coords="data.coords" :type="data.type" :maps="data.maps" />
            </ClientOnly>
          </TabPanel>
          <TabPanel id="plansMapPanel" :tab-index="1">
            <ClientOnly>
              <LazyDetailsRoomfinderMap
                v-if="data.maps.roomfinder?.available"
                :available="data.maps.roomfinder.available"
                :default-map-id="data.maps.roomfinder.default"
              />
            </ClientOnly>
          </TabPanel>
        </TabPanels>
        <TabList class="bg-zinc-100 flex space-x-1 rounded-md p-1 print:!hidden">
          <Tab :tab-index="0" as="template" @click="selectedMap = 'interactive'">
            <button
              type="button"
              class="focusable w-full rounded-md py-2.5 text-sm font-medium leading-5"
              :class="[
                selectedMap === 'interactive' ? 'text-zinc-900 bg-zinc-300 shadow' : 'text-zinc-800 bg-zinc-300/5 hover:text-zinc-900 hover:bg-zinc-500/20',
              ]"
            >
              {{ t("map.interactive") }}
            </button>
          </Tab>
          <Tab :tab-index="1" as="template" :disabled="!data.maps.roomfinder?.available" @click="selectedMap = 'plans'">
            <button
              type="button"
              class="focusable w-full rounded-md py-2.5 text-sm font-medium leading-5"
              :class="{
                'text-zinc-900 bg-zinc-300 shadow': selectedMap === 'plans',
                'text-zinc-800 bg-zinc-300/5': selectedMap !== 'plans',
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
    <DetailsSources :coords="data.coords" :sources="data.sources" :image="data.imgs?.length ? data.imgs[0] : undefined" />
  </div>
  <div v-else class="text-zinc-900 flex flex-col items-center gap-5 py-32">
    <Spinner class="h-8 w-8" />
    {{ t("Loading data...") }}
  </div>
  <ClientOnly>
    <LazyCalendarModal v-if="calendar.length" />
    <LazyEditProposalModal v-if="editProposal.open" />
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
  add_image: Bild hinzufügen
  add_first_image: Erstes Bild hinzufügen
  Loading data...: Lädt Daten...
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
  add_image: Add image
  add_first_image: Add first image
  Loading data...: Loading data...
</i18n>
