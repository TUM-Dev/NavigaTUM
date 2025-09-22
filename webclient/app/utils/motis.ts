import { decode } from "@googlemaps/polyline-codec";
import type { components } from "~/api_types";

type ItineraryResponse = components["schemas"]["ItineraryResponse"];
type ModeResponse = components["schemas"]["ModeResponse"];
type Coordinate = components["schemas"]["Coordinate"];
type PlaceResponse = components["schemas"]["PlaceResponse"];

/**
 * Decode Motis polyline geometry to coordinates
 */
export function decodeMotisGeometry(legGeometry: string): Coordinate[] {
  try {
    return decode(legGeometry, 6).map(([lat, lon]) => ({ lat, lon }));
  } catch (error) {
    console.error("Failed to decode polyline geometry:", error);
    return [];
  }
}

/**
 * Calculate bounding box for an itinerary
 */
export function calculateItineraryBounds(itinerary: ItineraryResponse): {
  minLat: number;
  maxLat: number;
  minLon: number;
  maxLon: number;
} {
  let minLat = Number.POSITIVE_INFINITY;
  let maxLat = Number.NEGATIVE_INFINITY;
  let minLon = Number.POSITIVE_INFINITY;
  let maxLon = Number.NEGATIVE_INFINITY;

  for (const leg of itinerary.legs) {
    const coordinates = decodeMotisGeometry(leg.leg_geometry);

    for (const coord of coordinates) {
      minLat = Math.min(minLat, coord.lat);
      maxLat = Math.max(maxLat, coord.lat);
      minLon = Math.min(minLon, coord.lon);
      maxLon = Math.max(maxLon, coord.lon);
    }

    // Also include start/end points
    minLat = Math.min(minLat, leg.from.lat, leg.to.lat);
    maxLat = Math.max(maxLat, leg.from.lat, leg.to.lat);
    minLon = Math.min(minLon, leg.from.lon, leg.to.lon);
    maxLon = Math.max(maxLon, leg.from.lon, leg.to.lon);
  }

  return { minLat, maxLat, minLon, maxLon };
}

/**
 * Calculate bounding box for a single leg
 */
export function calculateLegBounds(
  legGeometry: string,
  from: PlaceResponse,
  to: PlaceResponse
): {
  minLat: number;
  maxLat: number;
  minLon: number;
  maxLon: number;
} {
  const coordinates = decodeMotisGeometry(legGeometry);

  let minLat = Math.min(from.lat, to.lat);
  let maxLat = Math.max(from.lat, to.lat);
  let minLon = Math.min(from.lon, to.lon);
  let maxLon = Math.max(from.lon, to.lon);

  for (const coord of coordinates) {
    minLat = Math.min(minLat, coord.lat);
    maxLat = Math.max(maxLat, coord.lat);
    minLon = Math.min(minLon, coord.lon);
    maxLon = Math.max(maxLon, coord.lon);
  }

  return { minLat, maxLat, minLon, maxLon };
}

/**
 * Get styling for different transport modes
 */
export function getTransitModeStyle(
  mode: ModeResponse,
  routeColor: string,
  routeTextColor: string
): {
  color: string;
  textColor: string;
  weight: number;
  opacity: number;
  dashArray?: string;
} {
  // Special case for walking - use dashed gray line
  if (mode === "walk") {
    return {
      color: "#6B7280",
      textColor: "#FFFFFF",
      weight: 3,
      dashArray: "2,3",
      opacity: 0.8,
    };
  }

  // Define weight and opacity based on mode type
  const modeStyles: Record<string, { weight: number; opacity: number }> = {
    bus: { weight: 4, opacity: 1.0 },
    rail: { weight: 5, opacity: 1.0 },
    subway: { weight: 5, opacity: 1.0 },
    metro: { weight: 5, opacity: 1.0 },
    tram: { weight: 4, opacity: 1.0 },
    regional_rail: { weight: 5, opacity: 1.0 },
    regional_fast_rail: { weight: 5, opacity: 1.0 },
    long_distance: { weight: 6, opacity: 1.0 },
    night_rail: { weight: 6, opacity: 1.0 },
    highspeed_rail: { weight: 6, opacity: 1.0 },
    ferry: { weight: 4, opacity: 1.0 },
    airplane: { weight: 4, opacity: 1.0 },
    coach: { weight: 4, opacity: 1.0 },
    cable_car: { weight: 3, opacity: 1.0 },
    funicular: { weight: 3, opacity: 1.0 },
    areal_lift: { weight: 2, opacity: 1.0 },
    bike: { weight: 3, opacity: 0.9 },
    rental: { weight: 3, opacity: 0.9 },
    car: { weight: 4, opacity: 0.9 },
    car_parking: { weight: 3, opacity: 0.8 },
    odm: { weight: 3, opacity: 0.9 },
    flex: { weight: 3, opacity: 0.9 },
    transit: { weight: 4, opacity: 1.0 },
  };

  const styleProps = modeStyles[mode] || { weight: 3, opacity: 0.8 };

  return {
    color: routeColor,
    textColor: routeTextColor,
    weight: styleProps.weight,
    opacity: styleProps.opacity,
  };
}

/**
 * Format time from ISO string to local time
 */
export function formatTime(dateString: string): string {
  return new Date(dateString).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
}

/**
 * Format duration in seconds to human readable format
 */
export function formatDuration(seconds: number): string {
  if (seconds >= 3600) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  }
  if (seconds >= 60) {
    const minutes = Math.ceil(seconds / 60);
    return `${minutes}min`;
  }
  return `${seconds}s`;
}

/**
 * Format distance in meters to human readable format
 */
export function formatDistance(meters: number): string {
  if (meters >= 1000) {
    return `${(meters / 1000).toFixed(1)}km`;
  }
  return `${Math.round(meters)}m`;
}
