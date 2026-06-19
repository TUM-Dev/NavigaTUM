import type { RequestTransformFunction, StyleSpecification } from "maplibre-gl";

import type { components } from "~/api_types";

type LocationType = components["schemas"]["LocationTypeResponse"];

// martin vector tiles (…/martin/<source>/<z>/<x>/<y>); skips raster floors and TileJSON.
const MARTIN_VECTOR_TILE = /\/martin\/(?!level_)[^/]+\/\d+\/\d+\/\d+/;

// Asks martin for MLT instead of MVT. Needs the sources marked encoding:"mlt" (see loadBasemapStyle),
// or MapLibre feeds the MLT bytes to the MVT parser.
export const mltTransformRequest: RequestTransformFunction = (url) => {
  if (MARTIN_VECTOR_TILE.test(url))
    return { url, headers: { Accept: "application/vnd.maplibre-tile" } };
  return { url };
};

let basemapStyleText: Promise<string> | undefined;

function fetchBasemapStyle(): Promise<string> {
  const pending = fetch("https://nav.tum.de/martin/style/navigatum-basemap.json").then(
    (response) => {
      if (!response.ok)
        throw new Error(`failed to load basemap style: ${response.status} ${response.statusText}`);
      return response.text();
    }
  );
  // Drop a failed fetch so the next map retries instead of reusing the rejection.
  pending.catch(() => {
    if (basemapStyleText === pending) basemapStyleText = undefined;
  });
  return pending;
}

// Marks vector sources encoding:"mlt" so MapLibre decodes the MLT mltTransformRequest asks for. Done
// client-side to keep the shared style MVT-compatible. Fresh object per call (MapLibre mutates it).
export async function loadBasemapStyle(): Promise<StyleSpecification> {
  basemapStyleText ??= fetchBasemapStyle();
  const pending = basemapStyleText;
  const style: StyleSpecification = JSON.parse(await pending);
  for (const source of Object.values(style.sources)) {
    if (source.type === "vector") source.encoding = "mlt";
  }
  return style;
}

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
