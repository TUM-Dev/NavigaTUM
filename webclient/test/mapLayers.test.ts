import { describe, expect, it } from "vitest";
import {
  LAYER_REGISTRY,
  type LayerDef,
  parseEnabledLayers,
  parseLevel,
  resolveEnabledLayers,
  resolveLevel,
  serializeEnabledLayers,
} from "../app/composables/mapLayers";

// A two-layer registry to exercise ordering, unknown-id rejection, and multi-select without
// depending on the shipped registry only carrying WCs.
const REGISTRY: readonly LayerDef[] = [
  { id: "wcs", labelKey: "layers.wcs", icon: "M0", styleLayerIds: ["a", "b"], hintBelowZoom: 17 },
  {
    id: "elevators",
    labelKey: "layers.elevators",
    icon: "M1",
    styleLayerIds: ["c"],
    hintBelowZoom: 17,
  },
];

describe("parseEnabledLayers", () => {
  it("keeps known ids and drops unknown ones", () => {
    expect(parseEnabledLayers("wcs,bogus,elevators", REGISTRY)).toEqual(
      new Set(["wcs", "elevators"])
    );
  });

  it("trims whitespace and de-duplicates", () => {
    expect(parseEnabledLayers(" wcs , wcs ", REGISTRY)).toEqual(new Set(["wcs"]));
  });

  it("treats an empty, whitespace-only, or nullish value as no layers", () => {
    for (const input of ["", "   ", null, undefined]) {
      expect(parseEnabledLayers(input, REGISTRY)).toEqual(new Set());
    }
  });

  it("recognises the shipped WCs layer by default", () => {
    expect(parseEnabledLayers("wcs")).toEqual(new Set(["wcs"]));
  });
});

describe("serializeEnabledLayers", () => {
  it("emits ids in registry order regardless of input order", () => {
    expect(serializeEnabledLayers(["elevators", "wcs"], REGISTRY)).toBe("wcs,elevators");
  });

  it("round-trips through parseEnabledLayers", () => {
    const set = parseEnabledLayers("elevators,wcs", REGISTRY);
    expect(parseEnabledLayers(serializeEnabledLayers(set, REGISTRY), REGISTRY)).toEqual(set);
  });

  it("serialises nothing for an empty selection", () => {
    expect(serializeEnabledLayers([], REGISTRY)).toBe("");
  });
});

describe("resolveEnabledLayers precedence (URL > localStorage > default)", () => {
  it("uses the URL when present, ignoring localStorage", () => {
    expect(
      resolveEnabledLayers({ urlParam: "wcs", stored: "elevators", registry: REGISTRY })
    ).toEqual(new Set(["wcs"]));
  });

  it("honours an explicit empty URL value as all-off, beating the default", () => {
    expect(resolveEnabledLayers({ urlParam: "", stored: "wcs", registry: REGISTRY })).toEqual(
      new Set()
    );
  });

  it("falls back to localStorage when the URL is absent", () => {
    expect(
      resolveEnabledLayers({ urlParam: null, stored: "elevators", registry: REGISTRY })
    ).toEqual(new Set(["elevators"]));
  });

  it("honours an explicit empty stored value as all-off", () => {
    expect(resolveEnabledLayers({ urlParam: null, stored: "", registry: REGISTRY })).toEqual(
      new Set()
    );
  });

  it("defaults to every overlay on when neither URL nor localStorage is set", () => {
    expect(resolveEnabledLayers({ urlParam: null, stored: null, registry: REGISTRY })).toEqual(
      new Set(["wcs", "elevators"])
    );
  });

  it("defaults to WCs on for the shipped registry", () => {
    expect(resolveEnabledLayers({})).toEqual(new Set(LAYER_REGISTRY.map((l) => l.id)));
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
