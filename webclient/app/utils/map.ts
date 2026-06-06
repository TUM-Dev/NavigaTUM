import type { components } from "~/api_types";

type LocationType = components["schemas"]["LocationTypeResponse"];

/** Zoom level for framing a single location of the given type. */
export function zoomForLocationType(type: LocationType | undefined): number {
  if (type === "building") return 17;
  if (type === "room") return 18;
  return 16;
}
