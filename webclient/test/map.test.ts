import { describe, expect, it } from "vitest";
import { browseMapUrl, zoomForLocationType } from "../app/utils/map";

describe("zoomForLocationType", () => {
  it("frames buildings and rooms tighter than container types", () => {
    expect(zoomForLocationType("building")).toBe(17);
    expect(zoomForLocationType("room")).toBe(18);
    for (const type of ["site", "campus", "poi", undefined] as const) {
      expect(zoomForLocationType(type)).toBe(16);
    }
  });
});

describe("browseMapUrl", () => {
  it("builds the MapLibre hash with the type-appropriate zoom", () => {
    expect(browseMapUrl({ lat: 48.266921, lon: 11.670099 }, "building")).toBe(
      "/map#17/48.26692/11.67010"
    );
  });

  it("frames sites at the container zoom", () => {
    expect(browseMapUrl({ lat: 48.1, lon: 11.5 }, "site")).toBe("/map#16/48.10000/11.50000");
  });
});
