import type { LngLatLike, Map as MapLibreMap, MapMouseEvent } from "maplibre-gl";
import { Popup } from "maplibre-gl";
import type { MaybeRefOrGetter, Ref } from "vue";
import type { IndoorRoomPopupProps } from "~/components/IndoorRoomPopup.vue";

// The room fill layer; merged with the POIs into one unified popup on click.
export const ROOM_LAYER = "indoor-rooms";
// The combined POI layer whose markers open a popup.
export const POI_LAYER = "indoor-pois";
// Both indoor layers a click is resolved against, and that show a pointer cursor on hover.
export const INDOOR_INTERACTIVE_LAYERS = [ROOM_LAYER, POI_LAYER] as const;

export interface UseIndoorRoomPopupOptions {
  /** Read when a popup opens, so its OSM edit link deep-links the right scale. */
  readonly getZoom: () => number;
  /** Read when a popup opens; `0` when no floor is selected. */
  readonly getLevel: () => number;
}

export interface UseIndoorRoomPopup {
  /** Teleport host for the popup body; `null` when no room popup is open. */
  readonly popupTarget: Ref<HTMLElement | null>;
  /** Props for the teleported `IndoorRoomPopup`; `null` when none is open. */
  readonly roomPopup: Ref<IndoorRoomPopupProps | null>;
  /** Tear down whichever popup (room or raw-DOM) is open. */
  readonly closeRoomPopup: () => void;
  /** Anchor a room popup and teleport the Vue body into it. */
  readonly openRoomPopup: (state: IndoorRoomPopupProps) => void;
  /** Reuse the single popup instance for raw-DOM content (e.g. the card-validator popup). */
  readonly openDomPopup: (lngLat: LngLatLike, content: HTMLElement) => void;
  /** Merge the room and POI under the cursor into one popup; close it when nothing matches. */
  readonly resolveRoomPopupFromClick: (event: MapMouseEvent) => void;
  /** Show a pointer cursor while hovering the given layers. */
  readonly attachHoverCursor: (target: MapLibreMap, layers: readonly string[]) => void;
}

function truthy(value: unknown): boolean {
  return value === true || value === 1 || value === "true";
}

/**
 * The shared indoor room/POI popup: clicking a room polygon or toilet/shower node anchors a
 * MapLibre popup whose body is the Vue `IndoorRoomPopup`, teleported into `popupTarget`. A single
 * popup instance is reused, so opening a new one (or a raw-DOM popup via `openDomPopup`) replaces
 * any open one. Shared by `/map` and the location detail map.
 */
export function useIndoorRoomPopup(
  map: MaybeRefOrGetter<MapLibreMap | undefined>,
  options: UseIndoorRoomPopupOptions
): UseIndoorRoomPopup {
  // `shallowRef`: MapLibre owns the popup's deep state; Vue must not track it reactively.
  const popupInstance = shallowRef<Popup | undefined>(undefined);
  const popupTarget = shallowRef<HTMLElement | null>(null);
  const roomPopup = shallowRef<IndoorRoomPopupProps | null>(null);

  function closeRoomPopup(): void {
    popupInstance.value?.remove();
    popupInstance.value = undefined;
    roomPopup.value = null;
    popupTarget.value = null;
  }

  function openRoomPopup(state: IndoorRoomPopupProps): void {
    const m = toValue(map);
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

  function openDomPopup(lngLat: LngLatLike, content: HTMLElement): void {
    const m = toValue(map);
    if (!m) return;
    closeRoomPopup();
    popupInstance.value = new Popup({ closeButton: true, closeOnClick: false })
      .setLngLat(lngLat)
      .setDOMContent(content)
      .addTo(m);
  }

  function resolveRoomPopupFromClick(event: MapMouseEvent): void {
    const m = toValue(map);
    if (!m) return;

    const queryLayers = INDOOR_INTERACTIVE_LAYERS.filter((layer) => m.getLayer(layer));
    const features = m.queryRenderedFeatures(event.point, { layers: queryLayers });
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

    const flag = (key: string): boolean =>
      truthy(poi?.properties?.[key] ?? room?.properties?.[key]);
    openRoomPopup({
      refTum,
      lat: event.lngLat.lat,
      lng: event.lngLat.lng,
      zoom: options.getZoom(),
      level: options.getLevel(),
      isToilet,
      isShower,
      isMale: flag("is_male_toilet"),
      isFemale: flag("is_female_toilet"),
      isWheelchair: flag("is_wheelchair_toilet"),
    });
  }

  function attachHoverCursor(target: MapLibreMap, layers: readonly string[]): void {
    for (const layer of layers) {
      target.on("mouseenter", layer, () => {
        target.getCanvas().style.cursor = "pointer";
      });
      target.on("mouseleave", layer, () => {
        target.getCanvas().style.cursor = "";
      });
    }
  }

  return {
    popupTarget,
    roomPopup,
    closeRoomPopup,
    openRoomPopup,
    openDomPopup,
    resolveRoomPopupFromClick,
    attachHoverCursor,
  };
}
