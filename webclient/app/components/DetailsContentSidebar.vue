<script setup lang="ts">
import {
  mdiCalendarMonth,
  mdiClipboardCheck,
  mdiDirections,
  mdiLink,
  mdiPencil,
  mdiPlus,
  mdiShareVariant,
} from "@mdi/js";
import { useClipboard } from "@vueuse/core";
import type { components } from "~/api_types";
import type { DetailAction } from "~/components/DetailActionToolbar.vue";
import { emptyPropertyFields, useEditProposal } from "~/composables/editProposal";
import { categoryForEntity, FILTER_QUERY_PARAM } from "~/composables/mapLayers";
import { entityPath, isEntityType } from "~/utils/entityPath";
import { browseMapUrl } from "~/utils/map";

type LocationDetailsResponse = components["schemas"]["LocationDetailsResponse"];

const props = defineProps<{
  data: LocationDetailsResponse;
  mobileSheetState: "up" | "middle" | "down";
}>();

defineEmits(["openSlideshow"]);

const { t } = useI18n({ useScope: "local" });
const route = useRoute();
const runtimeConfig = useRuntimeConfig();
const editProposal = useEditProposal();
const calendar = useCalendar();

const navigationEnabled = computed(() => props.data.coords.accuracy !== "building");

const category = computed(() => categoryForEntity(props.data));
const exploreHereUrl = computed(() => browseMapUrl(props.data.coords, props.data.type));

const shareModalOpen = ref(false);

// Only ancestors with a routable type get a canonical /{type}/{id} link; the rest render as plain text.
// The current entity is appended as the `current` item so the trail is complete for screen readers.
const breadcrumbItems = computed(() => [
  ...props.data.parent_names.map((name, i) => {
    const id = props.data.parents[i];
    if (i === 0 || !id) return { name, to: "/" };
    const type = props.data.parent_types?.[i];
    return { name, to: type && isEntityType(type) ? entityPath(id, type) : undefined };
  }),
  { name: props.data.name, current: true },
]);

const clipboardSource = computed(() => `https://nav.tum.de${route.fullPath}`);
const {
  copy,
  copied,
  isSupported: clipboardIsSupported,
} = useClipboard({ source: clipboardSource });

const suggestImage = () => {
  if (!props.data) return;

  editProposal.value.selected = {
    id: props.data.id,
    name: props.data.name,
  };
  if (!editProposal.value.data.additional_context) {
    editProposal.value.data.additional_context = `I would like to suggest a new image for ${props.data.name} (${props.data.id}).`;
  }
  const floorIds = props.data.props.floors?.map((f) => f.id) ?? [];
  editProposal.value.locationPicker = {
    lat: props.data.coords.lat,
    lon: props.data.coords.lon,
    open: false,
    floors: floorIds,
    floor: floorIds[0] ?? null,
  };
  editProposal.value.open = true;
  editProposal.value.imageUpload.open = true;
};

const suggestEdit = () => {
  if (!props.data) return;

  editProposal.value.selected = {
    id: props.data.id,
    name: props.data.name,
  };
  const floorIds = props.data.props.floors?.map((f) => f.id) ?? [];
  editProposal.value.locationPicker = {
    lat: props.data.coords.lat,
    lon: props.data.coords.lon,
    open: false,
    floors: floorIds,
    floor: floorIds[0] ?? null,
  };

  // Start the name field empty rather than pre-filling `props.data.name`: that
  // is the decorated display name (`{id} ({name})`), not a curated name, so
  // pre-filling it would launder the generated string back into names.csv on
  // save. The current display name is offered as a read-only placeholder for
  // context, and a name edit is only sent once the user actually types one.
  editProposal.value.propertyFields = emptyPropertyFields();
  editProposal.value.originalPropertyFields = emptyPropertyFields();

  editProposal.value.open = true;
};

const suggestLocationFix = () => {
  if (!props.data) return;
  if (!editProposal.value.data.additional_context) {
    editProposal.value.data.additional_context = `The location for ${props.data.name} (${props.data.id}) is only accurate to building level. I can help provide a more precise location within the building.`;
  }
  editProposal.value.selected = {
    id: props.data.id,
    name: props.data.name,
  };
  const floorIds = props.data.props.floors?.map((f) => f.id) ?? [];
  editProposal.value.locationPicker = {
    lat: props.data.coords.lat,
    lon: props.data.coords.lon,
    open: true,
    floors: floorIds,
    floor: floorIds[0] ?? null,
  };
  editProposal.value.open = true;
};

const actions = computed<DetailAction[]>(() => [
  {
    key: "calendar",
    icon: mdiCalendarMonth,
    label: t("header.calendar"),
    shortLabel: t("header.calendar_short"),
    visible: Boolean(props.data.props?.calendar_url),
    onClick: () => {
      calendar.value = [...new Set([...calendar.value, route.params.id?.toString() ?? "404"])];
    },
  },
  {
    key: "navigation",
    icon: mdiDirections,
    label: t("header.start_navigation"),
    shortLabel: t("header.start_navigation_short"),
    visible: navigationEnabled.value,
    href: `/navigate?coming_from=${props.data.id}&coming_from_type=${props.data.type}&to=${props.data.id}&q_to=${props.data.name}`,
  },
  {
    key: "share",
    icon: mdiShareVariant,
    label: t("header.share"),
    shortLabel: t("header.share"),
    onClick: () => {
      shareModalOpen.value = true;
    },
  },
  {
    key: "suggest-change",
    icon: mdiPencil,
    label: t("header.suggest_edit"),
    shortLabel: t("header.suggest_edit_short"),
    onClick: suggestEdit,
  },
]);
</script>

<template>
  <div class="shrink-0">
    <!-- Image Section -->
    <div v-if="data?.imgs?.length && data.imgs[0]" class="relative shrink-0">
      <button type="button" class="focusable block w-full" @click="$emit('openSlideshow')">
        <NuxtImg
          :alt="t('image_alt')"
          :src="`${runtimeConfig.public.cdnURL}/cdn/lg/${data.imgs[0].name}`"
          class="bg-zinc-100 dark:bg-zinc-800 block md:h-64 w-full object-cover"
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
      class="bg-zinc-100 dark:bg-zinc-800 shrink-0 group hover:border-zinc-400 dark:hover:border-zinc-500 hover:bg-zinc-200 dark:hover:bg-zinc-700 border-2 rounded-2xl border-dashed border-zinc-300 dark:border-zinc-600 md:m-2 md:mb-0"
      :class="mobileSheetState === 'up' ? 'px-2' : 'mt-1'"
    >
      <button
        type="button"
        class="w-full flex flex-col items-center justify-center text-zinc-500 dark:text-zinc-400 group-hover:text-zinc-700 dark:group-hover:text-zinc-200 group-hover:border-zinc-400 dark:group-hover:border-zinc-500 transition-colors"
        :class="mobileSheetState === 'up' ? 'h-32' : 'h-20'"
        @click="suggestImage"
      >
        <MdiIcon :path="mdiPlus" :size="32" class="mb-2"/>
        <span class="text-sm font-medium">{{ t("add_first_image") }}</span>
      </button>
    </div>
  </div>

  <!-- Content Padding -->
  <div class="px-5 pb-8 pt-4 bg-zinc-50 dark:bg-zinc-900">
    <!-- Breadcrumbs -->
    <BreadcrumbList :items="breadcrumbItems" class="mb-2" />

    <!-- Title & Actions -->
    <div class="group flex py-1 rounded transition-colors flex-row items-center gap-2">
      <h1 class="text-zinc-800 dark:text-zinc-100 text-2xl font-bold leading-tight">{{ data.name }}</h1>
      <button
        v-if="clipboardIsSupported"
        :title="t('header.copy_link')"
        type="button"
        class="hidden group-hover:block text-zinc-800 dark:text-zinc-100"
        @click="copy(`https://nav.tum.de${route.fullPath}`)"
      >
        <MdiIcon :path="mdiClipboardCheck" :size="20" v-if="copied"/>
        <MdiIcon :path="mdiLink" :size="20" v-else/>
      </button>
    </div>

    <!-- Type -->
    <div class="flex flex-wrap items-baseline justify-between gap-2 mb-3">
      <span class="text-zinc-500 dark:text-zinc-400 text-sm font-medium">{{ data.type_common_name }}</span>
      <MapBridgeLink v-if="category" :to="`/map?${FILTER_QUERY_PARAM}=${category}`">
        {{ t(`category_bridge.${category}`) }}
      </MapBridgeLink>
    </div>

    <ShareModal v-model:open="shareModalOpen" :coords="data.coords" :name="data.name" :id="data.id"/>

    <!-- Toasts/Alerts -->
    <div class="flex flex-col gap-2 mb-4">
      <div
        v-if="data.coords.accuracy === 'building'"
        class="text-orange-900 dark:text-orange-50 bg-orange-50 dark:bg-orange-900 border border-orange-200 dark:border-orange-700 rounded p-3 text-sm flex flex-col gap-2"
      >
        <span>{{ t("msg.inaccurate_only_building") }}</span>
        <button type="button" class="text-orange-700 dark:text-orange-200 hover:text-orange-900 dark:hover:text-orange-50 text-xs font-bold uppercase self-start"
                @click="suggestLocationFix">
          {{ t("suggest_edit") }}
        </button>
      </div>
      <Toast
        v-if="data.type === 'room' && data.maps?.overlays?.default === null"
        level="warning"
        :msg="t('msg.no_floor_overlay')"
        id="details-no_floor_overlay"
      />
      <Toast v-if="data.props.comment" :msg="data.props.comment" id="details-comment"/>
    </div>

    <!-- Property Table -->
    <div class="mb-6">
      <DetailsPropertyTable :props="data.props"/>
    </div>

    <!-- Action bar -->
    <DetailActionToolbar :actions="actions" class="mb-8"/>

    <!-- Extra Sections -->
    <div class="flex flex-col gap-6">
      <DetailsBuildingOverviewSection :buildings="data.sections?.buildings_overview" :browse-map-url="exploreHereUrl"/>
      <ClientOnly>
        <LazyDetailsOpeningHoursCard
          v-if="data.opening_hours"
          :opening-hours="data.opening_hours"
        />
        <LazyDetailsMensaMenuCard
          v-if="data.mensa_canteen_id"
          :slug="data.mensa_canteen_id"
        />
        <LazyDetailsNearbyTransportSection :id="data.id"/>
        <LazyDetailsIrisCoverageCard
          v-if="data.props.iris_coverage_building_ids?.length"
          :building-ids="data.props.iris_coverage_building_ids"
        />
        <LazyDetailsRoomOverviewSection :rooms="data.sections?.rooms_overview" :browse-map-url="exploreHereUrl"/>
      </ClientOnly>
      <DetailsSources
        :coords="data.coords"
        :sources="data.sources"
        :image="data.imgs?.length ? data.imgs[0] : undefined"
        class="text-xs text-zinc-400 dark:text-zinc-500 mt-4"
      />
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  image_alt: Header-Bild, zeigt das Gebäude
  category_bridge:
    wcs: Alle Toiletten auf der Karte anzeigen
  header:
    calendar: Kalender öffnen
    calendar_short: Kalender
    copy_link: Link kopieren
    share: Teilen
    start_navigation: Navigation starten
    start_navigation_short: Navigation
    suggest_edit: Änderung vorschlagen
    suggest_edit_short: Bearbeiten
  add_first_image: Erstes Bild hinzufügen
  suggest_edit: Ich weiß wo es liegt
  msg:
    inaccurate_only_building: Die angezeigte Position zeigt nur die Position des Gebäude(teils). Die genaue Lage innerhalb des Gebäudes ist uns nicht bekannt.
    no_floor_overlay: Für den angezeigten Raum gibt es leider keine Indoor Karte.
en:
  image_alt: Header image, showing the building
  category_bridge:
    wcs: Show all toilets on the map
  header:
    calendar: Open calendar
    calendar_short: Calendar
    copy_link: Copy link
    share: Share
    start_navigation: Start navigation
    start_navigation_short: Navigate
    suggest_edit: Suggest a change
    suggest_edit_short: Edit
  add_first_image: Add first image
  suggest_edit: I know where it is
  msg:
    inaccurate_only_building: The displayed position only shows the position of the building(part). The exact position within the building is not known to us.
    no_floor_overlay: There is unfortunately no indoor map for the displayed room.
</i18n>
