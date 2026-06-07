import { describe, expect, it } from "vitest";
import { dedupeFeatures, diffMarkers } from "../app/utils/eventMarkers";

// `queryRenderedFeatures` returns one row per (feature, tile) pair, so an event sitting on a tile
// boundary comes back several times under the same stable `id`. These fixtures mimic that.
type Feat = { id: string; lon: number; lat: number };
const feat = (id: string, lon = 11.5, lat = 48.1): Feat => ({ id, lon, lat });

describe("dedupeFeatures", () => {
  it("collapses a feature split across tile boundaries to a single entry", () => {
    const result = dedupeFeatures([feat("a"), feat("a"), feat("b")]);
    expect(result.map((f) => f.id)).toEqual(["a", "b"]);
  });

  it("keeps the first occurrence (with its coordinates) and preserves order", () => {
    const result = dedupeFeatures([feat("b", 1, 1), feat("a", 2, 2), feat("b", 9, 9)]);
    expect(result.map((f) => f.id)).toEqual(["b", "a"]);
    expect(result[0]).toMatchObject({ id: "b", lon: 1, lat: 1 });
  });

  it("returns an empty array for empty input", () => {
    expect(dedupeFeatures([])).toEqual([]);
  });
});

describe("diffMarkers", () => {
  it("reports features not yet shown as added", () => {
    const diff = diffMarkers([feat("a"), feat("b")], new Set(["a"]));
    expect(diff.added.map((f) => f.id)).toEqual(["b"]);
  });

  it("reports ids that left the viewport as removed", () => {
    const diff = diffMarkers([feat("a")], new Set(["a", "gone"]));
    expect(diff.removed).toEqual(["gone"]);
  });

  it("reports features present in both as kept, not added", () => {
    const diff = diffMarkers([feat("a")], new Set(["a"]));
    expect(diff.kept.map((f) => f.id)).toEqual(["a"]);
    expect(diff.added).toEqual([]);
    expect(diff.removed).toEqual([]);
  });

  it("collapses a tile-split kept event to a single kept entry, never added", () => {
    const diff = diffMarkers([feat("a"), feat("a")], new Set(["a"]));
    expect(diff.kept.map((f) => f.id)).toEqual(["a"]);
    expect(diff.added).toEqual([]);
  });

  it("removes everything when nothing is queried (e.g. panned away)", () => {
    const diff = diffMarkers([], new Set(["a", "b"]));
    expect(diff.removed.sort()).toEqual(["a", "b"]);
    expect(diff.added).toEqual([]);
    expect(diff.kept).toEqual([]);
  });

  it("adds everything on the first render, with nothing shown yet", () => {
    const diff = diffMarkers([feat("a"), feat("b")], new Set());
    expect(diff.added.map((f) => f.id)).toEqual(["a", "b"]);
    expect(diff.removed).toEqual([]);
    expect(diff.kept).toEqual([]);
  });
});
