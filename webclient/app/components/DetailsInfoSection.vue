<script setup lang="ts">
import { mdiPlus } from "@mdi/js";
import type { components } from "~/api_types";
import { useEditProposal } from "~/composables/editProposal";

type LocationDetailsResponse = components["schemas"]["LocationDetailsResponse"];
type ImageInfoResponse = components["schemas"]["ImageInfoResponse"];

const props = defineProps<{ data: LocationDetailsResponse }>();

const shownImage = defineModel<ImageInfoResponse>("shown_image");
const slideshowOpen = defineModel<boolean>("slideshow_open", {
  required: true,
});
const { t } = useI18n({ useScope: "local" });
const runtimeConfig = useRuntimeConfig();
const editProposal = useEditProposal();

const suggestImage = () => {
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
  <!-- Information section (on mobile) -->
  <div v-if="data.props?.computed" class="col-5 col-sm-12 column mt-4 block lg:hidden">
    <h2 class="text-zinc-800 pb-3 text-lg font-semibold">
      {{ t("info_title") }}
    </h2>
    <DetailsPropertyTable :id="data.id" :props="data.props" :name="data.name" :navigation-enabled="data.coords.accuracy !== 'building'" />
  </div>

  <!-- Informationen card (desktop) -->
  <!-- Some elements are currently duplicate, which is not optimal but should be okay
       as long as only little information is there -->
  <div class="hidden lg:block">
    <div class="bg-white border-zinc-200 max-w-sm rounded-lg border shadow-md shadow-zinc-500/5 dark:bg-zinc-100">
      <div v-if="data.imgs?.length && data.imgs[0]" class="relative rounded-t-lg">
        <button type="button" class="focusable block w-full rounded-t-lg" @click="slideshowOpen = true">
          <NuxtImg
            :alt="t('image_alt')"
            :src="`${runtimeConfig.public.cdnURL}/cdn/lg/${data.imgs[0].name}`"
            class="bg-zinc-100 block h-auto w-full max-w-full rounded-t-lg"
            preload
            placeholder
            sizes="500px sm:200px md:300px md:400px"
            densities="x1 x2"
          />
        </button>
      </div>
      <div v-else-if="!data.imgs?.length" class="bg-zinc-100 group hover:border-zinc-400 hover:bg-zinc-200 border-2 border-dashed border-zinc-300 rounded-t-lg">
        <button
          type="button"
          class="w-full h-24 flex flex-col items-center justify-center text-zinc-500 group-hover:text-zinc-700 group-hover:border-zinc-400"
          @click="suggestImage"
        >
          <MdiIcon :path="mdiPlus" :size="32" class="mb-2" />
          <span class="text-sm font-medium">{{ t("add_first_image") }}</span>
        </button>
      </div>
      <div class="px-5 py-3">
        <span class="sr-only">{{ t("info_title") }}</span>
        <DetailsPropertyTable v-if="data" :id="data.id" :props="data.props" :name="data.name" :navigation-enabled="data.coords.accuracy !== 'building'" />
        <div class="mt-3 grid gap-2">
          <div
            v-if="data.coords.accuracy === 'building'"
            class="text-orange-900 flex-col bg-orange-100 border-orange-300 text-pretty rounded border p-1.5 text-sm leading-5 flex justify-between items-start"
          >
            <span>{{ t("msg.inaccurate_only_building") }}</span>
            <button type="button" class="pt-2 text-orange-600 hover:text-orange-800 text-sm font-medium ml-2" @click="suggestLocationFix">
              {{ t("suggest_edit") }}
            </button>
          </div>
          <Toast v-if="data.type === 'room' && data.maps?.overlays?.default === null" level="warning" :msg="t('msg.no_floor_overlay')" id="details-section-no_floor_overlay" />
          <Toast v-if="data.props?.comment" :msg="data.props.comment" id="details-section-comment" />
        </div>
      </div>
    </div>
    <!-- <button class="btn btn-link">Mehr Infos</button> -->
  </div>
  <ClientOnly>
    <LazyDetailsImageSlideshowModal
      v-if="slideshow_open && !!data.imgs"
      v-model:shown_image="shownImage"
      v-model:slideshow_open="slideshowOpen"
      :imgs="data.imgs"
    />
  </ClientOnly>
</template>

<i18n lang="yaml">
de:
  image_alt: Header-Bild, zeigt das Gebäude
  info_title: Informationen
  add_image: Bild hinzufügen
  add_first_image: Erstes Bild hinzufügen
  suggest_edit: Ich weiß wo es liegt
  msg:
    inaccurate_only_building: Die angezeigte Position zeigt nur die Position des Gebäude(teils). Die genaue Lage innerhalb des Gebäudes ist uns nicht bekannt.
    no_floor_overlay: Für den angezeigten Raum gibt es leider keine Indoor Karte.
en:
  image_alt: Header image, showing the building
  info_title: Information
  add_image: Add image
  add_first_image: Add first image
  suggest_edit: I know where it is
  msg:
    inaccurate_only_building: The displayed position only shows the position of the building(part). The exact position within the building is not known to us.
    no_floor_overlay: There is unfortunately no indoor map for the displayed room.
</i18n>
