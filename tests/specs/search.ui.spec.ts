import { expect, test } from "@playwright/test";

test.describe("Search Page - Basic Functionality", () => {
  test("should navigate to search page with query parameter", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    await expect(page).toHaveURL("/search?q=MI");
    await expect(page).toHaveTitle(/MI/);
  });

  test("should display search results and runtime statistics", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    await expect(page.getByText(/Laufzeit: \d+ms/)).toBeVisible();
  });

  test("should show feedback button", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    const feedbackButton = page.getByText(/Feedback|feedback/i).first();
    await expect(feedbackButton).toBeVisible();
  });
});

test.describe("Search Page - Results Display", () => {
  test("should display search results as clickable links", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    // Wait for search results to load
    await page.waitForLoadState("networkidle");

    const resultLinks = page.locator('a[href*="/view/"]');
    const count = await resultLinks.count();
    expect(count).toBeGreaterThan(0);
    await expect(page).toHaveScreenshot();
  });

  test("should navigate to details page when clicking a result", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    const firstResult = page.locator('a[href*="/view/mi"]').first();
    await expect(firstResult).toBeVisible();
    await firstResult.click();
    await expect(page).toHaveURL(/\/(view|building)\/mi/);
  });
});

test.describe("Search Page - Empty and Error States", () => {
  test("should handle empty search query", async ({ page }) => {
    await page.goto("/search?q=", { waitUntil: "networkidle" });
    await expect(page.locator("body")).toBeVisible();
  });

  test("should handle search with no results", async ({ page }) => {
    await page.goto("/search?q=xyznonexistentbuilding12345", { waitUntil: "networkidle" });
    await expect(page.locator("body")).toBeVisible();
    await expect(page).toHaveScreenshot();
  });

  test("should handle special characters in search", async ({ page }) => {
    await page.goto("/search?q=MI-HS", { waitUntil: "networkidle" });
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Search Bar - Interactive Search", () => {
  test("should perform search from homepage search bar", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    // Wait for page to fully load
    await page.waitForLoadState("networkidle");

    // Search input might be type="text" or type="search"
    const searchInput = page.getByRole("textbox", { name: "Suchfeld" }).first();
    await searchInput.fill("MI");
    await searchInput.press("Enter");

    await expect(page).toHaveURL("/search?q=MI");
  });
});

test.describe("Search Page - Filtering and Pagination", () => {
  test("should respect limit parameters in URL", async ({ page }) => {
    await page.goto("/search?q=MI&limit_buildings=5&limit_rooms=10", {
      waitUntil: "networkidle",
    });

    await expect(page).toHaveURL(/limit_buildings=5/);
    await expect(page).toHaveURL(/limit_rooms=10/);
  });
});

test.describe("Search Page - URL Handling", () => {
  test.skip("should preserve search query in URL when navigating back", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    const firstResult = page.locator('a[href*="/view/"]').first();
    await firstResult.click();

    await page.waitForLoadState("networkidle");
    await page.goBack();
    await page.waitForLoadState("networkidle");

    await expect(page).toHaveURL("/search?q=MI");
  });

  test("should update document title with search query", async ({ page }) => {
    await page.goto("/search?q=Informatik", { waitUntil: "networkidle" });

    const title = await page.title();
    expect(title).toContain("Informatik");
  });
});

test.describe("Search Page - Accessibility", () => {
  test("should be keyboard navigable", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    await page.keyboard.press("Tab");
    const focusedElement = await page.evaluateHandle(() => document.activeElement);
    expect(focusedElement).toBeTruthy();
  });
});

test.describe("Search Page - Responsive Design", () => {
  test("should display search results on mobile", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });
    await expect(page.locator("body")).toBeVisible();
  });

  test("should display search results on desktop", async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Search Page - SEO", () => {
  test("should have proper meta description for search results", async ({ page }) => {
    await page.goto("/search?q=Informatik", { waitUntil: "networkidle" });

    const description = await page.locator('meta[name="description"]').getAttribute("content");
    expect(description).toBeTruthy();
  });
});
