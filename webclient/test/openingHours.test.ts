import { describe, expect, it } from "vitest";
import {
  computeOpeningHoursState,
  type OpeningHoursDay,
  parseOpeningHoursWeek,
  WEEKDAY_KEYS,
} from "../app/utils/openingHours";

// A fixed Wednesday so the Monday-anchored week and the `isToday` flag are deterministic.
const WEDNESDAY = new Date(2026, 5, 10); // 2026-06-10 is a Wednesday.

function dayByKey(week: readonly OpeningHoursDay[], key: string): OpeningHoursDay {
  const day = week.find((d) => d.key === key);
  if (!day) throw new Error(`missing day ${key}`);
  return day;
}

describe("parseOpeningHoursWeek", () => {
  it("returns the seven Monday-first weekdays with today flagged", async () => {
    const week = await parseOpeningHoursWeek("Mo-Fr 08:00-22:00", WEDNESDAY);
    expect(week).not.toBeNull();
    expect(week?.map((d) => d.key)).toEqual([...WEEKDAY_KEYS]);
    expect(week?.filter((d) => d.isToday).map((d) => d.key)).toEqual(["we"]);
  });

  it("parses a single range and leaves uncovered days closed", async () => {
    const week = await parseOpeningHoursWeek("Mo-Fr 08:00-22:00", WEDNESDAY);
    if (!week) throw new Error("expected a parsed week");
    expect(dayByKey(week, "mo").ranges).toEqual([
      { from: "08:00", to: "22:00", comment: undefined },
    ]);
    // Weekends are not covered by the rule, so they read as closed (no ranges).
    expect(dayByKey(week, "sa").ranges).toEqual([]);
    expect(dayByKey(week, "su").ranges).toEqual([]);
  });

  it("parses multiple ranges in a single day", async () => {
    const week = await parseOpeningHoursWeek("Mo 08:00-12:00,13:00-17:00", WEDNESDAY);
    if (!week) throw new Error("expected a parsed week");
    expect(dayByKey(week, "mo").ranges.map((r) => `${r.from}-${r.to}`)).toEqual([
      "08:00-12:00",
      "13:00-17:00",
    ]);
  });

  it("renders an all-day rule as 00:00-24:00 rather than 00:00-00:00", async () => {
    const week = await parseOpeningHoursWeek("24/7", WEDNESDAY);
    if (!week) throw new Error("expected a parsed week");
    for (const day of week) {
      expect(day.ranges).toEqual([{ from: "00:00", to: "24:00", comment: undefined }]);
    }
  });

  it("expands a semester-style date range into the matching week only", async () => {
    const osm = "2026 Jun 08-2026 Jun 12 Mo-Fr 09:00-18:00";
    const inRange = await parseOpeningHoursWeek(osm, WEDNESDAY);
    const outOfRange = await parseOpeningHoursWeek(osm, new Date(2026, 6, 1)); // a July week.
    expect(dayByKey(inRange ?? [], "we").ranges).toEqual([
      { from: "09:00", to: "18:00", comment: undefined },
    ]);
    expect((outOfRange ?? []).every((d) => d.ranges.length === 0)).toBe(true);
  });

  it("returns null for a malformed OSM string instead of throwing", async () => {
    expect(await parseOpeningHoursWeek("Mo-Fr 08:00-99:99", WEDNESDAY)).toBeNull();
  });
});

describe("computeOpeningHoursState", () => {
  const WEEKDAYS = "Mo-Fr 08:00-22:00";

  it("reports open with the upcoming closing time during opening hours", async () => {
    const state = await computeOpeningHoursState(WEEKDAYS, new Date(2026, 5, 8, 10, 0)); // Mon 10:00.
    expect(state?.open).toBe(true);
    expect(state?.nextChange?.getTime()).toBe(new Date(2026, 5, 8, 22, 0).getTime());
  });

  it("reports closed with the next opening time outside opening hours", async () => {
    const state = await computeOpeningHoursState(WEEKDAYS, new Date(2026, 5, 7, 15, 0)); // Sun 15:00.
    expect(state?.open).toBe(false);
    expect(state?.nextChange?.getTime()).toBe(new Date(2026, 5, 8, 8, 0).getTime()); // Mon 08:00.
  });

  it("treats the closing minute itself as closed", async () => {
    const state = await computeOpeningHoursState(WEEKDAYS, new Date(2026, 5, 5, 22, 0)); // Fri 22:00.
    expect(state?.open).toBe(false);
  });

  it("reports an all-day schedule as open with no upcoming change", async () => {
    const state = await computeOpeningHoursState("24/7", new Date(2026, 5, 8, 3, 0));
    expect(state).toEqual({ open: true, nextChange: null });
  });

  it("honours a baked-in holiday date as closed", async () => {
    // Holidays reach the client as explicit `<date> off` rules, not `PH`.
    const osm = `${WEEKDAYS}; 2026 Jan 06 off`;
    const onHoliday = await computeOpeningHoursState(osm, new Date(2026, 0, 6, 10, 0)); // Tue holiday.
    const nextDay = await computeOpeningHoursState(osm, new Date(2026, 0, 13, 10, 0)); // ordinary Tue.
    expect(onHoliday?.open).toBe(false);
    expect(nextDay?.open).toBe(true);
  });

  it("returns null for a malformed OSM string instead of throwing", async () => {
    expect(await computeOpeningHoursState("Mo-Fr 08:00-99:99", new Date(2026, 5, 8))).toBeNull();
  });
});
