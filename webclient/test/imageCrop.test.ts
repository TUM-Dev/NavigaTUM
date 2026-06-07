import { describe, expect, it } from "vitest";
import { clampThumbOffset, thumbCropRect, thumbOffsetBounds } from "../app/utils/imageCrop";

describe("thumbOffsetBounds", () => {
  it("is half the difference of the axes", () => {
    expect(thumbOffsetBounds(800, 600).max).toBe(100);
    expect(thumbOffsetBounds(600, 800).max).toBe(100);
  });
  it("is zero for a square image", () => {
    expect(thumbOffsetBounds(500, 500).max).toBe(0);
  });
});

describe("clampThumbOffset", () => {
  it("clamps to the valid range and rounds", () => {
    expect(clampThumbOffset(800, 600, 999)).toBe(100);
    expect(clampThumbOffset(800, 600, -999)).toBe(-100);
    expect(clampThumbOffset(800, 600, 12.6)).toBe(13);
  });
});

describe("thumbCropRect", () => {
  it("centres a landscape crop at offset 0", () => {
    // 800×600 → 600² square centred horizontally: x = 400 - 300 = 100.
    expect(thumbCropRect(800, 600, 0)).toEqual({ x: 100, y: 0, size: 600 });
  });
  it("slides a landscape crop horizontally with the offset", () => {
    expect(thumbCropRect(800, 600, 50)).toEqual({ x: 150, y: 0, size: 600 });
    expect(thumbCropRect(800, 600, -50)).toEqual({ x: 50, y: 0, size: 600 });
  });
  it("centres a portrait crop and slides it vertically", () => {
    expect(thumbCropRect(600, 800, 0)).toEqual({ x: 0, y: 100, size: 600 });
    expect(thumbCropRect(600, 800, 40)).toEqual({ x: 0, y: 140, size: 600 });
  });
  it("clamps an out-of-range offset to keep the crop inside the image", () => {
    expect(thumbCropRect(800, 600, 9999)).toEqual({ x: 200, y: 0, size: 600 });
  });
  it("uses the whole square image regardless of offset", () => {
    expect(thumbCropRect(500, 500, 30)).toEqual({ x: 0, y: 0, size: 500 });
  });
});
