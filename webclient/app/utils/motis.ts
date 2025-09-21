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
 * Extract stops with platform display context and transport mode information
 */
export function extractStopsWithContext(itinerary: ItineraryResponse): any[] {
  const stopsWithContext: any[] = [];

  // Helper to check if a leg is walking
  const isWalkingLeg = (legIndex: number) => {
    const leg = itinerary.legs[legIndex];
    return leg && leg.mode === "walk";
  };

  // Helper to check if this is the journey start (first non-walking leg)
  const isJourneyStart = (legIndex: number) => {
    for (let i = 0; i < legIndex; i++) {
      if (!isWalkingLeg(i)) return false;
    }
    return true;
  };

  // Helper to check if this is the journey end (last non-walking leg)
  const isJourneyEnd = (legIndex: number) => {
    for (let i = legIndex + 1; i < itinerary.legs.length; i++) {
      if (!isWalkingLeg(i)) return false;
    }
    return true;
  };

  // Helper to check if a mode is rail-based
  const isRailMode = (mode: string) => {
    return [
      "rail",
      "highspeed_rail",
      "long_distance",
      "night_rail",
      "regional_fast_rail",
      "regional_rail",
      "subway",
      "metro",
      "tram",
    ].includes(mode);
  };

  for (let legIndex = 0; legIndex < itinerary.legs.length; legIndex++) {
    const leg = itinerary.legs[legIndex];
    if (!leg || leg.mode === "walk") continue;

    const prevLeg = legIndex > 0 ? itinerary.legs[legIndex - 1] : null;

    // Add starting point with context
    const fromStop = {
      ...leg.from,
      showPlatform: false,
      platformText: undefined as string | undefined,
      transportModes: [leg.mode], // Track what transport modes use this stop
      isImportant: isJourneyStart(legIndex) || prevLeg?.mode !== "walk", // Important if journey start or transfer
    };

    if (leg.from.track) {
      if (isJourneyStart(legIndex)) {
        // Journey start: show just platform number
        fromStop.showPlatform = true;
        fromStop.platformText = leg.from.track;
      } else if (prevLeg?.to.track && prevLeg.to.track !== leg.from.track) {
        // Transfer with platform change: show platform transition
        fromStop.showPlatform = true;
        fromStop.platformText = `${prevLeg.to.track} → ${leg.from.track}`;
      }
    }

    // Add if not already present, or merge transport modes if it exists
    const existingStop = stopsWithContext.find(
      (stop) => stop.lat === fromStop.lat && stop.lon === fromStop.lon
    );
    if (existingStop) {
      if (!existingStop.transportModes.includes(leg.mode)) {
        existingStop.transportModes.push(leg.mode);
      }
      // If this is important, mark the existing stop as important
      if (fromStop.isImportant) {
        existingStop.isImportant = true;
      }
    } else {
      stopsWithContext.push(fromStop);
    }

    // Add intermediate stops (never show platform - they are through stations)
    if (leg.intermediate_stops) {
      for (const stop of leg.intermediate_stops) {
        const existingIntermediateStop = stopsWithContext.find(
          (s) => s.lat === stop.lat && s.lon === stop.lon
        );
        if (existingIntermediateStop) {
          if (!existingIntermediateStop.transportModes.includes(leg.mode)) {
            existingIntermediateStop.transportModes.push(leg.mode);
          }
          // Intermediate stops are never important (through stations)
        } else {
          stopsWithContext.push({
            ...stop,
            showPlatform: false,
            transportModes: [leg.mode],
            isImportant: false, // Through stations are not important
          });
        }
      }
    }

    // Add ending point with context
    const toStop = {
      ...leg.to,
      showPlatform: false,
      platformText: undefined as string | undefined,
      transportModes: [leg.mode],
      isImportant:
        isJourneyEnd(legIndex) ||
        (legIndex < itinerary.legs.length - 1 && !isWalkingLeg(legIndex + 1)), // Important if journey end or transfer
    };

    if (leg.to.track && isJourneyEnd(legIndex)) {
      // Journey end: show just platform number
      toStop.showPlatform = true;
      toStop.platformText = leg.to.track;
    }

    // Add if not already present, or merge transport modes if it exists
    const existingToStop = stopsWithContext.find(
      (stop) => stop.lat === toStop.lat && stop.lon === toStop.lon
    );
    if (existingToStop) {
      if (!existingToStop.transportModes.includes(leg.mode)) {
        existingToStop.transportModes.push(leg.mode);
      }
      // If this is important, mark the existing stop as important
      if (toStop.isImportant) {
        existingToStop.isImportant = true;
      }
    } else {
      stopsWithContext.push(toStop);
    }
  }

  return stopsWithContext;
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
export function getStopMarkerStyle(stop: any): {
  color: string;
  size: "small" | "medium" | "large";
  icon?: string;
} {
  // Helper to check if any transport mode is rail-based
  const isRailStation = (transportModes?: string[]) => {
    if (!transportModes) return false;
    return transportModes.some((mode) =>
      [
        "rail",
        "highspeed_rail",
        "long_distance",
        "night_rail",
        "regional_fast_rail",
        "regional_rail",
        "subway",
        "metro",
      ].includes(mode)
    );
  };

  // Helper to check if any transport mode is tram
  const isTramStop = (transportModes?: string[]) => {
    if (!transportModes) return false;
    return transportModes.includes("tram");
  };

  // Helper to check if any transport mode is bus
  const isBusStop = (transportModes?: string[]) => {
    if (!transportModes) return false;
    return transportModes.some((mode) => ["bus", "coach"].includes(mode));
  };

  // Determine size based on importance
  const size = stop.isImportant ? "large" : "small";

  // Check if it's a train station (rail-based transport)
  if (isRailStation(stop.transportModes)) {
    return {
      color: "#3B82F6", // Blue for train stations
      size: size,
      icon: "train",
    };
  }

  // Check if it's a tram stop
  if (isTramStop(stop.transportModes)) {
    return {
      color: "#F59E0B", // Orange for tram
      size: size,
      icon: "tram",
    };
  }

  // Check if it's a bus stop
  if (isBusStop(stop.transportModes)) {
    return {
      color: "#EF4444", // Red for bus
      size: size,
      icon: "bus",
    };
  }

  // Check if it's a major station/transfer point (fallback)
  if (stop.vertex_type === "transit") {
    return {
      color: "#3B82F6",
      size: stop.isImportant ? "large" : "medium",
      icon: "transit",
    };
  }

  // Regular stop
  return {
    color: "#6B7280",
    size: stop.isImportant ? "medium" : "small",
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
