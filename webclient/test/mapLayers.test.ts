import { describe, expect, it } from "vitest";
import {
  FILTER_REGISTRY,
  type FilterDef,
  parseFilters,
  parseLevel,
  resolveActiveFilters,
  resolveLevel,
  serializeFilters,
} from "../app/composables/mapLayers";

// A two-filter registry to exercise ordering, unknown-id rejection, and multi-select without
// depending on the shipped registry only carrying WCs.
const REGISTRY: readonly FilterDef[] = [
  {
    id: "wcs",
    labelKey: "filters.wcs",
    icon: "M0",
    indoorValues: ["toilet", "shower"],
    hintBelowZoom: 17,
  },
  {
    id: "elevators",
    labelKey: "filters.elevators",
    icon: "M1",
    indoorValues: ["elevator"],
    hintBelowZoom: 17,
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
    expect(wcs?.indoorValues).toEqual(["toilet", "shower"]);
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
