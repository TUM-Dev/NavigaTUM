<script setup lang="ts">
import { selectedMap, useDetailsStore } from "@/stores/details";

const state = useDetailsStore();

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
  const rect = document.getElementById("map-container")?.getBoundingClientRect();
  // -1023px, -1023px is top left corner, 16px = 2*8px is element padding
  state.map.roomfinder.x = -1023 + (map.x / map.width) * (rect.width - 16);

  // We cannot use "height" here as it might be still zero before layouting
  // finished, so we use the aspect ratio here.
  state.map.roomfinder.y = -1023 + (map.y / map.height) * (rect.width - 16) * (map.height / map.width);

  state.map.roomfinder.width = map.width;
  state.map.roomfinder.height = map.height;

  if (fromUi) {
    document.getElementById("map-accordion").checked = false;
    /* window.setTimeout(() => {
                    document.getElementById("roomfinder-map-img").scrollIntoView(false);
                }, 50); */
    window.scrollTo(0, rect.top + state.map.roomfinder.y + 1023 - window.innerHeight / 2);
  }
}

function loadRoomfinderModalMap() {
  const map = state.data.maps.roomfinder.available[state.map.roomfinder.selected_index];

  const rect = document.getElementById("roomfinder-modal-container").getBoundingClientRect();
  // -1023px, -1023px is top left corner, 16px = 2*8px is element padding
  state.map.roomfinder.modalX = -1023 + (map.x / map.width) * (rect.width - 65);

  // We cannot use "height" here as it might be still zero before layouting
  // finished, so we use the aspect ratio here.
  state.map.roomfinder.modalY = -1023 + (map.y / map.height) * (rect.width - 65) * (map.height / map.width);
}
function delayedLoadRoomfinderModalMap() {
  setTimeout(loadRoomfinderModalMap, 1000);
}
</script>

<template>
  <a @click="state.map.roomfinder.modal_open = true" v-on:click="delayedLoadRoomfinderModalMap">
    <div
      class="roomfinder-map-container"
      v-bind:class="{ 'd-none': state.map.selected !== selectedMap.roomfinder }"
      v-if="state.data.maps.roomfinder?.available"
    >
      <img
        alt="Cross showing where the room is located on the hand-drawn roomfinder map image"
        src="../assets/map/roomfinder_cross-v2.webp"
        v-bind:style="{
          transform: 'translate(' + state.map.roomfinder.x + 'px, ' + state.map.roomfinder.y + 'px)',
        }"
        id="roomfinder-map-cross"
      />
      <img
        alt="Hand-drawn roomfinder map image"
        v-bind:src="
          '/cdn/maps/roomfinder/' + state.data.maps.roomfinder.available[state.map.roomfinder.selected_index].file
        "
        class="img-responsive"
        v-bind:width="state.map.roomfinder.width"
        v-bind:height="state.map.roomfinder.height"
        id="roomfinder-map-img"
      />
      <div>
        {{ $t("view_view.map.img_source") }}:
        {{ state.data.maps.roomfinder.available[state.map.roomfinder.selected_index].source }}
      </div>
    </div>
  </a>
  <div
    class="accordion"
    id="roomfinder-map-select"
    v-bind:class="{ 'd-none': state.map.selected !== selectedMap.roomfinder }"
    v-if="state.data.maps.roomfinder?.available"
  >
    <input id="map-accordion" type="checkbox" name="accordion-checkbox" hidden />
    <label for="map-accordion" class="btn btn-sm btn-block accordion-header">
      1:{{ state.data.maps.roomfinder.available[state.map.roomfinder.selected_index].scale }},
      {{ state.data.maps.roomfinder.available[state.map.roomfinder.selected_index].name }}
      <i class="icon icon-caret"></i>
    </label>
    <div class="accordion-body" v-if="state.data.maps?.roomfinder">
      <ul class="menu menu-nav">
        <li class="menu-item" v-for="(m, i) in state.data.maps.roomfinder.available">
          <button
            class="btn btn-sm"
            v-bind:aria-label="`show the map '` + m.name + `' at the scale 1:` + m.scale"
            v-bind:class="{
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
  <div class="modal modal-lg" id="roomfinder-modal" v-bind:class="{ active: state.map.roomfinder.modal_open }">
    <a
      class="modal-overlay"
      v-bind:aria-label="$t('view_view.roomfinder_modal.close')"
      @click="state.map.roomfinder.modal_open = false"
    >
    </a>
    <div class="modal-container modal-fullheight" id="roomfinder-modal-container">
      <div class="modal-header">
        <button
          class="btn btn-clear float-right"
          v-bind:aria-label="$t('view_view.roomfinder_modal.close')"
          @click="state.map.roomfinder.modal_open = false"
        ></button>
        <h5 class="modal-title">
          {{ $t("view_view.roomfinder_modal.header") }}
        </h5>
      </div>
      <div class="modal-body">
        <div class="roomfinder-map-container">
          <img
            alt="Cross showing where the room is located on the hand-drawn roomfinder map image"
            src="../assets/map/roomfinder_cross-v2.webp"
            v-bind:style="{
              transform: 'translate(' + state.map.roomfinder.modalX + 'px, ' + state.map.roomfinder.modalY + 'px)',
            }"
            id="roomfinder-modal-map-cross"
          />
          <img
            alt="Hand-drawn roomfinder map image"
            v-bind:src="
              '/cdn/maps/roomfinder/' + state.data.maps.roomfinder.available[state.map.roomfinder.selected_index].file
            "
            class="img-responsive"
            v-bind:width="state.map.roomfinder.width"
            v-bind:height="state.map.roomfinder.height"
            id="roomfinder-modal-map-img"
          />
          <div>
            {{ $t("view_view.map.img_source") }}:
            {{ state.data.maps.roomfinder.available[state.map.roomfinder.selected_index].source }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
@import "../assets/variables";

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
