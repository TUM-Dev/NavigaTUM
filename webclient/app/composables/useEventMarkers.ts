import type { GeoJSONFeature, Map as MapLibreMap, MapStyleImageMissingEvent } from "maplibre-gl";
import type { MaybeRefOrGetter, Ref } from "vue";

const LAYER_ID = "events_active-symbols";
const IMAGE_PX = 64;

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

// The properties the Martin `events_active` source advertises (matches map/martin/config.yaml).
// The popup renders these verbatim, so coercing here keeps the SFC API a plain string bag.
export interface EventPopupProps {
  readonly id: string;
  readonly name: string;
  readonly description: string;
  readonly imagePath: string;
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
    startsAt: asString(p.starts_at),
    endsAt: asString(p.ends_at),
    orgCode: asString(p.organising_org_code),
    orgNameDe: asString(p.organising_org_name_de),
    orgNameEn: asString(p.organising_org_name_en),
    lngLat: [lng, lat],
  };
}

/**
 * Renders the Martin `events_active` vector source as photo markers on the given map and exposes
 * the currently active event plus its projected screen position. The host component is responsible
 * for rendering the card (as a positioned overlay on desktop, or as a fullscreen sheet on mobile),
 * which keeps the Vue lifecycle entirely in Vue's hands - no DOM hand-off to MapLibre, no detached
 * subtrees that survive a popup close.
 *
 * Markers ride on a native MapLibre symbol layer, so scaling and fade with zoom come from
 * `interpolate-zoom` expressions and rendering stays on the GPU. Per-event photos are
 * registered on demand via `styleimagemissing`.
 */
export function useEventMarkers(map: MaybeRefOrGetter<MapLibreMap | undefined>): {
  readonly activeEvent: Ref<EventPopupProps | null>;
  readonly markerScreenPos: Ref<ScreenPos | null>;
  closeActiveEvent: () => void;
} {
  const { public: publicConfig } = useRuntimeConfig();
  // Tracks images we've already kicked off loading for so concurrent missing-image events
  // don't double-fetch the same URL.
  const pending = new Set<string>();
  const registered = new Set<string>();
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

    const onStyleImageMissing = async (event: MapStyleImageMissingEvent) => {
      const name = event.id;
      if (!name.startsWith("event-") || pending.has(name) || target.hasImage(name)) return;
      pending.add(name);
      try {
        const id = name.slice("event-".length);
        const features = target.querySourceFeatures("events_active", {
          sourceLayer: "events_active",
        });
        const feature = features.find((f) => String(f.id) === id);
        const rawImage =
          feature && typeof feature.properties?.image === "string" ? feature.properties.image : "";
        if (!rawImage) return;
        const imageData = await rasteriseCircular(`${publicConfig.cdnURL}${rawImage}`);
        if (!imageData || target.hasImage(name)) return;
        target.addImage(name, imageData);
        registered.add(name);
      } finally {
        pending.delete(name);
      }
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
      if (!target.getSource("events_active")) {
        target.addSource("events_active", {
          type: "vector",
          url: "https://nav.tum.de/martin/events_active",
          // Markers are invisible below zoom 15, so don't ask Martin for tiles below that.
          minzoom: 15,
        });
      }
      if (!target.getLayer(LAYER_ID)) {
        target.addLayer({
          id: LAYER_ID,
          type: "symbol",
          source: "events_active",
          "source-layer": "events_active",
          // The layer-level guard is what actually stops the SourceCache from fetching tiles
          // at lower zooms; the source `minzoom` only narrows the declared availability range.
          minzoom: 15,
          layout: {
            "icon-image": ["concat", "event-", ["to-string", ["id"]]],
            "icon-size": ["interpolate", ["linear"], ["zoom"], 15, 0.3, 17, 0.7, 20, 1.0],
            "icon-allow-overlap": true,
            "icon-anchor": "center",
          },
          paint: {
            "icon-opacity": ["interpolate", ["linear"], ["zoom"], 15, 0, 17, 1],
          },
        });
      }
      target.on("styleimagemissing", onStyleImageMissing);
      target.on("click", LAYER_ID, onMarkerClick);
      target.on("mouseenter", LAYER_ID, onMarkerEnter);
      target.on("mouseleave", LAYER_ID, onMarkerLeave);
      target.on("move", projectActiveMarker);
    };
    if (target.loaded()) attach();
    else target.once("load", attach);

    onCleanup(() => {
      target.off("styleimagemissing", onStyleImageMissing);
      target.off("click", LAYER_ID, onMarkerClick);
      target.off("mouseenter", LAYER_ID, onMarkerEnter);
      target.off("mouseleave", LAYER_ID, onMarkerLeave);
      target.off("move", projectActiveMarker);
      activeEvent.value = null;
      markerScreenPos.value = null;
      if (target.getLayer(LAYER_ID)) target.removeLayer(LAYER_ID);
      if (target.getSource("events_active")) target.removeSource("events_active");
      for (const name of registered) {
        if (target.hasImage(name)) target.removeImage(name);
      }
      registered.clear();
      pending.clear();
    });
  });

  return { activeEvent, markerScreenPos, closeActiveEvent };
}
