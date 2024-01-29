<script setup lang="ts">
import { useDetailsStore } from "@/stores/details";
import DetailsImageSlideshowModal from "@/components/DetailsImageSlideshowModal.vue";
import DetailsPropertyTable from "@/components/DetailsPropertyTable.vue";
import Toast from "@/components/Toast.vue";
import { useI18n } from "vue-i18n";

const state = useDetailsStore();
const { t } = useI18n({ useScope: "local" });
const appURL = import.meta.env.VITE_APP_URL;
</script>

<template>
  <!-- Information section (on mobile) -->
  <div v-if="state.data?.props?.computed" class="col-5 col-sm-12 column mt-4 block lg:hidden">
    <h2>{{ t("info_title") }}</h2>
    <DetailsPropertyTable />
  </div>

  <!-- Informationen card (desktop) -->
  <!-- Some elements are currently duplicate, which is not optimal but should be okay
       as long as only little information is there -->
  <div class="hidden lg:block">
    <div
      class="max-w-sm rounded-lg border border-gray-200 bg-white shadow-md shadow-neutral-500/5 dark:border-gray-700 dark:bg-gray-800 dark:shadow-white/20"
    >
      <button v-if="state.image.shown_image" type="button" @click="state.showImageSlideshow(true)">
        <img
          :alt="t('image_alt')"
          :src="`${appURL}/cdn/header/${state.image.shown_image.name}`"
          class="block h-auto w-full max-w-full rounded-t-lg bg-zinc-100"
        />
      </button>
      <div class="p-5">
        <h3 class="text-lg font-semibold">{{ t("info_title") }}</h3>
        <DetailsPropertyTable />
        <div class="mt-3 grid gap-2">
          <Toast
            v-if="state.data?.coords.accuracy === 'building'"
            level="warning"
            :msg="t('msg.inaccurate_only_building')"
          />
          <Toast
            v-if="state.data?.type === 'room' && state.data?.maps?.overlays?.default === null"
            level="warning"
            :msg="t('msg.no_floor_overlay')"
          />
          <Toast v-if="state.data?.props?.comment" :msg="state.data.props.comment" />
        </div>
      </div>
    </div>
    <!-- <button class="btn btn-link">Mehr Infos</button> -->
  </div>
  <DetailsImageSlideshowModal v-if="state.image.slideshow_open" />
</template>

<i18n lang="yaml">
de:
  image_alt: Header-Bild, zeigt das Gebäude
  info_title: Informationen
  msg:
    inaccurate_only_building: Die angezeigte Position zeigt nur die Position des Gebäude(teils). Die genaue Lage innerhalb des Gebäudes ist uns nicht bekannt.
    no_floor_overlay: Für den angezeigten Raum gibt es leider keine Indoor Karte.
en:
  image_alt: Header image, showing the building
  info_title: Information
  msg:
    inaccurate_only_building: The displayed position only shows the position of the building(part). The exact position within the building is not known to us.
    no_floor_overlay: There is unfortunately no indoor map for the displayed room.
</i18n>
