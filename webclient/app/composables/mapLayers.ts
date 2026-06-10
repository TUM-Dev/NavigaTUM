import { mdiToilet } from "@mdi/js";

/** A map filter that highlights one category by dimming everything else. */
export interface FilterDef {
  /** Stable key used in the `?filter=` query and in `localStorage`. */
  readonly id: string;
  /** i18n key (local scope of the `/map` page) for the panel label. */
  readonly labelKey: string;
  /** MDI path rendered as the panel icon. */
  readonly icon: string;
  /** The `indoor` property values this filter keeps vibrant. */
  readonly indoorValues: readonly string[];
  /** Below this zoom the data is not in the tiles yet, so we show a "zoom in" hint. */
  readonly hintBelowZoom: number;
}

export const FILTER_REGISTRY = [
  {
    id: "wcs",
    labelKey: "filters.wcs",
    icon: mdiToilet,
    indoorValues: ["toilet", "shower"],
    hintBelowZoom: 17,
  },
] as const satisfies readonly FilterDef[];

export type FilterId = (typeof FILTER_REGISTRY)[number]["id"];

export const FILTER_QUERY_PARAM = "filter";
export const LEVEL_QUERY_PARAM = "level";
export const ACTIVE_FILTERS_STORAGE_KEY = "map:activeFilters";
export const PANEL_COLLAPSED_STORAGE_KEY = "map:panelCollapsed";

// Mirrors the `FLOOR_LEVELS` ids in `FloorControl.ts`, duplicated to keep this module free of the
// maplibre import (which fails to load under the node test environment).
export const SELECTABLE_LEVELS: readonly number[] = [6, 5, 4, 3, 2, 1, 0, -1];
export const DEFAULT_LEVEL = 0;

/**
 * Parse a comma-separated `?filter=` value into the set of active filter ids, dropping anything
 * not in the registry. An empty or whitespace-only value yields an empty set, which is distinct
 * from the value being absent entirely.
 */
export function parseFilters(
  param: string | null | undefined,
  registry: readonly FilterDef[] = FILTER_REGISTRY
): Set<string> {
  const known = new Set(registry.map((f) => f.id));
  const active = new Set<string>();
  for (const raw of (param ?? "").split(",")) {
    const id = raw.trim();
    if (id && known.has(id)) active.add(id);
  }
  return active;
}

/** Serialise active filter ids back into a stable, registry-ordered `?filter=` value. */
export function serializeFilters(
  active: Iterable<string>,
  registry: readonly FilterDef[] = FILTER_REGISTRY
): string {
  const set = active instanceof Set ? active : new Set(active);
  return registry
    .map((f) => f.id)
    .filter((id) => set.has(id))
    .join(",");
}

/**
 * Resolve which filters start active, honouring precedence: an explicit `?filter=` in the URL
 * wins (even when empty, so a deep link survives a reload), then `localStorage`, then the default
 * of no filter active.
 */
export function resolveActiveFilters(opts: {
  urlParam?: string | null;
  stored?: string | null;
  registry?: readonly FilterDef[];
}): Set<string> {
  const registry = opts.registry ?? FILTER_REGISTRY;
  if (opts.urlParam !== undefined && opts.urlParam !== null)
    return parseFilters(opts.urlParam, registry);
  if (opts.stored !== undefined && opts.stored !== null) return parseFilters(opts.stored, registry);
  return new Set();
}

/** Parse a `?level=` value into a known integer floor, or `null` when absent or invalid. */
export function parseLevel(param: string | null | undefined): number | null {
  if (param === null || param === undefined || param.trim() === "") return null;
  const level = Number(param);
  if (!Number.isInteger(level) || !SELECTABLE_LEVELS.includes(level)) return null;
  return level;
}

/** Resolve the initial floor, defaulting to the ground floor when `?level=` is unusable. */
export function resolveLevel(param: string | null | undefined): number {
  return parseLevel(param) ?? DEFAULT_LEVEL;
}
