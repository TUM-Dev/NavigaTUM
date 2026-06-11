# `/map` URL contract: viewport + filter, no identity

The Browse map's URL is **MapLibre `#zoom/lat/lng` hash + `?filter=` + `?level=`** — no `focus_id`, no entity identity. Bridges from search and detail pages encode the parent Entity's `coords` directly in the hash, so links are resolvable client-side from data the source page already has. Identity is the detail page's job (`/{type}/{id}`); the Browse map stays a stateless explore surface.

## Considered options

- **Coords-only** (chosen). No new contract; no API call on Browse-map load; sidesteps the OSM-POI-has-no-NavigaTUM-id problem because we never claim identity here.
- **`?focus=<id>`**. Adds a server round-trip and a loading state to a page that is currently zero-API-call, and forces an id-space decision (NavigaTUM / OSM / event). Layerable later as an additive enhancement if highlights become wanted.
- **Hybrid hash + `?focus=`**. Two contracts, marginal benefit while no highlight exists.

## Consequences

- All Browse-map links from elsewhere carry zero identity.
- A future "highlight this entity" feature is an additive change, not a breaking redesign.
- POI markers stay coordinate-anchored OSM features.
