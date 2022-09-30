<script setup lang="ts">
import {
  getLocalStorageWithExpiry,
  setLocalStorageWithExpiry,
} from "@/utils/storage";
import mapboxgl from "mapbox-gl";
import { selectedMap, useDetailsStore } from "@/stores/details";
import { nextTick, ref } from "vue";
import { FloorControl } from "@/modules/FloorControl";

const map = ref<mapboxgl.Map | undefined>(undefined);
const marker = ref<mapboxgl.Marker | undefined>(undefined);
const marker2 = ref<mapboxgl.Marker | null>(null);
const floorControl = ref<FloorControl>(new FloorControl());

const state = useDetailsStore();

// The coordinate picker keeps backups of the subject and body
// in case someone writes a text and then after that clicks
// the set coordinate button in the feedback form.
// If we no backup has been made then, this would be lost after clicking confirm there.
const coord_picker = ref({
  backup_id: null as string | null,
  subject_backup: null as string | null,
  body_backup: null as string | null,
  force_reopen: false,
});
const initialLoaded = ref(false);

function confirmLocationPicker() {
  // add the current edits to the feedback
  const currentEdits = getLocalStorageWithExpiry("coordinate-feedback", {});
  const location = marker2.value?.getLngLat();
  currentEdits[this.state.data.id] = {
    coords: { lat: location.lat, lon: location.lng },
  };
  // save to local storage with ttl of 12h (garbage-collected on next read)
  setLocalStorageWithExpiry("coordinate-feedback", currentEdits, 12);

  marker2.value?.remove();
  marker2.value = null;

  // A feedback form is only opened when this is the only (and therefore
  // first coordinate). If there are more coordinates we can assume
  // someone is doing batch edits. They can then use the send button in
  // the coordinate counter at the top of the page.
  if (
    Object.keys(currentEdits).length === 1 ||
    state.coord_picker.force_reopen
  ) {
    state.coord_picker.force_reopen = false;
    openFeedbackForm();
  }

  // The helptext (which says thet you can edit multiple coordinates in bulk)
  // is also only shown if there is one edit.
  if (Object.keys(currentEdits).length === 1) {
    document
      .getElementById("feedback-coordinate-picker-helptext")
      ?.classList.remove("d-none");
  }
}
function cancelLocationPicker() {
  marker2.value?.remove();
  marker2.value = null;

  if (state.coord_picker.force_reopen) {
    state.coord_picker.force_reopen = false;
    this.openFeedbackForm();
  }
}
// TODO: make this interactive from other components
function addLocationPicker() {
  // If this is called from the feedback form using the edit coordinate
  // button, we temporarily save the current subject and body, so it is
  // not lost when being reopened
  if (
    window.feedback &&
    document.getElementById("feedback-modal")!.classList.contains("active")
  ) {
    coord_picker.value.backup_id = state.data!.id;
    coord_picker.value.subject_backup = (
      document.getElementById("feedback-subject") as HTMLInputElement
    ).value;
    coord_picker.value.body_backup = (
      document.getElementById("feedback-body") as HTMLTextAreaElement
    ).value;
    coord_picker.value.force_reopen = true; // reopen after confirm

    window.feedback.closeForm();
  }

  state.map.selected = selectedMap.interactive;

  // Verify that there isn't already a marker (could happen if you click 'assign
  // a location' multiple times from the 'missing accurate location' toast)
  if (marker2.value === null) {
    // Coordinates are either taken from the entry, or if there are already
    // some in the localStorage use them
    const currentEdits = getLocalStorageWithExpiry("coordinate-feedback", {});

    const { coords } = currentEdits[state.data.id] || state.data;
    marker2.value = new mapboxgl.Marker({
      draggable: true,
      color: "#ff0000",
    });
    marker2.value.setLngLat([coords.lon, coords.lat]).addTo(map.value!);
  }
}

loadInteractiveMap();
function loadInteractiveMap(fromUi = false) {
  const fromMap = state.map.selected;

  state.map.selected = selectedMap.interactive;

  const doMapUpdate = function () {
    // The map might or might not be initialized depending on the type
    // of navigation.
    if (document.getElementById("interactive-map")) {
      if (
        document
          .getElementById("interactive-map")!
          .classList.contains("mapboxgl-map")
      ) {
        marker.value!.remove();
      } else {
        map.value = initMap("interactive-map");

        document.getElementById("interactive-map")!.classList.remove("loading");
      }
    }
    marker.value = new mapboxgl.Marker({ element: createMarker() });
    const coords = state.data!.coords;
    marker.value.setLngLat([coords.lon, coords.lat]).addTo(map.value!);

    if (state.data!.maps && state.data!.maps.overlays)
      floorControl.value.updateFloors(state.data!.maps.overlays);
    else floorControl.value.resetFloors();

    const defaultZooms: { [index: string]: number | undefined } = {
      building: 17,
      room: 18,
    };
    if (fromMap === selectedMap.interactive) {
      map.value!.flyTo({
        center: [coords.lon, coords.lat],
        zoom: defaultZooms[state.data!.type] || 16,
        speed: 1,
        maxDuration: 2000,
      });
    } else {
      map.value!.setZoom(16);
      map.value!.setCenter([coords.lon, coords.lat]);
    }
  };

  // The map element should be visible when initializing
  if (!document.querySelector("#interactive-map .mapboxgl-canvas"))
    nextTick(doMapUpdate);
  else doMapUpdate();

  // To have an animation when the roomfinder is opened some time later,
  // the cursor is set to 'zero' while the interactive map is displayed.
  state.map.roomfinder.x = -1023 - 10;
  state.map.roomfinder.y = -1023 - 10;

  if (fromUi) {
    window.scrollTo(0, 0);
  }
}

function createMarker(hueRotation = 0) {
  const markerDiv = document.createElement("div");
  const markerIcon = document.createElement("span");
  markerIcon.style.backgroundImage = `url(@/assets/map/marker_pin.webp)`;
  markerIcon.style.width = `25px`;
  markerIcon.style.height = `36px`;
  markerIcon.style.filter = `hue-rotate(${hueRotation}deg)`;
  markerIcon.style.top = `-33px`;
  markerIcon.style.left = `-12px`;
  markerIcon.classList.add("marker");
  markerDiv.appendChild(markerIcon);
  const markerShadow = document.createElement("span");
  markerShadow.style.backgroundImage = `url(@/assets/map/marker_pin-shadow.webp)`;
  markerShadow.style.width = `38px`;
  markerShadow.style.height = `24px`;
  markerShadow.style.top = `-20px`;
  markerShadow.style.left = `-12px`;
  markerShadow.classList.add("marker");
  markerDiv.appendChild(markerShadow);
  return markerDiv;
}

function initMap(containerId: string) {
  mapboxgl.accessToken =
    "pk.eyJ1IjoiY29tbWFuZGVyc3Rvcm0iLCJhIjoiY2t6ZGJyNDBoMDU2ZzJvcGN2eTg2cWtxaSJ9.PY6Drc3tYHGqSy0UVmVnCg";
  const map = new mapboxgl.Map({
    container: containerId,

    // create the gl context with MSAA antialiasing, so custom layers are antialiasing.
    // slower, but prettier and therefore worth it for our use case
    antialias: true,

    // preview of the following style is available at
    // https://api.mapbox.com/styles/v1/commanderstorm/ckzdc14en003m14l9l8iqwotq.html?title=copy&access_token=pk.eyJ1IjoiY29tbWFuZGVyc3Rvcm0iLCJhIjoiY2t6ZGJyNDBoMDU2ZzJvcGN2eTg2cWtxaSJ9.PY6Drc3tYHGqSy0UVmVnCg&zoomwheel=true&fresh=true#16.78/48.264624/11.670726
    style:
      "mapbox://styles/commanderstorm/ckzdc14en003m14l9l8iqwotq?optimize=true",

    center: [11.5748, 48.14], // Approx Munich
    zoom: 11, // Zoomed out so that the whole city is visible

    logoPosition: "bottom-left",
  });
  const nav = new mapboxgl.NavigationControl();
  map.addControl(nav, "top-left");

  // (Browser) Fullscreen is enabled only on mobile, on desktop the map
  // is maximized instead. This is determined once to select the correct
  // container to maximize, and then remains unchanged even if the browser
  // is resized (not relevant for users but for developers).
  const isMobile =
    window.matchMedia &&
    window.matchMedia("only screen and (max-width: 480px)").matches;

  const fullscreenCtl = new mapboxgl.FullscreenControl({
    container: isMobile
      ? document.getElementById("interactive-map")
      : document.getElementById("interactive-map-container"),
  });
  // "Backup" the mapboxgl default fullscreen handler
  const defaultOnClickFullscreen = fullscreenCtl._onClickFullscreen;
  fullscreenCtl._onClickFullscreen = () => {
    if (isMobile) defaultOnClickFullscreen();
    else {
      if (fullscreenCtl._container.classList.contains("maximize")) {
        fullscreenCtl._container.classList.remove("maximize");
        document.body.classList.remove("no-scroll");
      } else {
        fullscreenCtl._container.classList.add("maximize");
        document.body.classList.add("no-scroll");
        // "instant" is not part of the spec but nonetheless implemented
        // by Firefox and Chrome
        window.scrollTo({ top: 0, behavior: "instant" });
      }

      fullscreenCtl._fullscreen =
        fullscreenCtl._container.classList.contains("maximize");
      fullscreenCtl._changeIcon();
      fullscreenCtl.map.resize();
    }
  };
  map.addControl(fullscreenCtl);

  const location = new mapboxgl.GeolocateControl({
    positionOptions: {
      enableHighAccuracy: true,
    },
    trackUserLocation: true,
    showUserHeading: true,
  });
  map.addControl(location);

  // Each source / style change causes the map to get
  // into "loading" state, so map.loaded() is not reliable
  // enough to know whether just the initial loading has
  // succeeded.
  map.on("load", () => {
    initialLoaded.value = true;
  });

  interface FloorChangedEvent {
    file: string | null;
    coords: number[][] | undefined;
  }

  floorControl.value.on("floor-changed", (args: FloorChangedEvent) => {
    const url = args.file ? `/cdn/maps/overlay/${args.file}` : null;
    setOverlayImage(url, args.coords);
  });
  map.addControl(floorControl.value, "bottom-left");

  return map;
}

// Set the currently visible overlay image in the map,
// or hide it if imgUrl is null.
function setOverlayImage(
  imgUrl: string | null,
  coords: number[][] | undefined
) {
  // Even if the map is initialized, it could be that
  // it hasn't loaded yet, so we need to postpone adding
  // the overlay layer.
  // However, the official `loaded()` function is a problem
  // here, because the map is shortly in a "loading" state
  // when source / style is changed, even though the initial
  // loading is complete (and only the initial loading seems
  // to be required to do changes here)
  if (!initialLoaded.value) {
    map.value!.on("load", () => setOverlayImage(imgUrl, coords));
    return;
  }

  if (imgUrl === null) {
    // Hide overlay
    if (map.value!.getLayer("overlay-layer"))
      map.value!.setLayoutProperty("overlay-layer", "visibility", "none");
    if (map.value!.getLayer("overlay-bg"))
      map.value!.setLayoutProperty("overlay-bg", "visibility", "none");
  } else {
    const source = map.value!.getSource("overlay-src") as
      | mapboxgl.ImageSource
      | undefined;
    if (!source)
      map.value!.addSource("overlay-src", {
        type: "image",
        url: imgUrl,
        coordinates: coords,
      });
    else
      source.updateImage({
        url: imgUrl,
        coordinates: coords,
      });

    const layer = map.value!.getLayer("overlay-layer") as
      | mapboxgl.BackgroundLayer
      | undefined;
    if (!layer) {
      map.value!.addLayer({
        id: "overlay-bg",
        type: "background",
        paint: {
          "background-color": "#ffffff",
          "background-opacity": 0.6,
        },
      });
      map.value!.addLayer({
        id: "overlay-layer",
        type: "raster",
        source: "overlay-src",
        paint: {
          "raster-fade-duration": 0,
        },
      });
    } else {
      map.value!.setLayoutProperty("overlay-layer", "visibility", "visible");
      map.value!.setLayoutProperty("overlay-bg", "visibility", "visible");
    }
  }
}
</script>

<template>
  <div class="toast toast-primary mb-2 location-picker" v-if="marker2">
    <div class="columns">
      <div class="column col col-sm-12">
        {{ $t("view_view.msg.correct_location.msg") }}
      </div>
      <div class="column col-auto col-sm-12 btns">
        <button class="btn btn-sm" @click="cancelLocationPicker">
          {{ $t("view_view.msg.correct_location.btn-cancel") }}
        </button>
        <button class="btn btn-sm" @click="confirmLocationPicker">
          <i class="icon icon-check"></i>
          {{ $t("view_view.msg.correct_location.btn-done") }}
        </button>
      </div>
    </div>
  </div>
  <div
    id="interactive-map-container"
    v-bind:class="{ 'd-none': state.map.selected !== selectedMap.interactive }"
  >
    <div>
      <div id="interactive-map" class="loading"></div>
    </div>
  </div>
</template>

<style lang="scss">
@import "mapbox-gl/dist/mapbox-gl.css";
@import "../assets/variables";

/* --- Map container --- */
#map-container {
  // This does not change anything (except using px instead of rem),
  // but ensures that roomfinder position calculations are predictable.
  padding: 0 8px;

  // The marker2 (draggable)
  .mapboxgl-marker + .mapboxgl-marker {
    animation: fade-in 0.1s linear 0.05s;
    animation-fill-mode: both;
  }
}

.toast.location-picker {
  animation: fade-in 0.1s linear 0.05s;
  animation-fill-mode: both;

  & .btns {
    margin: auto 0;
  }

  .toast {
    // Mobile
    margin-bottom: 9px;
    font-size: 0.7rem;
  }
}
/* --- Interactive map display --- */
#interactive-map-container {
  margin-bottom: 10px;
  aspect-ratio: 4 / 3; // Not yet supported by all browsers

  > div {
    padding-bottom: 75%; // 4:3 aspect ratio
    border: 1px solid $border-light;
    background-color: $container-loading-bg;
    position: relative;
  }

  &.maximize {
    position: absolute;
    top: -10px;
    left: 0;
    width: 100%;
    height: calc(100vh - 60px);
    z-index: 1000;

    > div {
      padding-bottom: 0;
      height: 100%;
    }
  }
}

#interactive-map {
  position: absolute;
  height: 100%;
  width: 100%;
}

.marker {
  position: absolute;
  pointer-events: none;
  padding: 0;
}

.mapboxgl-ctrl-group.floor-ctrl {
  max-width: 100%;
  display: none;
  overflow: hidden;

  &.visible {
    display: block;
  }

  &.closed #floor-list {
    display: none !important;
  }

  & button {
    &.active {
      background: #ececec;
    }

    & .arrow {
      font-weight: normal;
      font-size: 0.3rem;
      line-height: 0.9rem;
      vertical-align: top;
    }
  }

  &.reduced > .vertical-oc,
  &.reduced > .horizontal-oc {
    display: none !important;
  }

  & > .vertical-oc,
  & > .horizontal-oc {
    font-weight: bold;
    background: #ececec;
  }

  &.closed {
    & > .vertical-oc,
    & > .horizontal-oc {
      background: #fff;
    }

    &:hover > .vertical-oc,
    &:hover > .horizontal-oc {
      background: #f2f2f2;
    }
  }

  // vertical is default layout
  & > .horizontal-oc {
    display: none;
  }

  &.horizontal {
    & > .horizontal-oc {
      display: inline-block;
    }

    & > .vertical-oc {
      display: none;
    }

    & #floor-list {
      display: inline-block;
      width: calc(100% - 29px);
    }

    & button {
      display: inline-block;
      border-top: 0;
      border-left: 1px solid #ddd;

      &.arrow {
        font-size: 0.4rem;
        vertical-align: bottom;
        line-height: 1.1rem;
      }

      & + button {
        border-top: 0;
      }
    }
  }

  // mapbox logo
  & + .mapboxgl-ctrl {
    opacity: 0.4;
    pointer-events: none;
    z-index: -1;
  }
}

// 'sm' (mobile)
@media (max-width: 600px) {
  // The mapbox logo is taking away space from the layer
  // selection on the bottom left on mobile, so we move
  // it a bit
  .floor-ctrl.visible + .mapboxgl-ctrl {
    position: absolute;
    bottom: 2px;
    left: 42px;
  }
}
</style>
