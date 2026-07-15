<script setup lang="ts">
import type { GeoJSONSource } from "maplibre-gl";
import { GeolocateControl, Map as MapLibreMap, Marker, NavigationControl } from "maplibre-gl";
import type { components } from "~/api_types";
import { FloorControl } from "~/composables/FloorControl";
import { useSharedGeolocation } from "~/composables/geolocation";
import { useWebglGuard } from "~/composables/webglSupport";
import { zoomForLocationType } from "~/utils/map";
import {
  calculateItineraryBounds,
  calculateLegBounds,
  calculateStepBounds,
  decodeMotisGeometry,
  extractPlatformChangeMarkers,
  getTransitModeStyle,
} from "~/utils/motis";

type LocationDetailsResponse = components["schemas"]["LocationDetailsResponse"];
type Coordinate = components["schemas"]["Coordinate"];
type ItineraryResponse = components["schemas"]["ItineraryResponse"];
type StepInstructionResponse = components["schemas"]["StepInstructionResponse"];

// Simplified GeoJSON Feature type to avoid deep type inference
interface SimpleGeoJSONFeature {
  type: "Feature";
  properties: Record<string, unknown>;
  geometry: {
    type: "LineString";
    coordinates: number[][];
  };
}

const props = defineProps<{
  coords: LocationDetailsResponse["coords"];
  type: LocationDetailsResponse["type"];
}>();
// `shallowRef`: MapLibre owns its own deep state; Vue must not try to track it reactively.
const map = shallowRef<MapLibreMap | undefined>(undefined);
const marker = shallowRef<Marker | undefined>(undefined);
const afterLoaded = ref<() => void>(() => {});
const geolocateControl = ref<GeolocateControl | undefined>(undefined);
const floorControl = shallowRef<FloorControl | undefined>(undefined);
const { supported: webglSupport, attach: attachWebglGuard } = useWebglGuard();

// Geolocation state
const geolocationState = useSharedGeolocation();

// Motis routing state
const highlightedLegIndex = ref<number | null>(null);
const zoom = computed<number>(() => zoomForLocationType(props.type));

onMounted(async () => {
  if (!webglSupport.value) return;

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
      _marker.addTo(map.value);
      marker.value = _marker;
    }
  };

  // The map element should be visible when initializing
  if (document.querySelector("#interactive-indoor-map .maplibregl-canvas")) await doMapUpdate();
  else await nextTick(doMapUpdate);
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
  const mapInstance = new MapLibreMap({
    container: containerId,
    // Reflect the viewport in the URL hash so the map state is deep-linkable.
    hash: true,

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

    center: [11.670099, 48.266921],
    zoom: zoom.value,
  });
  attachWebglGuard(mapInstance);

  // Each source / style change causes the map to get
  // into "loading" state, so map.loaded() is not reliable
  // enough to know whether just the initial loading has
  // succeeded.
  mapInstance.on("load", () => {
    mapInstance.addControl(new NavigationControl({}), "top-left");

    const location = new GeolocateControl({
      positionOptions: {
        enableHighAccuracy: true,
      },
      trackUserLocation: true,
    });

    // Listen for geolocation events
    location.on("geolocate", (e) => {
      geolocationState.value.mapGeolocationActive = true;
      // Store the user location coordinates
      geolocationState.value.userLocation = {
        lat: e.coords.latitude,
        lon: e.coords.longitude,
      };
    });

    location.on("error", () => {
      geolocationState.value.mapGeolocationActive = false;
      geolocationState.value.userLocation = null;
      // Clear the triggering search bar ID so animation stops
      geolocationState.value.triggeringSearchBarId = null;
    });

    mapInstance.addControl(location);
    geolocateControl.value = location;

    // Not at setup: the FloorControl constructor touches `document`, absent on the server.
    const floors = new FloorControl();
    mapInstance.addControl(floors, "top-left");
    // Floors only render from zoom 17, so picking one while zoomed out would show nothing.
    floors.on("level-changed", (event: { level: number | null }) => {
      if (event.level !== null && mapInstance.getZoom() < 17) {
        mapInstance.easeTo({ zoom: 17, duration: 2000 });
      }
    });
    floorControl.value = floors;

    // Add Valhalla route source and layers
    mapInstance.addSource("route", {
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
    mapInstance.addLayer({
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
    mapInstance.addLayer({
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
    mapInstance.addSource("motis-routes", {
      type: "geojson",
      data: {
        type: "FeatureCollection",
        features: [],
      },
    });

    // Add multiple layers for different transport modes
    mapInstance.addLayer({
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
        "line-dasharray": [2, 3],
      },
    });

    mapInstance.addLayer({
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
    mapInstance.addLayer({
      id: "motis-route-highlighted",
      type: "line",
      source: "motis-routes",
      filter: ["==", ["get", "legIndex"], -1], // Initially show nothing
      layout: {
        "line-join": "round",
        "line-cap": "round",
      },
      paint: {
        "line-color": ["get", "color"],
        "line-width": 8,
        "line-opacity": 0.9,
      },
    });

    // Add source for platform change markers
    mapInstance.addSource("platform-changes", {
      type: "geojson",
      data: {
        type: "FeatureCollection",
        features: [],
      },
    });

    // Add platform change layers AFTER all route layers so they render on top
    mapInstance.addLayer({
      id: "platform-changes",
      type: "circle",
      source: "platform-changes",
      minzoom: 0,
      maxzoom: 24,
      paint: {
        "circle-radius": 6,
        "circle-color": "#FF6B35",
        "circle-stroke-width": 2,
        "circle-stroke-color": "#FFFFFF",
        "circle-opacity": 0.9,
      },
    });

    // Platform change text layer
    mapInstance.addLayer({
      id: "platform-changes-text",
      type: "symbol",
      source: "platform-changes",
      minzoom: 10,
      layout: {
        "text-field": ["get", "platformText"],
        "text-font": ["Roboto Regular"],
        "text-size": 11,
        "text-offset": [0, -1],
        "text-anchor": "bottom",
        "text-allow-overlap": true,
      },
      paint: {
        "text-color": "#FF6B35",
        "text-halo-color": "#FFFFFF",
        "text-halo-width": 2,
        "text-halo-blur": 0.5,
      },
    });

    afterLoaded.value();
  });

  return mapInstance;
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

/** Centre the map on a single location, used when there is no route to fit. */
function flyToCoords(
  coords: { lat: number; lon: number },
  type?: LocationDetailsResponse["type"],
  isAfterLoaded = false
) {
  if (!map.value || (!isAfterLoaded && !map.value.loaded())) {
    afterLoaded.value = () => flyToCoords(coords, type, true);
    return;
  }
  marker.value?.setLngLat([coords.lon, coords.lat]);
  map.value.flyTo({ center: [coords.lon, coords.lat], zoom: zoomForLocationType(type) });
}

function fitBounds(lon: [number, number], lat: [number, number]) {
  if (!map.value) {
    console.error("tried to fly to point but map has not loaded yet.. wtf??");
    return;
  }
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
  const routesSrc = map.value?.getSource("motis-routes") as GeoJSONSource | undefined;
  const platformChangesSrc = map.value?.getSource("platform-changes") as GeoJSONSource | undefined;

  if (!routesSrc || !platformChangesSrc || (!isAfterLoaded && !map.value?.loaded())) {
    afterLoaded.value = () => drawMotisItinerary(itinerary, true);
    return;
  }

  // Clear existing routes
  clearMotisRoutes();

  // Create GeoJSON features for each leg
  const routeFeatures: SimpleGeoJSONFeature[] = itinerary.legs.map((leg, index) => {
    const coordinates = decodeMotisGeometry(leg.leg_geometry);
    const style = getTransitModeStyle(leg.mode, leg.route_color, leg.route_text_color);

    return {
      type: "Feature",
      properties: {
        legIndex: index,
        mode: leg.mode,
        color: style.color,
        textColor: style.textColor,
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

  // Update the routes source
  routesSrc.setData({
    type: "FeatureCollection",
    features: routeFeatures,
  });

  // Create platform change markers
  const platformChanges = extractPlatformChangeMarkers(itinerary);
  const platformChangeFeatures = platformChanges.map((change) => {
    return {
      type: "Feature" as const,
      properties: {
        platformText: `${change.name}\nChange platforms\n${change.fromPlatform} to ${change.toPlatform}`,
        name: change.name,
      },
      geometry: {
        type: "Point" as const,
        coordinates: [change.lon, change.lat],
      },
    };
  });

  // Update platform changes source
  platformChangesSrc.setData({
    type: "FeatureCollection",
    features: platformChangeFeatures,
  });

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
 * Focus map on a single step of a leg, falling back to the whole leg when the
 * step's polyline cannot be decoded.
 */
function focusOnMotisStep(
  step: StepInstructionResponse,
  legIndex: number,
  itinerary: ItineraryResponse
) {
  const bounds = calculateStepBounds(step);
  if (bounds) {
    fitBounds([bounds.minLon, bounds.maxLon], [bounds.minLat, bounds.maxLat]);
  } else {
    focusOnMotisLeg(legIndex, itinerary);
  }
}

/**
 * Clear all Motis routes and symbols
 */
function clearMotisRoutes() {
  // Clear highlighted leg
  highlightedLegIndex.value = null;
  if (map.value?.loaded()) {
    map.value.setFilter("motis-route-highlighted", ["==", ["get", "legIndex"], -1]);
  }

  // Clear route data
  const routesSrc = map.value?.getSource("motis-routes") as GeoJSONSource | undefined;
  if (routesSrc) {
    routesSrc.setData({
      type: "FeatureCollection",
      features: [],
    });
  }
}

// Watch for geolocation trigger requests
watch(
  () => geolocationState.value.shouldTriggerMapGeolocation,
  (shouldTrigger) => {
    if (shouldTrigger) {
      geolocateControl.value?.trigger();
      geolocationState.value.shouldTriggerMapGeolocation = false;
    }
  }
);

function triggerGeolocation() {
  geolocateControl.value?.trigger();
}

/** Switch the floor selector (and thus the indoor layers) to the given level. */
function setFloor(level: number | null) {
  floorControl.value?.setLevel(level);
}

defineExpose({
  drawRoute,
  fitBounds,
  flyToCoords,
  drawMotisItinerary,
  highlightMotisLeg,
  focusOnMotisLeg,
  focusOnMotisStep,
  clearMotisRoutes,
  triggerGeolocation,
  setFloor,
});
</script>

<template>
  <div
    id="interactive-indoor-map-container"
    class="!h-full min-h-96 print:!hidden"
    :class="{
      'dark:bg-black bg-white border-zinc-300 dark:border-zinc-600 border': webglSupport,
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
    background-color: var(--color-blue-500);
  }

  > div {
    padding-bottom: 0;
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
