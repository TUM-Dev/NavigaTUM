import { expect, type Page, test } from "@playwright/test";

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
    is_unisex_toilet: false,
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
function eventFeature(name: string, startsInSeconds: number, endsInSeconds: number) {
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

// A style carrying one event in the `events` layer the page toggles and filters. Mirrors the real
// basemap: the layer ships hidden and only `applyOverlayVisibility` reveals it. A `circle` layer
// needs no sprite to be clickable, unlike the real `symbol` icon layer.
function styleWithEvent(feature: object) {
  return {
    version: 8,
    sources: {
      events_active: {
        type: "geojson",
        data: { type: "FeatureCollection", features: [feature] },
      },
    },
    layers: [
      {
        id: "events",
        type: "circle",
        source: "events_active",
        layout: { visibility: "none" },
        paint: { "circle-radius": 24 },
      },
    ],
  };
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

    const editLink = popup.getByRole("link", { name: "In OpenStreetMap bearbeiten" });
    await expect(editLink).toHaveAttribute(
      "href",
      /openstreetmap\.org\/edit#map=21\/48\.266921\d*\/11\.670099\d*/
    );
  });

  test("toggling Events flips the ?filter= query and reveals the time-window selector", async ({
    page,
  }) => {
    await stubBasemap(page, EMPTY_STYLE);
    await page.goto("/map", { waitUntil: "networkidle" });

    const nowRadio = page.getByRole("radio", { name: "Gerade aktiv" });
    await expect(nowRadio).toBeHidden();

    const events = page.getByRole("checkbox", { name: "Veranstaltungen" });
    await events.check();
    await expect(page).toHaveURL(/[?&]filter=events/);
    await expect(nowRadio).toBeChecked();
    await expect(page.getByRole("radio", { name: "Nächste 24 Stunden" })).not.toBeChecked();

    await events.uncheck();
    await expect(page).not.toHaveURL(/[?&]filter=events/);
    await expect(nowRadio).toBeHidden();
  });

  test("selecting the 24h window sets ?events_window= and survives a reload", async ({ page }) => {
    await stubBasemap(page, EMPTY_STYLE);
    await page.goto("/map?filter=events", { waitUntil: "networkidle" });

    await page.getByRole("radio", { name: "Nächste 24 Stunden" }).check();
    await expect(page).toHaveURL(/[?&]events_window=24h/);

    await page.reload({ waitUntil: "networkidle" });
    await expect(page.getByRole("radio", { name: "Nächste 24 Stunden" })).toBeChecked();

    // Back at the "now" default the param is dropped, keeping the URL clean.
    await page.getByRole("radio", { name: "Gerade aktiv" }).check();
    await expect(page).not.toHaveURL(/[?&]events_window=/);
  });

  test("a running event only shows its popup while the Events layer is enabled", async ({
    page,
  }) => {
    // Running right now: started an hour ago, ends in an hour.
    await stubBasemap(page, styleWithEvent(eventFeature("Sommerfest", -3600, 3600)));

    // Layer disabled: the style ships the events layer hidden, so the click hits nothing.
    await page.goto("/map", { waitUntil: "networkidle" });
    await expect(page.getByRole("region", { name: "Map" })).toBeVisible();
    await page.locator("#map-browse canvas").first().click();
    await expect(page.getByRole("heading", { name: "Sommerfest" })).toBeHidden();

    // Layer enabled: the marker sits at the default center, i.e. where a default click lands.
    await page.goto("/map?filter=events", { waitUntil: "networkidle" });
    await expect(page.getByRole("region", { name: "Map" })).toBeVisible();
    await page.locator("#map-browse canvas").first().click();

    await expect(page.getByRole("heading", { name: "Sommerfest" })).toBeVisible();
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

  test("an event starting in three hours only appears in the 24h window", async ({ page }) => {
    await stubBasemap(page, styleWithEvent(eventFeature("Hackathon", 3 * 3600, 5 * 3600)));

    // "Happening now" (the default): the future event is filtered out, so the click hits nothing.
    await page.goto("/map?filter=events", { waitUntil: "networkidle" });
    await expect(page.getByRole("region", { name: "Map" })).toBeVisible();
    await page.locator("#map-browse canvas").first().click();
    await expect(page.getByRole("heading", { name: "Hackathon" })).toBeHidden();

    // "Next 24 hours": the same event renders and carries its popup.
    await page.goto("/map?filter=events&events_window=24h", { waitUntil: "networkidle" });
    await expect(page.getByRole("region", { name: "Map" })).toBeVisible();
    await page.locator("#map-browse canvas").first().click();
    await expect(page.getByRole("heading", { name: "Hackathon" })).toBeVisible();
  });
});
