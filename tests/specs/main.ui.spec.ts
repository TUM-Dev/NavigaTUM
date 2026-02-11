import { expect, test } from "@playwright/test";

test.describe("Homepage", () => {
  test("should load the homepage successfully", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    await expect(page).toHaveTitle(/NavigaTUM/);
    await expect(page.locator("h1")).toContainText(/Standorte|Sites/);
    // await expect(page).toHaveScreenshot();
  });

  test("should display the main navigation header", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    // Check for navigation elements - use first() to avoid strict mode violation
    await expect(page.locator("nav").first()).toBeVisible();
  });

  test("should display search bar on homepage", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    const searchInput = page.getByRole("textbox", { name: "Suchfeld" }).first();
    await expect(searchInput).toBeVisible();
    // await expect(page).toHaveScreenshot();
  });

  test("should autofocus search bar on homepage", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    const searchInput = page.getByRole("textbox", { name: "Suchfeld" }).first();
    await expect(searchInput).toBeFocused();
  });

  test("should display footer with links", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    await expect(page.locator("footer")).toBeVisible();
  });
});

test.describe("Sites Overview", () => {
  test("should display all major campuses", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    await expect(page.getByText(/Garching/)).toBeVisible();
    await expect(page.getByText(/Stammgelände/)).toBeVisible();
    await expect(page.getByText(/Weihenstephan/)).toBeVisible();
    // await expect(page).toHaveScreenshot();
  });

  test("should navigate to campus details when clicking a campus", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    const garchingLink = page.locator('a[href*="/view/garching"]').first();
    await garchingLink.click();

    await expect(page).toHaveURL(/.*\/(view|campus)\/garching/);
  });

  test("should navigate to building details when clicking a building", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    const miLink = page.locator('a[href*="/view/mi"]').first();
    await miLink.click();

    await expect(page).toHaveURL(/\/(view|building)\/mi/);
  });

  test("should expand/collapse building lists", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    const moreButton = page.getByRole("button", { name: /mehr|more/i });
    const lessButton = page.getByRole("button", { name: /weniger|less/i });
    await expect(moreButton).toHaveCount(4);
    await expect(moreButton.first()).toBeVisible();
    await expect(lessButton).not.toBeVisible();
    await moreButton.first().click();
    // Wait for animation/re-render
    await expect(moreButton).toHaveCount(3);
    await expect(lessButton).toBeVisible();
    await expect(lessButton).toHaveCount(1);
    await lessButton.first().click();
    // Wait for animation/re-render
    await expect(lessButton).not.toBeVisible();
    await expect(moreButton).toHaveCount(4);
  });
});

test.describe("Language Switching", () => {
  test("should switch between German and English", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    const settingsButton = page.getByRole("button", { name: "Open preferences menu" });
    await expect(settingsButton).toBeVisible();
    await settingsButton.first().click();

    const languageSwitcherGerman = page.getByRole("tab", { name: "Deutsch" });
    await expect(languageSwitcherGerman).toBeVisible();
    await languageSwitcherGerman.first().click();
    await expect(page).toHaveURL("/");

    const prefDe = page.getByRole("heading", { name: "Präferenzen" });
    await expect(prefDe).toBeVisible();

    const languageSwitcherEnglish = page.getByRole("tab", { name: "English" });
    await expect(languageSwitcherEnglish).toBeVisible();
    await languageSwitcherEnglish.first().click();
    await expect(page).toHaveURL("/en");

    const prefEn = page.getByRole("heading", { name: "Preferences" });
    await expect(prefEn).toBeVisible();
  });
});

test.describe("Responsive Design", () => {
  test("should display correctly on mobile viewport", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto("/", { waitUntil: "networkidle" });

    await expect(page.locator("h1")).toBeVisible();
    // Search input might be type="text" or type="search"
    await expect(page.getByRole("textbox", { name: "Suchfeld" }).first()).toBeVisible();
  });

  test("should display correctly on desktop viewport", async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto("/", { waitUntil: "networkidle" });

    await expect(page.locator("h1")).toBeVisible();
  });
});

test.describe("Accessibility", () => {
  test("should have proper heading hierarchy", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    const h1 = await page.locator("h1").count();
    expect(h1).toBeGreaterThan(0);
  });

  test("should have focusable navigation elements", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    const firstLink = page.locator("a").first();
    await firstLink.focus();
    await expect(firstLink).toBeFocused();
  });
});

test.describe("SEO Meta Tags", () => {
  test("should have proper meta tags", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    const title = await page.title();
    expect(title.length).toBeGreaterThan(0);

    const description = await page.locator('meta[name="description"]').getAttribute("content");
    expect(description).toBeTruthy();
  });
});
