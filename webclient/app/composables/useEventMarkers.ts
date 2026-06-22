import { useIntervalFn } from "@vueuse/core";
import type {
  FilterSpecification,
  GeoJSONFeature,
  Map as MapLibreMap,
  MapSourceDataEvent,
  MapStyleImageMissingEvent,
  SymbolLayerSpecification,
} from "maplibre-gl";
import type { MaybeRefOrGetter, Ref } from "vue";
import { type EventSourceId, eventsExpiryFilter } from "~/composables/mapLayers";

const IMAGE_PX = 64;
// Markers fade to 0 below this zoom, so don't add layers or fetch tiles below it.
const MARKER_MINZOOM = 14;
const EXPIRY_INTERVAL_MS = 60_000;

function layerIdFor(source: EventSourceId): string {
  return `${source}-symbols`;
}

// Per-event photos register on demand; see `styleimagemissing`.
const MARKER_LAYOUT = {
  "icon-image": ["concat", "event-", ["to-string", ["id"]]],
  "icon-size": ["interpolate", ["linear"], ["zoom"], 14, 0.6, 16, 1.2, 19, 1.7],
  "icon-allow-overlap": true,
  "icon-anchor": "center",
  "text-field": ["get", "name"],
  "text-font": ["Roboto Regular"],
  "text-size": 11,
  "text-anchor": "top",
  // Offset is ems of text-size, but the photo's bottom edge sits 32 * icon-size px below the centred
  // point; track icon-size's zoom stops so the label keeps a constant gap below the photo.
  "text-offset": [
    "interpolate",
    ["linear"],
    ["zoom"],
    14,
    ["literal", [0, 2.2]],
    16,
    ["literal", [0, 4]],
    19,
    ["literal", [0, 5.4]],
  ],
  "text-optional": true,
} as const satisfies SymbolLayerSpecification["layout"];

const MARKER_PAINT = {
  "icon-opacity": ["interpolate", ["linear"], ["zoom"], 14, 0, 14.7, 1],
  // Fade the label out faster than the photo.
  "text-opacity": ["interpolate", ["linear"], ["zoom"], 14.3, 0, 15, 1],
  "text-color": "#E37222",
  "text-halo-color": "#ffffff",
  "text-halo-width": 1.2,
} as const satisfies SymbolLayerSpecification["paint"];

/**
 * Loads `url` and rasterises it as a circular sprite ready for `map.addImage`.
 * The circular crop is baked in so the symbol layer can render the image directly.
 */
async function rasteriseCircular(url: string): Promise<ImageData | null> {
  const img = new Image();
  img.crossOrigin = "anonymous";
  img.src = url;
  try {
    await img.decode();
  } catch {
    return null;
  }
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = IMAGE_PX;
  const ctx = canvas.getContext("2d");
  if (!ctx) return null;
  const radius = IMAGE_PX / 2;
  ctx.save();
  ctx.beginPath();
  ctx.arc(radius, radius, radius - 2, 0, Math.PI * 2);
  ctx.closePath();
  ctx.clip();
  // cover-fit: scale shorter side to image edge.
  const ratio = Math.max(IMAGE_PX / img.naturalWidth, IMAGE_PX / img.naturalHeight);
  const drawW = img.naturalWidth * ratio;
  const drawH = img.naturalHeight * ratio;
  ctx.drawImage(img, (IMAGE_PX - drawW) / 2, (IMAGE_PX - drawH) / 2, drawW, drawH);
  ctx.restore();
  // White ring around the photo, drawn after the clip is released.
  ctx.beginPath();
  ctx.arc(radius, radius, radius - 2, 0, Math.PI * 2);
  ctx.strokeStyle = "#ffffff";
  ctx.lineWidth = 3;
  ctx.stroke();
  return ctx.getImageData(0, 0, IMAGE_PX, IMAGE_PX);
}

// The properties the Martin event feeds advertise (matches map/martin/config.yaml).
// The popup renders these verbatim, so coercing here keeps the SFC API a plain string bag.
export interface EventPopupProps {
  readonly id: string;
  readonly name: string;
  readonly description: string;
  readonly imagePath: string;
  readonly imageAuthor: string;
  readonly startsAt: string;
  readonly endsAt: string;
  readonly orgCode: string;
  readonly orgNameDe: string;
  readonly orgNameEn: string;
  readonly lngLat: readonly [number, number];
}

export interface ScreenPos {
  readonly x: number;
  readonly y: number;
}

function asString(value: unknown): string {
  return typeof value === "string" ? value : "";
}

function readPopupProps(feature: GeoJSONFeature): EventPopupProps | null {
  const p = feature.properties ?? {};
  if (feature.geometry.type !== "Point") return null;
  const [lng, lat] = feature.geometry.coordinates as [number, number];
  return {
    id: String(feature.id ?? ""),
    name: asString(p.name),
    description: asString(p.description),
    imagePath: asString(p.image),
    imageAuthor: asString(p.image_author),
    startsAt: asString(p.starts_at),
    endsAt: asString(p.ends_at),
    orgCode: asString(p.organising_org_code),
    orgNameDe: asString(p.organising_org_name_de),
    orgNameEn: asString(p.organising_org_name_en),
    lngLat: [lng, lat],
  };
}

/**
 * Wires the event popup onto the given symbol layers (one per event feed): clicking a marker
 * exposes its `EventPopupProps` plus the marker's projected screen position, hovering sets the
 * cursor and a name tooltip. The host component renders the card (e.g. via `EventPopupOverlay`),
 * which keeps the Vue lifecycle in Vue's hands - no DOM hand-off to MapLibre, no detached subtrees
 * that survive a popup close.
 */
export function useEventPopup(
  map: MaybeRefOrGetter<MapLibreMap | undefined>,
  layerIds: readonly string[]
): {
  readonly activeEvent: Ref<EventPopupProps | null>;
  readonly markerScreenPos: Ref<ScreenPos | null>;
  closeActiveEvent: () => void;
} {
  const activeEvent = shallowRef<EventPopupProps | null>(null);
  const markerScreenPos = shallowRef<ScreenPos | null>(null);

  function closeActiveEvent(): void {
    activeEvent.value = null;
    markerScreenPos.value = null;
  }

  watchEffect((onCleanup) => {
    const target = toValue(map);
    if (!target) return;

    const projectActiveMarker = () => {
      const event = activeEvent.value;
      if (!event) {
        markerScreenPos.value = null;
        return;
      }
      const { x, y } = target.project(event.lngLat as [number, number]);
      markerScreenPos.value = { x, y };
    };

    const onMarkerClick = (event: { features?: GeoJSONFeature[] }) => {
      const feature = event.features?.[0];
      if (!feature) return;
      const next = readPopupProps(feature);
      if (!next) return;
      activeEvent.value = next;
      projectActiveMarker();
    };

    const onMarkerEnter = (event: { features?: GeoJSONFeature[] }) => {
      target.getCanvas().style.cursor = "pointer";
      const name = asString(event.features?.[0]?.properties?.name);
      if (name) target.getCanvasContainer().title = name;
    };
    const onMarkerLeave = () => {
      target.getCanvas().style.cursor = "";
      target.getCanvasContainer().title = "";
    };

    const attach = () => {
      for (const layerId of layerIds) {
        target.on("click", layerId, onMarkerClick);
        target.on("mouseenter", layerId, onMarkerEnter);
        target.on("mouseleave", layerId, onMarkerLeave);
      }
      target.on("move", projectActiveMarker);
    };
    if (target.loaded()) attach();
    else target.once("load", attach);

    onCleanup(() => {
      target.off("load", attach);
      for (const layerId of layerIds) {
        target.off("click", layerId, onMarkerClick);
        target.off("mouseenter", layerId, onMarkerEnter);
        target.off("mouseleave", layerId, onMarkerLeave);
      }
      target.off("move", projectActiveMarker);
      closeActiveEvent();
    });
  });

  return { activeEvent, markerScreenPos, closeActiveEvent };
}

export interface UseEventMarkersOptions {
  readonly sources: readonly EventSourceId[];
  /**
   * Feeds currently shown, re-evaluated reactively; defaults to all `sources`. `/map` flips this
   * between the two feeds via its window toggle; the detail map omits it and shows its single feed.
   */
  readonly visibleSources?: MaybeRefOrGetter<readonly EventSourceId[]>;
}

/**
 * Renders the given event feeds as photo + name-label markers and exposes the active event plus its
 * screen position via `useEventPopup`. Shared by `/map` (active + upcoming, one shown at a time) and
 * the detail map (active only). Ended markers are dropped by `eventsExpiryFilter` on an interval.
 */
export function useEventMarkers(
  map: MaybeRefOrGetter<MapLibreMap | undefined>,
  options: UseEventMarkersOptions
): {
  readonly activeEvent: Ref<EventPopupProps | null>;
  readonly markerScreenPos: Ref<ScreenPos | null>;
  closeActiveEvent: () => void;
} {
  const { public: publicConfig } = useRuntimeConfig();
  const { sources } = options;
  const layerIds = sources.map(layerIdFor);
  // Images already kicked off, so concurrent missing-image events don't double-fetch the same URL.
  const pending = new Set<string>();
  const registered = new Set<string>();

  const applyExpiry = (target: MapLibreMap): void => {
    const filter = eventsExpiryFilter(Date.now()) as FilterSpecification;
    for (const source of sources) {
      const layerId = layerIdFor(source);
      if (target.getLayer(layerId)) target.setFilter(layerId, filter);
    }
  };

  const applyVisibility = (target: MapLibreMap): void => {
    const visible = new Set(options.visibleSources ? toValue(options.visibleSources) : sources);
    for (const source of sources) {
      const layerId = layerIdFor(source);
      if (!target.getLayer(layerId)) continue;
      target.setLayoutProperty(layerId, "visibility", visible.has(source) ? "visible" : "none");
    }
  };

  watchEffect((onCleanup) => {
    const target = toValue(map);
    if (!target) return;

    // Rasterise and register one event's photo. `pending` is only set once a queryable feature is
    // found, so a request raised before the feature's tile is ready stays retryable rather than
    // being permanently swallowed.
    const ensureImage = async (name: string): Promise<void> => {
      if (pending.has(name) || target.hasImage(name)) return;
      const id = name.slice("event-".length);
      if (!id) return;
      // An id can appear in several feeds (upcoming ⊇ active) with the same photo; first hit wins.
      let rawImage = "";
      for (const source of sources) {
        const feature = target
          .querySourceFeatures(source, { sourceLayer: source })
          .find((f) => String(f.id) === id);
        if (feature && typeof feature.properties?.image === "string") {
          rawImage = feature.properties.image;
          break;
        }
      }
      if (!rawImage) return;
      pending.add(name);
      try {
        const imageData = await rasteriseCircular(`${publicConfig.cdnURL}${rawImage}`);
        if (imageData && !target.hasImage(name)) {
          target.addImage(name, imageData);
          registered.add(name);
        }
      } finally {
        pending.delete(name);
      }
    };

    const onStyleImageMissing = (event: MapStyleImageMissingEvent) => {
      if (event.id.startsWith("event-")) void ensureImage(event.id);
    };

    // `styleimagemissing` fires once per id and the miss is cached, so one raised before the tile's
    // features are queryable would never recover; re-drive registration when a feed's data settles.
    const onSourceData = (event: MapSourceDataEvent) => {
      if (!event.isSourceLoaded || !sources.some((source) => source === event.sourceId)) return;
      for (const source of sources) {
        for (const feature of target.querySourceFeatures(source, { sourceLayer: source })) {
          const id = String(feature.id ?? "");
          if (id) void ensureImage(`event-${id}`);
        }
      }
    };

    const attach = () => {
      for (const source of sources) {
        if (!target.getSource(source)) {
          target.addSource(source, {
            type: "vector",
            url: `https://nav.tum.de/martin/${source}`,
            // Markers are invisible below MARKER_MINZOOM, so don't ask Martin for tiles below that.
            minzoom: MARKER_MINZOOM,
          });
        }
        const layerId = layerIdFor(source);
        if (!target.getLayer(layerId)) {
          target.addLayer({
            id: layerId,
            type: "symbol",
            source,
            "source-layer": source,
            // The layer-level guard is what actually stops the SourceCache from fetching tiles at
            // lower zooms; the source `minzoom` only narrows the declared availability range.
            minzoom: MARKER_MINZOOM,
            layout: { ...MARKER_LAYOUT },
            paint: { ...MARKER_PAINT },
          });
        }
      }
      applyExpiry(target);
      applyVisibility(target);
      target.on("styleimagemissing", onStyleImageMissing);
      target.on("sourcedata", onSourceData);
    };
    if (target.loaded()) attach();
    else target.once("load", attach);

    onCleanup(() => {
      target.off("load", attach);
      target.off("styleimagemissing", onStyleImageMissing);
      target.off("sourcedata", onSourceData);
      for (const source of sources) {
        const layerId = layerIdFor(source);
        if (target.getLayer(layerId)) target.removeLayer(layerId);
        if (target.getSource(source)) target.removeSource(source);
      }
      for (const name of registered) {
        if (target.hasImage(name)) target.removeImage(name);
      }
      registered.clear();
      pending.clear();
    });
  });

  // Re-apply on `visibleSources` change; a no-op until `attach` has added the layers.
  watchEffect(() => {
    const target = toValue(map);
    if (target) applyVisibility(target);
  });

  // The expiry filter compares against the clock at evaluation time; re-evaluate it periodically so
  // markers drop off as their events end while the page stays open.
  useIntervalFn(() => {
    const target = toValue(map);
    if (target) applyExpiry(target);
  }, EXPIRY_INTERVAL_MS);

  return useEventPopup(map, layerIds);
}
