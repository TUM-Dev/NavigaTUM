<script setup lang="ts">
import { selectedMap, useDetailsStore } from "@/stores/details";
import {useI18n} from "vue-i18n";

const state = useDetailsStore();
const { t } = useI18n({ useScope: "local" });

defineExpose({
  loadRoomfinderMap,
  loadRoomfinderModalMap,
});
function loadRoomfinderMap(mapIndex: number, fromUi = false) {
  const map = state.data?.maps.roomfinder?.available[mapIndex];
  if (!map) {
    console.error({
      data: state.data,
      code: "invalid state for roomfinder load",
    });
    return;
  }
  state.map.selected = selectedMap.roomfinder;
  state.map.roomfinder.selected_id = map.id;
  state.map.roomfinder.selected_index = mapIndex;

  // Using the #map-container since the bounding rect is still all zero
  // if we switched here from interactive map
  const container = document.getElementById("map-container") as HTMLDivElement;
  const rect = container.getBoundingClientRect();
  // -1023px, -1023px is top left corner, 16px = 2*8px is element padding
  state.map.roomfinder.x = -1023 + (map.x / map.width) * (rect.width - 16);

  // We cannot use "height" here as it might be still zero before layouting
  // finished, so we use the aspect ratio here.
  state.map.roomfinder.y = -1023 + (map.y / map.height) * (rect.width - 16) * (map.height / map.width);

  state.map.roomfinder.width = map.width;
  state.map.roomfinder.height = map.height;

  if (fromUi) {
    const accordion = document.getElementById("map-accordion") as HTMLInputElement;
    accordion.checked = false;
    /* window.setTimeout(() => {
                    document.getElementById("roomfinder-map-img").scrollIntoView(false);
                }, 50); */
    window.scrollTo(0, rect.top + state.map.roomfinder.y + 1023 - window.innerHeight / 2);
  }
}

function loadRoomfinderModalMap() {
  const map = state.selectedRoomfinderMap();
  if (!map) return;

  const width = document.getElementById("roomfinder-modal-container")?.getBoundingClientRect().width || 100;
  // -1023px, -1023px is top left corner, 16px = 2*8px is element padding
  state.map.roomfinder.modalX = -1023 + (map.x / map.width) * (width - 65);

  // We cannot use "height" here as it might be still zero before layouting
  // finished, so we use the aspect ratio here.
  state.map.roomfinder.modalY = -1023 + (map.y / map.height) * (width - 65) * (map.height / map.width);
}
function delayedLoadRoomfinderModalMap() {
  setTimeout(loadRoomfinderModalMap, 1000);
}
</script>

<template>
  <a
    @click="state.map.roomfinder.modal_open = true"
    v-on:click="delayedLoadRoomfinderModalMap"
    :aria-label="t('roomfinder.open_detailed_modal')"
  >
    <div
      class="roomfinder-map-container"
      :class="{ 'd-none': state.map.selected !== selectedMap.roomfinder }"
      v-if="state.data?.maps.roomfinder?.available"
    >
      <img
        :alt="t('roomfinder.crosshair')"
        src="@/assets/map/roomfinder_cross-v2.webp"
        :style="{
          transform: `translate(${state.map.roomfinder.x}px, ${state.map.roomfinder.y}px)`,
        }"
        id="roomfinder-map-cross"
      />
      <img
        :alt="t('view_view.map.img_alt')"
        :src="'/cdn/maps/roomfinder/' + state.selectedRoomfinderMap().file"
        class="img-responsive"
        :width="state.map.roomfinder.width"
        :height="state.map.roomfinder.height"
        id="roomfinder-map-img"
      />
      <div>
        {{ t("img_source") }}:
        {{ state.selectedRoomfinderMap().source }}
      </div>
    </div>
  </a>
  <div
    class="accordion"
    id="roomfinder-map-select"
    :class="{ 'd-none': state.map.selected !== selectedMap.roomfinder }"
    v-if="state.data?.maps.roomfinder?.available"
  >
    <input id="map-accordion" type="checkbox" name="accordion-checkbox" hidden />
    <label for="map-accordion" class="btn btn-sm btn-block accordion-header">
      1:{{ state.selectedRoomfinderMap().scale }}, {{ state.selectedRoomfinderMap().name }}
      <i class="icon icon-caret" />
    </label>
    <div class="accordion-body" v-if="state.data.maps?.roomfinder">
      <ul class="menu menu-nav">
        <li class="menu-item" v-for="(m, i) in state.data.maps.roomfinder.available" :key="m.id">
          <button
            class="btn btn-sm"
            :aria-label="`show the map '${m.name}' at the scale 1:${m.scale}`"
            :class="{
              selected: m.id === state.map.roomfinder.selected_id,
            }"
            @click="loadRoomfinderMap(i, true)"
          >
            1:{{ m.scale }}, {{ m.name }}
          </button>
        </li>
      </ul>
    </div>
  </div>

  <!-- roomfinder-modal -->
  <div
    class="modal modal-lg"
    id="roomfinder-modal"
    :class="{ active: state.map.roomfinder.modal_open }"
    v-if="state.data?.maps.roomfinder?.available"
  >
    <a class="modal-overlay" :aria-label="t('close')" @click="state.map.roomfinder.modal_open = false" />
    <div class="modal-container modal-fullheight" id="roomfinder-modal-container">
      <div class="modal-header">
        <button
          class="btn btn-clear float-right"
          :aria-label="t('close')"
          @click="state.map.roomfinder.modal_open = false"
        />
        <h5 class="modal-title">{{ t("roomfinder.modal.header") }}</h5>
      </div>
      <div class="modal-body">
        <div class="roomfinder-map-container">
          <img
            :alt="t('roomfinder.crosshair')"
            src="@/assets/map/roomfinder_cross-v2.webp"
            :style="{
              transform: `translate(${state.map.roomfinder.modalX}px, ${state.map.roomfinder.modalY}px)`,
            }"
            id="roomfinder-modal-map-cross"
          />
          <img
            :alt="t('roomfinder.modal.img_alt')"
            :src="'/cdn/maps/roomfinder/' + state.selectedRoomfinderMap().file"
            class="img-responsive"
            :width="state.map.roomfinder.width"
            :height="state.map.roomfinder.height"
            id="roomfinder-modal-map-img"
          />
          <div>
            {{ t("img_source") }}:
            {{ state.selectedRoomfinderMap().source }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
@import "@/assets/variables";

/* --- Roomfinder display --- */
.roomfinder-map-container {
  overflow: hidden;
  position: relative;
  margin-bottom: 6px;

  > div {
    // Image source label
    position: absolute;
    bottom: 1px;
    right: 1px;
    padding: 1px 5px;
    color: $body-font-color;
    background-color: $container-loading-bg;
    font-size: 10px;
  }

  #roomfinder-map-cross {
    position: absolute;
    transition: transform 0.3s;
    pointer-events: none;
  }

  #roomfinder-map-img {
    width: 100%;
    display: block;
    cursor: pointer;
  }
}

#roomfinder-map-select > label {
  padding: 0.05rem 0.3rem;
}

.accordion-body {
  ul,
  button,
  li {
    font-size: 12px;
  }

  .selected {
    background: $roomfinder-selected-bg;
  }
}

/* --- Roomfinder Modal, shown on click on a roomfinder map --- */
#roomfinder-modal {
  align-items: baseline;

  & .modal-container {
    position: relative;
    padding-bottom: 4em;
    top: 5em;
  }

  #roomfinder-modal-map-cross {
    position: absolute;
    transition: transform 0.3s;
    pointer-events: none;
  }

  #roomfinder-modal-map-img {
    width: 100%; // Without this part the image doesn't fill the modal over the whole width.
  }
}
</style>
<i18n>
de:
  close: Schließen
  img_source: Bildquelle
  img_alt: Handgezeichnete Roofinder-Kartendarstellung
  roomfinder:
    open_detailed_modal: Show a larger popup of the map
    modal:
      header: Lageplan
      img_alt: Bild des Lageplans
    crosshair: Kreuz, das anzeigt, wo sich der Raum auf dem handgezeichneten Raumfinderkartenbild befindet
en:
  close: Close
  img_source: Image source
  img_alt: Hand-drawn roomfinder map image
  roomfinder:
    open_detailed_modal: Ein größeres Popup der Karte anzeigen
    modal:
      header: Site Plan
      img_alt: Image showing the Site Plan
    crosshair: Cross showing where the room is located on the hand-drawn roomfinder map image
</i18n>
