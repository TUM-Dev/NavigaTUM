export interface MarkerFeature {
  readonly id: string;
  readonly lon: number;
  readonly lat: number;
}

/**
 * Keep one feature per `id`. `queryRenderedFeatures` yields a row per tile, so a feature on a tile
 * boundary repeats under the same `id`.
 */
export function dedupeFeatures<T extends MarkerFeature>(raw: readonly T[]): T[] {
  const seen = new Set<string>();
  const out: T[] = [];
  for (const feature of raw) {
    if (seen.has(feature.id)) continue;
    seen.add(feature.id);
    out.push(feature);
  }
  return out;
}

export interface MarkerDiff<T extends MarkerFeature> {
  readonly added: T[];
  readonly removed: string[];
  readonly kept: T[];
}

/** Diff freshly-queried features against the shown ids. `rawNext` is deduped first. */
export function diffMarkers<T extends MarkerFeature>(
  rawNext: readonly T[],
  previousIds: ReadonlySet<string>
): MarkerDiff<T> {
  const next = dedupeFeatures(rawNext);
  const nextIds = new Set(next.map((f) => f.id));

  const added: T[] = [];
  const kept: T[] = [];
  for (const feature of next) {
    if (previousIds.has(feature.id)) kept.push(feature);
    else added.push(feature);
  }
  const removed: string[] = [];
  for (const id of previousIds) {
    if (!nextIds.has(id)) removed.push(id);
  }
  return { added, removed, kept };
}
