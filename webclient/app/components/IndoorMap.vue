<script setup lang="ts">
import { IndoorControl, MapServerHandler } from "maplibre-gl-indoor";
import { AttributionControl, FullscreenControl, GeolocateControl, Map, Marker, NavigationControl } from "maplibre-gl";
import { webglSupport } from "~/composables/webglSupport";
import type { IndoorMapOptions } from "maplibre-gl-indoor";
import type { components } from "~/api_types";

type DetailsResponse = components["schemas"]["DetailsResponse"];

const props = defineProps<{
  coords: DetailsResponse["coords"];
  type: DetailsResponse["type"];
}>();
const map = ref<Map | undefined>(undefined);
const marker = ref<Marker | undefined>(undefined);
const runtimeConfig = useRuntimeConfig();
const zoom = computed<number>(() => {
  if (props.type === "building") return 17;
  if (props.type === "room") return 18;
  return 16;
});

onMounted(async () => {
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

    if (map.value !== undefined) {
      const _marker = new Marker({ element: createMarker() });
      _marker.setLngLat([props.coords.lon, props.coords.lat]);
      _marker.addTo(map.value as Map);
      marker.value = _marker;
    }
  };

  // The map element should be visible when initializing
  if (!document.querySelector("#interactive-map .maplibregl-canvas")) await nextTick(doMapUpdate);
  else await doMapUpdate();
});

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

async function initMap(containerId: string): Promise<Map> {
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
    style: `${runtimeConfig.public.mapsURL}/maps/styles/navigatum-basemap/style.json`,

    center: [11.670099, 48.266921],
    zoom: zoom.value,

    // done manually, to have more control over when it is extended
    attributionControl: false,
  });

  // Each source / style change causes the map to get
  // into "loading" state, so map.loaded() is not reliable
  // enough to know whether just the initial loading has
  // succeeded.
  map.on("load", () => {
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
    if (isMobile) {
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

    // The attributionControl is automatically open, which takes up a lot of
    // space on the small map display that we have. That's why we add it ourselves
    // and then toggle it.
    // It's only added after loading because if we add it directly on map initialization
    // for some reason it doesn't work.
    const attrib = new AttributionControl({ compact: true });
    map.addControl(attrib);
    attrib._toggleAttribution();
  });

  const indoorOptions = { showFeaturesWithEmptyLevel: false } as IndoorMapOptions;
  const mapServerHandler = MapServerHandler.manage(
    `${runtimeConfig.public.apiURL}/api/maps/indoor`,
    map,
    indoorOptions,
  );

  // Add the specific control
  mapServerHandler.map.addControl(new IndoorControl(), "bottom-left");

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
</style>
