<script setup lang="ts">
import { until } from "@vueuse/core";
import {
  GeolocateControl,
  type IControl,
  Map as MapLibreMap,
  Marker,
  NavigationControl,
} from "maplibre-gl";
import type { components } from "~/api_types";
import { FloorControl } from "~/composables/FloorControl";
import { useIsMobile } from "~/composables/useIsMobile";
import { useWebglGuard } from "~/composables/webglSupport";
import { zoomForLocationType } from "~/utils/map";

const props = defineProps<{
  coords: LocationDetailsResponse["coords"];
  type: LocationDetailsResponse["type"];
  maps: LocationDetailsResponse["maps"];
  id: LocationDetailsResponse["id"];
  floors?: LocationDetailsResponse["props"]["floors"];
}>();
// `shallowRef`: MapLibre owns its own deep state; Vue must not try to track it reactively.
const map = shallowRef<MapLibreMap | undefined>(undefined);
const marker = shallowRef<Marker | undefined>(undefined);
const floorControl = shallowRef<FloorControl>(new FloorControl());
const mapContainer = ref<HTMLElement>();
const isMobile = useIsMobile();
const { supported: webglSupport, attach: attachWebglGuard } = useWebglGuard();
const zoom = computed<number>(() => zoomForLocationType(props.type));

const { activeEvent, markerScreenPos, closeActiveEvent } = useEventMarkers(map, {
  sources: ["events_active"],
});

// The floor control's active OSM level, fed to the room popup's OSM edit link; `0` while no floor.
const currentLevel = ref<number | null>(null);
const { popupTarget, roomPopup, closeRoomPopup, resolveRoomPopupFromClick, attachHoverCursor } =
  useIndoorRoomPopup(map, {
    getZoom: () => map.value?.getZoom() ?? zoom.value,
    getLevel: () => currentLevel.value ?? 0,
  });

const initialLoaded = ref(false);

type LocationDetailsResponse = components["schemas"]["LocationDetailsResponse"];

function loadInteractiveMap() {
  if (!webglSupport.value) return;

  const doMapUpdate = async () => {
    // The map might or might not be initialized depending on the type
    // of navigation.
    if (document.getElementById("interactive-legacy-map")) {
      if (document.getElementById("interactive-legacy-map")?.classList.contains("maplibregl-map")) {
        marker.value?.remove();
      } else {
        map.value = await initMap("interactive-legacy-map");

        document.getElementById("interactive-legacy-map")?.classList.remove("loading");
      }
    }
    if (map.value !== undefined) {
      const _marker = new Marker({ element: createMarker() });
      _marker.setLngLat([props.coords.lon, props.coords.lat]);
      _marker.addTo(map.value);
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

async function initMap(containerId: string): Promise<MapLibreMap> {
  const mapInstance = new MapLibreMap({
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

    style: await loadBasemapStyle(),
    transformRequest: mltTransformRequest,

    center: [11.5748, 48.14], // Approx Munich
    zoom: 11, // Zoomed out so that the whole city is visible
    validateStyle: import.meta.env.DEV,
    maplibreLogo: true,
  });
  attachWebglGuard(mapInstance);

  // Each source / style change causes the map to get
  // into "loading" state, so map.loaded() is not reliable
  // enough to know whether just the initial loading has
  // succeeded.
  mapInstance.on("load", () => {
    initialLoaded.value = true;

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
    mapInstance.addControl(new CombinedControlGroup(controls), "top-right");

    // Set available floors if provided
    if (props.floors && props.floors.length > 0) {
      const availableFloorIds = props.floors.map((floor) => floor.id);
      floorControl.value.setAvailableFloors(availableFloorIds);
      if (props.floors.length === 1) {
        floorControl.value.setLevel(availableFloorIds[0] ?? null);
      }
    }

    // Clicking a room polygon or toilet/shower node opens the shared indoor popup.
    mapInstance.on("click", resolveRoomPopupFromClick);
    attachHoverCursor(mapInstance, INDOOR_INTERACTIVE_LAYERS);
  });

  mapInstance.addControl(floorControl.value, "top-left");

  // Listen for floor level changes and adjust zoom if needed
  floorControl.value.on("level-changed", (event: { level: number | null }) => {
    currentLevel.value = event.level;
    // The open popup belongs to the floor that was visible; drop it when the floor switches.
    closeRoomPopup();
    if (event.level !== null && mapInstance) {
      const currentMapZoom = mapInstance.getZoom();
      // Our floors are only visible at zoom level 17+
      if (currentMapZoom < 17) {
        mapInstance.easeTo({
          zoom: 17,
          duration: 2000,
        });
      }
    }
  });

  return mapInstance;
}

// --- Loading components ---
onMounted(async () => {
  await until(mapContainer).toBeTruthy();
  loadInteractiveMap();
  window.scrollTo({ top: 0, behavior: "auto" });
});

onBeforeUnmount(() => {
  closeRoomPopup();
});
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

    <EventPopupOverlay :event="activeEvent" :screen-pos="markerScreenPos" @close="closeActiveEvent" />
    <Teleport v-if="popupTarget && roomPopup" :to="popupTarget">
      <IndoorRoomPopup v-bind="roomPopup" />
    </Teleport>
  </div>
</template>

<style lang="postcss">
@import "maplibre-gl/dist/maplibre-gl.css";

/* Popup stays white in both themes; pin dark text so nightwind cannot invert it. */
.maplibregl-popup-content {
  background: var(--color-white);
  color: var(--color-zinc-800);
}

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
</style>
