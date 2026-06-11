import type { components } from "~/api_types";

/**
 * Entity types that resolve to a type-specific canonical route.
 *
 * Aliases the API's closed `LocationEntryType` union: every `NavigaTUM` entity
 * type is routable. A type added on the server therefore fails to compile in
 * {@link entityPath} (and in the record below) until its route is decided.
 */
export type RoutableEntityType = components["schemas"]["LocationEntryType"];

// `Record` over the union so this list cannot drift from the API: a missing or
// extraneous key is a compile error.
const ROUTABLE_ENTITY_TYPES: Record<RoutableEntityType, true> = {
  campus: true,
  site: true,
  area: true,
  building: true,
  joined_building: true,
  room: true,
  virtual_room: true,
  poi: true,
};

/**
 * Narrows an opaque `type` string to a routable entity type.
 *
 * Search results carry the closed `LocationEntryType` union and no longer need
 * this guard. It remains for the API surfaces that are still stringly typed
 * (e.g. the calendar API, breadcrumb `parent_types` with their synthetic
 * `root`) and for runtime inputs like URL query parameters.
 */
export function isRoutableEntityType(type: string): type is RoutableEntityType {
  return Object.hasOwn(ROUTABLE_ENTITY_TYPES, type);
}

/** A canonical, un-localized in-app entity path. */
export type EntityPath =
  | `/campus/${string}`
  | `/site/${string}`
  | `/building/${string}`
  | `/room/${string}`
  | `/poi/${string}`;

/**
 * Canonical, un-localized in-app path for a routable entity (e.g. `/building/5510`).
 *
 * Mirrors the server's `redirect_url` mapping (`LocationKeyAlias::redirect_exact_match`
 * in `server/src/db/location.rs`) - keep the two in sync. The `type` is the strict
 * routable set so a typo or an unhandled new type fails to type-check rather than
 * silently falling back; callers holding an opaque API string narrow with
 * {@link isRoutableEntityType} first.
 */
export function entityPath(id: string, type: RoutableEntityType): EntityPath {
  switch (type) {
    case "campus":
      return `/campus/${id}`;
    case "site":
    case "area":
      return `/site/${id}`;
    case "building":
    case "joined_building":
      return `/building/${id}`;
    case "room":
    case "virtual_room":
      return `/room/${id}`;
    case "poi":
      return `/poi/${id}`;
    default:
      return type satisfies never;
  }
}
