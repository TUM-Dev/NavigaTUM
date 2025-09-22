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

  // Helper to find the next non-walking leg
  const findNextTransitLeg = (startIndex: number) => {
    for (let i = startIndex + 1; i < itinerary.legs.length; i++) {
      if (!isWalkingLeg(i)) return i;
    }
    return -1;
  };

  // Helper to find the previous non-walking leg
  const findPreviousTransitLeg = (startIndex: number) => {
    for (let i = startIndex - 1; i >= 0; i--) {
      if (!isWalkingLeg(i)) return i;
    }
    return -1;
  };

  for (let legIndex = 0; legIndex < itinerary.legs.length; legIndex++) {
    const leg = itinerary.legs[legIndex];
    if (!leg || leg.mode === "walk") continue;

    const prevTransitLegIndex = findPreviousTransitLeg(legIndex);
    const nextTransitLegIndex = findNextTransitLeg(legIndex);
    const prevTransitLeg = prevTransitLegIndex >= 0 ? itinerary.legs[prevTransitLegIndex] : null;
    const nextTransitLeg = nextTransitLegIndex >= 0 ? itinerary.legs[nextTransitLegIndex] : null;

    // Determine if this is a transfer stop
    const isTransferStart = prevTransitLeg && prevTransitLeg.to.name === leg.from.name;
    const isTransferEnd = nextTransitLeg && leg.to.name === nextTransitLeg.from.name;

    // Add starting point with context
    const fromStop = {
      ...leg.from,
      showPlatform: false,
      platformText: undefined as string | undefined,
      transportModes: [leg.mode],
      isImportant: isJourneyStart(legIndex) || isTransferStart,
      isTransfer: isTransferStart,
      transferType: undefined as string | undefined,
    };

    // Handle platform display for departure points
    if (leg.from.track) {
      if (isJourneyStart(legIndex)) {
        // Journey start: show departure platform
        fromStop.showPlatform = true;
        fromStop.platformText = `Pl. ${leg.from.track}`;
      } else if (isTransferStart && prevTransitLeg?.to.track) {
        // Transfer: show platform change with more explicit format
        const fromPlatform = prevTransitLeg.to.track;
        const toPlatform = leg.from.track;
        if (fromPlatform !== toPlatform) {
          fromStop.showPlatform = true;
          fromStop.platformText = `${fromPlatform} → ${toPlatform}`;
          fromStop.transferType = "platform_change";
        } else {
          // Same platform transfer
          fromStop.showPlatform = true;
          fromStop.platformText = `Pl. ${toPlatform}`;
          fromStop.transferType = "same_platform";
        }
      }
    }

    // Add if not already present, or merge transport modes if it exists
    const existingStop = stopsWithContext.find(
      (stop) =>
        Math.abs(stop.lat - fromStop.lat) < 0.0001 && Math.abs(stop.lon - fromStop.lon) < 0.0001
    );
    if (existingStop) {
      if (!existingStop.transportModes.includes(leg.mode)) {
        existingStop.transportModes.push(leg.mode);
      }
      // Merge platform information if this has more specific info
      if (fromStop.showPlatform && fromStop.platformText) {
        existingStop.showPlatform = true;
        existingStop.platformText = fromStop.platformText;
        existingStop.transferType = fromStop.transferType;
      }
      if (fromStop.isImportant) {
        existingStop.isImportant = true;
      }
      if (fromStop.isTransfer) {
        existingStop.isTransfer = true;
      }
    } else {
      stopsWithContext.push(fromStop);
    }

    // Add intermediate stops (through stations - minimal display)
    if (leg.intermediate_stops) {
      for (const stop of leg.intermediate_stops) {
        const existingIntermediateStop = stopsWithContext.find(
          (s) => Math.abs(s.lat - stop.lat) < 0.0001 && Math.abs(s.lon - stop.lon) < 0.0001
        );
        if (existingIntermediateStop) {
          if (!existingIntermediateStop.transportModes.includes(leg.mode)) {
            existingIntermediateStop.transportModes.push(leg.mode);
          }
        } else {
          stopsWithContext.push({
            ...stop,
            showPlatform: false,
            transportModes: [leg.mode],
            isImportant: false,
            isTransfer: false,
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
      isImportant: isJourneyEnd(legIndex) || isTransferEnd,
      isTransfer: isTransferEnd,
      transferType: undefined as string | undefined,
    };

    // Handle platform display for arrival points
    if (leg.to.track && isJourneyEnd(legIndex)) {
      // Journey end: show arrival platform
      toStop.showPlatform = true;
      toStop.platformText = `Pl. ${leg.to.track}`;
    }

    // Add if not already present, or merge transport modes if it exists
    const existingToStop = stopsWithContext.find(
      (stop) => Math.abs(stop.lat - toStop.lat) < 0.0001 && Math.abs(stop.lon - toStop.lon) < 0.0001
    );
    if (existingToStop) {
      if (!existingToStop.transportModes.includes(leg.mode)) {
        existingToStop.transportModes.push(leg.mode);
      }
      // Merge platform information if this has more specific info
      if (toStop.showPlatform && toStop.platformText) {
        existingToStop.showPlatform = true;
        existingToStop.platformText = toStop.platformText;
        existingToStop.transferType = toStop.transferType;
      }
      if (toStop.isImportant) {
        existingToStop.isImportant = true;
      }
      if (toStop.isTransfer) {
        existingToStop.isTransfer = true;
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
export function getTransitModeStyle(
  mode: ModeResponse,
  routeColor?: string
): {
  color: string;
  weight: number;
  opacity: number;
  dashArray?: string;
} {
  // Special case for walking - use dashed gray line
  if (mode === "walk") {
    return {
      color: "#6B7280",
      weight: 3,
      dashArray: "2,3",
      opacity: 0.8,
    };
  }

  // For all other modes, use the route color (no fallback)
  const color = routeColor && routeColor.length === 6 ? `#${routeColor}` : "#000000";

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
    color,
    weight: styleProps.weight,
    opacity: styleProps.opacity,
  };
}

/**
 * Get marker styling for different stop types
 */
export function getStopMarkerStyle(
  stop: any,
  routeColor?: string
): {
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

  // Determine size based on importance and transfer status
  let size: "small" | "medium" | "large" = "small";
  if (stop.isTransfer && stop.transferType === "platform_change") {
    size = "large"; // Make platform changes very prominent
  } else if (stop.isImportant) {
    size = "medium";
  }

  // Special styling for transfer stops with platform changes
  if (stop.isTransfer && stop.transferType === "platform_change") {
    return {
      color: routeColor && routeColor.length === 6 ? `#${routeColor}` : "#000000",
      size: "large", // Always make platform changes large
      icon: "platform_change", // Special icon for platform changes
    };
  }

  // Use route color for all stops (no fallbacks except black for missing color)
  const color = routeColor && routeColor.length === 6 ? `#${routeColor}` : "#000000";
  let icon = "circle";

  // Determine icon based on transport modes
  if (isRailStation(stop.transportModes)) {
    icon = "train";
  } else if (isTramStop(stop.transportModes)) {
    icon = "tram";
  } else if (isBusStop(stop.transportModes)) {
    icon = "bus";
  } else if (stop.vertex_type === "transit" || stop.isTransfer) {
    icon = "rail-metro";
  }

  return {
    color,
    size: size,
    icon: icon,
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
