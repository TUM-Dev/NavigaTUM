import type { components } from "~/api_types";

type LocationType = components["schemas"]["LocationTypeResponse"];

/** Zoom level for framing a single location of the given type. */
export function zoomForLocationType(type: LocationType | undefined): number {
  if (type === "building") return 17;
  if (type === "room") return 18;
  return 16;
}

/** Deep link to the Browse map, framed via the MapLibre `#zoom/lat/lng` hash so `/map` needs no API call. */
export function browseMapUrl(
  coords: { readonly lat: number; readonly lon: number },
  type: LocationType | undefined
): string {
  return `/map#${zoomForLocationType(type)}/${coords.lat.toFixed(5)}/${coords.lon.toFixed(5)}`;
}
