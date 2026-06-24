<script setup lang="ts">
import { mdiMapMarkerPlus } from "@mdi/js";
import { until } from "@vueuse/core";
import { GeolocateControl, Map as MapLibreMap, Marker, NavigationControl } from "maplibre-gl";
import { useWebglGuard } from "~/composables/webglSupport";

interface Props {
  initialLat: number;
  initialLon: number;
  zoom?: number;
  containerClass?: string;
  awaitingSelection?: boolean;
}
const lat = defineModel<number>("lat", { required: true });

const lon = defineModel<number>("lon", { required: true });

const props = withDefaults(defineProps<Props>(), {
  zoom: 17,
  containerClass: "aspect-4/3",
  awaitingSelection: false,
});
const { t } = useI18n({ useScope: "local" });

const map = shallowRef<MapLibreMap | undefined>(undefined);
const marker = shallowRef<Marker | undefined>(undefined);
const mapContainer = ref<HTMLElement>();
const { supported: webglSupport, attach: attachWebglGuard } = useWebglGuard();

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
    canvasContextAttributes: { antialias: true, preserveDrawingBuffer: false },
    style,
    transformRequest: mltTransformRequest,
    center: [props.initialLon, props.initialLat],
    zoom: props.zoom,
  });
  attachWebglGuard(mapInstance);

  mapInstance.on("load", () => {
    mapInstance.addControl(new NavigationControl({}), "top-left");
    mapInstance.addControl(
      new GeolocateControl({
        positionOptions: { enableHighAccuracy: true },
        trackUserLocation: false,
      })
    );

    const draggableMarker = new Marker({ element: createMarker(120), draggable: true });
    draggableMarker.setLngLat([lon.value, lat.value]);
    if (!props.awaitingSelection) draggableMarker.addTo(mapInstance);
    draggableMarker.on("dragend", () => {
      const lngLat = draggableMarker.getLngLat();
      lat.value = lngLat.lat;
      lon.value = lngLat.lng;
    });
    mapInstance.on("click", (e) => {
      draggableMarker.setLngLat([e.lngLat.lng, e.lngLat.lat]).addTo(mapInstance);
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

watch(
  () => props.awaitingSelection,
  (awaiting) => {
    if (!awaiting && marker.value && map.value) marker.value.addTo(map.value);
  }
);

onMounted(async () => {
  await until(mapContainer).toBeTruthy();
  await initMap();
});

onUnmounted(() => {
  if (map.value) map.value.remove();
});
</script>

<template>
  <div class="location-picker">
    <div
      class="border-zinc-300 dark:border-zinc-600 relative overflow-hidden rounded border"
      :class="[containerClass, { 'dark:bg-black bg-white': webglSupport }]"
    >
      <div v-if="webglSupport" ref="mapContainer" class="absolute inset-0 h-full w-full" />
      <LazyMapGLNotSupported v-else />
      <div
        v-if="awaitingSelection && webglSupport"
        class="pointer-events-none absolute inset-0 flex items-center justify-center p-3"
      >
        <span
          class="bg-zinc-900/75 text-white flex items-center gap-2 rounded px-3 py-1.5 text-center text-sm font-medium shadow backdrop-blur-sm"
        >
          <MdiIcon :path="mdiMapMarkerPlus" :size="18" class="flex-shrink-0" />
          {{ t("clickMap") }}
        </span>
      </div>
    </div>
    <p v-if="!awaitingSelection" class="text-zinc-500 dark:text-zinc-400 mt-1 text-center text-xs">
      {{ t("clickMap") }}
    </p>
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
    background-color: var(--color-blue-500);
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
