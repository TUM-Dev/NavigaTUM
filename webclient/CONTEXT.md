# Webclient

Glossary for the Nuxt 4 / Vue 3 frontend.

## Language

**Browse map**: `/map`. Category-filtered, no entity identity.
_Avoid_: "the map page".

**Detail map**: `DetailsInteractiveMap` on `/{type}/{id}`. Anchored on one Entity's coords.
_Avoid_: "interactive map" alone.

**Entity**: routable thing with a detail page (campus, site, area, building, joined building, room, virtual room, POI). Resolved via `entityPath()`.
_Avoid_: "item", "result".

**Category**: a class of Entities exposed as a Browse-map filter. Defined in `FILTER_REGISTRY` (`composables/mapLayers.ts`).
_Avoid_: "type" - `type` is the API entity discriminator.

**Address result**: Nominatim hit in the `addresses` search section, an `AddressEntry` over the wire. Not an Entity (no detail page); carries an open Nominatim `addresstype`, not the closed entity `type`.

**Category bridge**: inline link on a detail page whose Entity has a Category → `/map?filter=<id>`.

**Explore-here bridge**: inline link in a detail-page listing section (rooms-of-building, …) → `/map#zoom/lat/lng` from the parent's coords.

**Category shortcut**: top entry in search results when a query token matches a Category's keyword list → `/map?filter=<id>`. Keyword lists are broad: synonyms + misspellings, both languages.

## Relationships

- Entity ↔ Detail map: at most one.
- Entity ↔ Category: zero or one.
- Browse-map filter selects by Category, never by Entity.
- Bridges and shortcut all use `DetailsNearbyTransportSection.vue`'s inline-pill pattern; no page-level CTAs.
