import { mdiCalendarStar, mdiToilet } from "@mdi/js";

interface FilterDefBase {
  /** Stable key used in the `?filter=` query and in `localStorage`. */
  readonly id: string;
  /** i18n key (local scope of the `MapLayerPanel`) for the panel label. */
  readonly labelKey: string;
  /** MDI path rendered as the panel icon. */
  readonly icon: string;
  /** Below this zoom the data is not in the tiles yet, so we show a "zoom in" hint. */
  readonly hintBelowZoom: number;
  /**
   * Lowercase query tokens that surface this Category as a search shortcut. Intentionally broad:
   * synonyms and common misspellings in both languages, so the shortcut fires on natural queries.
   */
  readonly keywords: readonly string[];
}

/** A map filter that highlights one indoor category by dimming everything else. */
export interface IndoorFilterDef extends FilterDefBase {
  readonly kind: "indoor";
  /** The `indoor` property values this filter keeps vibrant. */
  readonly indoorValues: readonly string[];
}

/** A map filter that toggles the visibility of dedicated basemap-style layers. */
export interface OverlayFilterDef extends FilterDefBase {
  readonly kind: "overlay";
  /** The basemap-style layer ids this filter shows while active (they default to hidden). */
  readonly styleLayers: readonly string[];
}

export type FilterDef = IndoorFilterDef | OverlayFilterDef;

/** The basemap-style layer carrying the event markers. */
export const EVENTS_STYLE_LAYER = "events";
/** The events entry in the filter registry; its time window only applies while it is active. */
export const EVENTS_FILTER_ID = "events";
/** The WCs entry in the filter registry; its attribute filters only apply while it is active. */
export const WCS_FILTER_ID = "wcs";
/** The `indoor` values the WCs filter covers, shared with its attribute-filter expression. */
export const WCS_INDOOR_VALUES = ["toilet", "shower"] as const;

export const FILTER_REGISTRY = [
  {
    id: WCS_FILTER_ID,
    kind: "indoor",
    labelKey: "filters.wcs",
    icon: mdiToilet,
    indoorValues: WCS_INDOOR_VALUES,
    hintBelowZoom: 17,
    keywords: [
      "toilet",
      "toilets",
      "toliet",
      "toilette",
      "toiletten",
      "toillette",
      "wc",
      "wcs",
      "klo",
      "klos",
      "restroom",
      "restrooms",
      "loo",
      "bathroom",
      "bathrooms",
      "dusche",
      "duschen",
      "shower",
      "showers",
    ],
  },
  {
    id: EVENTS_FILTER_ID,
    kind: "overlay",
    labelKey: "filters.events",
    icon: mdiCalendarStar,
    styleLayers: [EVENTS_STYLE_LAYER],
    hintBelowZoom: 13,
    keywords: ["event", "events", "veranstaltung", "veranstaltungen"],
  },
] as const satisfies readonly FilterDef[];

export type FilterId = (typeof FILTER_REGISTRY)[number]["id"];

// The details API exposes usage only as the localized `type_common_name`, so WCs membership
// matches the sanitary names of that closed usage set (TUMonline names plus their
// `data/translations.yaml` English forms; user-added rooms hyphenate, e.g. "WC-Damen").
const WCS_TYPE_COMMON_NAME = /^WC([ -]|$)|^(Dusche|Shower)$/;

const QUERY_TOKEN_SEPARATOR = /\s+/;

/**
 * The Categories whose keyword list contains one of the query's whitespace-separated tokens, in
 * registry order. Tokens match exactly after case-folding: substring matching would collide with
 * room-code prefixes such as "GWC 101".
 */
export function categoriesForQuery(query: string): FilterId[] {
  const tokens = new Set(query.trim().toLowerCase().split(QUERY_TOKEN_SEPARATOR));
  return FILTER_REGISTRY.filter((f) => f.keywords.some((k) => tokens.has(k))).map((f) => f.id);
}

/** Map an Entity to the Category it belongs to, or `null` when it belongs to none. */
export function categoryForEntity(entity: {
  readonly type: string;
  readonly type_common_name: string;
}): FilterId | null {
  if (entity.type !== "room" && entity.type !== "poi") return null;
  if (WCS_TYPE_COMMON_NAME.test(entity.type_common_name)) return WCS_FILTER_ID;
  return null;
}

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

// The time-window query parameter is namespaced per layer (`events_…`), so further layers can
// add their own sub-filters without colliding.
export const EVENTS_WINDOW_QUERY_PARAM = "events_window";
export const EVENTS_WINDOWS = ["now", "24h"] as const;
export type EventsWindow = (typeof EVENTS_WINDOWS)[number];
export const DEFAULT_EVENTS_WINDOW: EventsWindow = "now";

/** Parse a `?events_window=` value, or `null` when absent or not a known window. */
export function parseEventsWindow(param: string | null | undefined): EventsWindow | null {
  const window = EVENTS_WINDOWS.find((w) => w === param);
  return window ?? null;
}

/** Resolve the initial time window, defaulting to "happening now" when unusable. */
export function resolveEventsWindow(param: string | null | undefined): EventsWindow {
  return parseEventsWindow(param) ?? DEFAULT_EVENTS_WINDOW;
}

/**
 * JSON shape of a MapLibre filter expression, kept structural so this module stays free of the
 * maplibre import (which fails to load under the node test environment).
 */
export type EventsFilterExpression = [
  "all",
  ["<=", ["get", string], number],
  [">=", ["get", string], number],
];

/**
 * Style filter keeping the events that overlap the selected window: "now" keeps currently-running
 * events, "24h" additionally keeps those starting within the next 24 hours. Compares against the
 * `*_epoch` second-precision properties the `events_active` view bakes into the tiles - their
 * `timestamptz` siblings render as session-timezone text, which an expression cannot compare.
 */
export function eventsWindowFilter(window: EventsWindow, nowMs: number): EventsFilterExpression {
  const nowSeconds = Math.floor(nowMs / 1000);
  const latestStart = window === "now" ? nowSeconds : nowSeconds + 24 * 3600;
  return [
    "all",
    ["<=", ["get", "starts_at_epoch"], latestStart],
    [">=", ["get", "ends_at_epoch"], nowSeconds],
  ];
}

// The WC attribute parameters are namespaced per layer (`wcs_…`), so further layers can add
// their own sub-filters without colliding.
export const WCS_WHEELCHAIR_QUERY_PARAM = "wcs_wheelchair";
export const WCS_GENDER_QUERY_PARAM = "wcs_gender";
export const WCS_GENDERS = ["male", "female", "unisex"] as const;
export type WcsGender = (typeof WCS_GENDERS)[number];

/** The tile property carrying each gender's flag. */
const WCS_GENDER_FLAG = {
  male: "is_male_toilet",
  female: "is_female_toilet",
  unisex: "is_unisex_toilet",
} as const satisfies Record<WcsGender, string>;

/**
 * Per-feature predicate selecting a gender. A toilet flagged both male and female is all-gender in
 * practice, so the unisex selection also matches it - not just toilets carrying the explicit
 * `unisex` flag.
 */
function wcsGenderCondition(gender: WcsGender): JsonExpression {
  if (gender === "unisex")
    return [
      "any",
      ["get", "is_unisex_toilet"],
      ["all", ["get", "is_male_toilet"], ["get", "is_female_toilet"]],
    ];
  return ["get", WCS_GENDER_FLAG[gender]];
}

/** Parse a `?wcs_gender=` value, or `null` when absent or not a known gender. */
export function parseWcsGender(param: string | null | undefined): WcsGender | null {
  const gender = WCS_GENDERS.find((g) => g === param);
  return gender ?? null;
}

/** Parse a `?wcs_wheelchair=` value; only the literal "true" enables the filter. */
export function parseWcsWheelchair(param: string | null | undefined): boolean {
  return param === "true";
}

/**
 * JSON shape of a MapLibre expression, kept structural so this module stays free of the maplibre
 * import (which fails to load under the node test environment).
 */
export type JsonExpression = readonly (string | number | boolean | JsonExpression)[];

/**
 * Per-feature `["get", flag]` predicates for each WC attribute the user selected. A WC feature
 * "matches" when every returned predicate is truthy on it. Empty when nothing is selected.
 */
export function wcsAttributeConditions(opts: {
  wheelchair: boolean;
  gender: WcsGender | null;
}): JsonExpression[] {
  const conditions: JsonExpression[] = [];
  if (opts.wheelchair) conditions.push(["get", "is_wheelchair_toilet"]);
  if (opts.gender) conditions.push(wcsGenderCondition(opts.gender));
  return conditions;
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
