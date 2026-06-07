// Helpers for turning a browser `datetime-local` value into the RFC3339 contract the server's
// event validator parses (`server/src/routes/feedback/proposed_edits/addition/event.rs`). A
// `datetime-local` input yields a wall-clock string with no zone (`YYYY-MM-DDTHH:MM`); event
// times are entered as Munich wall-clock, so we stamp them with Europe/Berlin's actual UTC offset
// at that instant (DST-aware).

const WALL_TIME_RE = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})/;
// `longOffset` renders "GMT+02:00"; a zero offset can render as a bare "GMT".
const GMT_OFFSET_RE = /GMT([+-])(\d{2}):(\d{2})/;

// Europe/Berlin's offset (in minutes east of UTC) in effect at the given instant, read from the
// IANA database via Intl rather than hard-coded so the DST cutover dates stay correct over time.
function berlinOffsetMinutesAt(date: Date): number {
  const tzName = new Intl.DateTimeFormat("en-US", {
    timeZone: "Europe/Berlin",
    timeZoneName: "longOffset",
  })
    .formatToParts(date)
    .find((p) => p.type === "timeZoneName")?.value;
  const match = GMT_OFFSET_RE.exec(tzName ?? "");
  if (!match) return 0;
  const sign = match[1] === "+" ? 1 : -1;
  return sign * (Number(match[2]) * 60 + Number(match[3]));
}

function formatOffset(minutes: number): string {
  const sign = minutes >= 0 ? "+" : "-";
  const abs = Math.abs(minutes);
  const hh = String(Math.floor(abs / 60)).padStart(2, "0");
  const mm = String(abs % 60).padStart(2, "0");
  return `${sign}${hh}:${mm}`;
}

/**
 * Interpret a `datetime-local` wall-clock value as Europe/Berlin local time and render it as
 * RFC3339 with the matching offset, e.g. `2026-06-10T16:00` → `2026-06-10T16:00:00+02:00`.
 * Returns `null` for malformed input so callers can surface a validation error.
 *
 * Wall times that fall in a DST gap/overlap are inherently ambiguous; the two-pass settle below
 * resolves the common case, and such inputs only occur in the one missing/duplicated hour a year.
 */
export function berlinWallTimeToRfc3339(wall: string): string | null {
  if (!WALL_TIME_RE.test(wall)) return null;
  // Fixed-width `YYYY-MM-DDTHH:MM` slices; the regex above guarantees the layout.
  const year = wall.slice(0, 4);
  const month = wall.slice(5, 7);
  const day = wall.slice(8, 10);
  const hour = wall.slice(11, 13);
  const minute = wall.slice(14, 16);
  // Treat the wall components as if they were UTC to get a first instant, read Berlin's offset
  // there, then re-read it at the corrected instant (wall − offset) to settle around DST cutovers.
  const wallAsUtcMs = Date.UTC(
    Number(year),
    Number(month) - 1,
    Number(day),
    Number(hour),
    Number(minute)
  );
  const firstGuess = berlinOffsetMinutesAt(new Date(wallAsUtcMs));
  const offset = berlinOffsetMinutesAt(new Date(wallAsUtcMs - firstGuess * 60_000));
  return `${year}-${month}-${day}T${hour}:${minute}:00${formatOffset(offset)}`;
}
