// Weekdays in OSM order. The assembler groups days that share the same hours
// into one rule, so this order defines both rule order and what counts as a
// consecutive `Mo-We`-style run inside a day selector.
export const OPENING_HOURS_DAYS = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"] as const;
export type OpeningHoursWeekday = (typeof OPENING_HOURS_DAYS)[number];

// Every well-formed `HH:MM` (24h) wall-clock time and nothing else, built as
// digit crossings (24 x 60 literals) so a malformed literal such as `"8:00"` is
// a compile error, not just a runtime rejection. `TIME_RE` is the runtime
// mirror of this type; keep the two in sync.
type Digit = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9";
type Hour = `${"0" | "1"}${Digit}` | `2${"0" | "1" | "2" | "3"}`;
type Minute = `${"0" | "1" | "2" | "3" | "4" | "5"}${Digit}`;
export type ClockTime = `${Hour}:${Minute}`;

const TIME_RE = /^([01]\d|2[0-3]):[0-5]\d$/;

export interface TimeRange {
  /** Opening time as `HH:MM` (24h); free-form until `isValidTimeRange` proves it. */
  from: string;
  /** Closing time as `HH:MM` (24h); free-form until `isValidTimeRange` proves it. */
  to: string;
}

declare const validTimeRange: unique symbol;

/**
 * A `TimeRange` proven well-formed: both ends are `ClockTime`s and `from`
 * precedes `to`. The ordering invariant has no structural encoding, so the
 * symbol brand makes the `isValidTimeRange` guard the only way to obtain one.
 */
export interface ValidTimeRange {
  readonly from: ClockTime;
  readonly to: ClockTime;
  readonly [validTimeRange]: true;
}

export type WeekSchedule = Record<OpeningHoursWeekday, TimeRange[]>;

// The editor components own the mutable draft types; the assemblers below take
// these deep read-only views instead, so the type system proves they never
// mutate a draft.
type DeepReadonly<T> = T extends readonly (infer U)[]
  ? readonly DeepReadonly<U>[]
  : T extends object
    ? { readonly [K in keyof T]: DeepReadonly<T[K]> }
    : T;
export type WeekScheduleView = DeepReadonly<WeekSchedule>;

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
export function isValidTimeRange(range: Readonly<TimeRange>): range is ValidTimeRange {
  return TIME_RE.test(range.from) && TIME_RE.test(range.to) && range.from < range.to;
}

// Drop invalid ranges, deduplicate, and sort so the hours are deterministic
// regardless of the order the user entered them, then join as one OSM time
// selector (e.g. `08:00-12:00,13:00-17:00`). Empty when nothing is valid.
export function osmRangeList(ranges: readonly Readonly<TimeRange>[]): string {
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

// An OSM weekday selector for the given days. Lone days stay `Fr`; a two-day
// run stays a plain list (`Mo,Tu` - a dash between adjacent days names no day
// in between and reads oddly); runs of three or more collapse to `Mo-We`.
// Runs join with commas (`Mo-We,Fr`).
function osmDaySelector(openDays: ReadonlySet<OpeningHoursWeekday>): string {
  const runs: { start: OpeningHoursWeekday; end: OpeningHoursWeekday; length: number }[] = [];
  let previousDayOpen = false;
  for (const day of OPENING_HOURS_DAYS) {
    const open = openDays.has(day);
    if (open) {
      const run = previousDayOpen ? runs.at(-1) : undefined;
      if (run) {
        run.end = day;
        run.length += 1;
      } else {
        runs.push({ start: day, end: day, length: 1 });
      }
    }
    previousDayOpen = open;
  }
  return runs
    .map(({ start, end, length }) => {
      if (length === 1) return start;
      return length === 2 ? `${start},${end}` : `${start}-${end}`;
    })
    .join(",");
}

/**
 * Assemble a structured week into a plain OSM `opening_hours` string.
 *
 * All days that share identical hours are grouped into a single rule, even
 * non-adjacent ones (`Tu,Th 13:00-13:30`, `Mo-We,Fr 08:00-18:00`); days with no
 * valid ranges are treated as closed and omitted entirely (their absence is
 * what marks them closed in OSM). Grouping is purely syntactic: every day still
 * appears in exactly one rule, so OSM's per-day override semantics are
 * unaffected. Returns `""` when every day is closed, which `hasWeeklyHours`
 * reads as "no weekly schedule to submit".
 */
export function buildOsmOpeningHours(week: WeekScheduleView): string {
  // Map insertion order is first-occurrence day order, which fixes the rule order.
  const daysByHours = new Map<string, Set<OpeningHoursWeekday>>();
  for (const day of OPENING_HOURS_DAYS) {
    const hours = osmRangeList(week[day]);
    if (!hours) continue;
    const days = daysByHours.get(hours);
    if (days) days.add(day);
    else daysByHours.set(hours, new Set([day]));
  }
  return Array.from(daysByHours, ([hours, days]) => `${osmDaySelector(days)} ${hours}`).join("; ");
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

export interface OpeningHoursDraft {
  mode: OpeningHoursMode;
  always: WeekSchedule;
  lecture: WeekSchedule;
  break: WeekSchedule;
  // Public holidays (`PH`), treated like an extra weekday: empty means closed.
  holiday: TimeRange[];
  sourceUrl: string;
}

export type OpeningHoursDraftView = DeepReadonly<OpeningHoursDraft>;

export function emptyOpeningHoursDraft(): OpeningHoursDraft {
  return {
    mode: "always",
    always: emptyWeekSchedule(),
    lecture: emptyWeekSchedule(),
    break: emptyWeekSchedule(),
    holiday: [],
    sourceUrl: "",
  };
}

// Whether the draft states any regular weekly hours. The holiday rule only
// augments a weekly schedule, so a bare `PH off` default must not count as a
// schedule worth submitting on its own.
export function hasWeeklyHours(draft: OpeningHoursDraftView): boolean {
  return activeWeeks(draft).some((week) => buildOsmOpeningHours(week) !== "");
}

// The OSM `PH` rule. Mirrors a weekday: hours when open, `PH off` when the
// holiday row is left empty (most facilities are shut on public holidays).
// The return type records that, unlike a weekly schedule, it is never empty.
export function buildPublicHolidayRule(ranges: readonly Readonly<TimeRange>[]): `PH ${string}` {
  const hours = osmRangeList(ranges);
  return hours ? `PH ${hours}` : "PH off";
}

// The week schedules that actually contribute for the chosen mode; the others
// are kept as drafts but ignored, so toggling modes never loses what was typed.
export function activeWeeks(
  draft: OpeningHoursDraftView
): readonly [WeekScheduleView] | readonly [WeekScheduleView, WeekScheduleView] {
  return draft.mode === "always" ? [draft.always] : [draft.lecture, draft.break];
}

export function draftHasInvalidRange(draft: OpeningHoursDraftView): boolean {
  const weekInvalid = activeWeeks(draft).some((week) =>
    OPENING_HOURS_DAYS.some((day) => week[day].some((range) => !isValidTimeRange(range)))
  );
  const holidayInvalid = draft.holiday.some((range) => !isValidTimeRange(range));
  return weekInvalid || holidayInvalid;
}

/**
 * Assemble a draft into the final OSM `opening_hours` string.
 *
 * `always` mode emits plain OSM; `semester` mode prefixes each schedule with the
 * `lecture:`/`break:` macros the pipeline expands against the semester list. The
 * public-holiday (`PH`) rule is always appended, defaulting to `PH off`, so the
 * result is never empty - callers gate submission on `hasWeeklyHours` instead.
 */
export function buildDraftOpeningHours(draft: OpeningHoursDraftView): string {
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
  rules.push(buildPublicHolidayRule(draft.holiday));
  return rules.join("; ");
}
