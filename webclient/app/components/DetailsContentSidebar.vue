<script setup lang="ts">
import { mdiCalendarMonth, mdiClipboardCheck, mdiLink, mdiPlus } from "@mdi/js";
import { useClipboard } from "@vueuse/core";
import type { components } from "~/api_types";
import { useEditProposal } from "~/composables/editProposal";

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
  editProposal.value.locationPicker = {
    lat: props.data.coords.lat,
    lon: props.data.coords.lon,
    open: false,
  };
  editProposal.value.open = true;
  editProposal.value.imageUpload.open = true;
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
  editProposal.value.locationPicker = {
    lat: props.data.coords.lat,
    lon: props.data.coords.lon,
    open: true,
  };
  editProposal.value.open = true;
};
</script>

<template>
  <div class="shrink-0">
    <!-- Image Section -->
    <div v-if="data?.imgs?.length && data.imgs[0]" class="relative shrink-0">
      <button type="button" class="focusable block w-full" @click="$emit('openSlideshow')">
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
</template>

<i18n lang="yaml">
de:
  image_alt: Header-Bild, zeigt das Gebäude
  header:
    calendar: Kalender öffnen
    copy_link: Link kopieren
  add_first_image: Erstes Bild hinzufügen
  suggest_edit: Ich weiß wo es liegt
  msg:
    inaccurate_only_building: Die angezeigte Position zeigt nur die Position des Gebäude(teils). Die genaue Lage innerhalb des Gebäudes ist uns nicht bekannt.
    no_floor_overlay: Für den angezeigten Raum gibt es leider keine Indoor Karte.
en:
  image_alt: Header image, showing the building
  header:
    calendar: Open calendar
    copy_link: Copy link
  add_first_image: Add first image
  suggest_edit: I know where it is
  msg:
    inaccurate_only_building: The displayed position only shows the position of the building(part). The exact position within the building is not known to us.
    no_floor_overlay: There is unfortunately no indoor map for the displayed room.
</i18n>