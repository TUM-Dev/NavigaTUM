import { describe, expect, it } from "vitest";
import {
  buildDraftOpeningHours,
  buildOsmOpeningHours,
  draftHasInvalidRange,
  emptyOpeningHoursDraft,
  emptyWeekSchedule,
  hasWeeklyHours,
  isValidTimeRange,
  scopeOsmRules,
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

describe("scopeOsmRules", () => {
  it("prefixes every rule individually", () => {
    expect(scopeOsmRules("Mo-Fr 08:00-20:00; Sa 09:00-14:00", "lecture")).toBe(
      "lecture: Mo-Fr 08:00-20:00; lecture: Sa 09:00-14:00"
    );
  });
  it("returns an empty string unchanged", () => {
    expect(scopeOsmRules("", "break")).toBe("");
  });
});

describe("opening-hours draft", () => {
  it("defaults holidays to closed", () => {
    expect(emptyOpeningHoursDraft().holiday.mode).toBe("closed");
  });

  it("reports weekly hours only for the active mode", () => {
    const draft = emptyOpeningHoursDraft();
    expect(hasWeeklyHours(draft)).toBe(false); // default PH off is not weekly hours
    draft.lecture.Mo = [{ from: "08:00", to: "20:00" }]; // inactive while mode is "always"
    expect(hasWeeklyHours(draft)).toBe(false);
    draft.always.Mo = [{ from: "08:00", to: "20:00" }];
    expect(hasWeeklyHours(draft)).toBe(true);
  });
});

describe("buildDraftOpeningHours", () => {
  it("emits plain OSM plus the default PH rule in always mode", () => {
    const draft = emptyOpeningHoursDraft();
    draft.always.Mo = [{ from: "08:00", to: "20:00" }];
    draft.always.Tu = [{ from: "08:00", to: "20:00" }];
    // The lecture/break drafts are ignored while mode is "always".
    draft.lecture.Mo = [{ from: "06:00", to: "07:00" }];
    expect(buildDraftOpeningHours(draft)).toBe("Mo-Tu 08:00-20:00; PH off");
  });

  it("combines lecture and break schedules with macros in semester mode", () => {
    const draft = emptyOpeningHoursDraft();
    draft.mode = "semester";
    draft.lecture.Mo = [{ from: "08:00", to: "20:00" }];
    draft.lecture.Tu = [{ from: "08:00", to: "20:00" }];
    draft.break.Mo = [{ from: "10:00", to: "16:00" }];
    expect(buildDraftOpeningHours(draft)).toBe(
      "lecture: Mo-Tu 08:00-20:00; break: Mo 10:00-16:00; PH off"
    );
  });

  it("emits only the populated period in semester mode", () => {
    const draft = emptyOpeningHoursDraft();
    draft.mode = "semester";
    draft.lecture.Mo = [{ from: "08:00", to: "20:00" }];
    expect(buildDraftOpeningHours(draft)).toBe("lecture: Mo 08:00-20:00; PH off");
  });

  it("ignores invalid ranges in inactive periods", () => {
    const draft = emptyOpeningHoursDraft();
    draft.always.Mo = [{ from: "08:00", to: "20:00" }];
    draft.break.Mo = [{ from: "20:00", to: "08:00" }]; // backwards, but inactive
    expect(draftHasInvalidRange(draft)).toBe(false);
    expect(buildDraftOpeningHours(draft)).toBe("Mo 08:00-20:00; PH off");
  });

  it("flags an invalid range in the active period", () => {
    const draft = emptyOpeningHoursDraft();
    draft.mode = "semester";
    draft.lecture.Mo = [{ from: "20:00", to: "08:00" }];
    expect(draftHasInvalidRange(draft)).toBe(true);
  });

  it("appends PH off for holidays closed", () => {
    const draft = emptyOpeningHoursDraft();
    draft.always.Mo = [{ from: "08:00", to: "20:00" }];
    expect(draft.holiday.mode).toBe("closed"); // the default
    expect(buildDraftOpeningHours(draft)).toBe("Mo 08:00-20:00; PH off");
  });

  it("appends PH hours for holidays open", () => {
    const draft = emptyOpeningHoursDraft();
    draft.mode = "semester";
    draft.lecture.Mo = [{ from: "08:00", to: "20:00" }];
    draft.holiday.mode = "open";
    draft.holiday.ranges = [{ from: "10:00", to: "14:00" }];
    expect(buildDraftOpeningHours(draft)).toBe("lecture: Mo 08:00-20:00; PH 10:00-14:00");
  });

  it("flags an invalid holiday range only when holidays are open", () => {
    const draft = emptyOpeningHoursDraft();
    draft.holiday.mode = "closed";
    draft.holiday.ranges = [{ from: "14:00", to: "10:00" }];
    expect(draftHasInvalidRange(draft)).toBe(false); // ranges ignored unless open
    draft.holiday.mode = "open";
    expect(draftHasInvalidRange(draft)).toBe(true);
  });
});
