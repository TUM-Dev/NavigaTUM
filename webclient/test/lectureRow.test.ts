import { afterAll, beforeAll, describe, expect, it } from "vitest";
import type { components } from "../app/api_types";
import {
  buildVisibleSearchEntries,
  collapsedHighlightTarget,
  collapsedUpwardHighlightTarget,
  findLectureHeaderIndex,
  firstUpcoming,
  formatUpcoming,
  LECTURE_EVENT_NAV_CAP,
  lectureEventPath,
  lectureTitle,
  toggleLectureFromMouse,
  useLectureRowExpansion,
} from "../app/utils/lectureRow";

type ResultEntry = components["schemas"]["ResultEntry"];
type UpcomingEvent = components["schemas"]["UpcomingEvent"];

const GERMAN_SAME_DAY_RE = /Do\.?, 15\. Oktober, 08:00-10:00/;
const ENGLISH_SAME_DAY_RE = /Thu 15 Oct, 08:00-10:00/;
const CROSS_DAY_RE = /15 Oct.*22:30.*16 Oct.*00:30/;

function makeLecture(overrides: Partial<ResultEntry> = {}): ResultEntry {
  return {
    id: "lecture_abc123",
    type: "lecture",
    name: "Einführung in die Informatik 1",
    subtext: "Vorlesung",
    title_de: "Einführung in die Informatik 1",
    title_en: "Introduction to Informatics 1",
    next_occurrence_at: "2026-10-15T06:00:00Z",
    upcoming: [
      {
        start_at: "2026-10-15T06:00:00Z", // 08:00 Europe/Berlin (CEST)
        end_at: "2026-10-15T08:00:00Z", // 10:00 Europe/Berlin (CEST)
        room_code: "5606.EG.011",
        room_name: "Testhörsaal",
      },
      {
        start_at: "2026-10-22T07:00:00Z", // CET kicks in after 2026-10-26; here still CEST
        end_at: "2026-10-22T09:00:00Z",
        room_code: "5606.EG.011",
        room_name: "Testhörsaal",
      },
    ],
    ...overrides,
  };
}

describe("lectureTitle", () => {
  it("returns the localised title for the active locale", () => {
    const entry = makeLecture();
    expect(lectureTitle(entry, "de")).toBe("Einführung in die Informatik 1");
    expect(lectureTitle(entry, "en")).toBe("Introduction to Informatics 1");
  });

  it("falls back to the other locale when the preferred one is missing", () => {
    const onlyGerman = makeLecture({ title_en: null });
    expect(lectureTitle(onlyGerman, "en")).toBe("Einführung in die Informatik 1");

    const onlyEnglish = makeLecture({ title_de: null });
    expect(lectureTitle(onlyEnglish, "de")).toBe("Introduction to Informatics 1");
  });

  it("falls back to `name` when both titles are absent (defensive)", () => {
    const stripped = makeLecture({ title_de: null, title_en: null, name: "Some Lecture" });
    expect(lectureTitle(stripped, "de")).toBe("Some Lecture");
  });
});

describe("firstUpcoming", () => {
  it("returns the first event when `upcoming` is populated", () => {
    const event = firstUpcoming(makeLecture());
    expect(event?.room_code).toBe("5606.EG.011");
  });

  it("returns null when `upcoming` is empty or absent", () => {
    expect(firstUpcoming(makeLecture({ upcoming: [] }))).toBeNull();
    expect(firstUpcoming(makeLecture({ upcoming: null }))).toBeNull();
  });
});

describe("lectureEventPath", () => {
  it("routes to /room/<room_code>", () => {
    const event: Pick<UpcomingEvent, "room_code"> = { room_code: "5606.EG.011" };
    expect(lectureEventPath(event)).toBe("/room/5606.EG.011");
  });
});

describe("formatUpcoming", () => {
  // Pin the zone so the Intl output matches what a Berlin-based user would see.
  const original = process.env.TZ;
  beforeAll(() => {
    process.env.TZ = "Europe/Berlin";
  });
  afterAll(() => {
    process.env.TZ = original;
  });

  it("emits a German same-day weekday + month-day + HH:MM range", () => {
    const event: UpcomingEvent = {
      start_at: "2026-10-15T06:00:00Z",
      end_at: "2026-10-15T08:00:00Z",
      room_code: "5606.EG.011",
      room_name: "Testhörsaal",
    };
    const formatted = formatUpcoming(event, "de");
    expect(formatted).toMatch(GERMAN_SAME_DAY_RE);
  });

  it("emits an English same-day short-month + HH:MM range", () => {
    const event: UpcomingEvent = {
      start_at: "2026-10-15T06:00:00Z",
      end_at: "2026-10-15T08:00:00Z",
      room_code: "5606.EG.011",
      room_name: "Testhörsaal",
    };
    expect(formatUpcoming(event, "en")).toMatch(ENGLISH_SAME_DAY_RE);
  });

  it("renders both date and time on each side when the event crosses Berlin midnight", () => {
    const event: UpcomingEvent = {
      // 2026-10-15 is still CEST in Berlin (DST ends 2026-10-25), so UTC -> Berlin = +2h.
      start_at: "2026-10-15T20:30:00Z", // 22:30 Berlin, Oct 15
      end_at: "2026-10-15T22:30:00Z", // 00:30 Berlin, Oct 16
      room_code: "5606.EG.011",
      room_name: "Testhörsaal",
    };
    const formatted = formatUpcoming(event, "en");
    // Two day labels and an explicit ` - ` separator distinguish cross-day from the same-day form.
    expect(formatted).toContain(" - ");
    expect(formatted).toMatch(CROSS_DAY_RE);
  });
});

describe("useLectureRowExpansion", () => {
  it("starts collapsed and toggles between expanded and collapsed", () => {
    const row = useLectureRowExpansion();
    expect(row.expanded.value).toBe(false);
    expect(row.isCollapsed.value).toBe(true);

    row.toggle();
    expect(row.expanded.value).toBe(true);
    expect(row.isCollapsed.value).toBe(false);

    row.toggle();
    expect(row.expanded.value).toBe(false);
    expect(row.isCollapsed.value).toBe(true);
  });

  it("collapse() forces back to collapsed regardless of current state", () => {
    const row = useLectureRowExpansion(true);
    expect(row.expanded.value).toBe(true);
    row.collapse();
    expect(row.expanded.value).toBe(false);
    row.collapse();
    expect(row.expanded.value).toBe(false);
  });
});

describe("buildVisibleSearchEntries", () => {
  type ResultsSection = components["schemas"]["ResultsSection"];

  function room(id: string): ResultEntry {
    return {
      id,
      type: "room",
      name: id,
      subtext: "",
    };
  }

  function events(n: number): UpcomingEvent[] {
    const list: UpcomingEvent[] = [];
    for (let i = 0; i < n; i++) {
      list.push({
        start_at: `2026-10-${(15 + i).toString().padStart(2, "0")}T06:00:00Z`,
        end_at: `2026-10-${(15 + i).toString().padStart(2, "0")}T08:00:00Z`,
        room_code: "5606.EG.011",
        room_name: "Testhörsaal",
      });
    }
    return list;
  }

  function lecture(id: string, eventCount: number): ResultEntry {
    return {
      id,
      type: "lecture",
      name: id,
      subtext: "",
      title_de: id,
      title_en: id,
      upcoming: events(eventCount),
    };
  }

  function section(facet: string, entries: ResultEntry[], n_visible: number): ResultsSection {
    return { facet, entries, estimatedTotalHits: entries.length, n_visible };
  }

  const emptyState = {
    expandedFacets: new Set<string>(),
    expandedLectures: new Set<string>(),
    lectureShowAll: new Set<string>(),
  };

  it("flattens non-lecture sections respecting n_visible", () => {
    const sections: ResultsSection[] = [section("rooms", [room("r1"), room("r2"), room("r3")], 2)];
    const out = buildVisibleSearchEntries(sections, emptyState);
    expect(out.map((e) => (e.kind === "result" ? e.entry.id : e.kind))).toEqual(["r1", "r2"]);
  });

  it("respects expandedFacets to lift the section cap", () => {
    const sections: ResultsSection[] = [section("rooms", [room("r1"), room("r2"), room("r3")], 2)];
    const out = buildVisibleSearchEntries(sections, {
      ...emptyState,
      expandedFacets: new Set(["rooms"]),
    });
    expect(out.map((e) => (e.kind === "result" ? e.entry.id : e.kind))).toEqual(["r1", "r2", "r3"]);
  });

  it("emits only the lecture header when expandedLectures does not contain its id", () => {
    const sections: ResultsSection[] = [section("lectures", [lecture("l1", 5)], 1)];
    const out = buildVisibleSearchEntries(sections, emptyState);
    expect(out).toHaveLength(1);
    expect(out[0]?.kind).toBe("result");
  });

  it("emits all events with no show-more when total events <= cap", () => {
    const sections: ResultsSection[] = [section("lectures", [lecture("l1", 3)], 1)];
    const out = buildVisibleSearchEntries(sections, {
      ...emptyState,
      expandedLectures: new Set(["l1"]),
    });
    expect(out.map((e) => e.kind)).toEqual(["result", "event", "event", "event"]);
  });

  it("caps events at LECTURE_EVENT_NAV_CAP and emits a show-more sentinel with the hidden count", () => {
    const sections: ResultsSection[] = [section("lectures", [lecture("l1", 7)], 1)];
    const out = buildVisibleSearchEntries(sections, {
      ...emptyState,
      expandedLectures: new Set(["l1"]),
    });
    // header + 3 events + show-more
    expect(out).toHaveLength(LECTURE_EVENT_NAV_CAP + 2);
    expect(out.at(-1)).toEqual({
      kind: "show_more_events",
      lectureId: "l1",
      hiddenCount: 7 - LECTURE_EVENT_NAV_CAP,
    });
  });

  it("drops the show-more sentinel once lectureShowAll opts into the full list", () => {
    const sections: ResultsSection[] = [section("lectures", [lecture("l1", 7)], 1)];
    const out = buildVisibleSearchEntries(sections, {
      ...emptyState,
      expandedLectures: new Set(["l1"]),
      lectureShowAll: new Set(["l1"]),
    });
    expect(out).toHaveLength(1 + 7);
    expect(out.every((e) => e.kind !== "show_more_events")).toBe(true);
  });

  it("indexes events from zero against the lecture's own occurrence list", () => {
    const sections: ResultsSection[] = [section("lectures", [lecture("l1", 5)], 1)];
    const out = buildVisibleSearchEntries(sections, {
      ...emptyState,
      expandedLectures: new Set(["l1"]),
    });
    const eventEntries = out.filter((e) => e.kind === "event");
    expect(eventEntries.map((e) => (e.kind === "event" ? e.eventIndex : -1))).toEqual([0, 1, 2]);
  });
});

describe("findLectureHeaderIndex", () => {
  function section(facet: string, entries: ResultEntry[], n_visible: number) {
    return { facet, entries, estimatedTotalHits: entries.length, n_visible };
  }

  function room(id: string): ResultEntry {
    return { id, type: "room", name: id, subtext: "" };
  }

  function lecture(id: string, eventCount: number): ResultEntry {
    const upcoming: UpcomingEvent[] = [];
    for (let i = 0; i < eventCount; i++) {
      upcoming.push({
        start_at: `2026-10-${(15 + i).toString().padStart(2, "0")}T06:00:00Z`,
        end_at: `2026-10-${(15 + i).toString().padStart(2, "0")}T08:00:00Z`,
        room_code: "5606.EG.011",
        room_name: "Testhörsaal",
      });
    }
    return { id, type: "lecture", name: id, subtext: "", title_de: id, title_en: id, upcoming };
  }

  it("returns the index of the lecture's header in the flattened list", () => {
    const sections = [
      section("rooms", [room("r1"), room("r2")], 2),
      section("lectures", [lecture("lA", 7)], 1),
    ];
    const flat = buildVisibleSearchEntries(sections, {
      expandedFacets: new Set(),
      expandedLectures: new Set(["lA"]),
      lectureShowAll: new Set(),
    });
    expect(findLectureHeaderIndex(flat, "lA")).toBe(2);
  });

  it("returns -1 when no header matches the lecture id", () => {
    const flat = buildVisibleSearchEntries([section("rooms", [room("r1")], 1)], {
      expandedFacets: new Set(),
      expandedLectures: new Set(),
      lectureShowAll: new Set(),
    });
    expect(findLectureHeaderIndex(flat, "missing")).toBe(-1);
  });
});

describe("collapsedHighlightTarget", () => {
  function section(facet: string, entries: ResultEntry[], n_visible: number) {
    return { facet, entries, estimatedTotalHits: entries.length, n_visible };
  }

  function room(id: string): ResultEntry {
    return { id, type: "room", name: id, subtext: "" };
  }

  function lecture(id: string, eventCount: number): ResultEntry {
    const upcoming: UpcomingEvent[] = [];
    for (let i = 0; i < eventCount; i++) {
      upcoming.push({
        start_at: `2026-10-${(15 + i).toString().padStart(2, "0")}T06:00:00Z`,
        end_at: `2026-10-${(15 + i).toString().padStart(2, "0")}T08:00:00Z`,
        room_code: "5606.EG.011",
        room_name: "Testhörsaal",
      });
    }
    return { id, type: "lecture", name: id, subtext: "", title_de: id, title_en: id, upcoming };
  }

  it("targets the slot right after the lecture header when entries follow the body", () => {
    const sections = [
      section("rooms", [room("r1"), room("r2")], 2),
      section("lectures", [lecture("lA", 7)], 1),
      section("pois", [room("p1")], 1),
    ];
    const flat = buildVisibleSearchEntries(sections, {
      expandedFacets: new Set(),
      expandedLectures: new Set(["lA"]),
      lectureShowAll: new Set(),
    });
    // [r1, r2, lA-header, e0, e1, e2, show-more, p1]; show-more sits at idx 6.
    expect(flat[6]?.kind).toBe("show_more_events");
    // Header is at 2, so post-collapse the highlight should land at idx 3 -
    // which in the new flat list is where p1 will sit.
    expect(collapsedHighlightTarget(flat, 6, "lA")).toBe(3);
  });

  it("wraps to index 0 when the show-more was the last entry in the list", () => {
    const sections = [
      section("rooms", [room("r1")], 1),
      section("lectures", [lecture("lA", 7)], 1),
    ];
    const flat = buildVisibleSearchEntries(sections, {
      expandedFacets: new Set(),
      expandedLectures: new Set(["lA"]),
      lectureShowAll: new Set(),
    });
    // [r1, lA-header, e0, e1, e2, show-more]; show-more at the tail, idx 5.
    expect(flat.length - 1).toBe(5);
    expect(collapsedHighlightTarget(flat, 5, "lA")).toBe(0);
  });

  it("falls back to 0 when the header cannot be located (defensive)", () => {
    const sections = [section("rooms", [room("r1")], 1)];
    const flat = buildVisibleSearchEntries(sections, {
      expandedFacets: new Set(),
      expandedLectures: new Set(),
      lectureShowAll: new Set(),
    });
    expect(collapsedHighlightTarget(flat, 0, "missing")).toBe(0);
  });
});

describe("collapsedUpwardHighlightTarget", () => {
  it("steps one slot above the header when it wasn't at the top", () => {
    // Header at idx 3 in a list of pre-collapse length 8 (body of size 4).
    // Post-collapse length doesn't matter here - the slot above is untouched.
    expect(collapsedUpwardHighlightTarget(3, 4)).toBe(2);
  });

  it("wraps to the post-collapse tail when the header was the first entry", () => {
    // Header at idx 0; post-collapse list has 3 entries → wrap to 2.
    expect(collapsedUpwardHighlightTarget(0, 3)).toBe(2);
  });

  it("returns 0 for a single-lecture-only list (wrap onto itself)", () => {
    // Pre-collapse: just header + body; post-collapse: only header remains.
    expect(collapsedUpwardHighlightTarget(0, 1)).toBe(0);
  });

  it("clamps to 0 when the post-collapse list is empty (defensive)", () => {
    expect(collapsedUpwardHighlightTarget(0, 0)).toBe(0);
  });
});

describe("toggleLectureFromMouse", () => {
  // Mouse-click expansion is the user's "scan this whole lecture" intent, so
  // it must flip both gates - distinct from the keyboard's expandedLectures-
  // only flow which caps at LECTURE_EVENT_NAV_CAP + show-more.
  it("adds the id to both expandedLectures and lectureShowAll when not yet expanded", () => {
    const before = {
      expandedFacets: new Set<string>(),
      expandedLectures: new Set<string>(),
      lectureShowAll: new Set<string>(),
    };
    const after = toggleLectureFromMouse(before, "lA");
    expect(after.expandedLectures).toEqual(new Set(["lA"]));
    expect(after.lectureShowAll).toEqual(new Set(["lA"]));
  });

  it("removes the id from both sets when collapsing a mouse-expanded lecture", () => {
    const before = {
      expandedFacets: new Set<string>(),
      expandedLectures: new Set(["lA"]),
      lectureShowAll: new Set(["lA"]),
    };
    const after = toggleLectureFromMouse(before, "lA");
    expect(after.expandedLectures).toEqual(new Set());
    expect(after.lectureShowAll).toEqual(new Set());
  });

  // A keyboard-expanded lecture has expandedLectures.has(id) === true but
  // lectureShowAll.has(id) === false. A mouse "toggle" on this row is read as
  // "collapse it" (expandedLectures already has the id) and must drop both.
  it("treats expandedLectures presence as the expansion gate and clears both on toggle", () => {
    const before = {
      expandedFacets: new Set<string>(),
      expandedLectures: new Set(["lA"]),
      lectureShowAll: new Set<string>(),
    };
    const after = toggleLectureFromMouse(before, "lA");
    expect(after.expandedLectures).toEqual(new Set());
    expect(after.lectureShowAll).toEqual(new Set());
  });

  it("leaves unrelated lecture ids untouched in both sets", () => {
    const before = {
      expandedFacets: new Set<string>(),
      expandedLectures: new Set(["lA", "lB"]),
      lectureShowAll: new Set(["lB"]),
    };
    const after = toggleLectureFromMouse(before, "lA");
    expect(after.expandedLectures).toEqual(new Set(["lB"]));
    expect(after.lectureShowAll).toEqual(new Set(["lB"]));
  });

  it("does not mutate the input sets (returns fresh sets)", () => {
    const inExpanded = new Set<string>();
    const inShowAll = new Set<string>();
    const before = {
      expandedFacets: new Set<string>(),
      expandedLectures: inExpanded,
      lectureShowAll: inShowAll,
    };
    toggleLectureFromMouse(before, "lA");
    expect(inExpanded.size).toBe(0);
    expect(inShowAll.size).toBe(0);
  });
});
