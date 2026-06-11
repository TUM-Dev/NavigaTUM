# `/map` URL contract: viewport + filter, no identity

`/map` is `#zoom/lat/lng` (MapLibre hash) + `?filter=` + `?level=`. No `focus_id`, no entity id. `/map` is the stateless explore surface; identity belongs to `/{type}/{id}`. Bridges from detail pages already encode the parent Entity's coords client-side, so `/map` makes no API call on load. A `?focus=<id>` resolving to a NavigaTUM id can be added later additively if highlights become wanted - POI markers carry their NavigaTUM id via the OSM `ref:tum` tag, not yet propagated through `style.lua` into the MVT.
