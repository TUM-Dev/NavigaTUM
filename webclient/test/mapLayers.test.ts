import { describe, expect, it } from "vitest";
import {
  categoriesForQuery,
  categoryForEntity,
  eventsWindowFilter,
  FILTER_REGISTRY,
  type FilterDef,
  parseEventsWindow,
  parseFilters,
  parseLevel,
  parseWcsGender,
  parseWcsWheelchair,
  resolveActiveFilters,
  resolveEventsWindow,
  resolveLevel,
  serializeFilters,
  wcsAttributeConditions,
} from "../app/composables/mapLayers";

// A two-filter registry to exercise ordering, unknown-id rejection, and multi-select without
// depending on the shipped registry only carrying WCs.
const REGISTRY: readonly FilterDef[] = [
  {
    id: "wcs",
    kind: "indoor",
    labelKey: "filters.wcs",
    icon: "M0",
    indoorValues: ["toilet", "shower"],
    hintBelowZoom: 17,
    keywords: ["wc", "toilet"],
  },
  {
    id: "elevators",
    kind: "indoor",
    labelKey: "filters.elevators",
    icon: "M1",
    indoorValues: ["elevator"],
    hintBelowZoom: 17,
    keywords: ["elevator", "aufzug"],
  },
];

describe("parseFilters", () => {
  it("keeps known ids and drops unknown ones", () => {
    expect(parseFilters("wcs,bogus,elevators", REGISTRY)).toEqual(new Set(["wcs", "elevators"]));
  });

  it("trims whitespace and de-duplicates", () => {
    expect(parseFilters(" wcs , wcs ", REGISTRY)).toEqual(new Set(["wcs"]));
  });

  it("treats an empty, whitespace-only, or nullish value as no filters", () => {
    for (const input of ["", "   ", null, undefined]) {
      expect(parseFilters(input, REGISTRY)).toEqual(new Set());
    }
  });

  it("recognises the shipped WCs filter by default", () => {
    expect(parseFilters("wcs")).toEqual(new Set(["wcs"]));
  });
});

describe("serializeFilters", () => {
  it("emits ids in registry order regardless of input order", () => {
    expect(serializeFilters(["elevators", "wcs"], REGISTRY)).toBe("wcs,elevators");
  });

  it("round-trips through parseFilters", () => {
    const set = parseFilters("elevators,wcs", REGISTRY);
    expect(parseFilters(serializeFilters(set, REGISTRY), REGISTRY)).toEqual(set);
  });

  it("serialises nothing for an empty selection", () => {
    expect(serializeFilters([], REGISTRY)).toBe("");
  });
});

describe("resolveActiveFilters precedence (URL > localStorage > default)", () => {
  it("uses the URL when present, ignoring localStorage", () => {
    expect(
      resolveActiveFilters({ urlParam: "wcs", stored: "elevators", registry: REGISTRY })
    ).toEqual(new Set(["wcs"]));
  });

  it("honours an explicit empty URL value as none active, beating localStorage", () => {
    expect(resolveActiveFilters({ urlParam: "", stored: "wcs", registry: REGISTRY })).toEqual(
      new Set()
    );
  });

  it("falls back to localStorage when the URL is absent", () => {
    expect(
      resolveActiveFilters({ urlParam: null, stored: "elevators", registry: REGISTRY })
    ).toEqual(new Set(["elevators"]));
  });

  it("defaults to no filter active when neither URL nor localStorage is set", () => {
    expect(resolveActiveFilters({ urlParam: null, stored: null, registry: REGISTRY })).toEqual(
      new Set()
    );
    expect(resolveActiveFilters({})).toEqual(new Set());
  });
});

describe("the shipped registry", () => {
  it("highlights toilets and showers under the WCs filter", () => {
    const wcs = FILTER_REGISTRY.find((f) => f.id === "wcs");
    expect(wcs?.kind).toBe("indoor");
    expect(wcs?.kind === "indoor" && [...wcs.indoorValues]).toEqual(["toilet", "shower"]);
  });

  it("toggles the events style layer under the events filter", () => {
    const events = FILTER_REGISTRY.find((f) => f.id === "events");
    expect(events?.kind).toBe("overlay");
    expect(events?.kind === "overlay" && [...events.styleLayers]).toEqual(["events"]);
  });
});

describe("parseEventsWindow", () => {
  it("accepts the two known windows", () => {
    expect(parseEventsWindow("now")).toBe("now");
    expect(parseEventsWindow("24h")).toBe("24h");
  });

  it("rejects unknown, empty, and absent values", () => {
    for (const input of ["12h", "NOW", "", null, undefined]) {
      expect(parseEventsWindow(input)).toBeNull();
    }
  });
});

describe("resolveEventsWindow", () => {
  it("defaults to 'now' when the value is unusable", () => {
    expect(resolveEventsWindow(null)).toBe("now");
    expect(resolveEventsWindow("bogus")).toBe("now");
    expect(resolveEventsWindow("24h")).toBe("24h");
  });
});

describe("eventsWindowFilter", () => {
  // 2026-06-11T12:00:00Z, chosen arbitrarily; the filter only depends on the passed-in clock.
  const NOW_MS = 1781179200_000;
  const NOW_S = 1781179200;

  it("keeps only currently-running events for the 'now' window", () => {
    expect(eventsWindowFilter("now", NOW_MS)).toEqual([
      "all",
      ["<=", ["get", "starts_at_epoch"], NOW_S],
      [">=", ["get", "ends_at_epoch"], NOW_S],
    ]);
  });

  it("extends the latest accepted start by 24 hours for the '24h' window", () => {
    expect(eventsWindowFilter("24h", NOW_MS)).toEqual([
      "all",
      ["<=", ["get", "starts_at_epoch"], NOW_S + 24 * 3600],
      [">=", ["get", "ends_at_epoch"], NOW_S],
    ]);
  });

  it("truncates sub-second clocks towards the past so boundary events stay visible", () => {
    const [, [, , latestStart]] = eventsWindowFilter("now", NOW_MS + 999);
    expect(latestStart).toBe(NOW_S);
  });
});

describe("parseWcsGender", () => {
  it("accepts the three known genders", () => {
    expect(parseWcsGender("male")).toBe("male");
    expect(parseWcsGender("female")).toBe("female");
    expect(parseWcsGender("unisex")).toBe("unisex");
  });

  it("rejects unknown, empty, and absent values", () => {
    for (const input of ["MALE", "diverse", "", null, undefined]) {
      expect(parseWcsGender(input)).toBeNull();
    }
  });
});

describe("parseWcsWheelchair", () => {
  it("enables only on the literal 'true'", () => {
    expect(parseWcsWheelchair("true")).toBe(true);
    for (const input of ["TRUE", "1", "yes", "false", "", null, undefined]) {
      expect(parseWcsWheelchair(input)).toBe(false);
    }
  });
});

describe("wcsAttributeConditions", () => {
  it("yields no condition when no attribute is selected, leaving the indoor dim unrefined", () => {
    expect(wcsAttributeConditions({ wheelchair: false, gender: null })).toEqual([]);
  });

  it("requires the wheelchair flag when accessibility is selected", () => {
    expect(wcsAttributeConditions({ wheelchair: true, gender: null })).toEqual([
      ["get", "is_wheelchair_toilet"],
    ]);
  });

  it("maps each gender onto its tile flag", () => {
    for (const [gender, flag] of [
      ["male", "is_male_toilet"],
      ["female", "is_female_toilet"],
      ["unisex", "is_unisex_toilet"],
    ] as const) {
      expect(wcsAttributeConditions({ wheelchair: false, gender })).toEqual([["get", flag]]);
    }
  });

  it("requires every selected attribute when combined", () => {
    expect(wcsAttributeConditions({ wheelchair: true, gender: "female" })).toEqual([
      ["get", "is_wheelchair_toilet"],
      ["get", "is_female_toilet"],
    ]);
  });
});

describe("parseLevel", () => {
  it("parses known integer floors including negatives and ground", () => {
    expect(parseLevel("0")).toBe(0);
    expect(parseLevel("3")).toBe(3);
    expect(parseLevel("-1")).toBe(-1);
  });

  it("tolerates a trailing .0 from the tile-source convention", () => {
    expect(parseLevel("2.0")).toBe(2);
  });

  it("rejects out-of-range, fractional, non-numeric, and absent values", () => {
    for (const input of ["7", "-2", "1.5", "abc", "", "  ", null, undefined]) {
      expect(parseLevel(input)).toBeNull();
    }
  });
});

describe("resolveLevel", () => {
  it("returns the parsed floor when valid", () => {
    expect(resolveLevel("3")).toBe(3);
    expect(resolveLevel("-1")).toBe(-1);
  });

  it("defaults to the ground floor when the value is unusable", () => {
    for (const input of ["7", "abc", "", null, undefined]) {
      expect(resolveLevel(input)).toBe(0);
    }
  });
});

describe("categoriesForQuery", () => {
  it("fires on natural queries against the shipped registry", () => {
    for (const query of ["toilets", "Klo", "find toilet", "WC", "toliet", "nearest restroom"]) {
      expect(categoriesForQuery(query)).toEqual(["wcs"]);
    }
  });

  it("requires exact token equality, not substrings", () => {
    for (const query of ["GWC 101", "wcs-something", "toilettenpapier", "showering"]) {
      expect(categoriesForQuery(query)).toEqual([]);
    }
  });

  it("returns nothing for empty or whitespace-only queries", () => {
    expect(categoriesForQuery("")).toEqual([]);
    expect(categoriesForQuery("   ")).toEqual([]);
  });

  it("returns multiple matches in registry order regardless of token order", () => {
    expect(categoriesForQuery("veranstaltung toilette")).toEqual(["wcs", "events"]);
    expect(categoriesForQuery("toilette veranstaltung")).toEqual(["wcs", "events"]);
  });
});

describe("categoryForEntity", () => {
  it("maps sanitary usage names to the WCs Category in both locales", () => {
    for (const name of [
      "WC",
      "WC Herren",
      "WC Men",
      "WC Damen",
      "WC Women",
      "WC Barrierefrei",
      "WC Barrier-free",
      "WC Vorraum",
      "WC Anteroom",
      "WC-Damen",
      "Dusche",
      "Shower",
    ]) {
      expect(categoryForEntity({ type: "room", type_common_name: name })).toBe("wcs");
    }
  });

  it("accepts poi-typed entities as Category members", () => {
    expect(categoryForEntity({ type: "poi", type_common_name: "Dusche" })).toBe("wcs");
  });

  it("maps non-sanitary usages to no Category", () => {
    for (const name of ["Büro", "Office", "Seminarraum", "Validierungsautomat", "Waschraum"]) {
      expect(categoryForEntity({ type: "room", type_common_name: name })).toBeNull();
    }
  });

  it("rejects usage names that merely contain a sanitary term", () => {
    for (const name of ["WCetera", "Vorraum WC", "Duschen", "Showers"]) {
      expect(categoryForEntity({ type: "room", type_common_name: name })).toBeNull();
    }
  });

  it("never maps container types, whatever their common name", () => {
    for (const type of ["building", "joined_building", "site", "campus", "area"]) {
      expect(categoryForEntity({ type, type_common_name: "WC" })).toBeNull();
    }
  });
});
