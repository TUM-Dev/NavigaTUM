<script setup lang="ts">
import { until } from "@vueuse/core";
import { GeolocateControl, Map as MapLibreMap, Marker, NavigationControl } from "maplibre-gl";
import { FloorControl } from "~/composables/FloorControl";
import { useWebglGuard } from "~/composables/webglSupport";

interface LocationPickerProps {
  initialLat: number;
  initialLon: number;
  zoom?: number;
  floors?: number[];
  initialFloor?: number | null;
}

interface LocationPickerEmits {
  coordinatesChanged: [lat: number, lon: number];
  confirm: [];
  cancel: [];
}

const modalOpen = defineModel<boolean>("open", { required: true });
const props = withDefaults(defineProps<LocationPickerProps>(), { zoom: 17 });
const emit = defineEmits<LocationPickerEmits>();
const { t } = useI18n({ useScope: "local" });

const map = ref<MapLibreMap | undefined>(undefined);
const marker = ref<Marker | undefined>(undefined);
const floorControl = ref<FloorControl>(new FloorControl());
const mapContainer = ref<HTMLElement>();
const isMapLoaded = ref(false);
const { supported: webglSupport, attach: attachWebglGuard } = useWebglGuard();

const coordinates = ref({
  lat: props.initialLat,
  lon: props.initialLon,
});

function createMarker(hueRotation = 120) {
  const markerDiv = document.createElement("div");
  const markerIcon = document.createElement("span");
  markerIcon.style.filter = `hue-rotate(${hueRotation}deg)`;
  markerIcon.classList.add("marker");
  markerIcon.classList.add("marker-pin");
  markerDiv.appendChild(markerIcon);
  const markerShadow = document.createElement("span");
  markerShadow.classList.add("marker");
  markerShadow.classList.add("marker-shadow");
  markerDiv.appendChild(markerShadow);
  return markerDiv;
}

async function initMap() {
  const container = mapContainer.value;
  if (!webglSupport.value || !container) return;

  const style = await loadBasemapStyle();
  const mapInstance = new MapLibreMap({
    container,
    hash: false,
    canvasContextAttributes: {
      antialias: true,
      preserveDrawingBuffer: false,
    },
    style,
    transformRequest: mltTransformRequest,
    center: [coordinates.value.lon, coordinates.value.lat],
    zoom: props.zoom,
  });
  attachWebglGuard(mapInstance);

  mapInstance.on("load", () => {
    isMapLoaded.value = true;

    // Add navigation controls
    mapInstance.addControl(new NavigationControl({}), "top-left");

    // Add location control
    const location = new GeolocateControl({
      positionOptions: {
        enableHighAccuracy: true,
      },
      trackUserLocation: false, // Don't continuously track, just allow one-time location
    });
    mapInstance.addControl(location);

    // Add floor control
    mapInstance.addControl(floorControl.value, "bottom-left");

    // Initialize floor state if provided
    if (props.floors && props.floors.length > 0) {
      floorControl.value.setAvailableFloors(props.floors);
      if (props.initialFloor != null) {
        floorControl.value.setLevel(props.initialFloor);
      }
    }

    // Create draggable marker
    const draggableMarker = new Marker({
      element: createMarker(120), // Green-ish marker to indicate it's editable
      draggable: true,
    });

    draggableMarker.setLngLat([coordinates.value.lon, coordinates.value.lat]);
    draggableMarker.addTo(mapInstance);

    // Handle marker drag events
    draggableMarker.on("dragend", () => {
      const lngLat = draggableMarker.getLngLat();
      coordinates.value = {
        lat: lngLat.lat,
        lon: lngLat.lng,
      };
      emit("coordinatesChanged", lngLat.lat, lngLat.lng);
    });

    // Allow clicking on map to move marker
    mapInstance.on("click", (e) => {
      const { lng, lat } = e.lngLat;
      draggableMarker.setLngLat([lng, lat]);
      coordinates.value = { lat, lon: lng };
      emit("coordinatesChanged", lat, lng);
    });

    marker.value = draggableMarker;
  });

  map.value = mapInstance;
}

// Watch for coordinate changes from parent
watch(
  () => [props.initialLat, props.initialLon],
  ([newLat, newLon]) => {
    const lat = newLat ?? 48.2624449;
    const lon = newLon ?? 11.6677914;
    coordinates.value = { lat, lon };
    if (marker.value) {
      marker.value.setLngLat([lon, lat]);
    }
    if (map.value) {
      map.value.flyTo({
        center: [lon, lat],
        zoom: props.zoom,
        speed: 1,
        maxDuration: 1000,
      });
    }
  },
  { immediate: false }
);

onMounted(async () => {
  // Without WebGL2 the map container is never rendered, so `until` below would wait forever.
  if (!webglSupport.value) return;
  await until(mapContainer).toBeTruthy();
  await initMap();
});

onUnmounted(() => {
  if (map.value) {
    map.value.remove();
  }
});

// Expose current coordinates
defineExpose({
  coordinates: readonly(coordinates),
});
</script>

<template>
  <Modal v-model="modalOpen" :title="t('select_location')" @close="emit('cancel')">
    <div class="location-picker">
      <!-- Instructions above map -->
      <div
        class="aspect-4/3 relative border border-zinc-300 dark:border-zinc-600 rounded-lg overflow-hidden"
        :class="{
          'dark:bg-black bg-white': webglSupport,
        }"
      >
        <div v-if="webglSupport" ref="mapContainer" class="absolute inset-0 h-full w-full" />
        <LazyMapGLNotSupported v-else />
      </div>
      <div class="text-sm text-center">
        {{ t("clickMap") }}
      </div>
    </div>

    <div class="flex gap-2 mt-4">
      <Btn variant="primary" @click="emit('confirm')">
        {{ t("confirm_location") }}
      </Btn>
      <Btn variant="secondary" @click="emit('cancel')">
        {{ t("cancel") }}
      </Btn>
    </div>
  </Modal>
</template>

<style lang="postcss">
@import "maplibre-gl/dist/maplibre-gl.css";

.location-picker {
  /* Reuse marker styles from DetailsInteractiveMap */
  .marker {
    position: absolute;
    pointer-events: none;
    padding: 0;

    &.marker-pin {
      background-image: url(~/assets/map/marker_pin.webp);
      width: 25px;
      height: 36px;
      top: -33px;
      left: -12px;
    }

    &.marker-shadow {
      background-image: url(~/assets/map/marker_pin-shadow.webp);
      width: 38px;
      height: 24px;
      top: -20px;
      left: -12px;
    }
  }

  /* User location dot styling */
  .maplibregl-user-location-dot,
  .maplibregl-user-location-dot::before {
    background-color: var(--color-blue-500);
  }

  /* Make the container properly sized */
  .maplibregl-map {
    border-radius: inherit;
  }

  /* Floor control styles */
  .maplibregl-ctrl-group.floor-ctrl {
    max-width: 100%;
    display: block;
    overflow: hidden;
    color: black;

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

    /* vertical is default layout */
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
        width: calc(100% - var(--map-ctrl-button-size));
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
  }
}
</style>
<i18n lang="yaml">
de:
  select_location: Standort auswählen
  clickMap: Klick auf der Karte, um eine Position auszuwählen
  cancel: Abbrechen
  confirm_location: Standort bestätigen
en:
  select_location: Select Location
  clickMap: Click anywhere on the map to select a location
  cancel: Cancel
  confirm_location: Confirm Location
</i18n>
