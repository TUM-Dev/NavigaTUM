import type { components } from "~/api_types";

/** The API's closed entity-type union. Every entity type has a canonical route. */
export type EntityType = components["schemas"]["LocationEntryType"];

// Runtime mirror of `EntityType` for narrowing opaque strings.
const ENTITY_TYPES = [
  "campus",
  "site",
  "area",
  "building",
  "joined_building",
  "room",
  "virtual_room",
  "poi",
] as const satisfies readonly EntityType[];

/**
 * Narrows an opaque `type` string (the calendar API, breadcrumb `parent_types`
 * with their synthetic `root`, URL query parameters) to an entity type.
 */
export function isEntityType(type: string): type is EntityType {
  return (ENTITY_TYPES as readonly string[]).includes(type);
}

/** A canonical, un-localized in-app entity path. */
export type EntityPath =
  | `/campus/${string}`
  | `/site/${string}`
  | `/building/${string}`
  | `/room/${string}`
  | `/poi/${string}`;

/**
 * Canonical, un-localized in-app path for an entity (e.g. `/building/5510`).
 *
 * Mirrors the server's `redirect_url` mapping (`LocationKeyAlias::redirect_exact_match`
 * in `server/src/db/location.rs`) - keep the two in sync. The `type` is the closed
 * union so a typo or an unhandled new type fails to type-check rather than
 * silently falling back; callers holding an opaque string narrow with
 * {@link isEntityType} first.
 */
export function entityPath(id: string, type: EntityType): EntityPath {
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
