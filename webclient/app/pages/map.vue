<script setup lang="ts">
import { until, useIntervalFn } from "@vueuse/core";
import type {
  AllPaintProperties,
  ExpressionSpecification,
  FilterSpecification,
  MapMouseEvent,
} from "maplibre-gl";
import { GeolocateControl, Map as MapLibreMap, NavigationControl, Popup } from "maplibre-gl";
import type { IndoorRoomPopupProps } from "~/components/IndoorRoomPopup.vue";
import { FloorControl } from "~/composables/FloorControl";
import {
  ACTIVE_FILTERS_STORAGE_KEY,
  DEFAULT_EVENTS_WINDOW,
  EVENTS_FILTER_ID,
  EVENTS_STYLE_LAYER,
  EVENTS_WINDOW_QUERY_PARAM,
  EVENTS_WINDOWS,
  type EventsWindow,
  eventsWindowFilter,
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
import { useEventPopup } from "~/composables/useEventMarkers";
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
// The room fill layer; merged with the POIs into one unified popup on click.
const ROOM_LAYER = "indoor-rooms";
// Both indoor layers a click is resolved against, and that show a pointer cursor on hover.
const INDOOR_INTERACTIVE_LAYERS = [ROOM_LAYER, POI_LAYER] as const;
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
// The anchored MapLibre popup; its body is the Vue `IndoorRoomPopup` teleported into `popupTarget`.
const popupInstance = shallowRef<Popup | undefined>(undefined);
const popupTarget = shallowRef<HTMLElement | null>(null);
const roomPopup = shallowRef<IndoorRoomPopupProps | null>(null);
const mapContainer = ref<HTMLElement | undefined>(undefined);
const { supported: webglSupport, attach: attachWebglGuard } = useWebglGuard();
const initialLoaded = ref(false);
const currentZoom = ref(INITIAL_ZOOM);
// The floor control's active OSM level; `null` while all floors are hidden.
const currentLevel = ref<number | null>(null);
// Reassigned wholesale so the panel's `Set` prop changes identity.
const activeFilters = ref<Set<string>>(new Set());
const panelCollapsed = ref(false);
const eventsWindow = ref<EventsWindow>(DEFAULT_EVENTS_WINDOW);
const wcsWheelchair = ref(false);
// `null` selects all genders, i.e. no gender condition.
const wcsGender = ref<WcsGender | null>(null);

const { activeEvent, markerScreenPos, closeActiveEvent } = useEventPopup(map, EVENTS_STYLE_LAYER);

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

/** Show the overlay filters' style layers while active; they ship hidden in the basemap style. */
function applyOverlayVisibility(): void {
  const m = map.value;
  if (!m) return;
  for (const filter of FILTER_REGISTRY) {
    if (filter.kind !== "overlay") continue;
    const visibility = activeFilters.value.has(filter.id) ? "visible" : "none";
    for (const layer of filter.styleLayers) {
      if (m.getLayer(layer)) m.setLayoutProperty(layer, "visibility", visibility);
    }
  }
}

/** Restrict the events layer to the selected time window, evaluated against the current clock. */
function applyEventsFilter(): void {
  const m = map.value;
  if (!m?.getLayer(EVENTS_STYLE_LAYER)) return;
  const filter = eventsWindowFilter(eventsWindow.value, Date.now());
  m.setFilter(EVENTS_STYLE_LAYER, filter as FilterSpecification);
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
  applyOverlayVisibility();
  // The popup belongs to the events layer; it must not outlive the layer being switched off.
  if (!filters.has(EVENTS_FILTER_ID)) closeActiveEvent();
});

watch(eventsWindow, (window) => {
  // Drop the param at the "now" default, so a bare /map URL stays clean.
  setQueryParam(EVENTS_WINDOW_QUERY_PARAM, window === DEFAULT_EVENTS_WINDOW ? null : window);
  applyEventsFilter();
  closeActiveEvent();
});

watch([wcsWheelchair, wcsGender], ([wheelchair, gender]) => {
  // Drop each param at its "no condition" default, so a bare /map URL stays clean.
  setQueryParam(WCS_WHEELCHAIR_QUERY_PARAM, wheelchair ? "true" : null);
  setQueryParam(WCS_GENDER_QUERY_PARAM, gender);
  applyFilterDim();
});

// The window filter compares against the clock at evaluation time; re-evaluate it periodically so
// markers drop off as their events end while the page stays open.
useIntervalFn(applyEventsFilter, 60_000);

watch(panelCollapsed, (collapsed) => {
  localStorage.setItem(PANEL_COLLAPSED_STORAGE_KEY, collapsed ? "1" : "0");
});

function truthy(value: unknown): boolean {
  return value === true || value === 1 || value === "true";
}

function closeRoomPopup(): void {
  popupInstance.value?.remove();
  popupInstance.value = undefined;
  roomPopup.value = null;
  popupTarget.value = null;
}

/** Anchor a fresh MapLibre popup and teleport the Vue body into its content element. */
function openRoomPopup(state: IndoorRoomPopupProps): void {
  const m = map.value;
  if (!m) return;
  popupInstance.value?.remove();
  const container = document.createElement("div");
  roomPopup.value = state;
  popupTarget.value = container;
  const popup = new Popup({ closeButton: true, closeOnClick: false })
    .setLngLat([state.lng, state.lat])
    .setDOMContent(container)
    .addTo(m);
  // The close button removes the popup imperatively; drop the Vue body so the teleport unmounts.
  popup.on("close", () => {
    roomPopup.value = null;
    popupTarget.value = null;
  });
  popupInstance.value = popup;
}

/**
 * Merge the topmost room fill and POI under the cursor into one popup: the room polygon
 * carries `ref_tum`, the toilet/shower flags ride on whichever feature has them. A bare
 * toilet/shower node without a room keeps the link-less popup; anything else closes it.
 */
function onIndoorClick(event: MapMouseEvent): void {
  const m = map.value;
  if (!m) return;
  const features = m.queryRenderedFeatures(event.point, { layers: [...INDOOR_INTERACTIVE_LAYERS] });
  const room = features.find((f) => f.layer.id === ROOM_LAYER);
  const poi = features.find((f) => f.layer.id === POI_LAYER);

  const refTumRaw = room?.properties?.ref_tum;
  const refTum = typeof refTumRaw === "string" && refTumRaw.length > 0 ? refTumRaw : null;
  const indoor = poi?.properties?.indoor ?? room?.properties?.indoor;
  const isToilet = indoor === "toilet";
  const isShower = indoor === "shower";

  // Untagged, non-toilet features identify nothing and have no fix to offer - leave them inert.
  if (!refTum && !isToilet && !isShower) {
    closeRoomPopup();
    return;
  }

  const flag = (key: string): boolean => truthy(poi?.properties?.[key] ?? room?.properties?.[key]);
  openRoomPopup({
    refTum,
    lat: event.lngLat.lat,
    lng: event.lngLat.lng,
    zoom: currentZoom.value,
    level: currentLevel.value ?? 0,
    isToilet,
    isShower,
    isMale: flag("is_male_toilet"),
    isFemale: flag("is_female_toilet"),
    isWheelchair: flag("is_wheelchair_toilet"),
  });
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
    applyOverlayVisibility();
    applyEventsFilter();

    m.on("click", onIndoorClick);
    for (const layer of INDOOR_INTERACTIVE_LAYERS) {
      m.on("mouseenter", layer, () => {
        m.getCanvas().style.cursor = "pointer";
      });
      m.on("mouseleave", layer, () => {
        m.getCanvas().style.cursor = "";
      });
    }
  });

  floorControl.on("level-changed", (event: { level: number | null }) => {
    currentLevel.value = event.level;
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
  closeRoomPopup();
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
        <Teleport v-if="popupTarget && roomPopup" :to="popupTarget">
          <IndoorRoomPopup v-bind="roomPopup" />
        </Teleport>
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
    24h: Nächste 24 Stunden
  wcs_filters:
    legend: Filter für Toiletten & Duschen
    wheelchair: Nur rollstuhlgerecht
    all_genders: Alle Geschlechter
  gender:
    male: Herren
    female: Damen
en:
  title: Map
  description: Browse the TUM map and filter for places such as toilets, showers, and events.
  events_window:
    legend: Time window for events
    now: Happening now
    24h: Next 24 hours
  wcs_filters:
    legend: Filters for toilets & showers
    wheelchair: Wheelchair-accessible only
    all_genders: All genders
  gender:
    male: Male
    female: Female
</i18n>

<style lang="postcss">
@import "maplibre-gl/dist/maplibre-gl.css";

/* Popup stays white in both themes; pin dark text so nightwind cannot invert it. */
.maplibregl-popup-content {
  background: var(--color-white);
  color: var(--color-zinc-800);
}
</style>
