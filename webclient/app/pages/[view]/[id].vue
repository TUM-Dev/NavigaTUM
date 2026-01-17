<script setup lang="ts">
import { useSwipe } from "@vueuse/core";
import type { components } from "~/api_types";
import DetailsContentSidebar from "~/components/DetailsContentSidebar.vue";
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

const expandSheet = () => {
  if (mobileSheetState.value === "down") {
    mobileSheetState.value = "middle";
  } else if (mobileSheetState.value === "middle") {
    mobileSheetState.value = "up";
  }
};

const collapseSheet = () => {
  if (mobileSheetState.value === "up") {
    mobileSheetState.value = "middle";
  } else if (mobileSheetState.value === "middle") {
    mobileSheetState.value = "down";
  }
};

const { isSwiping } = useSwipe(sheetContainer, {
  threshold: 30,
  onSwipeEnd: (_e, direction) => {
    const scroll = scrollContainer.value?.scrollTop;
    if (direction === "up") {
      expandSheet();
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
        <button
          type="button"
          class="md:hidden flex w-full justify-center pt-2 pb-2 shrink-0 bg-zinc-50"
          :aria-expanded="mobileSheetState === 'up' ? true : mobileSheetState === 'middle' ? true : false"
          aria-controls="sheet-content"
          :aria-label="t('Toggle details sheet')"
          @click="toggleMobileExpand"
          @keydown.arrow-up.prevent="expandSheet"
          @keydown.arrow-down.prevent="collapseSheet"
          :class="{
            'cursor-grab': !isSwiping,
            'cursor-grabbing': isSwiping,
            'rounded-t-3xl': mobileSheetState !== 'up',
          }"
        >
          <div class="w-12 h-1.5 rounded-full" :class="isSwiping ? 'bg-zinc-500' : 'bg-zinc-300'"></div>
        </button>

        <!-- Scrollable Content -->
        <div id="sheet-content" ref="scrollContainer" class="overflow-y-auto flex-1 p-0 scrollbar-thin flex flex-col">
          <DetailsContentSidebar v-if="data" :data="data" :mobile-sheet-state="mobileSheetState" @open-slideshow="slideshowOpen = true" />
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
  details_for: Details für
  Loading data...: Lädt Daten...
  Toggle details sheet: Detailansicht umschalten
en:
  details_for: Details for
  Loading data...: Loading data...
  Toggle details sheet: Toggle details sheet
</i18n>
