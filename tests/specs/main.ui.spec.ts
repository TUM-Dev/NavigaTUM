import { expect, test } from "@playwright/test";

test.describe("Homepage", () => {
  test("should load the homepage successfully", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });

    await expect(page).toHaveTitle(/NavigaTUM/);
    await expect(page.locator("h1")).toContainText(/Standorte|Sites/);
  });

  test("should display the main navigation header", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });

    // Check for navigation elements - use first() to avoid strict mode violation
    await expect(page.locator("nav").first()).toBeVisible();
  });

  test("should display search bar on homepage", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });

    const searchInput = page.getByRole("textbox", { name: "Suchfeld" }).first();
    await expect(searchInput).toBeVisible();
  });

  test("should display footer with links", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });

    await expect(page.locator("footer")).toBeVisible();
  });
});

test.describe("Sites Overview", () => {
  test("should display all major campuses", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });

    await expect(page.getByText(/Garching/)).toBeVisible();
    await expect(page.getByText(/Stammgelände/)).toBeVisible();
    await expect(page.getByText(/Weihenstephan/)).toBeVisible();
  });

  test("should navigate to campus details when clicking a campus", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });

    const garchingLink = page.locator('a[href*="/view/garching"]').first();
    await garchingLink.click();

    await expect(page).toHaveURL(/\/view\/garching/);
  });

  test("should navigate to building details when clicking a building", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });

    const miLink = page.locator('a[href*="/view/mi"]').first();
    await miLink.click();

    await expect(page).toHaveURL(/\/view\/mi/);
  });

  test("should expand/collapse building lists", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });

    const moreButton = page.getByRole("button", { name: /mehr|more/i }).first();
    if ((await moreButton.count()) > 0) {
      await moreButton.click();
      // Wait for animation/re-render
      await page.waitForTimeout(300);
      await expect(page.getByRole("button", { name: /weniger|less/i }).first()).toBeVisible();
    }
  });
});

test.describe("Language Switching", () => {
  test("should switch between German and English", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });

    const languageSwitcher = page.locator('a[href*="/en"]').first();
    if ((await languageSwitcher.count()) > 0) {
      await languageSwitcher.click();
      expect(page.url()).toContain("/en");
    }
  });
});

test.describe("Responsive Design", () => {
  test("should display correctly on mobile viewport", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto("/", { waitUntil: "domcontentloaded" });

    await expect(page.locator("h1")).toBeVisible();
    // Search input might be type="text" or type="search"
    await expect(
      page
        .locator(
          'input[type="search"], input[type="text"][placeholder*="earch"], input[placeholder*="uche"]'
        )
        .first()
    ).toBeVisible();
  });

  test("should display correctly on desktop viewport", async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto("/", { waitUntil: "domcontentloaded" });

    await expect(page.locator("h1")).toBeVisible();
  });
});

test.describe("Accessibility", () => {
  test("should have proper heading hierarchy", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });

    const h1 = await page.locator("h1").count();
    expect(h1).toBeGreaterThan(0);
  });

  test("should have focusable navigation elements", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });

    const firstLink = page.locator("a").first();
    await firstLink.focus();
    await expect(firstLink).toBeFocused();
  });
});

test.describe("Performance", () => {
  test("should load within reasonable time", async ({ page }) => {
    const startTime = Date.now();
    await page.goto("/", { waitUntil: "domcontentloaded" });
    const endTime = Date.now();

    const loadTime = endTime - startTime;
    expect(loadTime).toBeLessThan(5000); // 5 seconds
  });
});

test.describe("SEO Meta Tags", () => {
  test("should have proper meta tags", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });

    const title = await page.title();
    expect(title.length).toBeGreaterThan(0);

    const description = await page.locator('meta[name="description"]').getAttribute("content");
    expect(description).toBeTruthy();
  });
});
