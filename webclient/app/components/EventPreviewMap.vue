<script setup lang="ts">
import { Map as MapLibreMap, Marker } from "maplibre-gl";
import { useWebglGuard } from "~/composables/webglSupport";

export interface EventPreviewPopup {
  readonly name: string;
  readonly description: string;
  readonly startsAt: string;
  readonly endsAt: string;
  readonly orgCode: string;
  readonly orgNameDe: string;
  readonly orgNameEn: string;
  readonly imageSrc: string;
  readonly imageAuthor: string;
}

const props = defineProps<{
  lat: number;
  lon: number;
  imageUrl: string | null | undefined;
  popup?: EventPreviewPopup | null;
}>();

const PREVIEW_ZOOM = 17;
const POPUP_TOP_PADDING = 260;

const mapContainer = ref<HTMLElement>();
const map = shallowRef<MapLibreMap | undefined>(undefined);
const marker = shallowRef<Marker | undefined>(undefined);
const markerImg = shallowRef<HTMLImageElement | undefined>(undefined);
const loaded = ref(false);
const markerPos = shallowRef<{ x: number; y: number } | null>(null);
const { supported: webglSupport, attach: attachWebglGuard } = useWebglGuard();

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

function projectMarker(): void {
  const target = map.value;
  if (!target) return;
  const { x, y } = target.project([props.lon, props.lat]);
  markerPos.value = { x, y };
}

function applyPadding(): void {
  const target = map.value;
  if (!target) return;
  target.setPadding({ top: props.popup ? POPUP_TOP_PADDING : 0, bottom: 0, left: 0, right: 0 });
  target.setCenter([props.lon, props.lat]);
  projectMarker();
}

onMounted(async () => {
  const container = mapContainer.value;
  if (!webglSupport.value || !container) return;
  const style = await loadBasemapStyle();
  const instance = new MapLibreMap({
    container,
    style,
    transformRequest: mltTransformRequest,
    center: [props.lon, props.lat],
    zoom: PREVIEW_ZOOM,
    attributionControl: false,
    validateStyle: import.meta.env.DEV,
  });
  attachWebglGuard(instance);
  map.value = instance;
  instance.on("load", () => {
    loaded.value = true;
    applyPadding();
    syncMarker();
    projectMarker();
  });
  instance.on("move", projectMarker);
  instance.on("resize", projectMarker);
});

watch(
  () => [props.lat, props.lon] as const,
  ([lat, lon]) => {
    map.value?.setCenter([lon, lat]);
    marker.value?.setLngLat([lon, lat]);
    projectMarker();
  }
);
watch(() => props.imageUrl, syncMarker);
watch(
  () => Boolean(props.popup),
  async () => {
    await nextTick();
    map.value?.resize();
    applyPadding();
  }
);

onBeforeUnmount(() => {
  marker.value?.remove();
  map.value?.remove();
});
</script>

<template>
  <div
    class="border-zinc-300 dark:border-zinc-600 relative overflow-hidden rounded border"
    :class="popup ? 'h-[26rem]' : 'h-40'"
  >
    <div v-if="webglSupport" ref="mapContainer" class="absolute inset-0 h-full w-full" />
    <LazyMapGLNotSupported v-else />
    <div v-if="webglSupport && !loaded" class="absolute inset-0 flex items-center justify-center">
      <Spinner class="h-8 w-8 text-blue-500 dark:text-blue-400" />
    </div>
    <div
      v-if="popup && markerPos && loaded"
      class="pointer-events-none absolute z-20"
      :style="{ left: `${markerPos.x}px`, top: `${markerPos.y}px` }"
    >
      <div class="-mt-3 -translate-x-1/2 -translate-y-full">
        <EventPopupCard
          :name="popup.name"
          :description="popup.description"
          :image-src-override="popup.imageSrc"
          :image-author="popup.imageAuthor"
          :starts-at="popup.startsAt"
          :ends-at="popup.endsAt"
          :org-code="popup.orgCode"
          :org-name-de="popup.orgNameDe"
          :org-name-en="popup.orgNameEn"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
@import "maplibre-gl/dist/maplibre-gl.css";
</style>
