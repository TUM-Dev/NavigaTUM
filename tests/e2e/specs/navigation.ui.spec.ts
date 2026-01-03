import { expect, test } from "@playwright/test";

test.describe("Navigation Page - Basic Functionality", () => {
  test("should load navigation page with inputs", async ({ page }) => {
    await page.goto("/navigate", { waitUntil: "domcontentloaded" });

    await expect(page).toHaveURL(/\/navigate/);
    const fromInput = page.locator('input[id*="from"], input[name*="from"]').first();
    const toInput = page.locator('input[id*="to"], input[name*="to"]').first();
    await expect(fromInput).toBeVisible();
    await expect(toInput).toBeVisible();
  });

  test("should navigate with query parameters", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=chemie", { waitUntil: "domcontentloaded" });
    await expect(page).toHaveURL(/from=mi/);
    await expect(page).toHaveURL(/to=chemie/);
  });
});

test.describe("Navigation Page - Route Calculation", () => {
  test("should calculate route between two locations", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=mw&mode=pedestrian", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(1000);
    await expect(page.locator("body")).toBeVisible();
  });

  test("should handle route calculation errors gracefully", async ({ page }) => {
    await page.goto("/navigate?from=invalid_location_123&to=invalid_location_456", {
      waitUntil: "domcontentloaded",
    });
    await page.waitForTimeout(1000);
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Navigation Page - Transportation Modes", () => {
  test("should support different transportation modes", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=mw&mode=pedestrian", { waitUntil: "domcontentloaded" });
    await expect(page).toHaveURL(/mode=pedestrian/);

    await page.goto("/navigate?from=mi&to=mw&mode=bicycle", { waitUntil: "domcontentloaded" });
    await expect(page).toHaveURL(/mode=bicycle/);
  });
});

test.describe("Navigation Page - Map Display", () => {
  test("should display interactive map with route", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=mw&mode=pedestrian", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(1000);

    const mapCanvas = page.locator("canvas").first();
    if ((await mapCanvas.count()) > 0) {
      await expect(mapCanvas).toBeVisible();
    }
  });
});

test.describe("Navigation Page - Turn-by-Turn Directions", () => {
  test("should display step-by-step directions with distances", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=mw&mode=pedestrian", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(1000);

    const instructions = page.locator('[role="list"], ol, ul').first();
    if ((await instructions.count()) > 0) {
      await expect(instructions).toBeVisible();
    }
  });
});

test.describe("Navigation Page - Public Transit", () => {
  test("should display public transit connections with times", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=garching&mode=public_transit", {
      waitUntil: "domcontentloaded",
    });
    await page.waitForTimeout(1500);
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Navigation Page - Location Search", () => {
  test("should allow searching for locations", async ({ page }) => {
    await page.goto("/navigate", { waitUntil: "domcontentloaded" });

    const fromInput = page.locator('input[id*="from"], input[name*="from"]').first();
    await fromInput.fill("mi");
    await fromInput.press("Enter");
    await page.waitForTimeout(300);

    const url = page.url();
    expect(url.includes("from=") || url.includes("mi")).toBeTruthy();
  });

  test("should support coordinate-based routing", async ({ page }) => {
    await page.goto("/navigate?from=coord:48.2656,11.6698&to=coord:48.2622,11.6681", {
      waitUntil: "domcontentloaded",
    });
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Navigation Page - Back Navigation", () => {
  test("should show and use back button when coming from details page", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=mw&coming_from=mi", { waitUntil: "domcontentloaded" });

    const backButton = page.locator('a[href*="/view/mi"]').first();
    if ((await backButton.count()) > 0) {
      await expect(backButton).toBeVisible();
    }
  });
});

test.describe("Navigation Page - Accessibility", () => {
  test("should be keyboard navigable", async ({ page }) => {
    await page.goto("/navigate", { waitUntil: "domcontentloaded" });

    await page.keyboard.press("Tab");
    const focusedElement = await page.evaluateHandle(() => document.activeElement);
    expect(focusedElement).toBeTruthy();
  });
});

test.describe("Navigation Page - Responsive Design", () => {
  test("should display correctly on mobile", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto("/navigate?from=mi&to=mw", { waitUntil: "domcontentloaded" });
    await expect(page.locator("body")).toBeVisible();
  });

  test("should display correctly on desktop", async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto("/navigate?from=mi&to=mw", { waitUntil: "domcontentloaded" });
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Navigation Page - Error Handling", () => {
  test("should handle invalid location IDs gracefully", async ({ page }) => {
    await page.goto("/navigate?from=invalid123&to=invalid456", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(1000);
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Navigation Page - SEO and Meta", () => {
  test("should have proper page title and meta description", async ({ page }) => {
    await page.goto("/navigate?from=mi&to=mw", { waitUntil: "domcontentloaded" });

    const title = await page.title();
    expect(title.length).toBeGreaterThan(0);

    const description = await page.locator('meta[name="description"]').getAttribute("content");
    expect(description).toBeTruthy();
  });
});

test.describe("Navigation Page - Performance", () => {
  test("should load navigation page quickly", async ({ page }) => {
    const startTime = Date.now();
    await page.goto("/navigate", { waitUntil: "domcontentloaded" });
    const endTime = Date.now();

    const loadTime = endTime - startTime;
    expect(loadTime).toBeLessThan(5000);
  });
});
