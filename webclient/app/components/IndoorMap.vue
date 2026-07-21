<script setup lang="ts">
import type { ExpressionSpecification, GeoJSONSource } from "maplibre-gl";
import { GeolocateControl, Map as MapLibreMap, Marker, NavigationControl } from "maplibre-gl";
import type { components } from "~/api_types";
import { FloorControl } from "~/composables/FloorControl";
import { useSharedGeolocation } from "~/composables/geolocation";
import { motisStepFeatureId, type RouteHighlight } from "~/composables/useRouteHighlight";
import { useWebglGuard } from "~/composables/webglSupport";
import { zoomForLocationType } from "~/utils/map";
import {
  calculateItineraryBounds,
  calculateLegBounds,
  calculateStepBounds,
  decodeMotisGeometry,
  extractPlatformChangeMarkers,
  getTransitModeStyle,
  splitLegByLevel,
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

interface FeatureRef {
  source: string;
  id: number;
}

const props = defineProps<{
  coords: LocationDetailsResponse["coords"];
  type: LocationDetailsResponse["type"];
}>();
// Hovering or clicking a route segment on the map drives the same highlight state as the list.
const emit = defineEmits<{
  hoverRoute: [target: RouteHighlight | null];
  selectRoute: [target: RouteHighlight];
}>();

// A route segment carries the transient hover emphasis; a persistent selected emphasis; or neither.
// Hover wins over selected so the segment under the pointer always reads as the active one.
const HOVER_STATE: ExpressionSpecification = ["boolean", ["feature-state", "hover"], false];
const SELECTED_STATE: ExpressionSpecification = ["boolean", ["feature-state", "selected"], false];
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
// The itinerary currently drawn, so map-driven highlights can be addressed by leg index alone
// (only one itinerary's features live in the sources at a time).
const currentItineraryIndex = ref<number | null>(null);
// The single feature currently carrying each highlight key, so it can be cleared before the next.
const hoverFeature = shallowRef<FeatureRef | null>(null);
const selectedFeature = shallowRef<FeatureRef | null>(null);
// The floor the selector currently shows, so the walk route can emphasise its on-floor part.
const currentLevel = ref<number | null>(null);
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
      currentLevel.value = event.level;
      applyWalkLevelStyling(event.level);
      if (event.level !== null && mapInstance.getZoom() < 17) {
        mapInstance.easeTo({ zoom: 17, duration: 2000 });
      }
    });
    floorControl.value = floors;

    // Add Valhalla route source and layers. One feature per maneuver (keyed by `maneuverIndex`)
    // so a hovered/selected maneuver can be emphasised via feature-state.
    mapInstance.addSource("route", {
      type: "geojson",
      promoteId: "maneuverIndex",
      data: {
        type: "FeatureCollection",
        features: [],
      },
    });
    // Soft casing drawn beneath the route line to make the emphasised maneuver pop.
    mapInstance.addLayer({
      id: "route-halo",
      type: "line",
      source: "route",
      layout: {
        "line-join": "round",
        "line-cap": "round",
      },
      paint: {
        "line-color": "#ffffff",
        "line-width": ["case", HOVER_STATE, 14, SELECTED_STATE, 12, 0],
        "line-opacity": ["case", HOVER_STATE, 0.35, SELECTED_STATE, 0.25, 0],
        "line-blur": 1,
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
        "line-width": ["case", HOVER_STATE, 9, SELECTED_STATE, 8, 7],
        "line-opacity": ["case", HOVER_STATE, 1, SELECTED_STATE, 0.95, 0.9],
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

    // Add Motis route sources and layers. Segments are keyed by `legIndex` so a hovered/selected
    // leg is emphasised via feature-state (the level-split segments of one leg share the id).
    mapInstance.addSource("motis-routes", {
      type: "geojson",
      promoteId: "legIndex",
      data: {
        type: "FeatureCollection",
        features: [],
      },
    });

    // Soft casing beneath the route lines; its floor-aware paint is set by applyWalkLevelStyling.
    mapInstance.addLayer({
      id: "motis-route-halo",
      type: "line",
      source: "motis-routes",
      layout: {
        "line-join": "round",
        "line-cap": "round",
      },
      paint: {
        "line-color": "#ffffff",
        "line-width": 0,
        "line-opacity": 0,
        "line-blur": 1,
      },
    });

    // Add multiple layers for different transport modes. Walk width/opacity are floor-aware and
    // therefore set by applyWalkLevelStyling; the emphasis added here is a safe default before it runs.
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
        "line-width": ["+", ["get", "weight"], ["case", HOVER_STATE, 3, SELECTED_STATE, 2, 0]],
        "line-opacity": ["case", HOVER_STATE, 1, SELECTED_STATE, 0.95, ["get", "opacity"]],
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
        "line-width": ["+", ["get", "weight"], ["case", HOVER_STATE, 3, SELECTED_STATE, 2, 0]],
        "line-opacity": ["case", HOVER_STATE, 1, SELECTED_STATE, 0.95, ["get", "opacity"]],
      },
    });

    // Per-step overlay, invisible until a step is hovered/selected from the list, drawn on top of
    // its leg. Keyed by `stepId` (see motisStepFeatureId).
    mapInstance.addSource("motis-steps", {
      type: "geojson",
      promoteId: "stepId",
      data: {
        type: "FeatureCollection",
        features: [],
      },
    });
    mapInstance.addLayer({
      id: "motis-step-halo",
      type: "line",
      source: "motis-steps",
      layout: {
        "line-join": "round",
        "line-cap": "round",
      },
      paint: {
        "line-color": "#ffffff",
        "line-width": ["case", HOVER_STATE, 13, SELECTED_STATE, 11, 0],
        "line-opacity": ["case", HOVER_STATE, 0.35, SELECTED_STATE, 0.25, 0],
        "line-blur": 1,
      },
    });
    mapInstance.addLayer({
      id: "motis-step-highlight",
      type: "line",
      source: "motis-steps",
      layout: {
        "line-join": "round",
        "line-cap": "round",
      },
      paint: {
        "line-color": ["get", "color"],
        "line-width": ["case", HOVER_STATE, 7, SELECTED_STATE, 6, 0],
        "line-opacity": ["case", HOVER_STATE, 1, SELECTED_STATE, 0.9, 0],
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

    // Hovering or clicking a route line mirrors the list: emphasise it and report it upward. Steps
    // are excluded (they are an invisible overlay driven only from the list).
    for (const layerId of ["route", "motis-route-walk", "motis-route-transit"]) {
      mapInstance.on("mousemove", layerId, (event) => {
        const feature = event.features?.[0];
        if (!feature) return;
        mapInstance.getCanvas().style.cursor = "pointer";
        emit("hoverRoute", targetFromFeature(feature.layer.id, feature.properties));
      });
      mapInstance.on("mouseleave", layerId, () => {
        mapInstance.getCanvas().style.cursor = "";
        emit("hoverRoute", null);
      });
      mapInstance.on("click", layerId, (event) => {
        const feature = event.features?.[0];
        if (!feature) return;
        const target = targetFromFeature(feature.layer.id, feature.properties);
        if (target) emit("selectRoute", target);
      });
    }

    afterLoaded.value();
  });

  return mapInstance;
}

// One feature per maneuver (sliced from the leg shape by its shape-index range) so a hovered or
// selected maneuver can be emphasised in isolation via feature-state.
function drawRoute(
  shapes: readonly Coordinate[],
  maneuvers: readonly { begin_shape_index: number; end_shape_index: number }[],
  isAfterLoaded = false
) {
  const src = map.value?.getSource("route") as GeoJSONSource | undefined;
  if (!src || (!isAfterLoaded && !map.value?.loaded())) {
    afterLoaded.value = () => drawRoute(shapes, maneuvers, true);
    return;
  }
  map.value?.removeFeatureState({ source: "route" });
  // The `maneuverIndex` property (not the array position) is the feature id, so filtering out
  // degenerate maneuvers (e.g. the zero-length arrival) keeps the remaining ids aligned with the list.
  const features: SimpleGeoJSONFeature[] = maneuvers
    .map((maneuver, maneuverIndex) => ({
      type: "Feature" as const,
      properties: { maneuverIndex },
      geometry: {
        type: "LineString" as const,
        coordinates: shapes
          .slice(maneuver.begin_shape_index, maneuver.end_shape_index + 1)
          .map(({ lat, lon }) => [lon, lat]),
      },
    }))
    .filter((feature) => feature.geometry.coordinates.length >= 2);
  src?.setData({ type: "FeatureCollection", features });
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
function drawMotisItinerary(
  itinerary: ItineraryResponse,
  itineraryIndex: number,
  isAfterLoaded = false
) {
  const routesSrc = map.value?.getSource("motis-routes") as GeoJSONSource | undefined;
  const stepsSrc = map.value?.getSource("motis-steps") as GeoJSONSource | undefined;
  const platformChangesSrc = map.value?.getSource("platform-changes") as GeoJSONSource | undefined;

  if (!routesSrc || !stepsSrc || !platformChangesSrc || (!isAfterLoaded && !map.value?.loaded())) {
    afterLoaded.value = () => drawMotisItinerary(itinerary, itineraryIndex, true);
    return;
  }

  // Clear existing routes (and their feature-state) before drawing the newly selected itinerary.
  clearMotisRoutes();
  currentItineraryIndex.value = itineraryIndex;

  // One feature per single-floor run of each leg, so the active floor can be drawn solid while
  // off-floor runs of the same walk are ghosted (see applyWalkLevelStyling).
  const routeFeatures: SimpleGeoJSONFeature[] = itinerary.legs.flatMap((leg, index) => {
    const style = getTransitModeStyle(leg.mode, leg.route_color, leg.route_text_color);
    return splitLegByLevel(leg).map((segment) => ({
      type: "Feature" as const,
      properties: {
        legIndex: index,
        mode: leg.mode,
        level: segment.level,
        floorSelectable: segment.floorSelectable,
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
        coordinates: segment.coordinates.map(({ lat, lon }) => [lon, lat]),
      },
    }));
  });

  // Update the routes source
  routesSrc.setData({
    type: "FeatureCollection",
    features: routeFeatures,
  });

  // One feature per self-navigated step, keyed by `stepId`, for the invisible step overlay.
  const stepFeatures: SimpleGeoJSONFeature[] = itinerary.legs.flatMap((leg, legIndex) => {
    const style = getTransitModeStyle(leg.mode, leg.route_color, leg.route_text_color);
    return (leg.steps ?? [])
      .map((step, stepIndex) => ({
        type: "Feature" as const,
        properties: { stepId: motisStepFeatureId(legIndex, stepIndex), color: style.color },
        geometry: {
          type: "LineString" as const,
          coordinates: decodeMotisGeometry(step.polyline).map(({ lat, lon }) => [lon, lat]),
        },
      }))
      .filter((feature) => feature.geometry.coordinates.length >= 2);
  });
  stepsSrc.setData({
    type: "FeatureCollection",
    features: stepFeatures,
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

  // Re-apply floor emphasis to the freshly drawn walk segments.
  applyWalkLevelStyling(currentLevel.value);

  // Fit map to show entire route
  const bounds = calculateItineraryBounds(itinerary);
  fitBounds([bounds.minLon, bounds.maxLon], [bounds.minLat, bounds.maxLat]);
}

// Emphasise the part of the walk on the shown floor and ghost the rest, composed with the
// hover/selected feature-state so an emphasised leg stays legible on its floor. Segments on a
// non-selectable level (outdoors, or a floor the selector can't show) stay solid; with no floor
// selected the whole walk is solid. See splitLegByLevel for how legs are cut into per-floor runs.
function applyWalkLevelStyling(level: number | null) {
  if (!map.value?.loaded()) return;

  const onShownFloor: ExpressionSpecification =
    level === null
      ? ["literal", true]
      : ["any", ["!", ["get", "floorSelectable"]], ["==", ["get", "level"], level]];

  const walkWidth: ExpressionSpecification = [
    "+",
    ["case", onShownFloor, ["get", "weight"], 2],
    ["case", HOVER_STATE, 3, SELECTED_STATE, 2, 0],
  ];
  const walkOpacity: ExpressionSpecification = [
    "case",
    onShownFloor,
    ["case", HOVER_STATE, 1, SELECTED_STATE, 0.95, ["get", "opacity"]],
    HOVER_STATE,
    0.4,
    SELECTED_STATE,
    0.35,
    0.2,
  ];
  const haloWidth: ExpressionSpecification = [
    "+",
    ["case", onShownFloor, ["get", "weight"], 2],
    ["case", HOVER_STATE, 9, SELECTED_STATE, 7, 0],
  ];
  const haloOpacity: ExpressionSpecification = [
    "case",
    onShownFloor,
    ["case", HOVER_STATE, 0.32, SELECTED_STATE, 0.24, 0],
    HOVER_STATE,
    0.16,
    SELECTED_STATE,
    0.12,
    0,
  ];

  map.value.setPaintProperty("motis-route-walk", "line-opacity", walkOpacity);
  map.value.setPaintProperty("motis-route-walk", "line-width", walkWidth);
  map.value.setPaintProperty("motis-route-halo", "line-opacity", haloOpacity);
  map.value.setPaintProperty("motis-route-halo", "line-width", haloWidth);
}

// Translate a highlight target to the single feature (source + promoted id) that carries it. Motis
// ids are only valid for the itinerary currently drawn, so off-itinerary targets resolve to null.
function featureRefFor(target: RouteHighlight): FeatureRef | null {
  if (target.router === "valhalla") return { source: "route", id: target.maneuverIndex };
  if (target.itineraryIndex !== currentItineraryIndex.value) return null;
  if (target.stepIndex === null) return { source: "motis-routes", id: target.legIndex };
  return { source: "motis-steps", id: motisStepFeatureId(target.legIndex, target.stepIndex) };
}

// Build a highlight target from a clicked/hovered map feature (steps never reach here).
function targetFromFeature(
  layerId: string,
  properties: Record<string, unknown> | null
): RouteHighlight | null {
  if (layerId === "route") {
    const maneuverIndex = Number(properties?.maneuverIndex);
    return Number.isNaN(maneuverIndex) ? null : { router: "valhalla", maneuverIndex };
  }
  if (currentItineraryIndex.value === null) return null;
  const legIndex = Number(properties?.legIndex);
  return Number.isNaN(legIndex)
    ? null
    : { router: "motis", itineraryIndex: currentItineraryIndex.value, legIndex, stepIndex: null };
}

// Move a highlight key from its previous feature to the next, so at most one feature carries it.
function applyFeatureState(
  previous: FeatureRef | null,
  next: FeatureRef | null,
  key: "hover" | "selected"
): FeatureRef | null {
  const m = map.value;
  if (!m) return next;
  if (previous) m.removeFeatureState({ source: previous.source, id: previous.id }, key);
  if (next) m.setFeatureState({ source: next.source, id: next.id }, { [key]: true });
  return next;
}

function setHover(target: RouteHighlight | null) {
  if (!map.value?.loaded()) return;
  hoverFeature.value = applyFeatureState(
    hoverFeature.value,
    target ? featureRefFor(target) : null,
    "hover"
  );
}

function setSelected(target: RouteHighlight | null) {
  if (!map.value?.loaded()) return;
  selectedFeature.value = applyFeatureState(
    selectedFeature.value,
    target ? featureRefFor(target) : null,
    "selected"
  );
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
  const m = map.value;
  // Feature ids (leg/step indices) are reused across itineraries, so stale state must be dropped
  // before the next itinerary reuses those ids.
  if (m?.loaded()) {
    m.removeFeatureState({ source: "motis-routes" });
    m.removeFeatureState({ source: "motis-steps" });
  }
  if (hoverFeature.value?.source.startsWith("motis")) hoverFeature.value = null;
  if (selectedFeature.value?.source.startsWith("motis")) selectedFeature.value = null;
  currentItineraryIndex.value = null;

  for (const sourceId of ["motis-routes", "motis-steps"]) {
    const src = m?.getSource(sourceId) as GeoJSONSource | undefined;
    src?.setData({ type: "FeatureCollection", features: [] });
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
  setHover,
  setSelected,
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
