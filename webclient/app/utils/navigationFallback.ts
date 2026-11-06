import type { operations } from "~/api_types";

type NavigationResponse =
  operations["route_handler"]["responses"][200]["content"]["application/json"];
type RouteCosting = operations["route_handler"]["parameters"]["query"]["route_costing"];

export function needsNavigationFallback(
  from: string,
  to: string,
  status: string,
  data?: NavigationResponse | null
): boolean {
  if (!from || !to) return false;
  if (status === "error") return true;
  return (
    status === "success" &&
    data?.router === "motis" &&
    data.direct.length === 0 &&
    data.itineraries.length === 0
  );
}

export async function googleMapsEndpoint(
  id: string,
  label: string,
  loadCoords: (id: string) => Promise<{ lat: number; lon: number }>
): Promise<string | undefined> {
  const coordinate = id.startsWith("coord:") ? id.slice(6) : id;
  const parts = coordinate.split(",");
  if (parts.length === 2 && parts.every((part) => part.trim() !== "")) {
    const lat = Number(parts[0]);
    const lon = Number(parts[1]);
    if (Number.isFinite(lat) && Number.isFinite(lon) && Math.abs(lat) <= 90 && Math.abs(lon) <= 180)
      return `${lat},${lon}`;
  }
  try {
    const coords = await loadCoords(id);
    return `${coords.lat},${coords.lon}`;
  } catch {
    // External address IDs are not location-detail IDs; use the selected address instead.
    return label.trim() || undefined;
  }
}

export function googleMapsDirectionsUrl(
  from: string | undefined,
  to: string | undefined,
  mode: RouteCosting
): string | undefined {
  if (!to) return undefined;
  const params = new URLSearchParams({ api: "1", destination: to });
  if (from) params.set("origin", from);
  const travelMode = {
    pedestrian: "walking",
    bicycle: "bicycling",
    car: "driving",
    motorcycle: "driving",
    public_transit: "transit",
  } as const;
  if (mode) params.set("travelmode", travelMode[mode]);
  return `https://www.google.com/maps/dir/?${params}`;
}
