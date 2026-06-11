# Webclient

The Nuxt 4 / Vue 3 frontend. Glossary for terms whose meaning is specific to NavigaTUM's UI vocabulary.

## Language

### Map surfaces

**Browse map**:
The `/map` page. A Category-driven exploration surface, stateless about any one Entity. Owns the filter panel.
_Avoid_: "the map page" — ambiguous against the Detail map.

**Detail map**:
The `DetailsInteractiveMap` embedded on `/{type}/{id}`. Anchored on one Entity's `coords`.
_Avoid_: "interactive map" alone — every map here is interactive.

### Search results

**Entity**:
A routable thing with a detail page; resolved via `entityPath()` (campus, site, area, building, joined building, room, virtual room, POI).
_Avoid_: "item", "result" — both also cover non-routable hits.

**Category**:
A class of Entities exposed as a Browse-map filter. Defined in `FILTER_REGISTRY` (`composables/mapLayers.ts`).
_Avoid_: "type" — `type` is the API entity discriminator, not the filter taxonomy.

**Address result**:
A Nominatim hit in the `addresses` search section. No NavigaTUM id, no detail page; only surfaced on `/navigate`.
_Avoid_: treating it as an Entity.

### Detail ↔ Browse bridges

The detail page never links to the Browse map as a page-level CTA. Three narrow inline-link exceptions exist, all modelled on the "via Transitous" pill in `DetailsNearbyTransportSection.vue`:

**Category bridge**:
Inline link on a detail page whose Entity belongs to a Category. Opens the Browse map with that filter applied; wording is Category-shaped ("Show all toilets on the map").

**Explore-here bridge**:
Inline link in a *listing* section of a detail page (rooms-of-building, buildings-of-area, …). Opens the Browse map framed on the parent Entity's coords.

**Category shortcut**:
Top entry in search results (`AppSearchBar` dropdown and `/search`) when a query token matches a Category's keyword list. Links to the Browse map with that filter applied. Keyword lists are intentionally broad — multilingual synonyms and common misspellings — so the shortcut fires on natural queries.

## Relationships

- An Entity has at most one Detail map; belongs to zero or one Category.
- A Browse-map filter selects by Category, never by Entity identity.
- Category bridge gates on Category membership; Explore-here bridge gates on listed children.

## Example dialogue

> **Dev:** "Should 'Show on map' on a toilet search result open the Browse map or the Detail map?"
> **Domain expert:** "Neither directly — the Detail map is one click away through the row's detail link. The Browse map link from a toilet would be a Category bridge: 'all toilets', not 'this toilet.'"

## Flagged ambiguities

- "map" was used ambiguously between the Browse map and the Detail map in #3162 — resolved: distinct surfaces, distinct intents (explore vs. inspect).
- "Show on map" was used loosely — resolved: it is a Category or Explore-here bridge, never an entity-focus operation.
