// Types and pure derivation for the Studentische Vertretung IRIS learning-room availability card.
//
// The browser-side fetch, per-room alias resolution, and polling live in the
// `useIrisAvailability` composable. Everything here is pure (no Nuxt auto-imports,
// no network) so the parsing and alias-join logic stays unit-testable.

// Public, unauthenticated Studentische Vertretung IRIS endpoints. The API sends `Access-Control-Allow-Origin: *`,
// so the browser may fetch it directly - NavigaTUM never proxies live status.
export const IRIS_API_URL = "https://iris.asta.tum.de/api/";
export const IRIS_SITE_URL = "https://iris.asta.tum.de/";

// The four statuses Iris documents. Any other value is passed through verbatim.
export const IRIS_STATUSES = ["frei", "belegt", "WAAS", "unbekannt"] as const;
export type IrisStatus = (typeof IRIS_STATUSES)[number];

const KNOWN_STATUSES: ReadonlySet<string> = new Set(IRIS_STATUSES);

/** Whether `status` is one of the statuses we localize (vs. an unknown value to pass through). */
export function isKnownStatus(status: string): status is IrisStatus {
  return KNOWN_STATUSES.has(status);
}

export interface IrisOccupancy {
  // Occupancy as a fraction in `[0, 1]`. The raw Wi-Fi-counter value can dip slightly
  // negative around an empty room, so it is clamped on parse.
  readonly percent: number;
  // The colour band Iris computed for this occupancy, a hex string like `#5cb85c`. Rendered as-is.
  readonly color: string;
}

export interface IrisRoom {
  // `<arch_name>@<building_id>` (e.g. `1450@5504`). Globally unique and, when present,
  // a NavigaTUM room alias - this is the key the alias lookup resolves.
  readonly archName: string;
  readonly name: string;
  readonly code: string;
  // The NavigaTUM building id this room belongs to (Iris `gebaeude_code`, 1:1 with our ids).
  readonly buildingId: string;
  readonly status: string;
  // `belegt` rooms only: who booked the room and until when, as raw Iris strings. Null when absent.
  readonly bookedBy: string | null;
  readonly bookedUntil: string | null;
  // `WAAS` (Wi-Fi-counter) rooms only; null otherwise.
  readonly occupancy: IrisOccupancy | null;
}

// An Iris room whose alias resolved to a NavigaTUM entity, ready to render as a linked row.
export interface IrisRoomRow extends IrisRoom {
  // Canonical in-app path of the matched NavigaTUM room, e.g. `/room/5504.01.450`.
  readonly path: string;
}

function asString(value: unknown): string | null {
  if (typeof value === "string") return value;
  if (typeof value === "number" && Number.isFinite(value)) return String(value);
  return null;
}

// Iris encodes "no value" as the empty string rather than omitting the key.
function nonEmpty(value: unknown): string | null {
  const str = asString(value);
  return str !== null && str.trim() !== "" ? str : null;
}

function parseOccupancy(raw: Record<string, unknown>): IrisOccupancy | null {
  if (raw.status !== "WAAS") return null;
  const color = nonEmpty(raw.color);
  const percent = raw.percent;
  if (color === null || typeof percent !== "number" || Number.isNaN(percent)) return null;
  return { percent: Math.min(1, Math.max(0, percent)), color };
}

/**
 * Parse the Studentische Vertretung IRIS `GET /api/` body into the rooms we render.
 *
 * Tolerant by design: a malformed body yields an empty list and individual rooms missing the
 * fields we depend on are skipped, so a partial response degrades gracefully instead of throwing.
 */
export function parseIrisRooms(json: unknown): IrisRoom[] {
  if (typeof json !== "object" || json === null) return [];
  const raeume = (json as { raeume?: unknown }).raeume;
  if (!Array.isArray(raeume)) return [];

  const rooms: IrisRoom[] = [];
  for (const entry of raeume) {
    if (typeof entry !== "object" || entry === null) continue;
    const raw = entry as Record<string, unknown>;
    const archName = nonEmpty(raw.raum_nr_architekt);
    const buildingId = nonEmpty(raw.gebaeude_code);
    const status = nonEmpty(raw.status);
    if (archName === null || buildingId === null || status === null) continue;

    rooms.push({
      archName,
      name: nonEmpty(raw.raum_name) ?? nonEmpty(raw.raum_code) ?? archName,
      code: nonEmpty(raw.raum_code) ?? archName,
      buildingId,
      status,
      bookedBy: nonEmpty(raw.belegung_durch),
      bookedUntil: nonEmpty(raw.belegung_bis),
      occupancy: parseOccupancy(raw),
    });
  }
  return rooms;
}

/** A joined building (e.g. MI) passes every covered finger id, so its page unions them. */
export function roomsForBuildings(
  rooms: readonly IrisRoom[],
  buildingIds: readonly string[]
): IrisRoom[] {
  const wanted = new Set(buildingIds);
  return rooms.filter((room) => wanted.has(room.buildingId));
}

/** The integer occupancy percentage (`0`..`100`) to display for a WAAS room. */
export function occupancyPercent(occupancy: IrisOccupancy): number {
  return Math.round(occupancy.percent * 100);
}

/**
 * Best-effort `HH:MM` from an Iris booked-until timestamp like `2026-06-06 19:28:40`.
 *
 * Iris returns a timezone-less local-time string; a regex extraction avoids the engine-dependent
 * parsing of `new Date(...)` for that format. The raw value is returned when no time is found.
 */
const TIME_OF_DAY_RE = /(\d{1,2}):(\d{2})/;

export function bookedUntilTime(raw: string): string {
  const match = raw.match(TIME_OF_DAY_RE);
  if (!match) return raw;
  const [, hours = "", minutes = ""] = match;
  return `${hours.padStart(2, "0")}:${minutes}`;
}

/**
 * Join parsed rooms with the resolved alias→path map, preserving Iris order.
 *
 * `resolved` maps an `archName` to the matched NavigaTUM path, or to `null` when the alias was
 * looked up but no entity matched. Rooms that are unresolved (`undefined`) or unmatched (`null`)
 * are silently omitted, mirroring the build-time coverage join.
 */
export function buildRoomRows(
  rooms: readonly IrisRoom[],
  resolved: ReadonlyMap<string, string | null>
): IrisRoomRow[] {
  const rows: IrisRoomRow[] = [];
  for (const room of rooms) {
    const path = resolved.get(room.archName);
    if (!path) continue;
    rows.push({ ...room, path });
  }
  return rows;
}
