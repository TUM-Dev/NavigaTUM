<script setup lang="ts">
import { selectedMap, useDetailsStore } from "@/stores/details";
import { useI18n } from "vue-i18n";
import Modal from "@/components/Modal.vue";
import RoomfinderImageLocation from "@/components/RoomfinderImageLocation.vue";
import { ref } from "vue";

const state = useDetailsStore();
const { t } = useI18n({ useScope: "local" });

defineExpose({
  loadRoomfinderMap,
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

  if (fromUi) {
    const accordion = document.getElementById("map-accordion") as HTMLInputElement;
    accordion.checked = false;
    /* window.setTimeout(() => {
                    document.getElementById("roomfinder-map-img").scrollIntoView(false);
                }, 50); */
    window.scrollTo(0, 0);
  }
}

const modalOpen = ref(false);
</script>

<template>
  <template v-if="state.data?.maps.roomfinder?.available">
    <a :aria-label="t('roomfinder.open_detailed_modal')" @click="modalOpen = true">
      <RoomfinderImageLocation
        v-if="state.map.selected === selectedMap.roomfinder"
        id="rf_outer_image"
        :map="state.selectedRoomfinderMap()"
      />
    </a>

    <div
      id="roomfinder-map-select"
      class="accordion"
      :class="{ hidden: state.map.selected !== selectedMap.roomfinder }"
    >
      <input id="map-accordion" type="checkbox" name="accordion-checkbox" hidden />
      <label for="map-accordion" class="accordion-header btn btn-block btn-sm">
        1:{{ state.selectedRoomfinderMap().scale }}, {{ state.selectedRoomfinderMap().name }}
        <i class="icon icon-caret" />
      </label>
      <div v-if="state.data.maps?.roomfinder" class="accordion-body">
        <ul class="menu menu-nav">
          <li v-for="(m, i) in state.data.maps.roomfinder.available" :key="m.id" class="menu-item">
            <button
              type="button"
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

    <Modal v-model="modalOpen" :title="t('roomfinder.modal.header')" :classes="{ modal: 'items-baseline' }">
      <RoomfinderImageLocation id="rf_modal_image" :map="state.selectedRoomfinderMap()" /> </Modal
  ></template>
</template>

<i18n lang="yaml">
de:
  img_alt: Handgezeichnete Roofinder-Kartendarstellung
  roomfinder:
    open_detailed_modal: Show a larger popup of the map
    modal:
      header: Lageplan
en:
  img_alt: Hand-drawn roomfinder map image
  roomfinder:
    open_detailed_modal: Ein größeres Popup der Karte anzeigen
    modal:
      header: Site Plan
</i18n>
