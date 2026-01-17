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
  });

  // Each source / style change causes the map to get
  // into "loading" state, so map.loaded() is not reliable
  // enough to know whether just the initial loading has
  // succeeded.
  map.on("load", () => {
    initialLoaded.value = true;

    // controls
    map.addControl(
      new NavigationControl({
        showCompass: false,
      }),
      "top-right"
    );

    // (Browser) Fullscreen is enabled only on mobile, on desktop the map
    // is maximized instead. This is determined once to select the correct
    // container to maximize, and then remains unchanged even if the browser
    // is resized (not relevant for users but for developers).
    const isMobile = window.matchMedia("only screen and (max-width: 480px)").matches;
    const fullscreenContainer = isMobile
      ? document.getElementById("interactive-legacy-map")
      : document.getElementById("interactive-legacy-map-container");
    const fullscreenCtl = new FullscreenControl({
      container: fullscreenContainer as HTMLElement,
    });
    // "Backup" the maplibregl default fullscreen handler
    const defaultOnClickFullscreen = fullscreenCtl._onClickFullscreen;
    fullscreenCtl._onClickFullscreen = () => {
      if (isMobile) defaultOnClickFullscreen();
      else {
        if (fullscreenCtl._container.classList.contains("maximize")) {
          fullscreenCtl._container.classList.remove("maximize");
          document.body.classList.remove("overflow-y-hidden");
          fullscreenCtl._fullscreenButton.classList.remove("maplibregl-ctrl-shrink");
        } else {
          fullscreenCtl._container.classList.add("maximize");
          fullscreenCtl._fullscreenButton.classList.add("maplibregl-ctrl-shrink");
          document.body.classList.add("overflow-y-hidden");
          window.scrollTo({ top: 0, behavior: "auto" });
        }

        fullscreenCtl._fullscreen = fullscreenCtl._container.classList.contains("maximize");
        fullscreenCtl._fullscreenButton.ariaLabel = fullscreenCtl._fullscreen
          ? "Exit fullscreen"
          : "Enter fullscreen";
        fullscreenCtl._fullscreenButton.title = fullscreenCtl._fullscreen
          ? "Exit fullscreen"
          : "Enter fullscreen";
        fullscreenCtl._map.resize();
      }
    };
    // There is a bug that the map doesn't update to the new size
    // when changing between fullscreen in the mobile version.
    if (isMobile) {
      const fullscreenObserver = new ResizeObserver(() => {
        fullscreenCtl._map.resize();
      });
      fullscreenObserver.observe(fullscreenCtl._container);
    }
    map.addControl(fullscreenCtl, "top-right");

    const location = new GeolocateControl({
      positionOptions: {
        enableHighAccuracy: true,
      },
      trackUserLocation: true,
    });
    map.addControl(location, "top-right");

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
    class="mb-2.5 aspect-4/3 print:!hidden"
    :class="{
      'dark:bg-black bg-white border-zinc-300 border': webglSupport,
      'bg-red-300 text-red-950': !webglSupport,
    }"
  >
    <div v-if="webglSupport" id="interactive-legacy-map" class="absolute !h-full !w-full" />
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
