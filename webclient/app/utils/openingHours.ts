// Monday-first; the translated labels live in the rendering component, keyed by these.
export type WeekdayKey = "mo" | "tu" | "we" | "th" | "fr" | "sa" | "su";
export const WEEKDAY_KEYS: readonly WeekdayKey[] = ["mo", "tu", "we", "th", "fr", "sa", "su"];

/** A wall-clock boundary formatted as `HH:MM`, where a midnight close reads as `24:00`. */
export type TimeOfDay = `${number}:${number}`;

export interface OpeningHoursRange {
  readonly from: TimeOfDay;
  readonly to: TimeOfDay;
  readonly comment?: string;
}
export interface OpeningHoursDay {
  readonly key: WeekdayKey;
  readonly isToday: boolean;
  readonly ranges: readonly OpeningHoursRange[];
}

// One open interval as the `opening_hours` library yields it: `[from, to, unknown, comment]`.
type OpenInterval = readonly [Date, Date, boolean, string | undefined];

function formatBoundary(date: Date, dayEnd: Date): TimeOfDay {
  // opening_hours reports an interval ending at the following midnight as the next day's
  // 00:00; within a single day's row that boundary should read as 24:00.
  if (date.getTime() >= dayEnd.getTime()) return "24:00";
  const hours = String(date.getHours()).padStart(2, "0");
  const minutes = String(date.getMinutes()).padStart(2, "0");
  return `${hours}:${minutes}` as TimeOfDay;
}

/**
 * Expand a plain OSM `opening_hours` string into the seven days of the week containing
 * `reference`. Returns `null` when the string cannot be parsed, so callers can fall back to
 * the raw schedule rather than dropping it.
 */
export async function parseOpeningHoursWeek(
  osm: string,
  reference: Date
): Promise<readonly OpeningHoursDay[] | null> {
  try {
    const { default: OpeningHours } = await import("opening_hours");
    // The mode argument is omitted because the ESM build does not export the `mode` enum its
    // types advertise; the library defaults to time-range mode regardless.
    const oh = new OpeningHours(osm, null);

    const todayIndex = (reference.getDay() + 6) % 7; // 0 = Monday … 6 = Sunday.
    const monday = new Date(
      reference.getFullYear(),
      reference.getMonth(),
      reference.getDate() - todayIndex
    );

    return WEEKDAY_KEYS.map((key, index): OpeningHoursDay => {
      const dayStart = new Date(monday.getFullYear(), monday.getMonth(), monday.getDate() + index);
      const dayEnd = new Date(
        monday.getFullYear(),
        monday.getMonth(),
        monday.getDate() + index + 1
      );
      const intervals: readonly OpenInterval[] = oh.getOpenIntervals(dayStart, dayEnd);
      const ranges = intervals.map(
        ([from, to, , comment]): OpeningHoursRange => ({
          from: formatBoundary(from, dayEnd),
          to: formatBoundary(to, dayEnd),
          comment: comment || undefined,
        })
      );
      return { key, isToday: index === todayIndex, ranges };
    });
  } catch {
    return null;
  }
}

/** The live open/closed verdict of a schedule at one instant. */
export interface OpeningHoursLiveState {
  readonly open: boolean;
  /** When the current state ends; `null` when it never changes (`24/7`, or never opens again). */
  readonly nextChange: Date | null;
}

/**
 * Evaluate whether a schedule is open at `now` and when that state next changes. Holidays are
 * already baked into the OSM string as explicit dates by the pipeline, so no holiday context is
 * needed here. Returns `null` when the string cannot be parsed, mirroring `parseOpeningHoursWeek`.
 */
export async function computeOpeningHoursState(
  osm: string,
  now: Date
): Promise<OpeningHoursLiveState | null> {
  try {
    const { default: OpeningHours } = await import("opening_hours");
    // `null` for the same reason as in `parseOpeningHoursWeek`: the ESM build omits the `mode` enum.
    const oh = new OpeningHours(osm, null);
    return { open: oh.getState(now), nextChange: oh.getNextChange(now) ?? null };
  } catch {
    return null;
  }
}
