<script setup lang="ts">
import {
  FullscreenControl,
  GeolocateControl,
  Map as MapLibreMap,
  Marker,
  NavigationControl,
} from "maplibre-gl";
import { webglSupport } from "~/composables/webglSupport";

interface Props {
  initialLat: number;
  initialLon: number;
  zoom?: number;
}

const props = withDefaults(defineProps<Props>(), { zoom: 17 });
const lat = defineModel<number>("lat", { required: true });
const lon = defineModel<number>("lon", { required: true });
const { t } = useI18n({ useScope: "local" });

const map = ref<MapLibreMap | undefined>(undefined);
const marker = ref<Marker | undefined>(undefined);
const mapContainer = ref<HTMLElement>();

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

function initMap() {
  if (!webglSupport || !mapContainer.value) return;
  const mapInstance = new MapLibreMap({
    container: mapContainer.value,
    hash: false,
    canvasContextAttributes: { antialias: true, preserveDrawingBuffer: false },
    style: "https://nav.tum.de/martin/style/navigatum-basemap.json",
    center: [props.initialLon, props.initialLat],
    zoom: props.zoom,
  });

  mapInstance.on("load", () => {
    mapInstance.addControl(new NavigationControl({}), "top-left");
    const isMobile = window.matchMedia("only screen and (max-width: 480px)").matches;
    if (isMobile) {
      const fullscreenCtl = new FullscreenControl({ container: mapContainer.value as HTMLElement });
      mapInstance.addControl(fullscreenCtl);
    }
    mapInstance.addControl(
      new GeolocateControl({
        positionOptions: { enableHighAccuracy: true },
        trackUserLocation: false,
      })
    );

    const draggableMarker = new Marker({ element: createMarker(120), draggable: true });
    draggableMarker.setLngLat([lon.value, lat.value]).addTo(mapInstance);
    draggableMarker.on("dragend", () => {
      const lngLat = draggableMarker.getLngLat();
      lat.value = lngLat.lat;
      lon.value = lngLat.lng;
    });
    mapInstance.on("click", (e) => {
      draggableMarker.setLngLat([e.lngLat.lng, e.lngLat.lat]);
      lat.value = e.lngLat.lat;
      lon.value = e.lngLat.lng;
    });
    marker.value = draggableMarker;
  });

  map.value = mapInstance;
}

watch(
  () => [props.initialLat, props.initialLon],
  ([newLat, newLon]) => {
    if (marker.value && map.value) {
      marker.value.setLngLat([newLon ?? 11.5681, newLat ?? 48.149]);
      map.value.flyTo({
        center: [newLon ?? 11.5681, newLat ?? 48.149],
        zoom: props.zoom,
        speed: 1,
        maxDuration: 1000,
      });
    }
  },
  { immediate: false }
);

onMounted(() => {
  nextTick(() => {
    let timeoutInMs = 25;
    function pollMap() {
      if (mapContainer.value) initMap();
      else {
        setTimeout(pollMap, timeoutInMs);
        timeoutInMs *= 1.5;
      }
    }
    pollMap();
  });
});

onUnmounted(() => {
  if (map.value) map.value.remove();
});
</script>

<template>
  <div class="location-picker">
    <div
      class="aspect-4/3 border-zinc-300 relative overflow-hidden rounded-lg border"
      :class="{ 'dark:bg-black bg-white': webglSupport }"
    >
      <div v-if="webglSupport" ref="mapContainer" class="absolute inset-0 h-full w-full" />
      <LazyMapGLNotSupported v-else />
    </div>
    <p class="text-zinc-500 mt-1 text-center text-xs">{{ t("clickMap") }}</p>
  </div>
</template>

<style lang="postcss">
@import "maplibre-gl/dist/maplibre-gl.css";

.location-picker {
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

  .maplibregl-user-location-dot,
  .maplibregl-user-location-dot::before {
    @apply bg-blue-500;
  }

  .maplibregl-map {
    border-radius: inherit;
  }
}
</style>

<i18n lang="yaml">
de:
  clickMap: Klick auf die Karte, um die Position zu wählen
en:
  clickMap: Click on the map to set the position
</i18n>
