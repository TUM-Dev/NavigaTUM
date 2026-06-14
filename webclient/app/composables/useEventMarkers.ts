import { useIntervalFn } from "@vueuse/core";
import type {
  FilterSpecification,
  GeoJSONFeature,
  Map as MapLibreMap,
  MapStyleImageMissingEvent,
  SymbolLayerSpecification,
} from "maplibre-gl";
import type { MaybeRefOrGetter, Ref } from "vue";
import { type EventSourceId, eventsExpiryFilter } from "~/composables/mapLayers";

const IMAGE_PX = 64;
// Markers are invisible below this zoom (icon/text fade to 0), so neither tiles nor layers exist
// below it; one minzoom shared by every feed keeps the two maps' render identical.
const MARKER_MINZOOM = 15;
// The wall clock advances, so ended markers must drop off even on an idle page; re-evaluate the
// expiry filter on this cadence.
const EXPIRY_INTERVAL_MS = 60_000;

/** The symbol layer id `useEventMarkers` adds for a given tile source. */
function layerIdFor(source: EventSourceId): string {
  return `${source}-symbols`;
}

// Shared photo-marker symbology, identical across feeds so `/map` and the detail map render the
// same thing. Per-event photos register on demand (see `styleimagemissing`); the icon fades in with
// zoom and the name label trails beneath it, dropping when it would collide (`text-optional`).
const MARKER_LAYOUT = {
  "icon-image": ["concat", "event-", ["to-string", ["id"]]],
  "icon-size": ["interpolate", ["linear"], ["zoom"], 15, 0.6, 17, 1.2, 20, 1.7],
  "icon-allow-overlap": true,
  "icon-anchor": "center",
  "text-field": ["get", "name"],
  "text-font": ["Roboto Regular"],
  "text-size": 11,
  "text-anchor": "top",
  // The 64px photo is centred on the point, so its bottom edge sits `32 * icon-size` px below it
  // and moves as the icon scales. Offsets are ems of `text-size` (11px), so track the icon growth
  // across the same zoom stops to keep the label a constant gap under the photo at every zoom.
  "text-offset": [
    "interpolate",
    ["linear"],
    ["zoom"],
    15,
    ["literal", [0, 2.2]],
    17,
    ["literal", [0, 4]],
    20,
    ["literal", [0, 5.4]],
  ],
  "text-optional": true,
} as const satisfies SymbolLayerSpecification["layout"];

const MARKER_PAINT = {
  "icon-opacity": ["interpolate", ["linear"], ["zoom"], 15, 0, 17, 1],
  // Blend the label out faster than the photo: gone below 16, full only at 17.5.
  "text-opacity": ["interpolate", ["linear"], ["zoom"], 16, 0, 17.5, 1],
  // TUM orange with a white halo, carried over from the retired baked events layer.
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
 * cursor and a name tooltip. Only the visible layer ever fires these (hidden layers render no
 * features), so multiple layers never contend for the popup. The host component renders the card
 * (e.g. via `EventPopupOverlay`), which keeps the Vue lifecycle entirely in Vue's hands - no DOM
 * hand-off to MapLibre, no detached subtrees that survive a popup close.
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
  /** Tile feeds to render, each as its own photo-marker symbol layer. */
  readonly sources: readonly EventSourceId[];
  /**
   * Which feeds are currently shown, re-evaluated reactively; defaults to all `sources`. `/map`
   * drives this from the events filter and its window toggle, flipping one layer's visibility for
   * the other; the detail map omits it and leaves its single feed always visible.
   */
  readonly visibleSources?: MaybeRefOrGetter<readonly EventSourceId[]>;
}

/**
 * Renders the given Martin event feeds as photo + name-label markers on the map and exposes the
 * currently active event plus its projected screen position via `useEventPopup`. This is the single
 * render definition shared by `/map` (active + upcoming feeds, one visible at a time) and the
 * building detail map (the active feed only).
 *
 * Markers ride on native MapLibre symbol layers, so scaling and fade with zoom come from
 * `interpolate-zoom` expressions and rendering stays on the GPU. Per-event photos are registered on
 * demand via `styleimagemissing`. Ended markers are retired by the live-expiry filter on an
 * interval (see `eventsExpiryFilter`); appearance is gated entirely server-side, so this never
 * references a start time.
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
  // Tracks images we've already kicked off loading for so concurrent missing-image events
  // don't double-fetch the same URL.
  const pending = new Set<string>();
  const registered = new Set<string>();

  /** Drop markers whose event has ended on whichever feeds currently have a layer. */
  const applyExpiry = (target: MapLibreMap): void => {
    const filter = eventsExpiryFilter(Date.now()) as FilterSpecification;
    for (const source of sources) {
      const layerId = layerIdFor(source);
      if (target.getLayer(layerId)) target.setFilter(layerId, filter);
    }
  };

  /** Show only the requested feeds; the others' layers stay added but hidden. */
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

    const onStyleImageMissing = async (event: MapStyleImageMissingEvent) => {
      const name = event.id;
      if (!name.startsWith("event-") || pending.has(name) || target.hasImage(name)) return;
      pending.add(name);
      try {
        const id = name.slice("event-".length);
        // The same event id can appear in several feeds (upcoming ⊇ active), but its photo is the
        // same image; the first feed carrying it wins.
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
        const imageData = await rasteriseCircular(`${publicConfig.cdnURL}${rawImage}`);
        if (!imageData || target.hasImage(name)) return;
        target.addImage(name, imageData);
        registered.add(name);
      } finally {
        pending.delete(name);
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
    };
    if (target.loaded()) attach();
    else target.once("load", attach);

    onCleanup(() => {
      target.off("load", attach);
      target.off("styleimagemissing", onStyleImageMissing);
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

  // Reactively re-apply visibility as `visibleSources` changes; a no-op until the layers exist,
  // since `attach` applies the initial visibility itself.
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
