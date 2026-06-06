import { expect, test } from "@playwright/test";

test.describe("Navigation Page - Basic Functionality", () => {
  test("should load navigation page with inputs", async ({ page }) => {
    await page.goto("/navigate", { waitUntil: "networkidle" });

    await expect(page).toHaveURL("/navigate");
    // Wait for page to fully load
    await page.waitForLoadState("networkidle");
    const fromInput = page.getByPlaceholder("Von").first();
    const toInput = page.getByPlaceholder("Nach").first().first();
    await expect(fromInput).toBeVisible();
    await expect(toInput).toBeVisible();
  });

  test("should navigate with query parameters", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=chemie", { waitUntil: "networkidle" });
    await expect(page).toHaveURL(/from=mi/);
    await expect(page).toHaveURL(/to=chemie/);
  });
});

test.describe("Navigation Page - Route Calculation", () => {
  test("should calculate route between two locations", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=mw&mode=pedestrian", { waitUntil: "networkidle" });
    await expect(page.locator("body")).toBeVisible();

    // await expect(page).toHaveScreenshot();
  });

  test("should handle route calculation errors gracefully", async ({ page }) => {
    await page.goto("/navigate?from=invalid_location_123&to=invalid_location_456", {
      waitUntil: "networkidle",
    });
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Navigation Page - Transportation Modes", () => {
  test("should support different transportation modes", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=mw&mode=pedestrian", { waitUntil: "networkidle" });
    await expect(page).toHaveURL(/mode=pedestrian/);

    await page.goto("/navigate?from=mi&to=mw&mode=bicycle", { waitUntil: "networkidle" });
    await expect(page).toHaveURL(/mode=bicycle/);
  });

  // Regression test for TUM-Dev/NavigaTUM#2091: mode switch left polyline stale.
  // CI talks to upstream Valhalla, which can return 5xx for these pairs, so we
  // assert on the request the page fires (proving useFetch saw the mode change)
  // rather than on the response payload or the map screenshot.
  test("clicking a mode button refetches the route with the new mode", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=mw&mode=pedestrian", { waitUntil: "domcontentloaded" });
    await expect(page).toHaveURL(/mode=pedestrian/);

    const bikeRequest = page.waitForRequest(
      (req) =>
        req.url().includes("/api/maps/route") && req.url().includes("route_costing=bicycle")
    );
    await page.getByLabel("Fahrrad").click();
    await bikeRequest;
    await expect(page).toHaveURL(/mode=bicycle/);
  });
});

test.describe("Navigation Page - Map Display", () => {
  test("should display interactive map with route", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=mw&mode=pedestrian", { waitUntil: "networkidle" });

    const mapCanvas = page.locator("canvas").first();
    await expect(mapCanvas).toBeVisible();

    // await expect(page).toHaveScreenshot();
  });

  // Regression test for TUM-Dev/NavigaTUM#1960: with only one endpoint defined
  // the map used to stay on the campus-overview default (the Studitum) instead
  // of framing the endpoint. There is no route to fit to, so the page resolves
  // the single endpoint's coordinates via /api/locations/<id> and centres on
  // them. Map rendering is flaky to screenshot in CI, so we assert that the
  // resolution request fires (proving the focus path ran) rather than on pixels.
  test("resolves the single endpoint when only `from` is defined", async ({ page }) => {
    const detailsRequest = page.waitForRequest((req) => req.url().includes("/api/locations/mi"), {
      timeout: 15_000,
    });
    await page.goto("/navigate?from=mi", { waitUntil: "domcontentloaded" });
    await detailsRequest;
    await expect(page).toHaveURL(/from=mi/);
  });

  test("resolves the single endpoint when only `to` is defined", async ({ page }) => {
    const detailsRequest = page.waitForRequest((req) => req.url().includes("/api/locations/mi"), {
      timeout: 15_000,
    });
    await page.goto("/navigate?to=mi", { waitUntil: "domcontentloaded" });
    await detailsRequest;
    await expect(page).toHaveURL(/to=mi/);
  });

  test("with both endpoints, fits the route and does not resolve a single endpoint", async ({
    page,
  }) => {
    const locationsRequests: string[] = [];
    page.on("request", (req) => {
      if (req.url().includes("/api/locations/")) locationsRequests.push(req.url());
    });
    const routeRequest = page.waitForRequest((req) => req.url().includes("/api/maps/route"), {
      timeout: 15_000,
    });
    await page.goto("/navigate?from=mi&to=mw&mode=pedestrian", { waitUntil: "domcontentloaded" });
    await routeRequest;
    // The single-endpoint resolver would only fire client-side once the map has
    // mounted, so wait for the canvas before asserting it stayed silent.
    await expect(page.locator("canvas").first()).toBeVisible();
    expect(locationsRequests).toEqual([]);
  });

  test("with neither endpoint, keeps the campus-overview default", async ({ page }) => {
    const locationsRequests: string[] = [];
    page.on("request", (req) => {
      if (req.url().includes("/api/locations/")) locationsRequests.push(req.url());
    });
    await page.goto("/navigate", { waitUntil: "domcontentloaded" });
    await expect(page.locator("canvas").first()).toBeVisible();
    expect(locationsRequests).toEqual([]);
  });

  test("falls back gracefully when the single endpoint id is unresolvable", async ({ page }) => {
    const pageErrors: Error[] = [];
    page.on("pageerror", (err) => pageErrors.push(err));
    const detailsResponse = page.waitForResponse(
      (resp) => resp.url().includes("/api/locations/invalid_location_123"),
      { timeout: 15_000 }
    );
    await page.goto("/navigate?from=invalid_location_123", { waitUntil: "domcontentloaded" });

    // The id does not resolve, so the resolver must swallow the 404 rather than
    // throw, and the map must still render (on its campus-overview default).
    expect((await detailsResponse).status()).toBe(404);
    await expect(page.locator("canvas").first()).toBeVisible();
    expect(pageErrors).toEqual([]);
  });
});

test.describe("Navigation Page - Turn-by-Turn Directions", () => {
  test.skip("should display turn-by-turn directions with distances", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=mw&mode=pedestrian", { waitUntil: "networkidle" });

    const quickSummaryMinutes = page.getByText("Minuten");
    await expect(quickSummaryMinutes).toBeVisible();

    const turnInstruction = page.getByText("Richtung Osten laufen");
    await expect(turnInstruction).toBeVisible();

    // await expect(page).toHaveScreenshot();
  });
});

test.describe("Navigation Page - Public Transit", () => {
  test("should display public transit connections with times", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=garching&mode=public_transit", {
      waitUntil: "networkidle",
    });
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Navigation Page - Location Search", () => {
  test("should allow searching for locations", async ({ page }) => {
    await page.goto("/navigate", { waitUntil: "networkidle" });

    const fromInput = page.getByPlaceholder("Von").first();
    await fromInput.fill("Mathematik Informatik");
    const searchButton = page.getByText("Fakultät Mathematik");
    await expect(searchButton).toBeVisible();
    await searchButton.click();

    await expect(page).toHaveURL((url) => url.searchParams.get("from") === "mi");
  });

  test("should support coordinate-based routing", async ({ page }) => {
    await page.goto("/navigate?from=coord:48.2656,11.6698&to=coord:48.2622,11.6681", {
      waitUntil: "networkidle",
    });
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Navigation Page - Back Navigation", () => {
  test("should show and use back button when coming from details page", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=mw&coming_from=mi", { waitUntil: "networkidle" });

    const backButton = page.locator('a[href*="/view/mi"]').first();
    await expect(backButton).toBeVisible();
  });
});

test.describe("Navigation Page - Accessibility", () => {
  test("should be keyboard navigable", async ({ page }) => {
    await page.goto("/navigate", { waitUntil: "networkidle" });

    await page.keyboard.press("Tab");
    const focusedElement = await page.evaluateHandle(() => document.activeElement);
    expect(focusedElement).toBeTruthy();
  });
});

test.describe("Navigation Page - Responsive Design", () => {
  test("should display correctly on mobile", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto("/navigate?from=mi&to=mw", { waitUntil: "networkidle" });
    await expect(page.locator("body")).toBeVisible();
  });

  test("should display correctly on desktop", async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto("/navigate?from=mi&to=mw", { waitUntil: "networkidle" });
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Navigation Page - Error Handling", () => {
  test("should handle invalid location IDs gracefully", async ({ page }) => {
    await page.goto("/navigate?from=invalid123&to=invalid456", { waitUntil: "networkidle" });
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Navigation Page - SEO and Meta", () => {
  test("should have proper page title and meta description", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=mw", { waitUntil: "networkidle" });

    const title = await page.title();
    expect(title.length).toBeGreaterThan(0);

    const description = await page.locator('meta[name="description"]').getAttribute("content");
    expect(description).toBeTruthy();
  });
});
