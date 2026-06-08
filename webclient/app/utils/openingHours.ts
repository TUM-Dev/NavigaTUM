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

// Drop invalid ranges, deduplicate, and sort so the hours are deterministic
// regardless of the order the user entered them, then join as one OSM time
// selector (e.g. `08:00-12:00,13:00-17:00`). Empty when nothing is valid.
export function osmRangeList(ranges: readonly TimeRange[]): string {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const range of ranges) {
    if (!isValidTimeRange(range)) continue;
    const osm = `${range.from}-${range.to}`;
    if (seen.has(osm)) continue;
    seen.add(osm);
    out.push(osm);
  }
  return out.sort().join(",");
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
  const perDay = OPENING_HOURS_DAYS.map((day) => osmRangeList(week[day]));

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

// The semester-macro prefixes the data pipeline understands: `lecture:`
// (Vorlesungszeit) and `break:` (vorlesungsfreie Zeit). See the pipeline's
// `semester_block_expander`, which rewrites these into plain OSM date ranges.
export type SchedulePrefix = "lecture" | "break";

// Prefix every `;`-separated rule of a plain OSM string with a semester macro.
// Each rule needs its own prefix because `;` is also the macro-block separator,
// so `lecture: Mo-Fr 08:00; Sa 09:00` would only scope the first rule.
export function scopeOsmRules(plainOsm: string, prefix: SchedulePrefix): string {
  if (!plainOsm) return "";
  return plainOsm
    .split("; ")
    .map((rule) => `${prefix}: ${rule}`)
    .join("; ");
}

// `always`: one schedule year-round. `semester`: distinct lecture-period and
// lecture-free-period schedules, emitted with `lecture:`/`break:` macros.
export type OpeningHoursMode = "always" | "semester";

// Public-holiday (`PH`) handling. `unspecified` emits no rule (we make no claim);
// `closed` emits `PH off`; `open` emits `PH <hours>`.
export type HolidayMode = "unspecified" | "closed" | "open";

export interface HolidaySchedule {
  mode: HolidayMode;
  ranges: TimeRange[];
}

export interface OpeningHoursDraft {
  mode: OpeningHoursMode;
  always: WeekSchedule;
  lecture: WeekSchedule;
  break: WeekSchedule;
  holiday: HolidaySchedule;
  sourceUrl: string;
}

export function emptyOpeningHoursDraft(): OpeningHoursDraft {
  return {
    mode: "always",
    always: emptyWeekSchedule(),
    lecture: emptyWeekSchedule(),
    break: emptyWeekSchedule(),
    holiday: { mode: "unspecified", ranges: [] },
    sourceUrl: "",
  };
}

// The OSM `PH` rule for the holiday selection, or `""` when unspecified (or
// `open` without any valid hours, which states nothing).
export function buildHolidayRule(holiday: HolidaySchedule): string {
  if (holiday.mode === "closed") return "PH off";
  if (holiday.mode === "open") {
    const hours = osmRangeList(holiday.ranges);
    return hours ? `PH ${hours}` : "";
  }
  return "";
}

// The week schedules that actually contribute for the chosen mode; the others
// are kept as drafts but ignored, so toggling modes never loses what was typed.
export function activeWeeks(draft: OpeningHoursDraft): WeekSchedule[] {
  return draft.mode === "always" ? [draft.always] : [draft.lecture, draft.break];
}

export function draftHasInvalidRange(draft: OpeningHoursDraft): boolean {
  const weekInvalid = activeWeeks(draft).some((week) =>
    OPENING_HOURS_DAYS.some((day) => week[day].some((range) => !isValidTimeRange(range)))
  );
  const holidayInvalid =
    draft.holiday.mode === "open" && draft.holiday.ranges.some((range) => !isValidTimeRange(range));
  return weekInvalid || holidayInvalid;
}

/**
 * Assemble a draft into the final OSM `opening_hours` string.
 *
 * `always` mode emits plain OSM; `semester` mode prefixes each schedule with the
 * `lecture:`/`break:` macros the pipeline expands against the semester list. A
 * public-holiday (`PH`) rule, when set, is appended unconditionally. Returns `""`
 * when nothing is stated.
 */
export function buildDraftOpeningHours(draft: OpeningHoursDraft): string {
  const base =
    draft.mode === "always"
      ? buildOsmOpeningHours(draft.always)
      : [
          scopeOsmRules(buildOsmOpeningHours(draft.lecture), "lecture"),
          scopeOsmRules(buildOsmOpeningHours(draft.break), "break"),
        ]
          .filter(Boolean)
          .join("; ");

  const rules = base ? [base] : [];
  const holiday = buildHolidayRule(draft.holiday);
  if (holiday) rules.push(holiday);
  return rules.join("; ");
}
