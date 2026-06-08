// Mirrors `Resizer.resize_to_fixed_size` in `data/processors/images.py`: a crop of the target's
// aspect ratio is taken from the source and slid along the over-long axis by an integer `offset` of
// source pixels (`offsets.thumb` / `offsets.header` in `img-sources.yaml`). `offset = 0` centres it.
// The pipeline emits a 256² thumb and a 512×210 header; these helpers reproduce that crop so the
// submission form can pick each offset with a faithful preview.

export interface CropTarget {
  width: number;
  height: number;
}

export const THUMB_TARGET: CropTarget = { width: 256, height: 256 };
export const HEADER_TARGET: CropTarget = { width: 512, height: 210 };

export type CropAxis = "horizontal" | "vertical" | "none";

export interface CropRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface CropGeometry {
  axis: CropAxis;
  width: number;
  height: number;
}

// The crop rectangle's size and the axis it travels along, before the offset is applied. Matches the
// `int(new_width / 2)` truncation the pipeline uses, so the dimensions are even.
function cropGeometry(width: number, height: number, target: CropTarget): CropGeometry {
  const targetAspect = target.width / target.height;
  const sourceAspect = width / height;
  if (targetAspect < sourceAspect) {
    // Source is wider than the target: crop the width, keep full height, slide horizontally.
    return { axis: "horizontal", width: 2 * Math.floor((targetAspect * height) / 2), height };
  }
  if (targetAspect > sourceAspect) {
    // Source is taller than the target: crop the height, keep full width, slide vertically.
    return { axis: "vertical", width, height: 2 * Math.floor(width / targetAspect / 2) };
  }
  return { axis: "none", width, height };
}

export function cropAxis(width: number, height: number, target: CropTarget): CropAxis {
  return cropGeometry(width, height, target).axis;
}

/**
 * The inclusive `[-max, max]` range an offset may take before the crop would leave the image. A
 * source already matching the target aspect ratio has `max = 0` (nothing to choose).
 */
export function cropOffsetBounds(
  width: number,
  height: number,
  target: CropTarget
): { max: number } {
  const geo = cropGeometry(width, height, target);
  if (geo.axis === "horizontal") return { max: Math.floor((width - geo.width) / 2) };
  if (geo.axis === "vertical") return { max: Math.floor((height - geo.height) / 2) };
  return { max: 0 };
}

export function clampCropOffset(
  width: number,
  height: number,
  target: CropTarget,
  offset: number
): number {
  const { max } = cropOffsetBounds(width, height, target);
  return Math.max(-max, Math.min(max, Math.round(offset)));
}

export function cropRect(
  width: number,
  height: number,
  target: CropTarget,
  offset: number
): CropRect {
  const geo = cropGeometry(width, height, target);
  const clamped = clampCropOffset(width, height, target, offset);
  if (geo.axis === "horizontal") {
    return {
      x: Math.floor(width / 2) - Math.floor(geo.width / 2) + clamped,
      y: 0,
      width: geo.width,
      height,
    };
  }
  if (geo.axis === "vertical") {
    return {
      x: 0,
      y: Math.floor(height / 2) - Math.floor(geo.height / 2) + clamped,
      width,
      height: geo.height,
    };
  }
  return { x: 0, y: 0, width, height };
}

/**
 * Renders the offset crop at the target's pixel dimensions as an `image/webp` blob URL - the same
 * output the pipeline produces - so the form can preview exactly what will be generated. The caller
 * owns revoking the returned URL.
 */
export function cropToBlobUrl(
  image: HTMLImageElement,
  width: number,
  height: number,
  target: CropTarget,
  offset: number
): Promise<string | null> {
  const rect = cropRect(width, height, target, offset);
  const canvas = document.createElement("canvas");
  canvas.width = target.width;
  canvas.height = target.height;
  const ctx = canvas.getContext("2d");
  if (!ctx) return Promise.resolve(null);
  ctx.drawImage(image, rect.x, rect.y, rect.width, rect.height, 0, 0, target.width, target.height);
  return new Promise((resolve) => {
    canvas.toBlob((blob) => resolve(blob ? URL.createObjectURL(blob) : null), "image/webp", 0.9);
  });
}
