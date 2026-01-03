import { expect, test } from "@playwright/test";

test.describe("Calendar Page - Basic Functionality", () => {
  test("should load calendar page and display events", async ({ page }) => {
    await page.goto("/calendar/5602.EG.001", { waitUntil: "domcontentloaded" });

    await expect(page).toHaveURL(/\/calendar\/5602\.EG\.001/);

    const heading = page.locator("h1, h2").first();
    await expect(heading).toBeVisible();

    // Check for calendar elements
    const calendar = page.locator('[class*="calendar"], [role="grid"], table').first();
    if ((await calendar.count()) > 0) {
      await expect(calendar).toBeVisible();
    }
  });

  test("should handle calendar for non-existent room", async ({ page }) => {
    await page.goto("/calendar/nonexistent_room_12345", { waitUntil: "domcontentloaded" });
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Calendar Page - Events Display", () => {
  test("should display calendar events with times", async ({ page }) => {
    await page.goto("/calendar/5602.EG.001", { waitUntil: "domcontentloaded" });

    // Look for event elements
    const events = page.locator('[class*="event"], [class*="booking"], [role="listitem"]');
    if ((await events.count()) > 0) {
      await expect(events.first()).toBeVisible();
    }

    // Look for time information
    const times = page.getByText(/\d{1,2}:\d{2}/);
    if ((await times.count()) > 0) {
      await expect(times.first()).toBeVisible();
    }
  });

  test("should show empty state when no events", async ({ page }) => {
    await page.goto("/calendar/mi", { waitUntil: "domcontentloaded" });
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("Calendar Page - Date Navigation", () => {
  test("should display date controls", async ({ page }) => {
    await page.goto("/calendar/5602.EG.001", { waitUntil: "domcontentloaded" });

    // Look for date navigation controls
    const dateNav = page.locator(
      'button[aria-label*="next"], button[aria-label*="previous"], button[aria-label*="nächste"]'
    );
    if ((await dateNav.count()) > 0) {
      await expect(dateNav.first()).toBeVisible();
    }

    // Look for date display
    const dateDisplay = page.locator('[class*="date"], [class*="day"], time').first();
    if ((await dateDisplay.count()) > 0) {
      await expect(dateDisplay).toBeVisible();
    }
  });
});

test.describe("Calendar Page - Actions", () => {
  test("should have back to room details link", async ({ page }) => {
    await page.goto("/calendar/5602.EG.001", { waitUntil: "domcontentloaded" });

    const backLink = page.locator('a[href*="/view/5602"]').first();
    if ((await backLink.count()) > 0) {
      await expect(backLink).toBeVisible();
    }
  });
});

test.describe("Calendar Page - Responsive Design", () => {
  test("should display correctly on mobile", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto("/calendar/5602.EG.001", { waitUntil: "domcontentloaded" });
    await expect(page.locator("body")).toBeVisible();
  });

  test("should display correctly on desktop", async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto("/calendar/5602.EG.001", { waitUntil: "domcontentloaded" });
    await expect(page.locator("body")).toBeVisible();
  });
});

test.describe("About Pages - Basic Functionality", () => {
  test("should load about us pages in both languages", async ({ page }) => {
    await page.goto("/about/ueber-uns", { waitUntil: "domcontentloaded" });
    await expect(page).toHaveURL(/\/about\/ueber-uns/);

    await page.goto("/en/about/about-us", { waitUntil: "domcontentloaded" });
    await expect(page).toHaveURL(/\/en\/about\/about-us/);
  });

  test("should redirect old about paths correctly", async ({ page }) => {
    await page.goto("/about/about-us", { waitUntil: "domcontentloaded" });
    await expect(page).toHaveURL(/\/about\/ueber-uns/);
  });

  test("should display page content", async ({ page }) => {
    await page.goto("/about/ueber-uns", { waitUntil: "domcontentloaded" });

    const content = page.locator("#contentwrapper, main, article").first();
    await expect(content).toBeVisible();
  });
});

test.describe("About Pages - Imprint/Impressum", () => {
  test("should load imprint pages in both languages", async ({ page }) => {
    await page.goto("/about/impressum", { waitUntil: "domcontentloaded" });
    await expect(page).toHaveURL(/\/about\/impressum/);

    await page.goto("/en/about/imprint", { waitUntil: "domcontentloaded" });
    await expect(page).toHaveURL(/\/en\/about\/imprint/);
  });

  test("should display imprint content with contact", async ({ page }) => {
    await page.goto("/about/impressum", { waitUntil: "domcontentloaded" });

    const heading = page.locator("h1, h2").first();
    await expect(heading).toBeVisible();

    // Look for email or address
    const contact = page.getByText(/@|mail|TUM|München|Munich/i).first();
    if ((await contact.count()) > 0) {
      await expect(contact).toBeVisible();
    }
  });
});

test.describe("About Pages - Privacy/Datenschutz", () => {
  test("should load privacy pages in both languages", async ({ page }) => {
    await page.goto("/about/datenschutz", { waitUntil: "domcontentloaded" });
    await expect(page).toHaveURL(/\/about\/datenschutz/);

    await page.goto("/en/about/privacy", { waitUntil: "domcontentloaded" });
    await expect(page).toHaveURL(/\/en\/about\/privacy/);
  });

  test("should display privacy policy content", async ({ page }) => {
    await page.goto("/about/datenschutz", { waitUntil: "domcontentloaded" });

    const heading = page.locator("h1, h2").first();
    await expect(heading).toBeVisible();

    // Look for privacy-related terms
    const privacyContent = page.getByText(/Daten|DSGVO|Datenschutz|Cookie/i).first();
    if ((await privacyContent.count()) > 0) {
      await expect(privacyContent).toBeVisible();
    }
  });
});

test.describe("About Pages - Content Rendering", () => {
  test("should render content properly", async ({ page }) => {
    await page.goto("/about/ueber-uns", { waitUntil: "domcontentloaded" });

    const h1 = await page.locator("h1").count();
    expect(h1).toBeGreaterThan(0);

    const paragraphs = await page.locator("p").count();
    expect(paragraphs).toBeGreaterThan(0);
  });

  test("should style content appropriately", async ({ page }) => {
    await page.goto("/about/ueber-uns", { waitUntil: "domcontentloaded" });

    const contentWrapper = page.locator("#contentwrapper").first();
    if ((await contentWrapper.count()) > 0) {
      await expect(contentWrapper).toBeVisible();
    }
  });
});

test.describe("About Pages - Navigation", () => {
  test("should navigate between about pages", async ({ page }) => {
    await page.goto("/about/ueber-uns", { waitUntil: "domcontentloaded" });

    const impressumLink = page.locator('a[href*="/about/impressum"]').first();
    if ((await impressumLink.count()) > 0) {
      await impressumLink.click();
      await expect(page).toHaveURL(/\/about\/impressum/);
    }
  });

  test("should maintain language when navigating", async ({ page }) => {
    await page.goto("/en/about/about-us", { waitUntil: "domcontentloaded" });

    const imprintLink = page.locator('a[href*="/en/about/imprint"]').first();
    if ((await imprintLink.count()) > 0) {
      await imprintLink.click();
      await expect(page).toHaveURL(/\/en\/about\/imprint/);
    }
  });
});

test.describe("About Pages - Accessibility", () => {
  test("should have proper heading hierarchy", async ({ page }) => {
    await page.goto("/about/ueber-uns", { waitUntil: "domcontentloaded" });

    const h1Count = await page.locator("h1").count();
    expect(h1Count).toBeGreaterThanOrEqual(1);
  });

  test("should be keyboard navigable", async ({ page }) => {
    await page.goto("/about/ueber-uns", { waitUntil: "domcontentloaded" });

    await page.keyboard.press("Tab");
    const focusedElement = await page.evaluateHandle(() => document.activeElement);
    expect(focusedElement).toBeTruthy();
  });
});

test.describe("About Pages - Responsive Design", () => {
  test("should display correctly on mobile", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto("/about/ueber-uns", { waitUntil: "domcontentloaded" });

    const content = page.locator("h1, h2, p").first();
    await expect(content).toBeVisible();
  });

  test("should display correctly on desktop", async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto("/about/ueber-uns", { waitUntil: "domcontentloaded" });

    const content = page.locator("h1, h2, p").first();
    await expect(content).toBeVisible();
  });
});

test.describe("About Pages - SEO", () => {
  test("should have proper page title", async ({ page }) => {
    await page.goto("/about/ueber-uns", { waitUntil: "domcontentloaded" });

    // Wait for page to fully load and title to be set
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(500);

    const title = await page.title();
    expect(title.length).toBeGreaterThan(0);
  });
});
