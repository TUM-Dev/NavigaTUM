// Mirrors `Resizer.resize_to_fixed_size` in `data/processors/images.py`.

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

function cropGeometry(width: number, height: number, target: CropTarget): CropGeometry {
  const targetAspect = target.width / target.height;
  const sourceAspect = width / height;
  if (targetAspect < sourceAspect) {
    return { axis: "horizontal", width: 2 * Math.floor((targetAspect * height) / 2), height };
  }
  if (targetAspect > sourceAspect) {
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
 * Renders the offset crop at the target's pixel dimensions as an `image/webp` blob - the same
 * output the pipeline produces - so the form can preview exactly what will be generated. Pair with
 * `useObjectUrl` so the URL's lifetime tracks the blob without manual revoke bookkeeping.
 */
export function cropToBlob(
  image: CanvasImageSource,
  width: number,
  height: number,
  target: CropTarget,
  offset: number
): Promise<Blob | null> {
  const rect = cropRect(width, height, target, offset);
  const canvas = document.createElement("canvas");
  canvas.width = target.width;
  canvas.height = target.height;
  const ctx = canvas.getContext("2d");
  if (!ctx) return Promise.resolve(null);
  ctx.drawImage(image, rect.x, rect.y, rect.width, rect.height, 0, 0, target.width, target.height);
  return new Promise((resolve) => canvas.toBlob(resolve, "image/webp", 0.9));
}
