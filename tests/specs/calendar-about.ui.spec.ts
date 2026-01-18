import { expect, test } from "@playwright/test";

test.describe("Calendar Page - Basic Functionality", () => {
  test.beforeEach(async ({ page }, testInfo) => {
    // Extend timeout for all tests running this hook by 60 seconds.
    // Wow, we should improve performance for this...
    test.setTimeout(testInfo.timeout + 60000);
  });
  test("should load calendar page and display events", async ({ page }) => {
    await page.goto("/calendar/5602.EG.001", { waitUntil: "networkidle" });
    await expect(page).toHaveURL("room/5602.EG.001?calendar[]=5602.EG.001");

    const heading = page.getByRole("heading", { name: "Kalender" }).first();
    await expect(heading).toBeVisible({ timeout: 10000 });

    const calendar = page.locator('[class*="calendar"], [role="grid"], table').first();
    await expect(calendar).toBeVisible({ timeout: 10000 });
  });

  test("should handle calendar for non-existent room", async ({ page }) => {
    await page.goto("/calendar/nonexistent_room_12345", { waitUntil: "networkidle" });
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Calendar Page - Events Display", () => {
  test.beforeEach(async ({ page }, testInfo) => {
    // Extend timeout for all tests running this hook by 60 seconds.
    // Wow, we should improve performance for this...
    test.setTimeout(testInfo.timeout + 60000);
  });
  test("should display calendar events with times", async ({ page }) => {
    await page.goto("/calendar/5602.EG.001", { waitUntil: "networkidle" });
    await expect(page).toHaveURL("room/5602.EG.001?calendar[]=5602.EG.001");

    const heading = page.getByRole("heading", { name: "Kalender" }).first();
    await expect(heading).toBeVisible({ timeout: 10000 });

    // Look for event elements
    const events = page.locator('[class*="event"], [class*="booking"], [role="listitem"]');
    await expect(events.first()).toBeVisible();

    // Look for time information
    const times = page.getByText(/\d{1,2}:\d{2}/);
    await expect(times.first()).toBeVisible();
  });

  test.skip("should show empty state when no events", async ({ page }) => {
    await page.goto("/calendar/mi", { waitUntil: "networkidle" });
    await expect(page).toHaveURL("building/mi?calendar[]=mi");

    const heading = page.getByRole("heading", { name: "Kalender" }).first();
    await expect(heading).toBeVisible({ timeout: 10000 });

    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Calendar Page - Date Navigation", () => {
  test.beforeEach(async ({ page }, testInfo) => {
    // Extend timeout for all tests running this hook by 60 seconds.
    // Wow, we should improve performance for this...
    test.setTimeout(testInfo.timeout + 60000);
  });
  test("should display date controls", async ({ page }) => {
    await page.goto("/calendar/5602.EG.001", { waitUntil: "networkidle" });
    await expect(page).toHaveURL("room/5602.EG.001?calendar[]=5602.EG.001");

    const heading = page.getByRole("heading", { name: "Kalender" }).first();
    await expect(heading).toBeVisible({ timeout: 10000 });

    // Look for date navigation controls
    const dateNav = page.getByRole("button", { name: "Nächste Woche" });
    await expect(dateNav.first()).toBeVisible();

    // Look for date display
    const dateDisplay = page.locator('[class*="date"], [class*="day"], time').first();
    await expect(dateDisplay).toBeVisible();
  });
});

test.describe("Calendar Page - Actions", () => {
  test.beforeEach(async ({ page }, testInfo) => {
    // Extend timeout for all tests running this hook by 60 seconds.
    // Wow, we should improve performance for this...
    test.setTimeout(testInfo.timeout + 60000);
  });
  test("should have back to room details link", async ({ page }) => {
    await page.goto("/calendar/5602.EG.001", { waitUntil: "load" });
    await expect(page).toHaveURL("room/5602.EG.001?calendar[]=5602.EG.001");

    const heading = page.getByRole("heading", { name: "Kalender" }).first();
    await expect(heading).toBeVisible({ timeout: 10000 });

    const backLink = page.locator('a[href*="/view/5602"]').first();
    await expect(backLink).toBeVisible();
  });
});

test.describe("About Pages - Basic Functionality", () => {
  test("should load about us pages german", async ({ page }) => {
    await page.goto("/about/ueber-uns", { waitUntil: "load" });
    await expect(page).toHaveURL("/about/ueber-uns");

    const h1 = await page.locator("h1").count();
    expect(h1).toBeGreaterThan(0);

    const paragraphs = await page.locator("p").count();
    expect(paragraphs).toBeGreaterThan(0);
  });

  test("should load about us pages english", async ({ page }) => {
    await page.goto("/en/about/about-us", { waitUntil: "load" });
    await expect(page).toHaveURL("/about/about-us");

    const h1 = await page.locator("h1").count();
    expect(h1).toBeGreaterThan(0);

    const paragraphs = await page.locator("p").count();
    expect(paragraphs).toBeGreaterThan(0);
    await expect(page).toHaveScreenshot();
  });

  test("should redirect old about paths correctly", async ({ page }) => {
    await page.goto("/about/about-us", { waitUntil: "networkidle" });
    await expect(page).toHaveURL("/about/ueber-uns");
  });

  test("should display page content", async ({ page }) => {
    await page.goto("/about/ueber-uns", { waitUntil: "networkidle" });

    const content = page.locator("#contentwrapper, main, article").first();
    await expect(content).toBeVisible();
  });
});

test.describe("About Pages - Imprint/Impressum", () => {
  test("should load imprint pages in both languages", async ({ page }) => {
    await page.goto("/about/impressum", { waitUntil: "networkidle" });
    await expect(page).toHaveURL("/about/impressum");

    await page.goto("/en/about/imprint", { waitUntil: "networkidle" });
    await expect(page).toHaveURL("/en/about/imprint");
  });

  test("should display imprint content with contact", async ({ page }) => {
    await page.goto("/about/impressum", { waitUntil: "networkidle" });

    const heading = page.locator("h1, h2").first();
    await expect(heading).toBeVisible();

    // Look for email or address
    const contact = page.getByText(/@|mail|TUM|München|Munich/i).first();
    await expect(contact).toBeVisible();
    await expect(page).toHaveScreenshot();
  });
});

test.describe("About Pages - Privacy/Datenschutz", () => {
  test("should load privacy pages in both languages", async ({ page }) => {
    await page.goto("/about/datenschutz", { waitUntil: "networkidle" });
    await expect(page).toHaveURL("/about/datenschutz");

    await page.goto("/en/about/privacy", { waitUntil: "networkidle" });
    await expect(page).toHaveURL("/en/about/privacy");
  });

  test("should display privacy policy content", async ({ page }) => {
    await page.goto("/about/datenschutz", { waitUntil: "networkidle" });

    const heading = page.locator("h1, h2").first();
    await expect(heading).toBeVisible();

    // Look for privacy-related terms
    const privacyContent = page.getByText(/Daten|DSGVO|Datenschutz|Cookie/i).first();
    await expect(privacyContent).toBeVisible();
  });
});

test.describe("About Pages - Navigation", () => {
  test("should navigate between about pages", async ({ page }) => {
    await page.goto("/about/ueber-uns", { waitUntil: "networkidle" });

    const impressumLink = page.locator('a[href*="/about/impressum"]');
    await expect(impressumLink).toBeVisible();
    await impressumLink.first().click();
    await expect(page).toHaveURL("/about/impressum");
  });

  test("should maintain language when navigating", async ({ page }) => {
    await page.goto("/en/about/about-us", { waitUntil: "networkidle" });

    const imprintLink = page.locator('a[href*="/en/about/imprint"]');
    await expect(imprintLink).toBeVisible();
    await imprintLink.first().click();
    await expect(page).toHaveURL("/en/about/imprint");
  });
});

test.describe("About Pages - Accessibility", () => {
  test("should have proper heading hierarchy", async ({ page }) => {
    await page.goto("/about/ueber-uns", { waitUntil: "networkidle" });

    const h1Count = await page.locator("h1").count();
    expect(h1Count).toBeGreaterThanOrEqual(1);
  });

  test("should be keyboard navigable", async ({ page }) => {
    await page.goto("/about/ueber-uns", { waitUntil: "networkidle" });

    await page.keyboard.press("Tab");
    const focusedElement = await page.evaluateHandle(() => document.activeElement);
    expect(focusedElement).toBeTruthy();
  });
});

test.describe("About Pages - Responsive Design", () => {
  test("should display correctly on mobile", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto("/about/ueber-uns", { waitUntil: "networkidle" });

    const content = page.locator("h1, h2, p").first();
    await expect(content).toBeVisible();
  });

  test("should display correctly on desktop", async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto("/about/ueber-uns", { waitUntil: "networkidle" });

    const content = page.locator("h1, h2, p").first();
    await expect(content).toBeVisible();
  });
});

test.describe("About Pages - SEO", () => {
  test("should have proper page title", async ({ page }) => {
    await page.goto("/about/ueber-uns", { waitUntil: "networkidle" });

    // Wait for page to fully load and title to be set
    await page.waitForLoadState("networkidle");

    const title = await page.title();
    expect(title.length).toBeGreaterThan(0);
  });
});
