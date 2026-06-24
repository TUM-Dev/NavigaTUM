import { expect, type Page, test } from "@playwright/test";
import type { Feature, FeatureCollection, Point } from "geojson";
import geojsonvt from "geojson-vt";
import vtpbf from "vt-pbf";

// The browse map pulls its style from the production Martin tileserver, whose POI data drifts
// and is occasionally unavailable. Stub the style so the tests exercise our page (panel, URL
// state, popup) rather than live tiles. An empty style still fires `load`, so the controls,
// panel, and zoom-driven hint all render.
const EMPTY_STYLE = { version: 8, sources: {}, layers: [] };

// Garching centroid - the page's default center, so a feature placed here lands at canvas center.
const CENTER: [number, number] = [11.670099, 48.266921];

// A style carrying a single clickable toilet in the `indoor-pois` layer the page wires its popup
// handler to, so this deterministically drives the popup without live data. A `circle` layer
// needs no sprite to be clickable, unlike the real `symbol` icon layer.
function styleWithToilet(flags: Record<string, boolean>) {
  const allFlags = {
    is_male_toilet: false,
    is_female_toilet: false,
    is_wheelchair_toilet: false,
    is_shower: false,
    ...flags,
  };
  return {
    version: 8,
    sources: {
      "test-pois": {
        type: "geojson",
        data: {
          type: "FeatureCollection",
          features: [
            {
              type: "Feature",
              properties: { indoor: "toilet", ...allFlags },
              geometry: { type: "Point", coordinates: CENTER },
            },
          ],
        },
      },
    },
    layers: [
      { id: "indoor-pois", type: "circle", source: "test-pois", paint: { "circle-radius": 24 } },
    ],
  };
}

const STYLE_WITH_TOILET = styleWithToilet({ is_male_toilet: true, is_wheelchair_toilet: true });

async function stubBasemap(page: Page, style: object): Promise<void> {
  await page.route("https://nav.tum.de/martin/style/navigatum-basemap.json", (route) =>
    route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(style) })
  );
}

// An event feature shaped like the `events_active` tiles: display strings plus the epoch-second
// properties the page's time-window filter compares against.
function eventFeature(
  name: string,
  startsInSeconds: number,
  endsInSeconds: number
): Feature<Point> {
  const nowSeconds = Math.floor(Date.now() / 1000);
  const startsAtEpoch = nowSeconds + startsInSeconds;
  const endsAtEpoch = nowSeconds + endsInSeconds;
  return {
    type: "Feature",
    id: 1,
    properties: {
      name,
      description: "Eine Testveranstaltung.",
      image: "/cdn/thumb/test.webp",
      image_author: "Test Author",
      starts_at: new Date(startsAtEpoch * 1000).toISOString(),
      ends_at: new Date(endsAtEpoch * 1000).toISOString(),
      organising_org_id: 1,
      organising_org_code: "TUTEST",
      organising_org_name_de: "Lehrstuhl für Tests",
      organising_org_name_en: "Chair of Testing",
      starts_at_epoch: startsAtEpoch,
      ends_at_epoch: endsAtEpoch,
    },
    geometry: { type: "Point", coordinates: CENTER },
  };
}

type EventFeed = "events_active" | "events_upcoming";

// `useEventMarkers` owns the event layers now: it adds the two feed sources as Martin *vector*
// sources, one symbol layer each, and toggles their visibility with the window. The radio/URL tests
// don't need a rendered marker, so they hand it empty geojson feed sources, which the composable
// finds already present and so never reaches for the live vector tiles.
function emptyEventStyle() {
  const empty = { type: "geojson", data: { type: "FeatureCollection", features: [] } } as const;
  return { version: 8, sources: { events_active: empty, events_upcoming: empty }, layers: [] };
}

// The popup tests need a clickable marker, which means feeding the composable's vector layers real
// tiles (a geojson source can't satisfy their `source-layer`). This basemap carries a `glyphs` URL
// the symbol layer's label needs to lay out; `stubEventFeed` and `stubGlyphs` supply the rest.
//
// The feeds are declared here as `encoding: "mvt"` so `useEventMarkers` reuses them rather than
// adding its own MLT sources, and so `loadBasemapStyle` (which only defaults *unset* encodings to
// MLT) leaves them be: no JS MLT encoder exists to stub MLT tiles, so the marker→popup wiring is
// exercised over MVT while the MLT decode path is covered by `eventsExpiryFilter`'s unit test.
function vectorFeedSource(feed: EventFeed) {
  return { type: "vector", url: `https://nav.tum.de/martin/${feed}`, encoding: "mvt" } as const;
}
const VECTOR_BASEMAP = {
  version: 8,
  glyphs: "https://nav.tum.de/fonts/{fontstack}/{range}.pbf",
  sources: {
    events_active: vectorFeedSource("events_active"),
    events_upcoming: vectorFeedSource("events_upcoming"),
  },
  layers: [],
};

// Markers are per-event photo sprites: the composable rasterises the CDN image onto a canvas, so
// the route must answer with a CORS-clean bitmap or the icon never registers and the symbol has
// nothing to click. A 1x1 PNG suffices - the icon only needs to place at the feature's point. Match
// by path: the CDN host is config-driven (nav.tum.de in dev, localhost in the e2e compose).
const MARKER_PNG = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==",
  "base64"
);
async function stubEventImage(page: Page): Promise<void> {
  await page.route(/\/cdn\/thumb\/test\.webp$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: "image/png",
      headers: { "Access-Control-Allow-Origin": "*" },
      body: MARKER_PNG,
    })
  );
}

// The label glyphs only have to resolve, not render: an empty glyph range lets the symbol lay out
// (its text is `text-optional`) so the icon places and the marker becomes clickable.
async function stubGlyphs(page: Page): Promise<void> {
  await page.route(/\/fonts\/.*\.pbf$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: "application/x-protobuf",
      headers: { "Access-Control-Allow-Origin": "*" },
      body: Buffer.alloc(0),
    })
  );
}

// Serve one event feed as Martin-shaped vector tiles: a TileJSON the composable's vector source
// points at, plus per-tile MVT carrying the feature in a `source-layer` named after the feed (the
// name the composable's symbol layer queries). This exercises the real production tile path.
function stubEventFeed(page: Page, feed: EventFeed, features: Feature[]): Promise<void> {
  const collection: FeatureCollection = { type: "FeatureCollection", features };
  const index = geojsonvt(collection, { maxZoom: 24, buffer: 64 });
  const base = `https://nav.tum.de/martin/${feed}`;
  const corsJson = { "Access-Control-Allow-Origin": "*" };
  page.route(
    (url) => url.href === base,
    (route) =>
      route.fulfill({
        status: 200,
        contentType: "application/json",
        headers: corsJson,
        body: JSON.stringify({
          tilejson: "3.0.0",
          tiles: [`${base}/{z}/{x}/{y}`],
          minzoom: 15,
          maxzoom: 24,
          vector_layers: [{ id: feed }],
        }),
      })
  );
  return page.route(
    (url) => url.href.startsWith(`${base}/`),
    (route) => {
      const tilePath = route.request().url().slice(`${base}/`.length);
      const [z, x, y] = tilePath.split("/").map((n) => Number.parseInt(n, 10));
      const tile = index.getTile(z, x, y);
      if (!tile || tile.features.length === 0) {
        return route.fulfill({ status: 204, headers: corsJson });
      }
      const body = Buffer.from(vtpbf.fromGeojsonVt({ [feed]: tile }, { version: 2 }));
      return route.fulfill({
        status: 200,
        contentType: "application/x-protobuf",
        headers: corsJson,
        body,
      });
    }
  );
}

// Stand up both feeds (one usually empty) plus the glyph and image routes the markers need.
async function stubEvents(
  page: Page,
  feeds: { active?: Feature[]; upcoming?: Feature[] }
): Promise<void> {
  await stubGlyphs(page);
  await stubEventImage(page);
  await stubBasemap(page, VECTOR_BASEMAP);
  await stubEventFeed(page, "events_active", feeds.active ?? []);
  await stubEventFeed(page, "events_upcoming", feeds.upcoming ?? []);
}

// Clicking a canvas marker only lands once its tile and sprite have loaded, so retry the click
// until the popup opens rather than racing the async tile fetch and image registration.
async function clickEventMarker(page: Page, heading: string): Promise<void> {
  await expect(async () => {
    await page.locator("#map-browse canvas").first().click();
    await expect(page.getByRole("heading", { name: heading })).toBeVisible({ timeout: 1000 });
  }).toPass({ timeout: 15000 });
}

test.describe("Browse map (/map)", () => {
  // MapLibre GL v6 loads `maplibre-gl-worker.mjs` via `new URL('./...', import.meta.url)`,
  // a pattern Vite/Rollup cannot statically detect; without an explicit `setWorkerUrl()` the
  // asset is never emitted and the worker fetch 404s, leaving the map without tiles.
  test("loads its WebWorker without a 404", async ({ page }) => {
    const failures: string[] = [];
    page.on("requestfailed", (req) => {
      if (req.url().includes("maplibre-gl-worker")) failures.push(req.url());
    });
    await stubBasemap(page, EMPTY_STYLE);
    await page.goto("/map", { waitUntil: "networkidle" });

    await expect(page.getByRole("region", { name: "Map" })).toBeVisible();
    expect(failures, `maplibre worker fetch failed: ${failures.join(", ")}`).toEqual([]);
  });

  test("loads with the filter panel and no filter active by default", async ({ page }) => {
    await stubBasemap(page, EMPTY_STYLE);
    await page.goto("/map", { waitUntil: "networkidle" });

    await expect(page.getByRole("region", { name: "Map" })).toBeVisible();

    const panel = page.getByRole("region", { name: "Filter" });
    await expect(panel).toBeVisible();

    const wcs = page.getByRole("checkbox", { name: "Toiletten & Duschen" });
    await expect(wcs).not.toBeChecked();
    await expect(page).not.toHaveURL(/[?&]filter=wcs/);
  });

  test("toggling WCs flips the checkbox and the ?filter= query", async ({ page }) => {
    await stubBasemap(page, EMPTY_STYLE);
    await page.goto("/map", { waitUntil: "networkidle" });

    const wcs = page.getByRole("checkbox", { name: "Toiletten & Duschen" });
    await wcs.check();
    await expect(page).toHaveURL(/[?&]filter=wcs/);

    await wcs.uncheck();
    await expect(wcs).not.toBeChecked();
    await expect(page).not.toHaveURL(/[?&]filter=wcs/);
  });

  test("shows a zoom-in hint only below zoom 17 while the filter is active", async ({ page }) => {
    await stubBasemap(page, EMPTY_STYLE);

    await page.goto("/map?filter=wcs#15/48.2669/11.6701", { waitUntil: "networkidle" });
    await expect(page.getByText(/Hineinzoomen/)).toBeVisible();

    await page.goto("/map?filter=wcs#18/48.2669/11.6701", { waitUntil: "networkidle" });
    await expect(page.getByText(/Hineinzoomen/)).toHaveCount(0);
  });

  test("collapsing the panel persists across a reload", async ({ page }) => {
    await stubBasemap(page, EMPTY_STYLE);
    await page.goto("/map", { waitUntil: "networkidle" });

    const wcs = page.getByRole("checkbox", { name: "Toiletten & Duschen" });
    await expect(wcs).toBeVisible();

    await page.getByRole("button", { name: "Filter" }).click();
    await expect(wcs).toBeHidden();

    await page.reload({ waitUntil: "networkidle" });
    await expect(page.getByRole("region", { name: "Filter" })).toBeVisible();
    await expect(page.getByRole("checkbox", { name: "Toiletten & Duschen" })).toBeHidden();
  });

  test("clicking a toilet opens a popup with attributes and an OSM edit link", async ({ page }) => {
    await stubBasemap(page, STYLE_WITH_TOILET);
    await page.goto("/map", { waitUntil: "networkidle" });
    await expect(page.getByRole("region", { name: "Map" })).toBeVisible();

    // The feature sits at the default center, i.e. the canvas centre, which is what a default
    // click targets.
    await page.locator("#map-browse canvas").first().click();

    const popup = page.locator(".maplibregl-popup-content");
    await expect(popup).toBeVisible();
    await expect(popup).toContainText("Toilette");
    await expect(popup).toContainText("Herren");
    await expect(popup).toContainText("Rollstuhlgerecht");

    // The indoor popup links to the OSM indoor editor (osminedit), anchored on the click point.
    const editLink = popup.getByRole("link", { name: "In OpenStreetMap bearbeiten" });
    await expect(editLink).toHaveAttribute(
      "href",
      /osminedit\.pavie\.info\/#\d+\/48\.266921\d*\/11\.670099\d*\/\d+/
    );
  });

  test("toggling Events flips the ?filter= query and reveals the time-window selector", async ({
    page,
  }) => {
    await stubBasemap(page, emptyEventStyle());
    await page.goto("/map", { waitUntil: "networkidle" });

    const nowRadio = page.getByRole("radio", { name: "Gerade aktiv" });
    await expect(nowRadio).toBeHidden();

    const events = page.getByRole("checkbox", { name: "Veranstaltungen" });
    await events.check();
    await expect(page).toHaveURL(/[?&]filter=events/);
    await expect(nowRadio).toBeChecked();
    await expect(page.getByRole("radio", { name: "Nächste 2 Wochen" })).not.toBeChecked();

    await events.uncheck();
    await expect(page).not.toHaveURL(/[?&]filter=events/);
    await expect(nowRadio).toBeHidden();
  });

  test("selecting the 2-week window sets ?events_window= and survives a reload", async ({
    page,
  }) => {
    await stubBasemap(page, emptyEventStyle());
    await page.goto("/map?filter=events", { waitUntil: "networkidle" });

    await page.getByRole("radio", { name: "Nächste 2 Wochen" }).check();
    await expect(page).toHaveURL(/[?&]events_window=2weeks/);

    await page.reload({ waitUntil: "networkidle" });
    await expect(page.getByRole("radio", { name: "Nächste 2 Wochen" })).toBeChecked();

    // Back at the "now" default the param is dropped, keeping the URL clean.
    await page.getByRole("radio", { name: "Gerade aktiv" }).check();
    await expect(page).not.toHaveURL(/[?&]events_window=/);
  });

  test("a running event only shows its popup while the Events layer is enabled", async ({
    page,
  }) => {
    // Running right now: started an hour ago, ends in an hour, served on the active feed.
    await stubEvents(page, { active: [eventFeature("Sommerfest", -3600, 3600)] });

    // Filter off: both feed layers stay hidden, so the click hits nothing.
    await page.goto("/map", { waitUntil: "networkidle" });
    await expect(page.getByRole("region", { name: "Map" })).toBeVisible();
    await page.locator("#map-browse canvas").first().click();
    await expect(page.getByRole("heading", { name: "Sommerfest" })).toBeHidden();

    // Filter on: the active feed is shown and its marker sits at the default center, where the click lands.
    await page.goto("/map?filter=events", { waitUntil: "networkidle" });
    await expect(page.getByRole("region", { name: "Map" })).toBeVisible();
    await clickEventMarker(page, "Sommerfest");

    await expect(page.getByText("Lehrstuhl für Tests")).toBeVisible();
    const orgLink = page.getByRole("link", { name: /Veranstalter 'Lehrstuhl für Tests'/ });
    await expect(orgLink).toHaveAttribute("href", /\/view\/TUTEST/);

    await page.getByRole("button", { name: "Veranstaltungsdetails schließen" }).click();
    await expect(page.getByRole("heading", { name: "Sommerfest" })).toBeHidden();
  });

  test("enabling WCs reveals the attribute filters, which round-trip through the URL", async ({
    page,
  }) => {
    await stubBasemap(page, EMPTY_STYLE);
    await page.goto("/map", { waitUntil: "networkidle" });

    const wheelchair = page.getByRole("checkbox", { name: "Nur rollstuhlgerecht" });
    await expect(wheelchair).toBeHidden();

    const wcs = page.getByRole("checkbox", { name: "Toiletten & Duschen" });
    await wcs.check();
    await expect(wheelchair).toBeVisible();
    await expect(page.getByRole("radio", { name: "Alle Geschlechter" })).toBeChecked();

    await wheelchair.check();
    await expect(page).toHaveURL(/[?&]wcs_wheelchair=true/);
    await page.getByRole("radio", { name: "Herren" }).check();
    await expect(page).toHaveURL(/[?&]wcs_gender=male/);

    await page.reload({ waitUntil: "networkidle" });
    await expect(page.getByRole("checkbox", { name: "Nur rollstuhlgerecht" })).toBeChecked();
    await expect(page.getByRole("radio", { name: "Herren" })).toBeChecked();

    await page.getByRole("checkbox", { name: "Nur rollstuhlgerecht" }).uncheck();
    await page.getByRole("radio", { name: "Alle Geschlechter" }).check();
    await expect(page).not.toHaveURL(/[?&]wcs_(wheelchair|gender)=/);

    await page.getByRole("checkbox", { name: "Toiletten & Duschen" }).uncheck();
    await expect(page.getByRole("checkbox", { name: "Nur rollstuhlgerecht" })).toBeHidden();
  });

  test("a non-matching toilet stays clickable under the wheelchair-only filter", async ({
    page,
  }) => {
    await stubBasemap(page, styleWithToilet({ is_male_toilet: true }));

    await page.goto("/map?filter=wcs&wcs_wheelchair=true", { waitUntil: "networkidle" });
    await expect(page.getByRole("region", { name: "Map" })).toBeVisible();
    await page.locator("#map-browse canvas").first().click();
    await expect(page.locator(".maplibregl-popup-content")).toContainText("Toilette");
  });

  test("a non-matching toilet stays clickable under the gender filter", async ({ page }) => {
    await stubBasemap(page, styleWithToilet({ is_female_toilet: true }));

    await page.goto("/map?filter=wcs&wcs_gender=male", { waitUntil: "networkidle" });
    await expect(page.getByRole("region", { name: "Map" })).toBeVisible();
    await page.locator("#map-browse canvas").first().click();
    await expect(page.locator(".maplibregl-popup-content")).toContainText("Damen");
  });

  test("an upcoming event only appears in the 2-week window", async ({ page }) => {
    // Starts in three hours: it rides the upcoming feed, not the active one.
    await stubEvents(page, { upcoming: [eventFeature("Hackathon", 3 * 3600, 5 * 3600)] });

    // "Happening now" (the default) shows only the active feed, so the click hits nothing.
    await page.goto("/map?filter=events", { waitUntil: "networkidle" });
    await expect(page.getByRole("region", { name: "Map" })).toBeVisible();
    await page.locator("#map-browse canvas").first().click();
    await expect(page.getByRole("heading", { name: "Hackathon" })).toBeHidden();

    // "Next 2 weeks": the upcoming feed is shown and its marker carries the popup.
    await page.goto("/map?filter=events&events_window=2weeks", { waitUntil: "networkidle" });
    await expect(page.getByRole("region", { name: "Map" })).toBeVisible();
    await clickEventMarker(page, "Hackathon");
  });
});
