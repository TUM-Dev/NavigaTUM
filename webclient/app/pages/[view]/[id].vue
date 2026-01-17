<script setup lang="ts">
import { mdiCalendarMonth, mdiClipboardCheck, mdiLink, mdiPlus } from "@mdi/js";
import { useClipboard, useSwipe } from "@vueuse/core";
import type { components } from "~/api_types";
import { useEditProposal } from "~/composables/editProposal";

definePageMeta({
  validate(route) {
    return /(view|campus|site|building|room|poi)/.test(route.params.view as string);
  },
  layout: false,
});

type LocationDetailsResponse = components["schemas"]["LocationDetailsResponse"];
type ImageInfoResponse = components["schemas"]["ImageInfoResponse"];

const { t, locale } = useI18n({ useScope: "local" });
const localePath = useLocalePath();
const route = useRoute();

const searchBarFocused = ref(false);

const calendar = useCalendar();
const runtimeConfig = useRuntimeConfig();
const url = computed(
  () => `${runtimeConfig.public.apiURL}/api/locations/${route.params.id}?lang=${locale.value}`
);
const { data, error } = await useFetch<LocationDetailsResponse, string>(url, {
  dedupe: "cancel",
  credentials: "omit",
  retry: 120,
  retryDelay: 1000,
});

// Check if we need to redirect before showing error - use 301 for canonical URLs
if (data.value?.redirect_url) {
  const redirectPath = localePath(data.value.redirect_url as string);
  if (route.path !== redirectPath) {
    await navigateTo({ path: redirectPath, query: route.query }, { redirectCode: 301 });
  }
}

if (error.value) {
  showError({
    statusCode: 404,
    statusMessage: "Location not found",
  });
}

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

const suggestLocationFix = () => {
  if (!data.value) return;
  if (!editProposal.value.data.additional_context) {
    editProposal.value.data.additional_context = `The location for ${data.value.name} (${data.value.id}) is only accurate to building level. I can help provide a more precise location within the building.`;
  }
  editProposal.value.selected = {
    id: data.value.id,
    name: data.value.name,
  };
  editProposal.value.locationPicker = {
    lat: data.value.coords.lat,
    lon: data.value.coords.lon,
    open: true,
  };
  editProposal.value.open = true;
};

watchEffect(async () => {
  if (route.params.id === "root") {
    await navigateTo({ path: localePath("/"), replace: true });
  }
});

watch([data], () => {
  if (!data.value) return;
  slideshowOpen.value = false;
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
const canonicalUrl = computed(() => {
  if (!data.value?.redirect_url) return `https://nav.tum.de${route.fullPath}`;
  return `https://nav.tum.de${data.value.redirect_url}`;
});
useSeoMeta({
  title: title,
  ogTitle: title,
  description: description,
  ogDescription: description,
  ogImage: `https://nav.tum.de/api/locations/${route.params.id}/preview`,
  twitterCard: "summary_large_image",
});
useHead({
  link: [
    {
      rel: "canonical",
      href: canonicalUrl,
    },
  ],
});

// Mobile bottom sheet logic
type SheetState = "up" | "middle" | "down";
const mobileSheetState = ref<SheetState>("middle");
const sheetContainer = ref<HTMLElement | null>(null);
const scrollContainer = ref<HTMLElement | null>(null);

const toggleMobileExpand = () => {
  if (mobileSheetState.value === "middle") {
    mobileSheetState.value = "up";
  } else if (mobileSheetState.value === "up") {
    mobileSheetState.value = "down";
  } else {
    mobileSheetState.value = "middle";
  }
};

const { isSwiping } = useSwipe(sheetContainer, {
  threshold: 30,
  onSwipeEnd: (_e, direction) => {
    const scroll = scrollContainer.value?.scrollTop;
    if (direction === "up") {
      if (mobileSheetState.value === "down") {
        mobileSheetState.value = "middle";
      } else if (mobileSheetState.value === "middle") {
        mobileSheetState.value = "up";
      }
    } else if (direction === "down") {
      if (mobileSheetState.value === "up") {
        mobileSheetState.value = "middle";
      } else if (mobileSheetState.value === "middle" && scroll === 0) {
        mobileSheetState.value = "down";
      }
    }
  },
});
</script>

<template>
  <div class="h-screen flex flex-col overflow-hidden bg-zinc-50">
    <!-- Re-use AppNavHeader -->
    <AppNavHeader>
      <AppSearchBar v-model:search-bar-focused="searchBarFocused" />
    </AppNavHeader>

    <!-- Main Container: Desktop = Row, Mobile = Stack (Map + Overlay) -->
    <!-- Added pt-[65px] to account for fixed header -->
    <div class="relative flex-1 flex flex-col md:flex-row overflow-hidden pt-[65px]">
      <!-- Content Card / Sidebar
           Desktop: Static Sidebar (Left)
           Mobile: Bottom Sheet (Overlay)
      -->
      <div
        ref="sheetContainer"
        class="bg-zinc-50 z-20 flex flex-col border-zinc-200 transition-all duration-300 ease-in-out md:relative md:w-[60%] lg:w-[40%] xl:w-[35%] md:max-w-[40rem] md:h-full md:border-r md:shadow-none max-md:absolute max-md:inset-x-0 max-md:bottom-0 max-md:shadow-[0_-4px_6px_-1px_rgba(0,0,0,0.1)] max-md:rounded-t-2xl"
        :class="{
          'max-md:top-[65px]': mobileSheetState === 'up',
          'max-md:max-h-[50vh]': mobileSheetState === 'middle',
          'max-md:max-h-20': mobileSheetState === 'down',
        }"
      >
        <!-- Mobile Handle / Toggle -->
        <div
          class="md:hidden flex justify-center pt-2 pb-2 shrink-0 bg-zinc-50"
          @click="toggleMobileExpand"
          :class="{
            'cursor-grab': !isSwiping,
            'cursor-grabbing': isSwiping,
            'rounded-t-3xl': mobileSheetState !== 'up',
          }"
        >
          <div class="w-12 h-1.5 rounded-full" :class="isSwiping ? 'bg-zinc-500' : 'bg-zinc-300'"></div>
        </div>

        <!-- Scrollable Content -->
        <div ref="scrollContainer" class="overflow-y-auto flex-1 p-0 scrollbar-thin flex flex-col">
          <div class="shrink-0">
            <!-- Image Section -->
            <div v-if="data?.imgs?.length && data.imgs[0]" class="relative shrink-0">
              <button type="button" class="focusable block w-full" @click="slideshowOpen = true">
                <NuxtImg
                  :alt="t('image_alt')"
                  :src="`${runtimeConfig.public.cdnURL}/cdn/lg/${data.imgs[0].name}`"
                  class="bg-zinc-100 block md:h-64 w-full object-cover"
                  :class="mobileSheetState === 'up' ? 'h-32' : 'h-20'"
                  preload
                  placeholder
                  sizes="500px sm:600px"
                  densities="x1 x2"
                />
              </button>
            </div>
            <div
              v-else-if="!data?.imgs?.length"
              class="bg-zinc-100 shrink-0 group hover:border-zinc-400 hover:bg-zinc-200 border-2 rounded-2xl border-dashed border-zinc-300 md:m-2 md:mb-0"
              :class="mobileSheetState === 'up' ? 'px-2' : 'mt-1'"
            >
              <button
                type="button"
                class="w-full flex flex-col items-center justify-center text-zinc-500 group-hover:text-zinc-700 group-hover:border-zinc-400 transition-colors"
                :class="mobileSheetState === 'up' ? 'h-32' : 'h-20'"
                @click="suggestImage"
              >
                <MdiIcon :path="mdiPlus" :size="32" class="mb-2" />
                <span class="text-sm font-medium">{{ t("add_first_image") }}</span>
              </button>
            </div>
          </div>

          <!-- Content Padding -->
          <div class="px-5 pb-8 pt-4 bg-zinc-50">
            <!-- Breadcrumbs -->
            <BreadcrumbList
              :items="
                data.parent_names.map((n, i) => ({
                  name: n,
                  to: i > 0 ? '/view/' + data?.parents[i] : '/',
                }))
              "
              class="mb-2"
            />

            <!-- Title & Actions -->
            <div class="group flex py-1 rounded transition-colors flex-row items-center gap-2">
              <h1 class="text-zinc-800 text-2xl font-bold leading-tight">{{ data.name }}</h1>
              <button
                v-if="clipboardIsSupported"
                :title="t('header.copy_link')"
                type="button"
                class="hidden group-hover:block text-zinc-800"
                @click="copy(`https://nav.tum.de${route.fullPath}`)"
              >
                <MdiIcon :path="mdiClipboardCheck" :size="20" v-if="copied" />
                <MdiIcon :path="mdiLink" :size="20" v-else />
              </button>
            </div>

            <!-- Type & Buttons -->
            <div class="flex flex-wrap items-center justify-between gap-y-2 mb-6">
              <span class="text-zinc-500 text-sm font-medium">{{ data.type_common_name }}</span>
              <div class="flex flex-row items-center gap-3">
                <button
                  v-if="data.props?.calendar_url"
                  type="button"
                  class="focusable rounded-sm"
                  :title="t('header.calendar')"
                  @click="calendar = [...new Set([...calendar, route.params.id?.toString() ?? '404'])]"
                >
                  <MdiIcon :path="mdiCalendarMonth" :size="26" class="text-blue-600 hover:text-blue-900" />
                </button>
                <ShareButton :coords="data.coords" :name="data.name" :id="data.id" />
                <DetailsFeedbackButton />
              </div>
            </div>

            <!-- Toasts/Alerts -->
            <div class="flex flex-col gap-2 mb-4">
              <div
                v-if="data.coords.accuracy === 'building'"
                class="text-orange-900 bg-orange-50 border border-orange-200 rounded p-3 text-sm flex flex-col gap-2"
              >
                <span>{{ t("msg.inaccurate_only_building") }}</span>
                <button type="button" class="text-orange-700 hover:text-orange-900 text-xs font-bold uppercase self-start" @click="suggestLocationFix">
                  {{ t("suggest_edit") }}
                </button>
              </div>
              <Toast
                v-if="data.type === 'room' && data.maps?.overlays?.default === null"
                level="warning"
                :msg="t('msg.no_floor_overlay')"
                id="details-no_floor_overlay"
              />
              <Toast v-if="data.props.comment" :msg="data.props.comment" id="details-comment" />
            </div>

            <!-- Property Table -->
            <div class="mb-8">
              <DetailsPropertyTable :id="data.id" :props="data.props" :name="data.name" :navigation-enabled="data.coords.accuracy !== 'building'" />
            </div>

            <!-- Extra Sections -->
            <div class="flex flex-col gap-6">
              <DetailsBuildingOverviewSection :buildings="data.sections?.buildings_overview" />
              <ClientOnly>
                <LazyDetailsRoomOverviewSection :rooms="data.sections?.rooms_overview" />
              </ClientOnly>
              <DetailsSources
                :coords="data.coords"
                :sources="data.sources"
                :image="data.imgs?.length ? data.imgs[0] : undefined"
                class="text-xs text-zinc-400 mt-4"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Map Layer (Right/Background) -->
      <div
        v-if="data"
        class="absolute z-0 md:relative md:flex-1 w-full full-screen-map-wrapper"
        :class="{
          'max-md:bottom-[80px]': mobileSheetState === 'down',
          'max-md:bottom-[50vh]': mobileSheetState === 'middle',
          'max-md:bottom-0': mobileSheetState === 'up',
          'max-md:top-[65px]': true,
        }"
      >
        <ClientOnly>
          <DetailsInteractiveMap :id="data.id" :coords="data.coords" :type="data.type" :maps="data.maps" :floors="data.props.floors" class="h-full w-full" />
        </ClientOnly>
      </div>

      <!-- Loading State -->
      <div v-else class="absolute inset-0 z-10 flex items-center justify-center bg-zinc-50/80 backdrop-blur-sm">
        <div class="flex flex-col items-center gap-5">
          <Spinner class="h-8 w-8" />
          {{ t("Loading data...") }}
        </div>
      </div>
    </div>
  </div>

  <!-- Modals -->
  <ClientOnly>
    <LazyCalendarModal v-if="calendar.length" />
    <LazyEditProposalModal v-if="editProposal.open" />
    <LazyDetailsImageSlideshowModal
      v-if="slideshowOpen && !!data?.imgs"
      v-model:shown_image="shownImage"
      v-model:slideshow_open="slideshowOpen"
      :imgs="data.imgs"
    />
  </ClientOnly>
</template>

<style scoped>
/* Force MapLibre to fill height and remove padding hack from the component */
.full-screen-map-wrapper :deep(#interactive-legacy-map-container) {
  margin-bottom: 0 !important;
  height: 100% !important;
  width: 100% !important;
  aspect-ratio: auto !important;
}

.full-screen-map-wrapper :deep(#interactive-legacy-map-container > div) {
  padding-bottom: 0 !important;
  height: 100% !important;
}
</style>

<i18n lang="yaml">
de:
  image_alt: Header-Bild, zeigt das Gebäude
  details_for: Details für
  map:
    interactive: Interaktive Karte
    plans: Lagepläne
  header:
    calendar: Kalender öffnen
    copy_link: Link kopieren
    favorites: Zu Favoriten hinzufügen
  add_image: Bild hinzufügen
  add_first_image: Erstes Bild hinzufügen
  suggest_edit: Ich weiß wo es liegt
  Loading data...: Lädt Daten...
  msg:
    inaccurate_only_building: Die angezeigte Position zeigt nur die Position des Gebäude(teils). Die genaue Lage innerhalb des Gebäudes ist uns nicht bekannt.
    no_floor_overlay: Für den angezeigten Raum gibt es leider keine Indoor Karte.
en:
  image_alt: Header image, showing the building
  details_for: Details for
  map:
    interactive: Interactive Map
    plans: Site Plans
  header:
    calendar: Open calendar
    copy_link: Copy link
    favorites: Add to favorites
  add_image: Add image
  add_first_image: Add first image
  suggest_edit: I know where it is
  Loading data...: Loading data...
  msg:
    inaccurate_only_building: The displayed position only shows the position of the building(part). The exact position within the building is not known to us.
    no_floor_overlay: There is unfortunately no indoor map for the displayed room.
</i18n>
