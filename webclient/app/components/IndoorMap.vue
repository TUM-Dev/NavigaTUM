<script setup lang="ts">
// @ts-expect-error library does not provide proper types
import { addIndoorTo, IndoorControl, IndoorMap } from "map-gl-indoor";
import { AttributionControl, FullscreenControl, GeolocateControl, Map, Marker, NavigationControl } from "maplibre-gl";
import { webglSupport } from "~/composables/webglSupport";
// @ts-expect-error library does not provide proper types
import type { MaplibreMapWithIndoor } from "map-gl-indoor";
import type { components } from "~/api_types";
import { indoorLayers } from "~/composables/indoorLayer";

const props = defineProps<{ data: DetailsResponse }>();
const map = ref<MaplibreMapWithIndoor | undefined>(undefined);
const marker = ref<Marker | undefined>(undefined);
const runtimeConfig = useRuntimeConfig();

const initialLoaded = ref(false);

type DetailsResponse = components["schemas"]["DetailsResponse"];

onMounted(async () => {
  await loadInteractiveMap();
});

async function loadInteractiveMap() {
  if (!webglSupport) return;

  const doMapUpdate = async function () {
    // The map might or might not be initialized depending on the type
    // of navigation.
    if (document.getElementById("interactive-map")) {
      if (document.getElementById("interactive-map")?.classList.contains("maplibregl-map")) {
        marker.value?.remove();
      } else {
        map.value = await initMap("interactive-map");

        document.getElementById("interactive-map")?.classList.remove("loading");
      }
    }

    marker.value = new Marker({ element: createMarker() });
    const coords = props.data.coords;
    if (map.value !== undefined) {
      marker.value.setLngLat([coords.lon, coords.lat]);
      marker.value.addTo(map.value as MaplibreMapWithIndoor);
    }

    const defaultZooms: { [index: string]: number | undefined } = {
      building: 17,
      room: 18,
    };

    map.value?.flyTo({
      center: [8.3909479, 49.0332499],
      zoom: defaultZooms[props.data.type || "undefined"] || 16,
      speed: 1,
      maxDuration: 2000,
    });
  };

  // The map element should be visible when initializing
  if (!document.querySelector("#interactive-map .maplibregl-canvas")) await nextTick(doMapUpdate);
  else await doMapUpdate();
}

function createMarker(hueRotation = 0): HTMLDivElement {
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

async function initMap(containerId: string) {
  const map = new Map({
    container: containerId,
    // to make sure that users can share urls
    hash: true,

    // create the gl context with MSAA antialiasing, so custom layers are antialiasing.
    // slower, but prettier and therefore worth it for our use case
    antialias: true,

    // without this true, printing the webpage is not possible
    // with this true the performance is halfed though...
    // => we are deliberetely not supporing printing of this part of the webpage
    preserveDrawingBuffer: false,

    // preview of the following style is available at
    // https://nav.tum.de/maps/
    style: `${runtimeConfig.public.mapsURL}/maps/styles/osm-liberty/style.json`,

    center: [11.5748, 48.14], // Approx Munich
    zoom: 11, // Zoomed out so that the whole city is visible

    // done manually, to have more controll over when it is extended
    attributionControl: false,
  }) as MaplibreMapWithIndoor;

  map.addControl(new NavigationControl({}), "top-left");

  // (Browser) Fullscreen is enabled only on mobile, on desktop the map
  // is maximized instead. This is determined once to select the correct
  // container to maximize, and then remains unchanged even if the browser
  // is resized (not relevant for users but for developers).
  const isMobile = window.matchMedia && window.matchMedia("only screen and (max-width: 480px)").matches;
  const fullscreenContainer = isMobile
    ? document.getElementById("interactive-map")
    : document.getElementById("interactive-map-container");
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
      fullscreenCtl._fullscreenButton.ariaLabel = fullscreenCtl._fullscreen ? "Exit fullscreen" : "Enter fullscreen";
      fullscreenCtl._fullscreenButton.title = fullscreenCtl._fullscreen ? "Exit fullscreen" : "Enter fullscreen";
      fullscreenCtl._map.resize();
    }
  };
  // There is a bug that the map doesn't update to the new size
  // when changing between fullscreen in the mobile version.
  if (isMobile && ResizeObserver) {
    const fullscreenObserver = new ResizeObserver(() => {
      fullscreenCtl._map.resize();
    });
    fullscreenObserver.observe(fullscreenCtl._container);
  }
  map.addControl(fullscreenCtl);

  const location = new GeolocateControl({
    positionOptions: {
      enableHighAccuracy: true,
    },
    trackUserLocation: true,
  });
  map.addControl(location);

  // Each source / style change causes the map to get
  // into "loading" state, so map.loaded() is not reliable
  // enough to know whether just the initial loading has
  // succeeded.
  map.on("load", () => {
    initialLoaded.value = true;

    // The attributionControl is automatically open, which takes up a lot of
    // space on the small map display that we have. That's why we add it ourselves
    // and then toggle it.
    // It's only added after loading because if we add it directly on map initialization
    // for some reason it doesn't work.
    const attrib = new AttributionControl({ compact: true });
    map.addControl(attrib);
    attrib._toggleAttribution();
  });

  addIndoorTo(map);

  // Retrieve the geojson from the path and add the map
  const geojson = await (await fetch("/example.geojson")).json();
  const indoorMap = IndoorMap.fromGeojson(geojson, { indoorLayers, showFeaturesWithEmptyLevel: true });
  await map.indoor.addMap(indoorMap);

  // Add the specific control
  map.addControl(new IndoorControl(), "bottom-left");

  return map;
}
</script>

<template>
  <div
    id="interactive-map-container"
    class="mb-2.5 aspect-4/3 print:!hidden"
    :class="{
      'dark:bg-black bg-white border-zinc-300 border': webglSupport,
      'bg-red-300 text-red-950': !webglSupport,
    }"
  >
    <div v-if="webglSupport" id="interactive-map" class="absolute !h-full !w-full" />
    <LazyMapGLNotSupported v-else />
  </div>
</template>

<style lang="postcss">
@import "maplibre-gl/dist/maplibre-gl.css";

/* --- Interactive map display --- */
#interactive-map-container {
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

/* mapboxgl is intentional due to map-gl-indoor */
.maplibregl-ctrl-bottom-left .mapboxgl-ctrl {
  margin-left: 10px;
  margin-bottom: 10px;
  pointer-events: auto;

  &.mapboxgl-ctrl-group {
    background: #fff;
    border-radius: 4px;
    box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.1);

    button {
      background-color: transparent;
      border: 0;
      box-sizing: border-box;
      cursor: pointer;
      display: block;
      height: 29px;
      outline: none;
      padding: 0;
      width: 29px;
    }
  }

  .mapboxgl-ctrl-group:empty {
    box-shadow: none;
  }
}
</style>
