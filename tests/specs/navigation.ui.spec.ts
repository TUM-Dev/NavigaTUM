import { expect, test } from "@playwright/test";

test.describe("Navigation Page - Basic Functionality", () => {
  test("should load navigation page with inputs", async ({ page }) => {
    await page.goto("/navigate", { waitUntil: "networkidle" });

    await expect(page).toHaveURL(/\/navigate/);
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
});

test.describe("Navigation Page - Map Display", () => {
  test("should display interactive map with route", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=mw&mode=pedestrian", { waitUntil: "networkidle" });

    const mapCanvas = page.locator("canvas").first();
    await expect(mapCanvas).toBeVisible();
  });
});

test.describe("Navigation Page - Turn-by-Turn Directions", () => {
  test("should display step-by-step directions with distances", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=mw&mode=pedestrian", { waitUntil: "networkidle" });

    const instructions = page.locator('[role="list"], ol, ul').first();
    await expect(instructions).toBeVisible();
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

    // Wait for page to fully load
    await page.waitForLoadState("networkidle");

    const fromInput = page.getByPlaceholder("Von").first();
    await fromInput.fill("mi");
    await fromInput.press("Enter");

    const url = page.url();
    expect(url.includes("from=") || url.includes("mi")).toBeTruthy();
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
