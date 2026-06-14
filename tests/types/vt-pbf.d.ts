// `@types/vt-pbf` types `fromGeojsonVt`'s layer values as the whole geojson-vt index, but the
// function actually wants a single tile (the `{ features }` object `getTile` returns), so we declare
// the slice of the API we use ourselves.
declare module "vt-pbf" {
  interface VtTile {
    readonly features: readonly unknown[];
  }
  export function fromGeojsonVt(
    layers: Record<string, VtTile>,
    options?: { version?: number }
  ): Uint8Array;
  const vtpbf: { fromGeojsonVt: typeof fromGeojsonVt };
  export default vtpbf;
}
