// Mirrors `Resizer.resize_to_fixed_size` in `data/processors/images.py`: the square thumb the event
// marker renders is `min(width, height)` per side and slides along the longer axis by an integer
// `offset` of source pixels (`offsets.thumb` in `img-sources.yaml`). `offset = 0` centres it. These
// helpers reproduce that crop so the submission form can pick the offset with a faithful preview.

export interface ThumbCropRect {
  /** left edge of the crop square, in source pixels */
  x: number;
  /** top edge of the crop square, in source pixels */
  y: number;
  /** side length of the (square) crop, in source pixels */
  size: number;
}

/**
 * The inclusive `[-max, max]` range an offset may take before the crop square would leave the
 * image. A square source has `max = 0` (nothing to choose).
 */
export function thumbOffsetBounds(width: number, height: number): { max: number } {
  return { max: Math.floor(Math.abs(width - height) / 2) };
}

export function clampThumbOffset(width: number, height: number, offset: number): number {
  const { max } = thumbOffsetBounds(width, height);
  return Math.max(-max, Math.min(max, Math.round(offset)));
}

export function thumbCropRect(width: number, height: number, offset: number): ThumbCropRect {
  const size = Math.min(width, height);
  const clamped = clampThumbOffset(width, height, offset);
  if (width >= height) {
    // Landscape (or square): the square slides horizontally; full height is kept.
    return { x: Math.floor(width / 2) - Math.floor(size / 2) + clamped, y: 0, size };
  }
  // Portrait: the square slides vertically; full width is kept.
  return { x: 0, y: Math.floor(height / 2) - Math.floor(size / 2) + clamped, size };
}

/**
 * Renders the offset crop to a 256×256 `image/webp` blob URL - the same dimensions the pipeline
 * emits - so the live map marker shows exactly what will be generated. The caller owns revoking the
 * returned URL.
 */
export function cropToThumbBlobUrl(
  image: HTMLImageElement,
  width: number,
  height: number,
  offset: number,
  outSize = 256
): Promise<string | null> {
  const { x, y, size } = thumbCropRect(width, height, offset);
  const canvas = document.createElement("canvas");
  canvas.width = outSize;
  canvas.height = outSize;
  const ctx = canvas.getContext("2d");
  if (!ctx) return Promise.resolve(null);
  ctx.drawImage(image, x, y, size, size, 0, 0, outSize, outSize);
  return new Promise((resolve) => {
    canvas.toBlob((blob) => resolve(blob ? URL.createObjectURL(blob) : null), "image/webp", 0.9);
  });
}
