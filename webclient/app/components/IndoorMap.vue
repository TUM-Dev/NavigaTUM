<script setup lang="ts">
import type { GeoJSONSource } from "maplibre-gl";
import {
  FullscreenControl,
  GeolocateControl,
  Map as MapLibreMap,
  Marker,
  NavigationControl,
} from "maplibre-gl";
import type { IndoorMapOptions } from "maplibre-gl-indoor";
import { IndoorControl, MapServerHandler } from "maplibre-gl-indoor";
import type { components } from "~/api_types";
import { webglSupport } from "~/composables/webglSupport";
import {
  calculateItineraryBounds,
  calculateLegBounds,
  decodeMotisGeometry,
  extractAllStops,
  getStopMarkerStyle,
  getTransitModeStyle,
} from "~/utils/motis";

type LocationDetailsResponse = components["schemas"]["LocationDetailsResponse"];
type Coordinate = components["schemas"]["Coordinate"];
type ItineraryResponse = components["schemas"]["ItineraryResponse"];
type MotisLegResponse = components["schemas"]["MotisLegResponse"];
type PlaceResponse = components["schemas"]["PlaceResponse"];

// Simplified GeoJSON Feature type to avoid deep type inference
interface SimpleGeoJSONFeature {
  type: "Feature";
  properties: Record<string, any>;
  geometry: {
    type: "LineString";
    coordinates: number[][];
  };
}

const props = defineProps<{
  coords: LocationDetailsResponse["coords"];
  type: LocationDetailsResponse["type"];
}>();
const map = ref<MapLibreMap | undefined>(undefined);
const marker = ref<Marker | undefined>(undefined);
const afterLoaded = ref<() => void>(() => {});
const runtimeConfig = useRuntimeConfig();

// Motis routing state
const motisMarkers = ref<Marker[]>([]);
const highlightedLegIndex = ref<number | null>(null);
const zoom = computed<number>(() => {
  if (props.type === "building") return 17;
  if (props.type === "room") return 18;
  return 16;
});

onMounted(async () => {
  if (!webglSupport) return;

  const doMapUpdate = async () => {
    // The map might or might not be initialized depending on the type
    // of navigation.
    if (document.getElementById("interactive-indoor-map")) {
      if (document.getElementById("interactive-indoor-map")?.classList.contains("maplibregl-map")) {
        marker.value?.remove();
      } else {
        map.value = await initMap("interactive-indoor-map");

        document.getElementById("interactive-indoor-map")?.classList.remove("loading");
      }
    }

    if (map.value !== undefined) {
      const _marker = new Marker({ element: createMarker() });
      _marker.setLngLat([props.coords.lon, props.coords.lat]);
      _marker.addTo(map.value as MapLibreMap);
      marker.value = _marker;
    }
  };

  // The map element should be visible when initializing
  if (!document.querySelector("#interactive-indoor-map .maplibregl-canvas"))
    await nextTick(doMapUpdate);
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

async function initMap(containerId: string): Promise<MapLibreMap> {
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

    style: "https://nav.tum.de/tiles/style/navigatum-basemap.json",

    center: [11.670099, 48.266921],
    zoom: zoom.value,
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
    const isMobile = window.matchMedia("only screen and (max-width: 480px)").matches;
    const fullscreenContainer = isMobile
      ? document.getElementById("interactive-indoor-map")
      : document.getElementById("interactive-indoor-map-container");
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
    map.addControl(fullscreenCtl);

    const location = new GeolocateControl({
      positionOptions: {
        enableHighAccuracy: true,
      },
      trackUserLocation: true,
    });
    map.addControl(location);

    // Add Valhalla route source and layers
    map.addSource("route", {
      type: "geojson",
      data: {
        type: "Feature",
        properties: {},
        geometry: {
          type: "LineString",
          coordinates: [],
        },
      },
    });
    map.addLayer({
      id: "route",
      type: "line",
      source: "route",
      layout: {
        "line-join": "round",
        "line-cap": "round",
      },
      paint: {
        "line-color": "#e37222",
        "line-width": 7,
      },
    });
    map.addLayer({
      id: "route-symbol",
      type: "symbol",
      source: "route",
      layout: {
        "icon-image": "triangle",
        "icon-size": 0.25,
      },
      paint: {
        "icon-color": "#e37222",
      },
    });

    // Add Motis route sources and layers
    map.addSource("motis-routes", {
      type: "geojson",
      data: {
        type: "FeatureCollection",
        features: [],
      },
    });

    // Add multiple layers for different transport modes
    map.addLayer({
      id: "motis-route-walk",
      type: "line",
      source: "motis-routes",
      filter: ["==", ["get", "mode"], "walk"],
      layout: {
        "line-join": "round",
        "line-cap": "round",
      },
      paint: {
        "line-color": ["get", "color"],
        "line-width": ["get", "weight"],
        "line-opacity": ["get", "opacity"],
        "line-dasharray": [5, 5],
      },
    });

    map.addLayer({
      id: "motis-route-transit",
      type: "line",
      source: "motis-routes",
      filter: ["!=", ["get", "mode"], "walk"],
      layout: {
        "line-join": "round",
        "line-cap": "round",
      },
      paint: {
        "line-color": ["get", "color"],
        "line-width": ["get", "weight"],
        "line-opacity": ["get", "opacity"],
      },
    });

    // Highlighted leg layer
    map.addLayer({
      id: "motis-route-highlighted",
      type: "line",
      source: "motis-routes",
      filter: ["==", ["get", "legIndex"], -1], // Initially show nothing
      layout: {
        "line-join": "round",
        "line-cap": "round",
      },
      paint: {
        "line-color": "#FF6B35",
        "line-width": 8,
        "line-opacity": 0.9,
      },
    });
    afterLoaded.value();
  });

  const indoorOptions = {
    showFeaturesWithEmptyLevel: false,
  } as IndoorMapOptions;
  const mapServerHandler = MapServerHandler.manage(
    `${runtimeConfig.public.apiURL}/api/maps/indoor`,
    map,
    indoorOptions
  );

  // Add the specific control
  mapServerHandler.map.addControl(new IndoorControl(), "bottom-left");

  return map;
}

function drawRoute(shapes: readonly Coordinate[], isAfterLoaded = false) {
  const src = map.value?.getSource("route") as GeoJSONSource | undefined;
  if (!src || (!isAfterLoaded && !map.value?.loaded())) {
    afterLoaded.value = () => drawRoute(shapes, true);
    return;
  }
  // cannot be undefined as returned from above if.. come on typescript
  src?.setData({
    type: "Feature",
    properties: {},
    geometry: {
      type: "LineString",
      coordinates: shapes.map(({ lat, lon }) => [lon, lat]),
    },
  });
  const latitudes = shapes.map(({ lat }) => lat);
  const longitudes = shapes.map(({ lon }) => lon);
  fitBounds(
    [Math.min(...longitudes), Math.max(...longitudes)],
    [Math.min(...latitudes), Math.max(...latitudes)]
  );
}

function fitBounds(lon: [number, number], lat: [number, number]) {
  if (!map.value) {
    console.error("tried to fly to point but map has not loaded yet.. wtf??");
    return;
  }
  console.log("zooming to", { lat, lon });
  // below function zooms exactly to the values.
  // adding a bit of padding looks nicer
  const paddingLat = (lat[1] - lat[0]) * 0.1;
  const paddingLon = (lon[1] - lon[0]) * 0.1;
  map.value.fitBounds(
    [
      { lat: lat[0] - paddingLat, lng: lon[0] - paddingLon },
      { lat: lat[1] + paddingLat, lng: lon[1] + paddingLon },
    ],
    { maxZoom: 19 }
  );
}

/**
 * Draw Motis itinerary on the map
 */
function drawMotisItinerary(itinerary: ItineraryResponse, isAfterLoaded = false) {
  const src = map.value?.getSource("motis-routes") as GeoJSONSource | undefined;
  if (!src || (!isAfterLoaded && !map.value?.loaded())) {
    afterLoaded.value = () => drawMotisItinerary(itinerary, true);
    return;
  }

  // Clear existing markers
  clearMotisRoutes();

  // Create GeoJSON features for each leg
  const features: SimpleGeoJSONFeature[] = itinerary.legs.map((leg, index) => {
    const coordinates = decodeMotisGeometry(leg.leg_geometry);
    const style = getTransitModeStyle(leg.mode);

    return {
      type: "Feature",
      properties: {
        legIndex: index,
        mode: leg.mode,
        color: style.color,
        weight: style.weight,
        opacity: style.opacity,
        routeShortName: leg.route_short_name || null,
        headsign: leg.headsign || null,
        fromName: leg.from.name,
        toName: leg.to.name,
      },
      geometry: {
        type: "LineString",
        coordinates: coordinates.map(({ lat, lon }) => [lon, lat]),
      },
    };
  });

  // Update the source with new features
  src.setData({
    type: "FeatureCollection",
    features,
  });

  // Add stop markers
  const stops = extractAllStops(itinerary);
  for (const stop of stops) {
    const markerStyle = getStopMarkerStyle(stop);
    const markerDiv = createTransitStopMarker(stop, markerStyle);

    const marker = new Marker({ element: markerDiv }) as any;
    marker.setLngLat([stop.lon, stop.lat]);
    if (map.value) {
      marker.addTo(map.value as MapLibreMap);
    }
    motisMarkers.value.push(marker as Marker);
  }

  // Fit map to show entire route
  const bounds = calculateItineraryBounds(itinerary);
  fitBounds([bounds.minLon, bounds.maxLon], [bounds.minLat, bounds.maxLat]);
}

/**
 * Highlight a specific leg of the Motis route
 */
function highlightMotisLeg(legIndex: number) {
  if (!map.value?.loaded()) return;

  highlightedLegIndex.value = legIndex;

  // Update the filter for the highlighted layer
  map.value.setFilter("motis-route-highlighted", ["==", ["get", "legIndex"], legIndex]);
}

/**
 * Focus map on a specific leg
 */
function focusOnMotisLeg(legIndex: number, itinerary: ItineraryResponse) {
  if (legIndex < 0 || legIndex >= itinerary.legs.length) return;

  const leg = itinerary.legs[legIndex];
  if (!leg) return;
  const bounds = calculateLegBounds(leg.leg_geometry, leg.from, leg.to);

  fitBounds([bounds.minLon, bounds.maxLon], [bounds.minLat, bounds.maxLat]);
}

/**
 * Clear all Motis routes and markers
 */
function clearMotisRoutes() {
  // Clear markers
  for (const marker of motisMarkers.value) {
    marker.remove();
  }
  motisMarkers.value = [];

  // Clear highlighted leg
  highlightedLegIndex.value = null;
  if (map.value?.loaded()) {
    map.value.setFilter("motis-route-highlighted", ["==", ["get", "legIndex"], -1]);
  }

  // Clear route data
  const src = map.value?.getSource("motis-routes") as GeoJSONSource | undefined;
  if (src) {
    src.setData({
      type: "FeatureCollection",
      features: [],
    });
  }
}

/**
 * Create a transit stop marker element
 */
function createTransitStopMarker(
  stop: PlaceResponse,
  style: { color: string; size: "small" | "medium" | "large"; icon?: string }
): HTMLDivElement {
  const markerDiv = document.createElement("div");
  markerDiv.className = "motis-stop-marker";

  const markerIcon = document.createElement("div");
  markerIcon.className = `motis-stop-icon motis-stop-${style.size}`;
  markerIcon.style.backgroundColor = style.color;
  markerIcon.title = stop.name;

  // Add platform/track info if available
  if (stop.track) {
    const trackInfo = document.createElement("div");
    trackInfo.className = "motis-stop-track";
    trackInfo.textContent = stop.track;
    markerIcon.appendChild(trackInfo);
  }

  markerDiv.appendChild(markerIcon);
  return markerDiv;
}

defineExpose({
  drawRoute,
  fitBounds,
  drawMotisItinerary,
  highlightMotisLeg,
  focusOnMotisLeg,
  clearMotisRoutes,
});
</script>

<template>
  <div
    id="interactive-indoor-map-container"
    class="!h-full min-h-96 print:!hidden"
    :class="{
      'dark:bg-black bg-white border-zinc-300 border': webglSupport,
      'bg-red-300 text-red-950': !webglSupport,
    }"
  >
    <div v-if="webglSupport" id="interactive-indoor-map" class="relative !h-full min-h-96 !w-full" />
    <MapGLNotSupported v-else />
  </div>
</template>

<style lang="postcss">
@import "maplibre-gl/dist/maplibre-gl.css";

/* --- Interactive map display --- */
#interactive-indoor-map-container {
  /* --- User location dot --- */

  .maplibregl-user-location-dot,
  .maplibregl-user-location-dot::before {
    @apply bg-blue-500;
  }

  > div {
    padding-bottom: 0;
  }

  &.maximize {
    position: absolute;
    top: 60px;
    left: 0;
    height: calc(100vh - 150px);
    z-index: 1000;
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

/* --- Motis transit stop markers --- */
.motis-stop-marker {
  position: relative;
  pointer-events: none;
}

.motis-stop-icon {
  border: 2px solid white;
  border-radius: 50%;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 8px;
  font-weight: bold;

  &.motis-stop-small {
    width: 8px;
    height: 8px;
    transform: translate(-4px, -4px);
  }

  &.motis-stop-medium {
    width: 12px;
    height: 12px;
    transform: translate(-6px, -6px);
  }

  &.motis-stop-large {
    width: 16px;
    height: 16px;
    transform: translate(-8px, -8px);
  }
}

.motis-stop-track {
  position: absolute;
  top: -16px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.8);
  color: white;
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 9px;
  white-space: nowrap;
  pointer-events: none;
}
</style>
