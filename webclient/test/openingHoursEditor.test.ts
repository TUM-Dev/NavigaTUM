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
} from "../app/utils/openingHoursEditor";

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

  it("groups non-adjacent days with identical hours into one rule", () => {
    const w = week({
      Mo: [{ from: "08:00", to: "20:00" }],
      Tu: [{ from: "08:00", to: "20:00" }],
      Fr: [{ from: "08:00", to: "20:00" }],
      Sa: [{ from: "09:00", to: "14:00" }],
    });
    // We/Th are closed and omitted; Fr joins the Mo,Tu rule as a day-list entry.
    expect(buildOsmOpeningHours(w)).toBe("Mo,Tu,Fr 08:00-20:00; Sa 09:00-14:00");
  });

  it("renders a two-day run as a list and three or more as a range", () => {
    const hours = [{ from: "08:00", to: "18:00" }];
    // `Mo-Tu` names no day in between, so the dash only starts at three days.
    expect(buildOsmOpeningHours(week({ Mo: hours, Tu: hours }))).toBe("Mo,Tu 08:00-18:00");
    expect(buildOsmOpeningHours(week({ Mo: hours, Tu: hours, We: hours }))).toBe(
      "Mo-We 08:00-18:00"
    );
  });

  it("groups lone same-hours days into a day list", () => {
    const w = week({
      Tu: [{ from: "13:00", to: "13:30" }],
      Th: [{ from: "13:00", to: "13:30" }],
    });
    expect(buildOsmOpeningHours(w)).toBe("Tu,Th 13:00-13:30");
  });

  it("mixes a consecutive run and a lone day in one selector", () => {
    const w = week({
      Mo: [{ from: "08:00", to: "18:00" }],
      Tu: [{ from: "08:00", to: "18:00" }],
      We: [{ from: "08:00", to: "18:00" }],
      Fr: [{ from: "08:00", to: "18:00" }],
    });
    expect(buildOsmOpeningHours(w)).toBe("Mo-We,Fr 08:00-18:00");
  });

  it("keeps days with different hours in separate rules", () => {
    const w = week({
      Tu: [{ from: "13:00", to: "13:30" }],
      Th: [{ from: "13:00", to: "14:00" }],
    });
    expect(buildOsmOpeningHours(w)).toBe("Tu 13:00-13:30; Th 13:00-14:00");
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
  it("defaults holidays to closed (no PH ranges)", () => {
    expect(emptyOpeningHoursDraft().holiday).toEqual([]);
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
    expect(buildDraftOpeningHours(draft)).toBe("Mo,Tu 08:00-20:00; PH off");
  });

  it("combines lecture and break schedules with macros in semester mode", () => {
    const draft = emptyOpeningHoursDraft();
    draft.mode = "semester";
    draft.lecture.Mo = [{ from: "08:00", to: "20:00" }];
    draft.lecture.Tu = [{ from: "08:00", to: "20:00" }];
    draft.break.Mo = [{ from: "10:00", to: "16:00" }];
    expect(buildDraftOpeningHours(draft)).toBe(
      "lecture: Mo,Tu 08:00-20:00; break: Mo 10:00-16:00; PH off"
    );
  });

  it("emits only the populated period in semester mode", () => {
    const draft = emptyOpeningHoursDraft();
    draft.mode = "semester";
    draft.lecture.Mo = [{ from: "08:00", to: "20:00" }];
    expect(buildDraftOpeningHours(draft)).toBe("lecture: Mo 08:00-20:00; PH off");
  });

  it("emits one macro prefix for same-hours days in semester mode", () => {
    const draft = emptyOpeningHoursDraft();
    draft.mode = "semester";
    draft.lecture.Tu = [{ from: "13:00", to: "13:30" }];
    draft.lecture.Th = [{ from: "13:00", to: "13:30" }];
    // Grouping into a day list keeps this a single rule, so `scopeOsmRules`
    // does not repeat the `lecture:` prefix per day.
    expect(buildDraftOpeningHours(draft)).toBe("lecture: Tu,Th 13:00-13:30; PH off");
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

  it("appends PH off when the holiday row is empty", () => {
    const draft = emptyOpeningHoursDraft();
    draft.always.Mo = [{ from: "08:00", to: "20:00" }];
    expect(buildDraftOpeningHours(draft)).toBe("Mo 08:00-20:00; PH off");
  });

  it("appends PH hours when the holiday row has hours", () => {
    const draft = emptyOpeningHoursDraft();
    draft.mode = "semester";
    draft.lecture.Mo = [{ from: "08:00", to: "20:00" }];
    draft.holiday = [{ from: "10:00", to: "14:00" }];
    expect(buildDraftOpeningHours(draft)).toBe("lecture: Mo 08:00-20:00; PH 10:00-14:00");
  });

  it("flags an invalid holiday range", () => {
    const draft = emptyOpeningHoursDraft();
    draft.always.Mo = [{ from: "08:00", to: "20:00" }];
    expect(draftHasInvalidRange(draft)).toBe(false);
    draft.holiday = [{ from: "14:00", to: "10:00" }];
    expect(draftHasInvalidRange(draft)).toBe(true);
  });
});
