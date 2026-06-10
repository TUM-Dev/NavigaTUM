import { mdiToilet } from "@mdi/js";

/** A toggleable overlay on the `/map` page. A new layer = one `LayerDef` + one split style layer. */
export interface LayerDef {
  /** Stable key used in the `?layers=` query and in `localStorage`. */
  readonly id: string;
  /** i18n key (local scope of the `/map` page) for the panel label. */
  readonly labelKey: string;
  /** MDI path rendered as the panel icon. */
  readonly icon: string;
  /** MapLibre style-layer ids whose `visibility` this overlay flips. */
  readonly styleLayerIds: readonly string[];
  /** Below this zoom the data is not in the tiles yet, so we show a "zoom in" hint. */
  readonly hintBelowZoom: number;
}

export const LAYER_REGISTRY = [
  {
    id: "wcs",
    labelKey: "layers.wcs",
    icon: mdiToilet,
    styleLayerIds: ["indoor-toilets", "indoor-showers"],
    hintBelowZoom: 17,
  },
] as const satisfies readonly LayerDef[];

export type LayerId = (typeof LAYER_REGISTRY)[number]["id"];

export const LAYERS_QUERY_PARAM = "layers";
export const LEVEL_QUERY_PARAM = "level";
export const ENABLED_LAYERS_STORAGE_KEY = "map:enabledLayers";
export const PANEL_COLLAPSED_STORAGE_KEY = "map:panelCollapsed";

// Mirrors the `FLOOR_LEVELS` ids in `FloorControl.ts`, duplicated to keep this module free of the
// maplibre import (which fails to load under the node test environment).
export const SELECTABLE_LEVELS: readonly number[] = [6, 5, 4, 3, 2, 1, 0, -1];
export const DEFAULT_LEVEL = 0;

/**
 * Parse a comma-separated `?layers=` value into the set of enabled layer ids, dropping
 * anything not in the registry. An empty or whitespace-only value yields an empty set
 * (every overlay off), which is distinct from the value being absent entirely.
 */
export function parseEnabledLayers(
  param: string | null | undefined,
  registry: readonly LayerDef[] = LAYER_REGISTRY
): Set<string> {
  const known = new Set(registry.map((layer) => layer.id));
  const enabled = new Set<string>();
  for (const raw of (param ?? "").split(",")) {
    const id = raw.trim();
    if (id && known.has(id)) enabled.add(id);
  }
  return enabled;
}

/** Serialise enabled layer ids back into a stable, registry-ordered `?layers=` value. */
export function serializeEnabledLayers(
  enabled: Iterable<string>,
  registry: readonly LayerDef[] = LAYER_REGISTRY
): string {
  const set = enabled instanceof Set ? enabled : new Set(enabled);
  return registry
    .map((layer) => layer.id)
    .filter((id) => set.has(id))
    .join(",");
}

/**
 * Resolve which overlays start enabled, honouring precedence: an explicit `?layers=` in the
 * URL wins (even when it selects nothing, so an "all off" deep link survives a reload), then
 * `localStorage`, then the default of every registry overlay on.
 */
export function resolveEnabledLayers(opts: {
  urlParam?: string | null;
  stored?: string | null;
  registry?: readonly LayerDef[];
}): Set<string> {
  const registry = opts.registry ?? LAYER_REGISTRY;
  if (opts.urlParam !== undefined && opts.urlParam !== null)
    return parseEnabledLayers(opts.urlParam, registry);
  if (opts.stored !== undefined && opts.stored !== null)
    return parseEnabledLayers(opts.stored, registry);
  return new Set(registry.map((layer) => layer.id));
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
