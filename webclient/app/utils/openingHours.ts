// Weekdays in OSM order. The assembler collapses runs of consecutive days that
// share the same hours, so this order also defines what counts as "consecutive".
export const OPENING_HOURS_DAYS = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"] as const;
export type OpeningHoursDay = (typeof OPENING_HOURS_DAYS)[number];

export interface TimeRange {
  /** Opening time as `HH:MM` (24h). */
  from: string;
  /** Closing time as `HH:MM` (24h). */
  to: string;
}

export type WeekSchedule = Record<OpeningHoursDay, TimeRange[]>;

const TIME_RE = /^([01]\d|2[0-3]):[0-5]\d$/;

export function emptyWeekSchedule(): WeekSchedule {
  return {
    Mo: [],
    Tu: [],
    We: [],
    Th: [],
    Fr: [],
    Sa: [],
    Su: [],
  };
}

// A range is valid when both ends are `HH:MM` and it does not run backwards.
// Overnight spans (e.g. 22:00-02:00) are intentionally rejected for v1.
export function isValidTimeRange(range: TimeRange): boolean {
  return TIME_RE.test(range.from) && TIME_RE.test(range.to) && range.from < range.to;
}

// Drop invalid ranges, deduplicate, and sort so a day's hours are deterministic
// regardless of the order the user entered them.
function normalizeDayRanges(ranges: readonly TimeRange[]): string[] {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const range of ranges) {
    if (!isValidTimeRange(range)) continue;
    const osm = `${range.from}-${range.to}`;
    if (seen.has(osm)) continue;
    seen.add(osm);
    out.push(osm);
  }
  return out.sort();
}

/**
 * Assemble a structured week into a plain OSM `opening_hours` string.
 *
 * Days that share identical hours are collapsed into a single `Mo-Fr 08:00-20:00`
 * rule; days with no valid ranges are treated as closed and omitted entirely
 * (their absence is what marks them closed in OSM). Returns `""` when nothing is
 * open, which callers use to keep the submit action disabled.
 */
export function buildOsmOpeningHours(week: WeekSchedule): string {
  const perDay = OPENING_HOURS_DAYS.map((day) => normalizeDayRanges(week[day]).join(","));

  const rules: string[] = [];
  let runStart = 0;
  while (runStart < OPENING_HOURS_DAYS.length) {
    const hours = perDay[runStart] ?? "";
    if (!hours) {
      runStart += 1;
      continue;
    }
    let runEnd = runStart;
    while (runEnd + 1 < OPENING_HOURS_DAYS.length && perDay[runEnd + 1] === hours) {
      runEnd += 1;
    }
    const days =
      runStart === runEnd
        ? OPENING_HOURS_DAYS[runStart]
        : `${OPENING_HOURS_DAYS[runStart]}-${OPENING_HOURS_DAYS[runEnd]}`;
    rules.push(`${days} ${hours}`);
    runStart = runEnd + 1;
  }

  return rules.join("; ");
}
