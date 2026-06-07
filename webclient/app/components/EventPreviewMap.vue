<script setup lang="ts">
import { Map as MapLibreMap, Marker } from "maplibre-gl";
import { webglSupport } from "~/composables/webglSupport";

// Live preview of how the proposed event will show up as a #3136 photo marker, rendered from the
// not-yet-uploaded image (a local `blob:` URL) at the picked coordinate. Marker-only on purpose -
// the popup card is a separate concern - so this re-creates the circular white-ringed look of the
// `events_active` symbol layer with a plain DOM marker instead of the Martin vector source.
const props = defineProps<{
  lat: number;
  lon: number;
  imageUrl: string | null;
}>();

const PREVIEW_ZOOM = 17;

const mapContainer = ref<HTMLElement>();
// `shallowRef`: MapLibre owns its own deep state; Vue must not track it reactively.
const map = shallowRef<MapLibreMap | undefined>(undefined);
const marker = shallowRef<Marker | undefined>(undefined);
const markerImg = shallowRef<HTMLImageElement | undefined>(undefined);
const loaded = ref(false);

function createMarkerElement(): HTMLElement {
  const el = document.createElement("div");
  Object.assign(el.style, {
    width: "56px",
    height: "56px",
    borderRadius: "9999px",
    overflow: "hidden",
    border: "3px solid #ffffff",
    boxShadow: "0 1px 4px rgba(0, 0, 0, 0.4)",
    background: "#e4e4e7",
  });
  const img = document.createElement("img");
  Object.assign(img.style, {
    width: "100%",
    height: "100%",
    objectFit: "cover",
    display: "block",
  });
  el.appendChild(img);
  markerImg.value = img;
  return el;
}

function syncMarker(): void {
  const target = map.value;
  if (!target) return;
  if (!props.imageUrl) {
    marker.value?.remove();
    marker.value = undefined;
    return;
  }
  if (markerImg.value) markerImg.value.src = props.imageUrl;
  if (!marker.value) {
    marker.value = new Marker({ element: createMarkerElement(), anchor: "center" });
    if (markerImg.value) markerImg.value.src = props.imageUrl;
  }
  marker.value.setLngLat([props.lon, props.lat]).addTo(target);
}

onMounted(() => {
  if (!webglSupport || !mapContainer.value) return;
  const instance = new MapLibreMap({
    container: mapContainer.value,
    style: "https://nav.tum.de/martin/style/navigatum-basemap.json",
    center: [props.lon, props.lat],
    zoom: PREVIEW_ZOOM,
    attributionControl: false,
    validateStyle: import.meta.env.DEV,
  });
  map.value = instance;
  instance.on("load", () => {
    loaded.value = true;
    syncMarker();
  });
});

// Track live coordinate edits (e.g. dragging the picker) without animating, so the marker stays
// glued to the chosen spot.
watch(
  () => [props.lat, props.lon] as const,
  ([lat, lon]) => {
    map.value?.setCenter([lon, lat]);
    marker.value?.setLngLat([lon, lat]);
  }
);
watch(() => props.imageUrl, syncMarker);

onBeforeUnmount(() => {
  marker.value?.remove();
  map.value?.remove();
});
</script>

<template>
  <div class="aspect-4/3 border-zinc-300 dark:border-zinc-600 relative overflow-hidden rounded border">
    <div v-if="webglSupport" ref="mapContainer" class="absolute inset-0 h-full w-full" />
    <LazyMapGLNotSupported v-else />
    <div v-if="webglSupport && !loaded" class="absolute inset-0 flex items-center justify-center">
      <Spinner class="h-8 w-8 text-blue-500 dark:text-blue-400" />
    </div>
  </div>
</template>

<style scoped>
@import "maplibre-gl/dist/maplibre-gl.css";
</style>
