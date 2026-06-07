/** The minimum a feature needs for marker reconciliation: a stable id and a position. */
export interface MarkerFeature {
  readonly id: string;
  readonly lon: number;
  readonly lat: number;
}

/**
 * Collapse features that share an `id` down to a single entry, keeping the first occurrence.
 *
 * `queryRenderedFeatures` returns one row per (feature, tile) pair, so an event whose point sits on
 * a tile boundary comes back several times under the same stable `id`. We render one marker per event.
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

/** The change between the markers currently shown and the freshly-queried set. */
export interface MarkerDiff<T extends MarkerFeature> {
  /** Features new to the viewport - the caller must create a marker for each. */
  readonly added: T[];
  /** Ids of markers no longer present - the caller must tear each one down. */
  readonly removed: string[];
  /** Features present in both - the caller keeps the marker (and may move it). */
  readonly kept: T[];
}

/**
 * Reconcile the freshly-queried features against the ids of the markers currently shown.
 *
 * `rawNext` is the raw `queryRenderedFeatures` result and is deduped by {@link dedupeFeatures}
 * first, so a kept event split across tiles collapses to a single `kept` entry rather than churning.
 */
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
