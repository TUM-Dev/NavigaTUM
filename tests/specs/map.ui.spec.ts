import { expect, type Page, test } from "@playwright/test";

// The browse map pulls its style from the production Martin tileserver, whose POI data drifts
// and is occasionally unavailable. Stub the style so the tests exercise our page (panel, URL
// state, popup) rather than live tiles. An empty style still fires `load`, so the controls,
// panel, and zoom-driven hint all render.
const EMPTY_STYLE = { version: 8, sources: {}, layers: [] };

// Garching centroid - the page's default center, so a feature placed here lands at canvas center.
const CENTER: [number, number] = [11.670099, 48.266921];

// A style carrying a single clickable `indoor-toilets` feature. The page wires its popup click
// handler to that layer id, so this deterministically drives the popup without live data. A
// `circle` layer needs no sprite to be clickable, unlike the real `symbol` icon layer.
const STYLE_WITH_TOILET = {
  version: 8,
  sources: {
    "test-pois": {
      type: "geojson",
      data: {
        type: "FeatureCollection",
        features: [
          {
            type: "Feature",
            properties: { indoor: "toilet", is_male_toilet: true, is_wheelchair_toilet: true },
            geometry: { type: "Point", coordinates: CENTER },
          },
        ],
      },
    },
  },
  layers: [
    { id: "indoor-toilets", type: "circle", source: "test-pois", paint: { "circle-radius": 24 } },
  ],
};

async function stubBasemap(page: Page, style: object): Promise<void> {
  await page.route("https://nav.tum.de/martin/style/navigatum-basemap.json", (route) =>
    route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(style) })
  );
}

test.describe("Browse map (/map)", () => {
  test("loads with the layer panel and WCs enabled by default", async ({ page }) => {
    await stubBasemap(page, EMPTY_STYLE);
    await page.goto("/map", { waitUntil: "networkidle" });

    await expect(page.getByRole("region", { name: "Map" })).toBeVisible();

    const panel = page.getByRole("region", { name: "Ebenen" });
    await expect(panel).toBeVisible();

    const wcs = page.getByRole("checkbox", { name: "Toiletten & Duschen" });
    await expect(wcs).toBeChecked();
    // The default selection is reflected into the URL so deep links round-trip.
    await expect(page).toHaveURL(/[?&]layers=wcs/);
  });

  test("toggling WCs flips the checkbox and the ?layers= query", async ({ page }) => {
    await stubBasemap(page, EMPTY_STYLE);
    await page.goto("/map", { waitUntil: "networkidle" });

    const wcs = page.getByRole("checkbox", { name: "Toiletten & Duschen" });
    await expect(wcs).toBeChecked();

    await wcs.uncheck();
    await expect(wcs).not.toBeChecked();
    // An explicit empty value (not an absent key) so an "all off" state survives a reload.
    await expect(page).toHaveURL(/[?&]layers=(?:#|&|$)/);

    await wcs.check();
    await expect(page).toHaveURL(/[?&]layers=wcs/);
  });

  test("shows a zoom-in hint only below zoom 17", async ({ page }) => {
    await stubBasemap(page, EMPTY_STYLE);

    await page.goto("/map#15/48.2669/11.6701", { waitUntil: "networkidle" });
    await expect(page.getByText(/Hineinzoomen/)).toBeVisible();

    await page.goto("/map#18/48.2669/11.6701", { waitUntil: "networkidle" });
    await expect(page.getByText(/Hineinzoomen/)).toHaveCount(0);
  });

  test("collapsing the panel persists across a reload", async ({ page }) => {
    await stubBasemap(page, EMPTY_STYLE);
    await page.goto("/map", { waitUntil: "networkidle" });

    const wcs = page.getByRole("checkbox", { name: "Toiletten & Duschen" });
    await expect(wcs).toBeVisible();

    await page.getByRole("button", { name: "Ebenen" }).click();
    await expect(wcs).toBeHidden();

    await page.reload({ waitUntil: "networkidle" });
    await expect(page.getByRole("region", { name: "Ebenen" })).toBeVisible();
    await expect(page.getByRole("checkbox", { name: "Toiletten & Duschen" })).toBeHidden();
  });

  test("clicking a toilet marker opens a popup with attributes and an OSM edit link", async ({
    page,
  }) => {
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
});
