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

  test("should focus search bar when typing on homepage", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    const searchInput = page.getByRole("textbox", { name: "Suchfeld" }).first();
    
    // Initially, search bar should not be focused
    await expect(searchInput).not.toBeFocused();
    
    // Simulate typing a lowercase character
    await page.keyboard.press("a");
    
    // Now search bar should be focused
    await expect(searchInput).toBeFocused();
  });

  test("should focus search bar when typing uppercase letters", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    const searchInput = page.getByRole("textbox", { name: "Suchfeld" }).first();
    
    await expect(searchInput).not.toBeFocused();
    
    // Type Shift+A (uppercase A)
    await page.keyboard.press("A");
    
    await expect(searchInput).toBeFocused();
  });

  test("should not focus search bar when pressing Tab key", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    const searchInput = page.getByRole("textbox", { name: "Suchfeld" }).first();
    
    await expect(searchInput).not.toBeFocused();
    
    // Press Tab - should not focus search bar
    await page.keyboard.press("Tab");
    
    await expect(searchInput).not.toBeFocused();
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

    // The /view/* link must resolve to the canonical /campus/ path and the
    // detail page must actually render. Asserting on URL alone - or on text
    // that also exists on the home page - would not catch #2888, where the
    // URL changed but the home page never unmounted.
    await expect(page).toHaveURL(/\/campus\/garching/);
    await expect(page.locator("main")).toContainText(/Anzahl Räume/);
    await expect(page.locator("main")).not.toContainText("Stammgelände");
  });

  test("should navigate to building details when clicking a building", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    const miLink = page.locator('a[href*="/view/mi"]').first();
    await miLink.click();

    await expect(page).toHaveURL(/\/building\/mi/);
    await expect(page.locator("main")).toContainText(/Anzahl Räume/);
    await expect(page.locator("main")).not.toContainText("Stammgelände");
  });

  test("should fully mount details after /view redirect (regression #2888)", async ({ page }) => {
    // Regression: clicking a `/view/{id}` link from the prerendered home page
    // updated `window.location` but left the home page mounted instead of the
    // location detail page. Asserting on URL alone - as the previous tests did
    // - was not enough to catch this. The whole click-then-render flow has to
    // land on a detail page that no longer contains the home-page sites grid.
    await page.goto("/", { waitUntil: "networkidle" });

    const physikLink = page.locator('a[href*="/view/physik"]').first();
    await physikLink.click();

    await expect(page).toHaveURL(/\/site\/physik/);
    await expect(page.locator("main")).toContainText("Physik");
    // The home page's "Stammgelände" card must no longer be visible - its
    // presence is the smoking gun that the home page never unmounted.
    await expect(page.locator("main")).not.toContainText("Stammgelände");
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
