<script setup lang="ts">
import {
  FullscreenControl,
  GeolocateControl,
  Map as MapLibreMap,
  Marker,
  NavigationControl,
} from "maplibre-gl";
import type { components } from "~/api_types";
import { FloorControl } from "~/composables/FloorControl";
import { webglSupport } from "~/composables/webglSupport";

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
const zoom = computed<number>(() => {
  if (props.type === "building") return 17;
  if (props.type === "room") return 18;
  return 16;
});

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
  if (!document.querySelector("#interactive-legacy-map .maplibregl-canvas")) nextTick(doMapUpdate);
  else doMapUpdate();
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
    validateStyle: false,
    maplibreLogo: true,
  });

  // Each source / style change causes the map to get
  // into "loading" state, so map.loaded() is not reliable
  // enough to know whether just the initial loading has
  // succeeded.
  map.on("load", () => {
    initialLoaded.value = true;

    const isMobile = window.matchMedia("only screen and (max-width: 480px)").matches;
    const fullscreenCtl = new FullscreenControl();
    map.addControl(fullscreenCtl, "top-right");

    // controls
    const controls = [];
    if (!isMobile) {
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
        floorControl.value.setLevel(availableFloorIds[0] || null);
      }
    }
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
onMounted(() => {
  nextTick(() => {
    // Even though 'mounted' is called there is no guarantee apparently,
    // that we can reference the map by ID in the DOM yet. For this reason we
    // try to poll now (Not the best solution probably)
    let timeoutInMs = 25;

    function pollMap() {
      const canLoadMap = document.getElementById("interactive-legacy-map") !== null;
      if (canLoadMap) {
        loadInteractiveMap();
        window.scrollTo({ top: 0, behavior: "auto" });
      } else {
        console.info(
          `'mounted' called, but page is not mounted yet. Retrying map-load in ${timeoutInMs}ms`
        );
        setTimeout(pollMap, timeoutInMs);
        timeoutInMs *= 1.5;
      }
    }

    pollMap();
  });
});
</script>

<template>
  <div
    id="interactive-legacy-map-container"
    class="mb-2.5 aspect-4/3 print:!hidden relative"
    :class="{
      'dark:bg-black bg-white border-zinc-300 border': webglSupport,
      'bg-red-300 text-red-950': !webglSupport,
    }"
  >
    <Spinner v-if="webglSupport && !initialLoaded" class="h-12 w-12 text-blue-500 absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-10" />
    <div
      v-if="webglSupport"
      id="interactive-legacy-map"
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
    @apply bg-blue-500;
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
