import { describe, expect, it } from "vitest";
import {
  clampCropOffset,
  cropAxis,
  cropOffsetBounds,
  cropRect,
  HEADER_TARGET,
  THUMB_TARGET,
} from "../app/utils/imageCrop";

describe("thumb crop (256×256 square target)", () => {
  it("bounds are half the difference of the axes", () => {
    expect(cropOffsetBounds(800, 600, THUMB_TARGET).max).toBe(100);
    expect(cropOffsetBounds(600, 800, THUMB_TARGET).max).toBe(100);
    expect(cropOffsetBounds(500, 500, THUMB_TARGET).max).toBe(0);
  });

  it("slides a landscape crop horizontally", () => {
    expect(cropAxis(800, 600, THUMB_TARGET)).toBe("horizontal");
    expect(cropRect(800, 600, THUMB_TARGET, 0)).toEqual({ x: 100, y: 0, width: 600, height: 600 });
    expect(cropRect(800, 600, THUMB_TARGET, 50)).toEqual({ x: 150, y: 0, width: 600, height: 600 });
  });

  it("slides a portrait crop vertically", () => {
    expect(cropAxis(600, 800, THUMB_TARGET)).toBe("vertical");
    expect(cropRect(600, 800, THUMB_TARGET, 40)).toEqual({ x: 0, y: 140, width: 600, height: 600 });
  });

  it("clamps an out-of-range offset to keep the crop inside the image", () => {
    expect(clampCropOffset(800, 600, THUMB_TARGET, 9999)).toBe(100);
    expect(cropRect(800, 600, THUMB_TARGET, 9999)).toEqual({
      x: 200,
      y: 0,
      width: 600,
      height: 600,
    });
  });

  it("uses the whole square image regardless of offset", () => {
    expect(cropAxis(500, 500, THUMB_TARGET)).toBe("none");
    expect(cropRect(500, 500, THUMB_TARGET, 30)).toEqual({ x: 0, y: 0, width: 500, height: 500 });
  });
});

describe("header crop (512×210 banner target)", () => {
  it("crops the height of a square source and slides vertically", () => {
    expect(cropAxis(1000, 1000, HEADER_TARGET)).toBe("vertical");
    const rect = cropRect(1000, 1000, HEADER_TARGET, 0);
    expect(rect.width).toBe(1000);
    expect(rect.height).toBe(410);
    expect(rect.x).toBe(0);
    expect(rect.y).toBe(295);
  });

  it("crops the width of an ultra-wide source and slides horizontally", () => {
    expect(cropAxis(4000, 500, HEADER_TARGET)).toBe("horizontal");
    const rect = cropRect(4000, 500, HEADER_TARGET, 0);
    expect(rect.height).toBe(500);
    expect(rect.width).toBe(1218);
    expect(rect.y).toBe(0);
  });
});
