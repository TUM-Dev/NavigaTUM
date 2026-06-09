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
  type LectureExpansionState,
  lectureEventPath,
  lectureTitle,
  toggleLectureFromMouse,
  useLectureRowExpansion,
  type VisibleSearchEntry,
} from "../app/utils/lectureRow";

// Entries are now grouped by section facet rather than carrying a per-entry
// discriminator; the helpers below build the two concrete entry shapes.
type ResultEntry = components["schemas"]["LocationEntry"] | components["schemas"]["LectureEntry"];
type ResultsSection = components["schemas"]["ResultsSection"];
type UpcomingEvent = components["schemas"]["UpcomingEvent"];

const SAME_DAY_DE_RE = /Do\.?, 15\. Oktober, 08:00-10:00/;
const SAME_DAY_EN_RE = /Thu 15 Oct, 08:00-10:00/;
const CROSS_MIDNIGHT_RE = /15 Oct.*22:30.*16 Oct.*00:30/;

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
        start_at: "2026-10-22T07:00:00Z", // CET kicks in after 2026-10-26; here still CEST.
        end_at: "2026-10-22T09:00:00Z",
        room_code: "5606.EG.011",
        room_name: "Testhörsaal",
      },
    ],
    ...overrides,
  };
}

function room(id: string): ResultEntry {
  return { id, type: "room", name: id, subtext: "" };
}

function events(n: number): UpcomingEvent[] {
  const list: UpcomingEvent[] = [];
  for (let i = 0; i < n; i++) {
    const day = (15 + i).toString().padStart(2, "0");
    list.push({
      start_at: `2026-10-${day}T06:00:00Z`,
      end_at: `2026-10-${day}T08:00:00Z`,
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

function state(overrides: Partial<LectureExpansionState> = {}): LectureExpansionState {
  return {
    expandedFacets: new Set(),
    expandedLectures: new Set(),
    lectureShowAll: new Set(),
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
    expect(lectureTitle(makeLecture({ title_en: null }), "en")).toBe(
      "Einführung in die Informatik 1"
    );
    expect(lectureTitle(makeLecture({ title_de: null }), "de")).toBe(
      "Introduction to Informatics 1"
    );
  });

  it("falls back to `name` when both titles are absent (defensive)", () => {
    const stripped = makeLecture({ title_de: null, title_en: null, name: "Some Lecture" });
    expect(lectureTitle(stripped, "de")).toBe("Some Lecture");
  });
});

describe("firstUpcoming", () => {
  it("returns the first event when `upcoming` is populated", () => {
    expect(firstUpcoming(makeLecture())?.room_code).toBe("5606.EG.011");
  });

  it("returns null when `upcoming` is empty or absent", () => {
    expect(firstUpcoming(makeLecture({ upcoming: [] }))).toBeNull();
    expect(firstUpcoming(makeLecture({ upcoming: null }))).toBeNull();
  });
});

describe("lectureEventPath", () => {
  it("routes to /room/<room_code>", () => {
    expect(lectureEventPath({ room_code: "5606.EG.011" })).toBe("/room/5606.EG.011");
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

  const sameDayEvent: UpcomingEvent = {
    start_at: "2026-10-15T06:00:00Z",
    end_at: "2026-10-15T08:00:00Z",
    room_code: "5606.EG.011",
    room_name: "Testhörsaal",
  };

  it.each([
    ["de", SAME_DAY_DE_RE],
    ["en", SAME_DAY_EN_RE],
  ] as const)("emits the localised same-day range for %s", (locale, expected) => {
    expect(formatUpcoming(sameDayEvent, locale)).toMatch(expected);
  });

  it("renders both date and time on each side when the event crosses Berlin midnight", () => {
    // CEST until 2026-10-25, so UTC + 2h yields 22:30 → 00:30 Berlin.
    const event: UpcomingEvent = {
      start_at: "2026-10-15T20:30:00Z",
      end_at: "2026-10-15T22:30:00Z",
      room_code: "5606.EG.011",
      room_name: "Testhörsaal",
    };
    const formatted = formatUpcoming(event, "en");
    expect(formatted).toContain(" - ");
    expect(formatted).toMatch(CROSS_MIDNIGHT_RE);
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
  function ids(out: readonly VisibleSearchEntry[]): readonly string[] {
    return out.map((e) => (e.kind === "result" ? e.entry.id : e.kind));
  }

  it("flattens non-lecture sections respecting n_visible", () => {
    const sections = [section("rooms", [room("r1"), room("r2"), room("r3")], 2)];
    expect(ids(buildVisibleSearchEntries(sections, state()))).toEqual(["r1", "r2"]);
  });

  it("respects expandedFacets to lift the section cap", () => {
    const sections = [section("rooms", [room("r1"), room("r2"), room("r3")], 2)];
    const out = buildVisibleSearchEntries(sections, state({ expandedFacets: new Set(["rooms"]) }));
    expect(ids(out)).toEqual(["r1", "r2", "r3"]);
  });

  it("emits only the lecture header when expandedLectures does not contain its id", () => {
    const sections = [section("lectures", [lecture("l1", 5)], 1)];
    const out = buildVisibleSearchEntries(sections, state());
    expect(out).toHaveLength(1);
    expect(out[0]?.kind).toBe("result");
  });

  it("emits all events with no show-more when total events <= cap", () => {
    const sections = [section("lectures", [lecture("l1", 3)], 1)];
    const out = buildVisibleSearchEntries(sections, state({ expandedLectures: new Set(["l1"]) }));
    expect(out.map((e) => e.kind)).toEqual(["result", "event", "event", "event"]);
  });

  it("caps events at LECTURE_EVENT_NAV_CAP and emits a show-more sentinel with the hidden count", () => {
    const sections = [section("lectures", [lecture("l1", 7)], 1)];
    const out = buildVisibleSearchEntries(sections, state({ expandedLectures: new Set(["l1"]) }));
    // header + 3 events + show-more.
    expect(out).toHaveLength(LECTURE_EVENT_NAV_CAP + 2);
    expect(out.at(-1)).toEqual({
      kind: "show_more_events",
      lectureId: "l1",
      hiddenCount: 7 - LECTURE_EVENT_NAV_CAP,
    });
  });

  it("drops the show-more sentinel once lectureShowAll opts into the full list", () => {
    const sections = [section("lectures", [lecture("l1", 7)], 1)];
    const out = buildVisibleSearchEntries(
      sections,
      state({
        expandedLectures: new Set(["l1"]),
        lectureShowAll: new Set(["l1"]),
      })
    );
    expect(out).toHaveLength(1 + 7);
    expect(out.every((e) => e.kind !== "show_more_events")).toBe(true);
  });

  it("indexes events from zero against the lecture's own occurrence list", () => {
    const sections = [section("lectures", [lecture("l1", 5)], 1)];
    const out = buildVisibleSearchEntries(sections, state({ expandedLectures: new Set(["l1"]) }));
    const eventEntries = out.filter((e) => e.kind === "event");
    expect(eventEntries.map((e) => (e.kind === "event" ? e.eventIndex : -1))).toEqual([0, 1, 2]);
  });
});

describe("findLectureHeaderIndex", () => {
  it("returns the index of the lecture's header in the flattened list", () => {
    const sections = [
      section("rooms", [room("r1"), room("r2")], 2),
      section("lectures", [lecture("lA", 7)], 1),
    ];
    const flat = buildVisibleSearchEntries(sections, state({ expandedLectures: new Set(["lA"]) }));
    expect(findLectureHeaderIndex(flat, "lA")).toBe(2);
  });

  it("returns -1 when no header matches the lecture id", () => {
    const flat = buildVisibleSearchEntries([section("rooms", [room("r1")], 1)], state());
    expect(findLectureHeaderIndex(flat, "missing")).toBe(-1);
  });
});

describe("collapsedHighlightTarget", () => {
  it("targets the slot right after the lecture header when entries follow the body", () => {
    const sections = [
      section("rooms", [room("r1"), room("r2")], 2),
      section("lectures", [lecture("lA", 7)], 1),
      section("pois", [room("p1")], 1),
    ];
    const flat = buildVisibleSearchEntries(sections, state({ expandedLectures: new Set(["lA"]) }));
    // [r1, r2, lA-header, e0, e1, e2, show-more, p1]; collapsing lA lands at p1's slot (3).
    expect(flat[6]?.kind).toBe("show_more_events");
    expect(collapsedHighlightTarget(flat, 6, "lA")).toBe(3);
  });

  it("wraps to index 0 when the show-more was the last entry in the list", () => {
    const sections = [
      section("rooms", [room("r1")], 1),
      section("lectures", [lecture("lA", 7)], 1),
    ];
    const flat = buildVisibleSearchEntries(sections, state({ expandedLectures: new Set(["lA"]) }));
    // [r1, lA-header, e0, e1, e2, show-more]; show-more at the tail, idx 5.
    expect(flat.length - 1).toBe(5);
    expect(collapsedHighlightTarget(flat, 5, "lA")).toBe(0);
  });

  it("falls back to 0 when the header cannot be located (defensive)", () => {
    const flat = buildVisibleSearchEntries([section("rooms", [room("r1")], 1)], state());
    expect(collapsedHighlightTarget(flat, 0, "missing")).toBe(0);
  });
});

describe("collapsedUpwardHighlightTarget", () => {
  it("steps one slot above the header when it wasn't at the top", () => {
    expect(collapsedUpwardHighlightTarget(3, 4)).toBe(2);
  });

  it("wraps to the post-collapse tail when the header was the first entry", () => {
    expect(collapsedUpwardHighlightTarget(0, 3)).toBe(2);
  });

  it("returns 0 for a single-lecture-only list (wrap onto itself)", () => {
    expect(collapsedUpwardHighlightTarget(0, 1)).toBe(0);
  });

  it("clamps to 0 when the post-collapse list is empty (defensive)", () => {
    expect(collapsedUpwardHighlightTarget(0, 0)).toBe(0);
  });
});

describe("toggleLectureFromMouse", () => {
  it("adds the id to both expandedLectures and lectureShowAll when not yet expanded", () => {
    const after = toggleLectureFromMouse(state(), "lA");
    expect(after.expandedLectures).toEqual(new Set(["lA"]));
    expect(after.lectureShowAll).toEqual(new Set(["lA"]));
  });

  it("removes the id from both sets when collapsing a mouse-expanded lecture", () => {
    const after = toggleLectureFromMouse(
      state({ expandedLectures: new Set(["lA"]), lectureShowAll: new Set(["lA"]) }),
      "lA"
    );
    expect(after.expandedLectures).toEqual(new Set());
    expect(after.lectureShowAll).toEqual(new Set());
  });

  it("treats expandedLectures presence as the expansion gate and clears both on toggle", () => {
    const after = toggleLectureFromMouse(state({ expandedLectures: new Set(["lA"]) }), "lA");
    expect(after.expandedLectures).toEqual(new Set());
    expect(after.lectureShowAll).toEqual(new Set());
  });

  it("leaves unrelated lecture ids untouched in both sets", () => {
    const after = toggleLectureFromMouse(
      state({
        expandedLectures: new Set(["lA", "lB"]),
        lectureShowAll: new Set(["lB"]),
      }),
      "lA"
    );
    expect(after.expandedLectures).toEqual(new Set(["lB"]));
    expect(after.lectureShowAll).toEqual(new Set(["lB"]));
  });

  it("does not mutate the input sets (returns fresh sets)", () => {
    const inExpanded = new Set<string>();
    const inShowAll = new Set<string>();
    toggleLectureFromMouse(
      state({ expandedLectures: inExpanded, lectureShowAll: inShowAll }),
      "lA"
    );
    expect(inExpanded.size).toBe(0);
    expect(inShowAll.size).toBe(0);
  });
});
