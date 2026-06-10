<script setup lang="ts">
import { until } from "@vueuse/core";
import type { MapGeoJSONFeature, MapLayerMouseEvent } from "maplibre-gl";
import {
  FullscreenControl,
  GeolocateControl,
  Map as MapLibreMap,
  NavigationControl,
  Popup,
} from "maplibre-gl";
import { FloorControl } from "~/composables/FloorControl";
import {
  ENABLED_LAYERS_STORAGE_KEY,
  LAYER_REGISTRY,
  LAYERS_QUERY_PARAM,
  LEVEL_QUERY_PARAM,
  PANEL_COLLAPSED_STORAGE_KEY,
  resolveEnabledLayers,
  resolveLevel,
  serializeEnabledLayers,
} from "~/composables/mapLayers";
import { webglSupport } from "~/composables/webglSupport";

definePageMeta({ layout: "navigation" });

const { t } = useI18n({ useScope: "local" });
const route = useRoute();

useSeoMeta({
  title: () => t("title"),
  description: () => t("description"),
});

const GARCHING_CENTER: [number, number] = [11.670099, 48.266921];
const INITIAL_ZOOM = 17;

// `shallowRef`: MapLibre owns its own deep state; Vue must not track it reactively.
const map = shallowRef<MapLibreMap | undefined>(undefined);
const poiPopup = shallowRef<Popup | undefined>(undefined);
const mapContainer = ref<HTMLElement | undefined>(undefined);
const initialLoaded = ref(false);
const currentZoom = ref(INITIAL_ZOOM);
// Reassigned wholesale so the panel's `Set` prop changes identity.
const enabledLayers = ref<Set<string>>(new Set(LAYER_REGISTRY.map((l) => l.id)));
const panelCollapsed = ref(false);

// Style-layer ids whose markers open a popup.
const allStyleLayerIds = LAYER_REGISTRY.flatMap((l) => l.styleLayerIds);

/** Read a query value as a single string, distinguishing "absent" (null) from "present but empty" (""). */
function queryString(key: string): string | null {
  if (!(key in route.query)) return null;
  const raw = route.query[key];
  const value = Array.isArray(raw) ? raw[0] : raw;
  return value ?? "";
}

/** Update one query parameter in place, leaving MapLibre's `#zoom/lat/lng` hash untouched. */
function setQueryParam(key: string, value: string | null): void {
  const url = new URL(window.location.href);
  if (value === null) url.searchParams.delete(key);
  else url.searchParams.set(key, value);
  window.history.replaceState(window.history.state, "", url.toString());
}

function applyLayerVisibility(): void {
  const m = map.value;
  if (!m) return;
  for (const layer of LAYER_REGISTRY) {
    const visibility = enabledLayers.value.has(layer.id) ? "visible" : "none";
    for (const styleLayerId of layer.styleLayerIds) {
      if (m.getLayer(styleLayerId)) m.setLayoutProperty(styleLayerId, "visibility", visibility);
    }
  }
}

function toggleLayer(id: string): void {
  const next = new Set(enabledLayers.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  enabledLayers.value = next;
}

watch(enabledLayers, (layers) => {
  const serialized = serializeEnabledLayers(layers);
  localStorage.setItem(ENABLED_LAYERS_STORAGE_KEY, serialized);
  // Reflect the exact selection (empty too) so an all-off deep link survives a reload.
  setQueryParam(LAYERS_QUERY_PARAM, serialized);
  applyLayerVisibility();
});

watch(panelCollapsed, (collapsed) => {
  localStorage.setItem(PANEL_COLLAPSED_STORAGE_KEY, collapsed ? "1" : "0");
});

function truthy(value: unknown): boolean {
  return value === true || value === 1 || value === "true";
}

function buildPopupContent(feature: MapGeoJSONFeature, lng: number, lat: number): HTMLElement {
  const props = feature.properties ?? {};
  const isShower = props.indoor === "shower";

  const root = document.createElement("div");
  root.className = "flex flex-col gap-1 text-sm";

  const title = document.createElement("p");
  title.className = "font-semibold";
  title.textContent = isShower ? t("poi.shower") : t("poi.toilet");
  root.appendChild(title);

  const addRow = (text: string) => {
    const row = document.createElement("p");
    row.className = "opacity-70";
    row.textContent = text;
    root.appendChild(row);
  };

  if (!isShower) {
    const genders: string[] = [];
    if (truthy(props.is_unisex_toilet)) genders.push(t("gender.unisex"));
    if (truthy(props.is_male_toilet)) genders.push(t("gender.male"));
    if (truthy(props.is_female_toilet)) genders.push(t("gender.female"));
    if (genders.length) addRow(`${t("gender.label")}: ${genders.join(", ")}`);
    if (truthy(props.is_wheelchair_toilet)) addRow(t("wheelchair_accessible"));
  }

  // Location-only OSM edit link; no element id flows through the tiles.
  const edit = document.createElement("a");
  edit.href = `https://www.openstreetmap.org/edit#map=21/${lat.toFixed(7)}/${lng.toFixed(7)}`;
  edit.target = "_blank";
  edit.rel = "noopener noreferrer";
  edit.textContent = t("edit_in_osm");
  root.appendChild(edit);

  return root;
}

function openPoiPopup(event: MapLayerMouseEvent): void {
  const m = map.value;
  const feature = event.features?.[0];
  if (!m || !feature) return;

  // Anchor on the marker's coordinates, not the click point.
  const [lng, lat] =
    feature.geometry.type === "Point"
      ? (feature.geometry.coordinates as [number, number])
      : [event.lngLat.lng, event.lngLat.lat];

  poiPopup.value?.remove();
  poiPopup.value = new Popup({ closeButton: true, closeOnClick: true })
    .setLngLat([lng, lat])
    .setDOMContent(buildPopupContent(feature, lng, lat))
    .addTo(m);
}

function initMap(): MapLibreMap {
  // Not at setup: its constructor touches `document`, absent on the server.
  const floorControl = new FloorControl();
  const m = new MapLibreMap({
    container: mapContainer.value as HTMLElement,
    // Reflect the viewport in the URL hash so the map state is deep-linkable.
    hash: true,
    canvasContextAttributes: { antialias: true, preserveDrawingBuffer: false },
    style: "https://nav.tum.de/martin/style/navigatum-basemap.json",
    center: GARCHING_CENTER,
    zoom: INITIAL_ZOOM,
    validateStyle: import.meta.env.DEV,
  });

  m.on("zoom", () => {
    currentZoom.value = m.getZoom();
  });

  m.on("load", () => {
    initialLoaded.value = true;
    // Seed from the hash-set zoom; `load` can precede the first `zoom` event.
    currentZoom.value = m.getZoom();
    m.addControl(new NavigationControl({ showCompass: false }), "top-right");
    m.addControl(
      new GeolocateControl({
        positionOptions: { enableHighAccuracy: true },
        trackUserLocation: true,
      }),
      "top-right"
    );
    m.addControl(new FullscreenControl(), "top-right");
    m.addControl(floorControl, "top-right");
    // Ground floor by default.
    floorControl.setLevel(resolveLevel(queryString(LEVEL_QUERY_PARAM)));

    applyLayerVisibility();

    for (const styleLayerId of allStyleLayerIds) {
      m.on("click", styleLayerId, openPoiPopup);
      m.on("mouseenter", styleLayerId, () => {
        m.getCanvas().style.cursor = "pointer";
      });
      m.on("mouseleave", styleLayerId, () => {
        m.getCanvas().style.cursor = "";
      });
    }
  });

  floorControl.on("level-changed", (event: { level: number | null }) => {
    setQueryParam(LEVEL_QUERY_PARAM, event.level === null ? null : String(event.level));
  });

  return m;
}

onMounted(async () => {
  if (!webglSupport) return;
  enabledLayers.value = resolveEnabledLayers({
    urlParam: queryString(LAYERS_QUERY_PARAM),
    stored: localStorage.getItem(ENABLED_LAYERS_STORAGE_KEY),
  });
  panelCollapsed.value = localStorage.getItem(PANEL_COLLAPSED_STORAGE_KEY) === "1";
  // <ClientOnly> mounts its slot after this hook fires, so wait for the container to exist.
  await until(mapContainer).toBeTruthy();
  map.value = initMap();
});

onBeforeUnmount(() => {
  poiPopup.value?.remove();
  map.value?.remove();
});
</script>

<template>
  <div class="relative h-[calc(100vh-60px)] w-full">
    <ClientOnly>
      <template v-if="webglSupport">
        <div v-if="!initialLoaded" class="absolute inset-0 z-10 flex items-center justify-center">
          <Spinner class="h-12 w-12 text-blue-500 dark:text-blue-400" />
        </div>
        <div id="map-browse" ref="mapContainer" class="h-full w-full" />
        <MapLayerPanel
          :layers="LAYER_REGISTRY"
          :enabled="enabledLayers"
          :collapsed="panelCollapsed"
          :zoom="currentZoom"
          @toggle="toggleLayer"
          @update:collapsed="(v) => (panelCollapsed = v)"
        />
      </template>
      <MapGLNotSupported v-else />
    </ClientOnly>
  </div>
</template>

<i18n lang="yaml">
de:
  title: Karte
  description: Durchsuche die TUM-Karte und blende Ebenen wie Toiletten und Duschen ein.
  edit_in_osm: In OpenStreetMap bearbeiten
  wheelchair_accessible: Rollstuhlgerecht
  poi:
    toilet: Toilette
    shower: Dusche
  gender:
    label: Geschlecht
    male: Herren
    female: Damen
    unisex: Unisex
en:
  title: Map
  description: Browse the TUM map and toggle layers such as toilets and showers.
  edit_in_osm: Edit in OpenStreetMap
  wheelchair_accessible: Wheelchair accessible
  poi:
    toilet: Toilet
    shower: Shower
  gender:
    label: Gender
    male: Male
    female: Female
    unisex: Unisex
</i18n>

<style lang="postcss">
@import "maplibre-gl/dist/maplibre-gl.css";

/* Popup stays white in both themes; pin dark text + blue link so nightwind cannot invert them. */
.maplibregl-popup-content {
  background: var(--color-white);
  color: var(--color-zinc-800);
}

.maplibregl-popup-content a {
  color: var(--color-blue-600);
}

.maplibregl-popup-content a:hover {
  text-decoration: underline;
}
</style>
