import { decode } from "@googlemaps/polyline-codec";
import type { components } from "~/api_types";
import { SELECTABLE_LEVELS } from "~/composables/mapLayers";

type ItineraryResponse = components["schemas"]["ItineraryResponse"];
type ModeResponse = components["schemas"]["ModeResponse"];
type Coordinate = components["schemas"]["Coordinate"];
type PlaceResponse = components["schemas"]["PlaceResponse"];
type StepInstructionResponse = components["schemas"]["StepInstructionResponse"];
type MotisLegResponse = components["schemas"]["MotisLegResponse"];
type AlertResponse = components["schemas"]["AlertResponse"];

// Platform change marker type
export interface PlatformChangeMarker {
  lat: number;
  lon: number;
  name: string;
  fromPlatform: string;
  toPlatform: string;
}

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
 * Calculate the bounding box of a single step's polyline, or `null` when the
 * polyline cannot be decoded.
 */
export function calculateStepBounds(step: StepInstructionResponse): {
  minLat: number;
  maxLat: number;
  minLon: number;
  maxLon: number;
} | null {
  const coordinates = decodeMotisGeometry(step.polyline);
  const first = coordinates[0];
  if (!first) return null;

  let minLat = first.lat;
  let maxLat = first.lat;
  let minLon = first.lon;
  let maxLon = first.lon;

  for (const coord of coordinates) {
    minLat = Math.min(minLat, coord.lat);
    maxLat = Math.max(maxLat, coord.lat);
    minLon = Math.min(minLon, coord.lon);
    maxLon = Math.max(maxLon, coord.lon);
  }

  return { minLat, maxLat, minLon, maxLon };
}

/**
 * Extract platform change markers from an itinerary
 */
export function extractPlatformChangeMarkers(itinerary: ItineraryResponse): PlatformChangeMarker[] {
  const markers: PlatformChangeMarker[] = [];

  // Find all transit legs (non-walking)
  const transitLegs = itinerary.legs.filter((leg) => leg.mode !== "walk");

  for (let i = 0; i < transitLegs.length - 1; i++) {
    const currentLeg = transitLegs[i];
    const nextLeg = transitLegs[i + 1];

    if (!currentLeg || !nextLeg) continue;

    // Only show platform changes for exact same station with different tracks
    const isTransferAtSameStation = currentLeg.to.name === nextLeg.from.name;
    const hasBothTracks = currentLeg.to.track && nextLeg.from.track;
    const isDifferentPlatforms = hasBothTracks && currentLeg.to.track !== nextLeg.from.track;

    if (isTransferAtSameStation && isDifferentPlatforms) {
      const marker = {
        lat: currentLeg.to.lat,
        lon: currentLeg.to.lon,
        name: currentLeg.to.name,
        fromPlatform: currentLeg.to.track || "",
        toPlatform: nextLeg.from.track || "",
      };

      markers.push(marker);
    }
  }

  return markers;
}

// A segment only ghosts with the floor selector when it sits on a floor the selector can
// represent. Motis reports level 0 for outdoor geometry too, so level 0 always renders solid.
export function isFloorSelectableLevel(level: number): boolean {
  return level !== 0 && SELECTABLE_LEVELS.includes(level);
}

/** A contiguous run of a leg's geometry that stays on a single OSM level. */
export interface LegLevelSegment {
  readonly level: number;
  // Whether this run participates in floor-selector ghosting (see isFloorSelectableLevel).
  readonly floorSelectable: boolean;
  readonly coordinates: Coordinate[];
}

/**
 * Split a leg's geometry into runs that each stay on one OSM level, so the map can render
 * the part on the active floor solid and ghost the rest. Self-navigated legs are cut along
 * their steps' `from_level`; legs without usable step geometry stay a single run.
 */
export function splitLegByLevel(leg: MotisLegResponse): LegLevelSegment[] {
  const steps = leg.steps ?? [];
  const segments: LegLevelSegment[] = [];
  for (const step of steps) {
    const coordinates = decodeMotisGeometry(step.polyline);
    if (coordinates.length === 0) continue;
    const level = step.from_level;
    const last = segments[segments.length - 1];
    // Steps on the same level draw as one line; the shared vertex would otherwise double up.
    if (last && last.level === level) last.coordinates.push(...coordinates.slice(1));
    else segments.push({ level, floorSelectable: isFloorSelectableLevel(level), coordinates });
  }
  if (segments.length > 0) return segments;

  const level = leg.from.level;
  return [
    {
      level,
      floorSelectable: isFloorSelectableLevel(level),
      coordinates: decodeMotisGeometry(leg.leg_geometry),
    },
  ];
}

/**
 * The floors a self-navigated leg touches (endpoints and every step), lowest first, or an
 * empty array when the leg never leaves a single floor. Lets the list flag vertical movement
 * on a collapsed leg without expanding it.
 */
export function legFloorSpan(leg: MotisLegResponse): number[] {
  const levels = new Set<number>([leg.from.level, leg.to.level]);
  for (const step of leg.steps ?? []) {
    levels.add(step.from_level);
    levels.add(step.to_level);
  }
  if (levels.size <= 1) return [];
  return [...levels].sort((a, b) => a - b);
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

// Below this, a deviation is jitter in the feed rather than something a traveller can act on.
const DELAY_DEADBAND_MS = 30_000;

/**
 * Minutes a trip runs behind (positive) or ahead of (negative) its timetable slot,
 * or `null` when there is nothing to compare or the deviation is within the deadband.
 */
export function delayMinutes(
  scheduled: string | null | undefined,
  actual: string | null | undefined
): number | null {
  if (!scheduled || !actual) return null;
  const diff = Date.parse(actual) - Date.parse(scheduled);
  if (Number.isNaN(diff) || Math.abs(diff) < DELAY_DEADBAND_MS) return null;
  return Math.round(diff / 60_000);
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

// Modes covered under own power (turn-by-turn steps), not a boarded transit vehicle.
const SELF_NAVIGATED_MODES: ReadonlySet<ModeResponse> = new Set([
  "walk",
  "bike",
  "car",
  "car_parking",
]);

export function isSelfNavigatedLeg(leg: MotisLegResponse): boolean {
  return SELF_NAVIGATED_MODES.has(leg.mode);
}

export interface TimelineTime {
  readonly scheduled: string;
  readonly actual: string;
  readonly realTime: boolean;
  readonly cancelled: boolean;
}

export interface TimelineNode {
  readonly name: string;
  readonly track: string | null;
  readonly level: number;
  readonly alerts: readonly AlertResponse[];
  readonly time: TimelineTime;
}

export interface TimelineEdge {
  readonly leg: MotisLegResponse;
  readonly legIndex: number;
  readonly selfNavigated: boolean;
}

export interface ItineraryTimeline {
  // Always `edges.length + 1` entries: one leading departure, one per leg arrival.
  readonly nodes: readonly TimelineNode[];
  readonly edges: readonly TimelineEdge[];
}

function departureNode(leg: MotisLegResponse): TimelineNode {
  return {
    name: leg.from.name,
    track: leg.from.track ?? leg.from.scheduled_track ?? null,
    level: leg.from.level,
    alerts: leg.from.alerts ?? [],
    time: {
      scheduled: leg.scheduled_start_time,
      actual: leg.start_time,
      realTime: leg.real_time,
      cancelled: leg.cancelled ?? false,
    },
  };
}

function arrivalNode(leg: MotisLegResponse): TimelineNode {
  return {
    name: leg.to.name,
    track: leg.to.track ?? leg.to.scheduled_track ?? null,
    level: leg.to.level,
    alerts: leg.to.alerts ?? [],
    time: {
      scheduled: leg.scheduled_end_time,
      actual: leg.end_time,
      realTime: leg.real_time,
      cancelled: leg.cancelled ?? false,
    },
  };
}

/**
 * Legs as a stops-as-nodes timeline: each physical stop appears once. An interior node
 * shows the boarding time when a transit leg departs from it, else the arrival time.
 */
export function buildItineraryTimeline(itinerary: ItineraryResponse): ItineraryTimeline {
  const legs = itinerary.legs;
  const firstLeg = legs[0];
  if (!firstLeg) return { nodes: [], edges: [] };

  const nodes: TimelineNode[] = [departureNode(firstLeg)];
  for (let i = 1; i < legs.length; i++) {
    const leg = legs[i];
    const previous = legs[i - 1];
    if (!leg || !previous) continue;
    nodes.push(isSelfNavigatedLeg(leg) ? arrivalNode(previous) : departureNode(leg));
  }
  const lastLeg = legs[legs.length - 1];
  if (lastLeg) nodes.push(arrivalNode(lastLeg));

  const edges: TimelineEdge[] = legs.map((leg, legIndex) => ({
    leg,
    legIndex,
    selfNavigated: isSelfNavigatedLeg(leg),
  }));

  return { nodes, edges };
}
