export function wallTimeToRfc3339(wall: string): string | null {
  const instant = new Date(wall);
  return Number.isNaN(instant.getTime()) ? null : instant.toISOString();
}

// Inverse of `wallTimeToRfc3339`: the instant as a browser-local `datetime-local` value.
export function rfc3339ToWallTime(iso: string): string | null {
  const instant = new Date(iso);
  if (Number.isNaN(instant.getTime())) return null;
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${instant.getFullYear()}-${pad(instant.getMonth() + 1)}-${pad(instant.getDate())}T${pad(instant.getHours())}:${pad(instant.getMinutes())}`;
}

// Campus events happen in Munich; their dates render in Europe/Berlin regardless of the visitor's zone.
const EVENT_TZ = "Europe/Berlin";

// Events recur yearly, so an old or future edition existing is exactly when proposing a new
// event is legitimate; only a same-occurrence collision is worth flagging. The window is a
// season wide of either side so it tolerates date drift and December/January edition shifts.
const SAME_OCCURRENCE_WINDOW_MS = 9 * 30 * 24 * 60 * 60 * 1000;

// Finds an already-existing event the user is about to duplicate by typing its name freehand.
// Returns the newest qualifying entry (or null), so the caller can offer to update it instead.
// Generic over the entry so the full match is handed back for adoption, not just its name/date.
export function findDuplicateEventByName<T extends { name: string; starts_at: string }>(
  entries: readonly T[],
  name: string,
  now: number
): T | null {
  const needle = name.trim().toLowerCase();
  if (needle.length < 2) return null;
  let newest: T | null = null;
  for (const entry of entries) {
    const startsAt = Date.parse(entry.starts_at);
    if (entry.name.trim().toLowerCase() !== needle) continue;
    if (Math.abs(startsAt - now) > SAME_OCCURRENCE_WINDOW_MS) continue;
    // Self-contained newest pick: the API sorts starts_at:desc, but the helper must not lean on
    // that order so its result is correct for any caller and verifiable in isolation.
    if (!newest || startsAt > Date.parse(newest.starts_at)) newest = entry;
  }
  return newest;
}

export function formatEventDateRange(
  startsAt: string,
  endsAt: string,
  locale: "de" | "en"
): string {
  const start = new Date(startsAt);
  const end = new Date(endsAt);
  if (Number.isNaN(start.getTime()) || Number.isNaN(end.getTime())) return "";
  return new Intl.DateTimeFormat(locale === "de" ? "de-DE" : "en-GB", {
    timeZone: EVENT_TZ,
    day: "numeric",
    month: "short",
    year: "numeric",
  }).formatRange(start, end);
}

// FullCalendar nulls `end` for zero- or negative-duration events (end <= start), so the end
// label must tolerate null and collapse to just the start time (#3424).
export function calendarEventEndLabel(end: Date | null): string {
  if (end === null) return "";
  return ` - ${end.toLocaleTimeString("de", { timeStyle: "short" })}`;
}
