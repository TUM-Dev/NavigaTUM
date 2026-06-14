<script setup lang="ts">
import { until } from "@vueuse/core";
import type {
  AllPaintProperties,
  ExpressionSpecification,
  MapGeoJSONFeature,
  MapLayerMouseEvent,
} from "maplibre-gl";
import { GeolocateControl, Map as MapLibreMap, NavigationControl, Popup } from "maplibre-gl";
import { FloorControl } from "~/composables/FloorControl";
import {
  ACTIVE_FILTERS_STORAGE_KEY,
  DEFAULT_EVENTS_WINDOW,
  EVENT_SOURCE_BY_WINDOW,
  EVENTS_FILTER_ID,
  EVENTS_WINDOW_QUERY_PARAM,
  EVENTS_WINDOWS,
  type EventSourceId,
  type EventsWindow,
  FILTER_QUERY_PARAM,
  FILTER_REGISTRY,
  type JsonExpression,
  LEVEL_QUERY_PARAM,
  PANEL_COLLAPSED_STORAGE_KEY,
  parseWcsGender,
  parseWcsWheelchair,
  resolveActiveFilters,
  resolveEventsWindow,
  resolveLevel,
  serializeFilters,
  WCS_FILTER_ID,
  WCS_GENDER_QUERY_PARAM,
  WCS_GENDERS,
  WCS_WHEELCHAIR_QUERY_PARAM,
  type WcsGender,
  wcsAttributeConditions,
} from "~/composables/mapLayers";
import { useEventMarkers } from "~/composables/useEventMarkers";
import { useWebglGuard } from "~/composables/webglSupport";

definePageMeta({ layout: "browse-map" });

const { t } = useI18n({ useScope: "local" });
const route = useRoute();

useSeoMeta({
  title: () => t("title"),
  description: () => t("description"),
});

const GARCHING_CENTER: [number, number] = [11.670099, 48.266921];
const INITIAL_ZOOM = 17;
// Opacity the non-matching indoor content fades to while a filter is active.
const DIM = 0.2;
// Translucent white wash over the basemap; lighter than the indoor dim so it reads as "a bit".
const BASEMAP_SCRIM = "rgba(255, 255, 255, 0.45)";
const SCRIM_LAYER = "filter-basemap-dim";
// The combined POI layer whose markers open a popup.
const POI_LAYER = "indoor-pois";
// MapLibre v6 tightened {set,get}PaintProperty signatures: the property name must be a literal
// key of AllPaintProperties, and the value matches that key's spec. The three opacity props
// below all resolve to DataDrivenPropertyValueSpecification<number>.
type OpacityProp = "icon-opacity" | "fill-opacity" | "text-opacity";
type OpacityValue = AllPaintProperties[OpacityProp];

// Indoor layers carrying an `indoor` field: keep the filter's values vibrant, dim the rest
// per-feature (so e.g. the toilet room fill stays its bathroom colour while other rooms fade).
const PER_FEATURE_TARGETS = [
  { id: "indoor-pois", prop: "icon-opacity", type: "symbol" },
  { id: "indoor-rooms", prop: "fill-opacity", type: "fill" },
] as const satisfies readonly {
  readonly id: string;
  readonly prop: OpacityProp;
  readonly type: string;
}[];
// Indoor content with no per-feature meaning: dim wholesale while a filter is active.
const FLAT_TARGETS = [
  { id: "indoor-pois", prop: "text-opacity", type: "symbol" },
] as const satisfies readonly {
  readonly id: string;
  readonly prop: OpacityProp;
  readonly type: string;
}[];

// `shallowRef`: MapLibre owns its own deep state; Vue must not track it reactively.
const map = shallowRef<MapLibreMap | undefined>(undefined);
const poiPopup = shallowRef<Popup | undefined>(undefined);
const mapContainer = ref<HTMLElement | undefined>(undefined);
const { supported: webglSupport, attach: attachWebglGuard } = useWebglGuard();
const initialLoaded = ref(false);
const currentZoom = ref(INITIAL_ZOOM);
// Reassigned wholesale so the panel's `Set` prop changes identity.
const activeFilters = ref<Set<string>>(new Set());
const panelCollapsed = ref(false);
const eventsWindow = ref<EventsWindow>(DEFAULT_EVENTS_WINDOW);
const wcsWheelchair = ref(false);
// `null` selects all genders, i.e. no gender condition.
const wcsGender = ref<WcsGender | null>(null);

// `/map` carries both event feeds; the events filter gates them on, and the window toggle flips
// which single feed is shown ("now" → active lead-in, "2weeks" → the 14-day upcoming horizon).
const eventsVisibleSources = computed<EventSourceId[]>(() =>
  activeFilters.value.has(EVENTS_FILTER_ID) ? [EVENT_SOURCE_BY_WINDOW[eventsWindow.value]] : []
);
const { activeEvent, markerScreenPos, closeActiveEvent } = useEventMarkers(map, {
  sources: ["events_active", "events_upcoming"],
  visibleSources: eventsVisibleSources,
});

// Original paint values captured before the first dim, so toggling a filter off restores them.
const originalPaint = new Map<string, OpacityValue | undefined>();

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

/** The `indoor` values the active indoor filters keep vibrant. */
function vibrantIndoorValues(): string[] {
  return FILTER_REGISTRY.flatMap((f) =>
    f.kind === "indoor" && activeFilters.value.has(f.id) ? [...f.indoorValues] : []
  );
}

function rememberPaint(m: MapLibreMap, id: string, prop: OpacityProp): void {
  const key = `${id}::${prop}`;
  if (!originalPaint.has(key)) originalPaint.set(key, m.getPaintProperty(id, prop) ?? 1);
}

/** The id of the lowest indoor-source layer, used to slot the basemap scrim just beneath it. */
function firstIndoorLayerId(m: MapLibreMap): string | undefined {
  return m.getStyle().layers.find((l) => "source" in l && l.source === "indoor")?.id;
}

/** Highlight the active filters by dimming everything else; restore the original paint when none. */
function applyFilterDim(): void {
  const m = map.value;
  if (!m) return;
  const vibrant = vibrantIndoorValues();
  const active = vibrant.length > 0;

  // Fade the whole basemap a little with one scrim slotted beneath the indoor layers.
  const scrim = m.getLayer(SCRIM_LAYER);
  if (active && !scrim) {
    m.addLayer(
      { id: SCRIM_LAYER, type: "background", paint: { "background-color": BASEMAP_SCRIM } },
      firstIndoorLayerId(m)
    );
  } else if (!active && scrim) {
    m.removeLayer(SCRIM_LAYER);
  }

  const attributeConditions = activeFilters.value.has(WCS_FILTER_ID)
    ? wcsAttributeConditions({ wheelchair: wcsWheelchair.value, gender: wcsGender.value })
    : [];
  const indoorMatch: JsonExpression = ["in", ["get", "indoor"], ["literal", vibrant]];
  const vibrantPredicate = (
    attributeConditions.length === 0 ? indoorMatch : ["all", indoorMatch, ...attributeConditions]
  ) as ExpressionSpecification;

  for (const { id, prop, type } of PER_FEATURE_TARGETS) {
    if (m.getLayer(id)?.type !== type) continue;
    rememberPaint(m, id, prop);
    const original = originalPaint.get(`${id}::${prop}`);
    // The case-expression branch must be a concrete expression/literal - v6 rejects the legacy
    // `{ stops: ... }` form here. Our basemap publishes these as plain numbers, so fall back to 1
    // when the original is anything else.
    const vibrantValue = typeof original === "number" ? original : 1;
    m.setPaintProperty(id, prop, active ? ["case", vibrantPredicate, vibrantValue, DIM] : original);
  }
  for (const { id, prop, type } of FLAT_TARGETS) {
    if (m.getLayer(id)?.type !== type) continue;
    rememberPaint(m, id, prop);
    m.setPaintProperty(id, prop, active ? DIM : originalPaint.get(`${id}::${prop}`));
  }
}

function toggleFilter(id: string): void {
  const next = new Set(activeFilters.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  activeFilters.value = next;
}

watch(activeFilters, (filters) => {
  const serialized = serializeFilters(filters);
  localStorage.setItem(ACTIVE_FILTERS_STORAGE_KEY, serialized);
  // Drop the param when nothing is active (the default), so a bare /map URL stays clean.
  setQueryParam(FILTER_QUERY_PARAM, serialized || null);
  applyFilterDim();
  // `useEventMarkers` toggles the feeds reactively off `eventsVisibleSources`; the popup belongs to
  // the events markers and must not outlive them being switched off.
  if (!filters.has(EVENTS_FILTER_ID)) closeActiveEvent();
});

watch(eventsWindow, (window) => {
  // Drop the param at the "now" default, so a bare /map URL stays clean.
  setQueryParam(EVENTS_WINDOW_QUERY_PARAM, window === DEFAULT_EVENTS_WINDOW ? null : window);
  // The shown feed changes (see `eventsVisibleSources`); the open popup may belong to the other one.
  closeActiveEvent();
});

watch([wcsWheelchair, wcsGender], ([wheelchair, gender]) => {
  // Drop each param at its "no condition" default, so a bare /map URL stays clean.
  setQueryParam(WCS_WHEELCHAIR_QUERY_PARAM, wheelchair ? "true" : null);
  setQueryParam(WCS_GENDER_QUERY_PARAM, gender);
  applyFilterDim();
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
    const male = truthy(props.is_male_toilet);
    const female = truthy(props.is_female_toilet);
    // A toilet serving both is all-gender; show it as unisex rather than "male, female".
    const genders: string[] = [];
    if (male && female) genders.push(t("gender.unisex"));
    else {
      if (male) genders.push(t("gender.male"));
      if (female) genders.push(t("gender.female"));
    }
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
  // Only toilets and showers carry a popup; other POIs in the shared layer are not interactive.
  const indoor = feature.properties?.indoor;
  if (indoor !== "toilet" && indoor !== "shower") return;

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
  attachWebglGuard(m);

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
    m.addControl(floorControl, "top-right");

    // Drop the legacy raster floor-plan overlays; they sit above the POIs and would hide them.
    // FloorControl's getLayer guards then skip them while still swapping the vector source per floor.
    for (const layer of m.getStyle().layers) {
      if (!layer.id.startsWith("indoor-raster-floor-")) continue;
      const source = "source" in layer ? layer.source : undefined;
      m.removeLayer(layer.id);
      if (typeof source === "string" && m.getSource(source)) m.removeSource(source);
    }

    // Ground floor by default.
    floorControl.setLevel(resolveLevel(queryString(LEVEL_QUERY_PARAM)));

    applyFilterDim();

    m.on("click", POI_LAYER, openPoiPopup);
    m.on("mouseenter", POI_LAYER, () => {
      m.getCanvas().style.cursor = "pointer";
    });
    m.on("mouseleave", POI_LAYER, () => {
      m.getCanvas().style.cursor = "";
    });
  });

  floorControl.on("level-changed", (event: { level: number | null }) => {
    setQueryParam(LEVEL_QUERY_PARAM, event.level === null ? null : String(event.level));
  });

  return m;
}

onMounted(async () => {
  if (!webglSupport.value) return;
  activeFilters.value = resolveActiveFilters({
    urlParam: queryString(FILTER_QUERY_PARAM),
    stored: localStorage.getItem(ACTIVE_FILTERS_STORAGE_KEY),
  });
  eventsWindow.value = resolveEventsWindow(queryString(EVENTS_WINDOW_QUERY_PARAM));
  wcsWheelchair.value = parseWcsWheelchair(queryString(WCS_WHEELCHAIR_QUERY_PARAM));
  wcsGender.value = parseWcsGender(queryString(WCS_GENDER_QUERY_PARAM));
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
          :filters="FILTER_REGISTRY"
          :active="activeFilters"
          :collapsed="panelCollapsed"
          :zoom="currentZoom"
          @toggle="toggleFilter"
          @update:collapsed="(v) => (panelCollapsed = v)"
        >
          <template #filter-wcs>
            <fieldset class="px-2 pb-1">
              <legend class="sr-only">{{ t("wcs_filters.legend") }}</legend>
              <div class="flex flex-col gap-1 ps-7">
                <label
                  class="text-zinc-600 dark:text-zinc-300 flex cursor-pointer items-center gap-2 text-sm"
                >
                  <input
                    type="checkbox"
                    class="h-3.5 w-3.5 accent-blue-600 dark:accent-blue-400"
                    :checked="wcsWheelchair"
                    @change="wcsWheelchair = !wcsWheelchair"
                  />
                  {{ t("wcs_filters.wheelchair") }}
                </label>
                <label
                  v-for="gender in [null, ...WCS_GENDERS]"
                  :key="gender ?? 'all'"
                  class="text-zinc-600 dark:text-zinc-300 flex cursor-pointer items-center gap-2 text-sm"
                >
                  <input
                    type="radio"
                    name="wcs-gender"
                    class="h-3.5 w-3.5 accent-blue-600 dark:accent-blue-400"
                    :checked="wcsGender === gender"
                    @change="wcsGender = gender"
                  />
                  {{ gender === null ? t("wcs_filters.all_genders") : t(`gender.${gender}`) }}
                </label>
              </div>
            </fieldset>
          </template>
          <template #filter-events>
            <fieldset class="px-2 pb-1">
              <legend class="sr-only">{{ t("events_window.legend") }}</legend>
              <div class="flex flex-col gap-1 ps-7">
                <label
                  v-for="window in EVENTS_WINDOWS"
                  :key="window"
                  class="text-zinc-600 dark:text-zinc-300 flex cursor-pointer items-center gap-2 text-sm"
                >
                  <input
                    type="radio"
                    name="events-window"
                    class="h-3.5 w-3.5 accent-blue-600 dark:accent-blue-400"
                    :checked="eventsWindow === window"
                    @change="eventsWindow = window"
                  />
                  {{ t(`events_window.${window}`) }}
                </label>
              </div>
            </fieldset>
          </template>
        </MapLayerPanel>
        <EventPopupOverlay
          :event="activeEvent"
          :screen-pos="markerScreenPos"
          @close="closeActiveEvent"
        />
      </template>
      <MapGLNotSupported v-else />
    </ClientOnly>
  </div>
</template>

<i18n lang="yaml">
de:
  title: Karte
  description: Durchsuche die TUM-Karte und filtere nach Orten wie Toiletten, Duschen und Veranstaltungen.
  events_window:
    legend: Zeitfenster für Veranstaltungen
    now: Gerade aktiv
    2weeks: Nächste 2 Wochen
  wcs_filters:
    legend: Filter für Toiletten & Duschen
    wheelchair: Nur rollstuhlgerecht
    all_genders: Alle Geschlechter
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
  description: Browse the TUM map and filter for places such as toilets, showers, and events.
  events_window:
    legend: Time window for events
    now: Happening now
    2weeks: Next 2 weeks
  wcs_filters:
    legend: Filters for toilets & showers
    wheelchair: Wheelchair-accessible only
    all_genders: All genders
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
