<script setup lang="ts">
import { useDetailsStore } from "@/stores/details";
import DetailsImageSlideshowModal from "@/components/DetailsImageSlideshowModal.vue";
import DetailsPropertyTable from "@/components/DetailsPropertyTable.vue";
import { useI18n } from "vue-i18n";

const state = useDetailsStore();
const { t } = useI18n({ useScope: "local" });
const appURL = import.meta.env.VITE_APP_URL;
</script>

<template>
  <!-- Information section (on mobile) -->
  <div v-if="state.data?.props?.computed" class="column col-5 col-sm-12 show-sm mt-4">
    <h2>{{ t("info_title") }}</h2>
    <DetailsPropertyTable />
  </div>

  <!-- Informationen card (desktop) -->
  <!-- Some elements are currently duplicate, which is not optimal but should be okay
       as long as only little information is there -->
  <div class="col-5 col-md-12 column hide-sm">
    <div class="card">
      <a
        v-if="state.image.shown_image"
        class="c-hand card-image"
        @click="state.showImageSlideshow(state.image.shown_image_id || 0)"
      >
        <img
          :alt="t('image_alt')"
          :src="`${appURL}/cdn/header/${state.image.shown_image.name}`"
          class="bg-zinc-100 block h-auto max-w-full w-full"
        />
      </a>
      <div class="card-header">
        <div class="card-title h5">{{ t("info_title") }}</div>
      </div>
      <div class="card-body">
        <DetailsPropertyTable />
        <div v-if="state.data?.coords.accuracy === 'building'" class="toast toast-warning mt-3">
          {{ t("msg.inaccurate_only_building") }}<br />
        </div>
        <div
          v-if="state.data?.type === 'room' && state.data?.maps?.overlays?.default === null"
          class="toast toast-warning mt-3"
        >
          {{ t("msg.no_floor_overlay") }}
        </div>
        <div v-if="state.data?.props?.comment" class="toast mt-3">
          {{ state.data.props.comment }}
        </div>
      </div>
      <!-- <div class="card-footer">
          <button class="btn btn-link">Mehr Infos</button>
      </div> -->
    </div>
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
