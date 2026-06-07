import type { Map as MapLibreMap, MapSourceDataEvent } from "maplibre-gl";
import { Marker } from "maplibre-gl";
import type { MaybeRefOrGetter } from "vue";

const SOURCE_ID = "events_active";

// Markers render via this class; the .event-marker styles live on the consuming map component
// alongside the other map CSS.
function createMarkerElement(image: string, name: string): HTMLElement {
  const wrapper = document.createElement("div");
  wrapper.className = "event-marker";
  wrapper.title = name;
  if (!image) return wrapper;
  const img = document.createElement("img");
  img.src = image;
  img.alt = name;
  img.loading = "lazy";
  img.decoding = "async";
  img.draggable = false;
  img.addEventListener("error", () => img.remove(), { once: true });
  wrapper.appendChild(img);
  return wrapper;
}

/**
 * Renders the Martin `events_active` vector source as photo markers on the given map.
 *
 * `querySourceFeatures` is used (not `queryRenderedFeatures`) so we never need a visible
 * styling layer just to keep the features queryable.
 */
export function useEventMarkers(map: MaybeRefOrGetter<MapLibreMap | undefined>): void {
  const { public: publicConfig } = useRuntimeConfig();
  const markers = new Map<string, Marker>();

  const sync = (target: MapLibreMap) => {
    if (!target.getSource(SOURCE_ID)) return;
    const seen = new Set<string>();
    for (const feature of target.querySourceFeatures(SOURCE_ID, { sourceLayer: SOURCE_ID })) {
      if (feature.id === undefined) continue;
      if (feature.geometry.type !== "Point") continue;
      const [lon, lat] = feature.geometry.coordinates;
      if (typeof lon !== "number" || typeof lat !== "number") continue;
      const id = String(feature.id);
      // Tiles overlap on boundaries, so the same feature can come back several times.
      if (seen.has(id)) continue;
      seen.add(id);
      const existing = markers.get(id);
      if (existing) {
        existing.setLngLat([lon, lat]);
        continue;
      }
      const props = feature.properties ?? {};
      const rawImage = typeof props.image === "string" ? props.image : "";
      const name = typeof props.name === "string" ? props.name : "";
      const image = rawImage ? `${publicConfig.cdnURL}${rawImage}` : "";
      const marker = new Marker({ element: createMarkerElement(image, name) }).setLngLat([lon, lat]);
      marker.addTo(target);
      markers.set(id, marker);
    }
    for (const [id, marker] of markers) {
      if (seen.has(id)) continue;
      marker.remove();
      markers.delete(id);
    }
  };

  watchEffect((onCleanup) => {
    const target = toValue(map);
    if (!target) return;

    const onSourceData = (event: MapSourceDataEvent) => {
      if (event.sourceId !== SOURCE_ID || !event.isSourceLoaded) return;
      sync(target);
    };
    const onMoveEnd = () => sync(target);

    const attach = () => {
      if (!target.getSource(SOURCE_ID)) {
        target.addSource(SOURCE_ID, {
          type: "vector",
          url: `https://nav.tum.de/martin/${SOURCE_ID}`,
        });
      }
      target.on("sourcedata", onSourceData);
      target.on("moveend", onMoveEnd);
    };
    if (target.loaded()) attach();
    else target.once("load", attach);

    onCleanup(() => {
      for (const marker of markers.values()) marker.remove();
      markers.clear();
      target.off("sourcedata", onSourceData);
      target.off("moveend", onMoveEnd);
    });
  });
}
