<script setup lang="ts">
import { until } from "@vueuse/core";
import {
  FullscreenControl,
  GeolocateControl,
  type MapGeoJSONFeature,
  type IControl,
  Map as MapLibreMap,
  Marker,
  NavigationControl,
} from "maplibre-gl";
import type { components } from "~/api_types";
import { FloorControl } from "~/composables/FloorControl";
import { useIsMobile } from "~/composables/useIsMobile";
import { webglSupport } from "~/composables/webglSupport";
import { dedupeFeatures, diffMarkers, type MarkerFeature } from "~/utils/eventMarkers";
import { zoomForLocationType } from "~/utils/map";

const props = defineProps<{
  coords: LocationDetailsResponse["coords"];
  type: LocationDetailsResponse["type"];
  maps: LocationDetailsResponse["maps"];
  id: LocationDetailsResponse["id"];
  floors?: LocationDetailsResponse["props"]["floors"];
}>();
const map = ref<MapLibreMap | undefined>(undefined);
const marker = ref<Marker | undefined>(undefined);
const floorControl = ref<FloorControl>(new FloorControl());
const mapContainer = ref<HTMLElement>();
const isMobile = useIsMobile();
const zoom = computed<number>(() => zoomForLocationType(props.type));

const initialLoaded = ref(false);

type LocationDetailsResponse = components["schemas"]["LocationDetailsResponse"];

function loadInteractiveMap() {
  if (!webglSupport) return;

  const doMapUpdate = () => {
    // The map might or might not be initialized depending on the type
    // of navigation.
    if (document.getElementById("interactive-legacy-map")) {
      if (document.getElementById("interactive-legacy-map")?.classList.contains("maplibregl-map")) {
        marker.value?.remove();
      } else {
        map.value = initMap("interactive-legacy-map");

        document.getElementById("interactive-legacy-map")?.classList.remove("loading");
      }
    }
    if (map.value !== undefined) {
      const _marker = new Marker({ element: createMarker() });
      _marker.setLngLat([props.coords.lon, props.coords.lat]);
      // @ts-expect-error somehow this is too deep for typescript
      _marker.addTo(map.value as MapLibreMap);
      marker.value = _marker;
    }

    map.value?.flyTo({
      center: [props.coords.lon, props.coords.lat],
      zoom: zoom.value,
      speed: 1,
      maxDuration: 2000,
    });
  };

  // The map element should be visible when initializing
  if (document.querySelector("#interactive-legacy-map .maplibregl-canvas")) doMapUpdate();
  else nextTick(doMapUpdate);
}

function createMarker(hueRotation = 0) {
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

// --- Active-event photo markers ---
// `events_active` is not in the basemap style; we add it (plus an invisible backing layer that keeps
// its tiles loaded and features queryable) at runtime and reconcile HTML markers on every settle.
const MARTIN_BASE_URL = "https://nav.tum.de/martin";
const EVENTS_SOURCE_ID = "events_active";
const EVENTS_LAYER_ID = "events_active-backing";
// Viewport-scoped and moderated, so this is a defensive ceiling, not an expected value.
const MAX_EVENT_MARKERS = 200;

// Blue inner ring + white outer ring + soft shadow, distinct from the orange location pin; the blue
// fill shows until the photo loads and behind the fallback glyph.
const EVENT_MARKER_CLASSES =
  "box-border size-10 cursor-pointer overflow-hidden rounded-full border-2 border-blue-500 bg-blue-500 shadow-md ring-2 ring-white";

type EventMarkerFeature = MarkerFeature & { image: string; name: string };

const eventMarkers = new Map<string, Marker>();

function toEventFeature(feature: MapGeoJSONFeature): EventMarkerFeature | null {
  if (feature.id === undefined) return null;
  if (feature.geometry.type !== "Point") return null;
  const [lon, lat] = feature.geometry.coordinates;
  if (typeof lon !== "number" || typeof lat !== "number") return null;
  const props = feature.properties ?? {};
  return {
    id: String(feature.id),
    lon,
    lat,
    image: typeof props.image === "string" ? props.image : "",
    name: typeof props.name === "string" ? props.name : "",
  };
}

// Generic event glyph (calendar), white on the blue ring, shown when an image is missing or broken.
function showEventFallback(element: HTMLElement): void {
  element.classList.add("flex", "items-center", "justify-center", "text-white");
  const ns = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(ns, "svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("fill", "none");
  svg.setAttribute("stroke", "currentColor");
  svg.setAttribute("stroke-width", "2");
  svg.setAttribute("stroke-linecap", "round");
  svg.setAttribute("stroke-linejoin", "round");
  svg.setAttribute("aria-hidden", "true");
  svg.setAttribute("class", "size-1/2");
  const rect = document.createElementNS(ns, "rect");
  rect.setAttribute("x", "3");
  rect.setAttribute("y", "4");
  rect.setAttribute("width", "18");
  rect.setAttribute("height", "18");
  rect.setAttribute("rx", "2");
  const path = document.createElementNS(ns, "path");
  path.setAttribute("d", "M16 2v4M8 2v4M3 10h18");
  svg.append(rect, path);
  element.appendChild(svg);
}

function createEventMarker(feature: EventMarkerFeature): Marker {
  const element = document.createElement("div");
  element.className = EVENT_MARKER_CLASSES;

  if (feature.image) {
    const image = document.createElement("img");
    // The ring shows immediately; the photo lazy-swaps in. `[filter:none]` keeps real-world imagery
    // safe from any dark-mode inversion.
    image.className = "block size-full rounded-full object-cover [filter:none]";
    image.alt = feature.name;
    image.loading = "lazy";
    image.decoding = "async";
    image.draggable = false;
    image.addEventListener(
      "error",
      () => {
        image.remove();
        showEventFallback(element);
      },
      { once: true }
    );
    element.appendChild(image);
    image.src = feature.image;
  } else {
    showEventFallback(element);
  }

  return new Marker({ element }).setLngLat([feature.lon, feature.lat]);
}

function syncEventMarkers(): void {
  const currentMap = map.value;
  if (!currentMap?.getLayer(EVENTS_LAYER_ID)) return;

  const features = dedupeFeatures(
    currentMap
      .queryRenderedFeatures({ layers: [EVENTS_LAYER_ID] })
      .map(toEventFeature)
      .filter((feature): feature is EventMarkerFeature => feature !== null)
  );
  // Sort by id so overlapping markers keep a stable z-order and the cap keeps a stable subset.
  features.sort((a, b) => a.id.localeCompare(b.id, undefined, { numeric: true }));
  if (features.length > MAX_EVENT_MARKERS) {
    console.warn(
      `events_active returned ${features.length} markers; capping at ${MAX_EVENT_MARKERS}`
    );
    features.length = MAX_EVENT_MARKERS;
  }

  const { added, removed, kept } = diffMarkers(features, new Set(eventMarkers.keys()));
  for (const id of removed) {
    eventMarkers.get(id)?.remove();
    eventMarkers.delete(id);
  }
  for (const feature of added) {
    const eventMarker = createEventMarker(feature);
    // @ts-expect-error somehow this is too deep for typescript
    eventMarker.addTo(currentMap);
    eventMarkers.set(feature.id, eventMarker);
  }
  for (const feature of kept) {
    eventMarkers.get(feature.id)?.setLngLat([feature.lon, feature.lat]);
  }
  features.forEach((feature, index) => {
    const element = eventMarkers.get(feature.id)?.getElement();
    if (element) element.style.zIndex = String(index);
  });
}

function teardownEventMarkers(): void {
  for (const eventMarker of eventMarkers.values()) eventMarker.remove();
  eventMarkers.clear();
  const currentMap = map.value;
  if (currentMap) {
    currentMap.off("idle", syncEventMarkers);
    currentMap.off("moveend", syncEventMarkers);
  }
}

function initMap(containerId: string): MapLibreMap {
  const map = new MapLibreMap({
    container: containerId,
    // while having the hash in the url is nice, it is overridden on map load anyway => not much use
    hash: false,

    canvasContextAttributes: {
      // create the gl context with MSAA antialiasing, so custom layers are antialiasing.
      // slower, but prettier and therefore worth it for our use case
      antialias: true,

      // without this true, printing the webpage is not possible
      // with this true the performance is halfed though...
      // => we are deliberetely not supporing printing of this part of the webpage
      preserveDrawingBuffer: false,
    },

    style: "https://nav.tum.de/martin/style/navigatum-basemap.json",

    center: [11.5748, 48.14], // Approx Munich
    zoom: 11, // Zoomed out so that the whole city is visible
    validateStyle: import.meta.env.DEV,
    maplibreLogo: true,
  });

  // Each source / style change causes the map to get
  // into "loading" state, so map.loaded() is not reliable
  // enough to know whether just the initial loading has
  // succeeded.
  map.on("load", () => {
    initialLoaded.value = true;

    const fullscreenCtl = new FullscreenControl();
    map.addControl(fullscreenCtl, "top-right");

    // controls
    const controls: IControl[] = [];
    if (!isMobile.value) {
      controls.push(
        new NavigationControl({
          showCompass: false,
        })
      );
    }

    controls.push(
      new GeolocateControl({
        positionOptions: {
          enableHighAccuracy: true,
        },
        trackUserLocation: true,
      })
    );
    map.addControl(new CombinedControlGroup(controls), "top-right");

    // Set available floors if provided
    if (props.floors && props.floors.length > 0) {
      const availableFloorIds = props.floors.map((floor) => floor.id);
      floorControl.value.setAvailableFloors(availableFloorIds);
      if (props.floors.length === 1) {
        floorControl.value.setLevel(availableFloorIds[0] ?? null);
      }
    }

    map.addSource(EVENTS_SOURCE_ID, {
      type: "vector",
      url: `${MARTIN_BASE_URL}/${EVENTS_SOURCE_ID}`,
    });
    map.addLayer({
      id: EVENTS_LAYER_ID,
      type: "circle",
      source: EVENTS_SOURCE_ID,
      "source-layer": EVENTS_SOURCE_ID,
      // Invisible: present only so tiles load and features stay queryable; markers are the visuals.
      paint: { "circle-radius": 0, "circle-opacity": 0 },
    });
    map.on("idle", syncEventMarkers);
    map.on("moveend", syncEventMarkers);
  });

  map.addControl(floorControl.value, "top-left");

  // Listen for floor level changes and adjust zoom if needed
  floorControl.value.on("level-changed", (event: { level: number | null }) => {
    if (event.level !== null && map) {
      const currentMapZoom = map.getZoom();
      // Our floors are only visible at zoom level 17+
      if (currentMapZoom < 17) {
        map.easeTo({
          zoom: 17,
          duration: 2000,
        });
      }
    }
  });

  return map;
}

// --- Loading components ---
onMounted(async () => {
  await until(mapContainer).toBeTruthy();
  loadInteractiveMap();
  window.scrollTo({ top: 0, behavior: "auto" });
});

// Tear down event markers and their listeners so nothing leaks across client-side navigation.
onBeforeUnmount(teardownEventMarkers);
</script>

<template>
  <div
    id="interactive-legacy-map-container"
    class="mb-2.5 aspect-4/3 print:!hidden relative"
    :class="{
      'dark:bg-black bg-white border-zinc-300 dark:border-zinc-600 border': webglSupport,
    }"
  >
    <div v-if="webglSupport && !initialLoaded" class="absolute inset-0 z-10 flex items-center justify-center">
      <Spinner class="h-12 w-12 text-blue-500 dark:text-blue-400" />
    </div>
    <div
      v-if="webglSupport"
      id="interactive-legacy-map"
      ref="mapContainer"
      class="absolute !h-full !w-full transition-opacity duration-300"
      :class="{ 'opacity-0': !initialLoaded }"
    />
    <LazyMapGLNotSupported v-else />
  </div>
</template>

<style lang="postcss">
@import "maplibre-gl/dist/maplibre-gl.css";

/* --- Interactive map display --- */
#interactive-legacy-map-container {
  /* --- User location dot --- */

  .maplibregl-user-location-dot,
  .maplibregl-user-location-dot::before {
    background-color: var(--color-blue-500);
  }

  > div {
    padding-bottom: 75%; /* 4:3 aspect ratio */
  }

  &.maximize {
    position: absolute;
    top: 60px;
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

.maplibregl-ctrl-group {
  border-radius: 2px !important;
}

.maplibregl-ctrl-group.floor-ctrl {
  max-width: 100%;
  display: block;
  overflow: hidden;

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
}
</style>
