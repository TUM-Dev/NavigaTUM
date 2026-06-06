/** Entity types that resolve to a type-specific canonical route. */
export const ROUTABLE_ENTITY_TYPES = [
  "campus",
  "site",
  "area",
  "building",
  "joined_building",
  "room",
  "virtual_room",
  "poi",
] as const;

export type RoutableEntityType = (typeof ROUTABLE_ENTITY_TYPES)[number];

/**
 * Narrows an opaque API `type` string to a routable entity type.
 *
 * Search results carry `type` as a bare `string` because address results use a
 * Nominatim `addresstype` (e.g. `road`) that has no canonical entity route. Gate
 * {@link entityPath} on this so non-routable results render without an entity link.
 */
export function isRoutableEntityType(type: string): type is RoutableEntityType {
  return (ROUTABLE_ENTITY_TYPES as readonly string[]).includes(type);
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
