import { afterAll, beforeAll, describe, expect, it } from "vitest";
import {
  calendarEventEndLabel,
  findDuplicateEventByName,
  formatEventDateRange,
  rfc3339ToWallTime,
  wallTimeToRfc3339,
} from "../app/utils/datetime";

// Pin the zone so the wall-time conversions are deterministic regardless of where the suite runs.
const original = process.env.TZ;
beforeAll(() => {
  process.env.TZ = "Europe/Berlin";
});
afterAll(() => {
  process.env.TZ = original;
});

describe("wallTimeToRfc3339", () => {
  it("converts a summer (CEST, +02:00) wall time to UTC", () => {
    expect(wallTimeToRfc3339("2026-06-10T16:00")).toBe("2026-06-10T14:00:00.000Z");
  });

  it("converts a winter (CET, +01:00) wall time to UTC", () => {
    expect(wallTimeToRfc3339("2026-01-10T16:00")).toBe("2026-01-10T15:00:00.000Z");
  });

  it("returns null for malformed input", () => {
    expect(wallTimeToRfc3339("")).toBeNull();
    expect(wallTimeToRfc3339("not-a-date")).toBeNull();
  });
});

describe("rfc3339ToWallTime", () => {
  it("converts a summer (CEST, +02:00) instant to the local wall time", () => {
    expect(rfc3339ToWallTime("2026-06-10T14:00:00Z")).toBe("2026-06-10T16:00");
  });

  it("converts a winter (CET, +01:00) instant to the local wall time", () => {
    expect(rfc3339ToWallTime("2026-01-10T15:00:00Z")).toBe("2026-01-10T16:00");
  });

  it("round-trips through wallTimeToRfc3339 at minute precision", () => {
    const instant = "2026-06-19T21:59:00.000Z";
    const wall = rfc3339ToWallTime(instant);
    expect(wall).not.toBeNull();
    expect(wallTimeToRfc3339(wall as string)).toBe(instant);
  });

  it("returns null for malformed input", () => {
    expect(rfc3339ToWallTime("")).toBeNull();
    expect(rfc3339ToWallTime("not-a-date")).toBeNull();
  });
});

describe("formatEventDateRange", () => {
  // ICU range output differs in separators across versions; assert the parts, not the exact string.
  it("renders a same-month range with both days and the year", () => {
    const range = formatEventDateRange("2026-06-15T14:00:00Z", "2026-06-19T21:59:00Z", "de");
    expect(range).toContain("15.");
    expect(range).toContain("19.");
    expect(range).toContain("2026");
  });

  it("renders a range across years with both years", () => {
    const range = formatEventDateRange("2026-12-30T14:00:00Z", "2027-01-02T21:59:00Z", "en");
    expect(range).toContain("2026");
    expect(range).toContain("2027");
  });

  it("renders the day in Europe/Berlin, not UTC", () => {
    // 22:30 UTC on the 15th is already June 16 in Berlin.
    const range = formatEventDateRange("2026-06-15T22:30:00Z", "2026-06-15T23:00:00Z", "en");
    expect(range).toContain("16");
    expect(range).not.toContain("15");
  });

  it("returns an empty string for malformed input", () => {
    expect(formatEventDateRange("not-a-date", "2026-06-19T21:59:00Z", "de")).toBe("");
  });
});

describe("findDuplicateEventByName", () => {
  const NOW = Date.parse("2026-06-13T00:00:00Z");
  const entry = (name: string, startsAt: string) => ({ name, starts_at: startsAt });

  it("returns the entry whose name matches and which falls inside the window", () => {
    const garnix = entry("GARNIX Festival", "2026-06-19T18:00:00Z");
    expect(findDuplicateEventByName([garnix], "GARNIX Festival", NOW)).toBe(garnix);
  });

  it("picks the newest qualifier regardless of input order", () => {
    const older = entry("GARNIX Festival", "2026-04-01T18:00:00Z");
    const newer = entry("GARNIX Festival", "2026-08-01T18:00:00Z");
    expect(findDuplicateEventByName([older, newer], "GARNIX Festival", NOW)).toBe(newer);
    expect(findDuplicateEventByName([newer, older], "GARNIX Festival", NOW)).toBe(newer);
  });

  it("matches case-insensitively and ignores surrounding whitespace", () => {
    const garnix = entry("GARNIX Festival", "2026-06-19T18:00:00Z");
    expect(findDuplicateEventByName([garnix], "  garnix festival  ", NOW)).toBe(garnix);
  });

  it("returns null when no name matches", () => {
    const garnix = entry("GARNIX Festival", "2026-06-19T18:00:00Z");
    expect(findDuplicateEventByName([garnix], "TUNIX Festival", NOW)).toBeNull();
  });

  const DAY_MS = 24 * 60 * 60 * 1000;
  const offsetEntry = (days: number) =>
    entry("GARNIX Festival", new Date(NOW + days * DAY_MS).toISOString());

  it.each([
    ["a recent past edition (8 months ago)", -8 * 30],
    ["a near-future edition (8 months ahead)", 8 * 30],
  ])("flags %s as a duplicate", (_label, days) => {
    const recent = offsetEntry(days);
    expect(findDuplicateEventByName([recent], "GARNIX Festival", NOW)).toBe(recent);
  });

  it.each([
    ["a distant past edition (10 months ago)", -10 * 30],
    ["a distant future edition (10 months ahead)", 10 * 30],
  ])("ignores %s, since that is a legitimate new occurrence", (_label, days) => {
    expect(findDuplicateEventByName([offsetEntry(days)], "GARNIX Festival", NOW)).toBeNull();
  });

  it.each(["", "   ", "a", " a "])("returns null for the too-short query %j", (query) => {
    const a = entry("a", "2026-06-19T18:00:00Z");
    expect(findDuplicateEventByName([a], query, NOW)).toBeNull();
  });
});

describe("calendarEventEndLabel", () => {
  // #3424: FullCalendar nulls `end` for zero-duration events (e.g. "2331.EG.113"),
  // and the template used to call `.toLocaleTimeString` on it directly, crashing the calendar.
  it("returns an empty string when end is null", () => {
    expect(calendarEventEndLabel(null)).toBe("");
  });

  it("renders a ` - HH:MM` suffix for a real end", () => {
    expect(calendarEventEndLabel(new Date("2026-07-06T08:00:00Z"))).toBe(" - 10:00");
  });
});
