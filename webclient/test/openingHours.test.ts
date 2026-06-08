import { describe, expect, it } from "vitest";
import {
  buildOsmOpeningHours,
  emptyWeekSchedule,
  isValidTimeRange,
  type WeekSchedule,
} from "../app/utils/openingHours";

function week(partial: Partial<WeekSchedule>): WeekSchedule {
  return { ...emptyWeekSchedule(), ...partial };
}

describe("isValidTimeRange", () => {
  it("accepts a well-formed range", () => {
    expect(isValidTimeRange({ from: "08:00", to: "20:00" })).toBe(true);
  });
  it("rejects a backwards range", () => {
    expect(isValidTimeRange({ from: "20:00", to: "08:00" })).toBe(false);
  });
  it("rejects an equal range", () => {
    expect(isValidTimeRange({ from: "08:00", to: "08:00" })).toBe(false);
  });
  it("rejects malformed times", () => {
    expect(isValidTimeRange({ from: "8:00", to: "20:00" })).toBe(false);
    expect(isValidTimeRange({ from: "08:00", to: "24:00" })).toBe(false);
    expect(isValidTimeRange({ from: "", to: "20:00" })).toBe(false);
  });
});

describe("buildOsmOpeningHours", () => {
  it("returns an empty string when nothing is open", () => {
    expect(buildOsmOpeningHours(emptyWeekSchedule())).toBe("");
  });

  it("collapses consecutive identical days into a range", () => {
    const w = week({
      Mo: [{ from: "08:00", to: "20:00" }],
      Tu: [{ from: "08:00", to: "20:00" }],
      We: [{ from: "08:00", to: "20:00" }],
      Th: [{ from: "08:00", to: "20:00" }],
      Fr: [{ from: "08:00", to: "20:00" }],
    });
    expect(buildOsmOpeningHours(w)).toBe("Mo-Fr 08:00-20:00");
  });

  it("emits distinct rules and omits closed days", () => {
    const w = week({
      Mo: [{ from: "08:00", to: "20:00" }],
      Tu: [{ from: "08:00", to: "20:00" }],
      Fr: [{ from: "08:00", to: "20:00" }],
      Sa: [{ from: "09:00", to: "14:00" }],
    });
    // We/Th are closed, so the Mo-Tu run breaks before Fr.
    expect(buildOsmOpeningHours(w)).toBe("Mo-Tu 08:00-20:00; Fr 08:00-20:00; Sa 09:00-14:00");
  });

  it("renders a single open day without a range dash", () => {
    expect(buildOsmOpeningHours(week({ We: [{ from: "10:00", to: "12:00" }] }))).toBe(
      "We 10:00-12:00"
    );
  });

  it("joins multiple ranges in one day, sorted and comma-separated", () => {
    const w = week({
      Mo: [
        { from: "13:00", to: "17:00" },
        { from: "08:00", to: "12:00" },
      ],
    });
    expect(buildOsmOpeningHours(w)).toBe("Mo 08:00-12:00,13:00-17:00");
  });

  it("does not collapse days whose range sets differ", () => {
    const w = week({
      Mo: [{ from: "08:00", to: "12:00" }],
      Tu: [
        { from: "08:00", to: "12:00" },
        { from: "13:00", to: "17:00" },
      ],
    });
    expect(buildOsmOpeningHours(w)).toBe("Mo 08:00-12:00; Tu 08:00-12:00,13:00-17:00");
  });

  it("drops invalid and duplicate ranges before assembling", () => {
    const w = week({
      Mo: [
        { from: "08:00", to: "20:00" },
        { from: "08:00", to: "20:00" }, // duplicate
        { from: "21:00", to: "19:00" }, // backwards, dropped
      ],
    });
    expect(buildOsmOpeningHours(w)).toBe("Mo 08:00-20:00");
  });
});
