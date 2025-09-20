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
      maxLat = Math.min(maxLat, coord.lat);
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
 * Extract all stops from an itinerary
 */
export function extractAllStops(itinerary: ItineraryResponse): PlaceResponse[] {
  const stops: PlaceResponse[] = [];

  for (const leg of itinerary.legs) {
    // Add starting point (avoid duplicates)
    if (!stops.some((stop) => stop.lat === leg.from.lat && stop.lon === leg.from.lon)) {
      stops.push(leg.from);
    }

    // Add intermediate stops
    if (leg.intermediate_stops) {
      for (const stop of leg.intermediate_stops) {
        if (!stops.some((s) => s.lat === stop.lat && s.lon === stop.lon)) {
          stops.push(stop);
        }
      }
    }

    // Add ending point (avoid duplicates)
    if (!stops.some((stop) => stop.lat === leg.to.lat && stop.lon === leg.to.lon)) {
      stops.push(leg.to);
    }
  }

  return stops;
}

/**
 * Extract only transfer stops (where users change vehicles)
 */
export function extractTransferStops(itinerary: ItineraryResponse): PlaceResponse[] {
  const transferStops: PlaceResponse[] = [];

  // First stop is the origin
  if (itinerary.legs.length > 0 && itinerary.legs[0]) {
    transferStops.push(itinerary.legs[0].from);
  }

  // Add intermediate transfer points (where one leg ends and another begins)
  for (let i = 0; i < itinerary.legs.length - 1; i++) {
    const currentLeg = itinerary.legs[i];
    const nextLeg = itinerary.legs[i + 1];

    // If this isn't a walking connection, it's a transfer
    if (currentLeg && nextLeg && (currentLeg.mode !== "walk" || nextLeg.mode !== "walk")) {
      transferStops.push(currentLeg.to);
    }
  }

  // Last stop is the destination
  if (itinerary.legs.length > 0) {
    const lastLeg = itinerary.legs[itinerary.legs.length - 1];
    if (lastLeg) {
      transferStops.push(lastLeg.to);
    }
  }

  return transferStops;
}

/**
 * Get styling for different transport modes
 */
export function getTransitModeStyle(mode: ModeResponse): {
  color: string;
  weight: number;
  opacity: number;
  dashArray?: string;
} {
  const styles: Record<
    string,
    {
      color: string;
      weight: number;
      opacity: number;
      dashArray?: string;
    }
  > = {
    walk: {
      color: "#6B7280",
      weight: 3,
      dashArray: "5,5",
      opacity: 0.8,
    },
    bus: {
      color: "#EF4444",
      weight: 4,
      opacity: 1.0,
    },
    rail: {
      color: "#3B82F6",
      weight: 5,
      opacity: 1.0,
    },
    subway: {
      color: "#8B5CF6",
      weight: 5,
      opacity: 1.0,
    },
    metro: {
      color: "#8B5CF6",
      weight: 5,
      opacity: 1.0,
    },
    tram: {
      color: "#F59E0B",
      weight: 4,
      opacity: 1.0,
    },
    regional_rail: {
      color: "#059669",
      weight: 5,
      opacity: 1.0,
    },
    regional_fast_rail: {
      color: "#059669",
      weight: 5,
      opacity: 1.0,
    },
    long_distance: {
      color: "#DC2626",
      weight: 6,
      opacity: 1.0,
    },
    night_rail: {
      color: "#1F2937",
      weight: 6,
      opacity: 1.0,
    },
    highspeed_rail: {
      color: "#7C2D12",
      weight: 6,
      opacity: 1.0,
    },
    ferry: {
      color: "#0891B2",
      weight: 4,
      opacity: 1.0,
    },
    airplane: {
      color: "#6366F1",
      weight: 4,
      opacity: 1.0,
    },
    coach: {
      color: "#DC2626",
      weight: 4,
      opacity: 1.0,
    },
    cable_car: {
      color: "#9CA3AF",
      weight: 3,
      opacity: 1.0,
    },
    funicular: {
      color: "#6B7280",
      weight: 3,
      opacity: 1.0,
    },
    areal_lift: {
      color: "#E5E7EB",
      weight: 2,
      opacity: 1.0,
    },
    bike: {
      color: "#16A34A",
      weight: 3,
      opacity: 0.9,
    },
    rental: {
      color: "#CA8A04",
      weight: 3,
      opacity: 0.9,
    },
    car: {
      color: "#374151",
      weight: 4,
      opacity: 0.9,
    },
    car_parking: {
      color: "#6B7280",
      weight: 3,
      opacity: 0.8,
    },
    odm: {
      color: "#8B5CF6",
      weight: 3,
      opacity: 0.9,
    },
    flex: {
      color: "#F59E0B",
      weight: 3,
      opacity: 0.9,
    },
    transit: {
      color: "#3B82F6",
      weight: 4,
      opacity: 1.0,
    },
    other: {
      color: "#9CA3AF",
      weight: 3,
      opacity: 0.8,
    },
  };

  return (
    styles[mode] || {
      color: "#6B7280",
      weight: 3,
      opacity: 0.8,
    }
  );
}

/**
 * Get marker styling for different stop types
 */
export function getStopMarkerStyle(stop: PlaceResponse): {
  color: string;
  size: "small" | "medium" | "large";
  icon?: string;
} {
  // Check if it's a major station/transfer point
  if (stop.vertex_type === "transit") {
    return {
      color: "#3B82F6",
      size: "large",
      icon: "transit",
    };
  }

  // Regular stop
  return {
    color: "#6B7280",
    size: "medium",
    icon: "stop",
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
