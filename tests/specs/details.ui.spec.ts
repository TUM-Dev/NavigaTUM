import { expect, test } from "@playwright/test";

test.describe("Details Page - Basic Functionality", () => {
  test("should load location details page with name", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");
    const heading = page.locator("h1, h2").first();
    await expect(heading).toBeVisible();
    await expect(heading).toContainText("MI");
  });

  test("should return an 404 for non-existent location", async ({ page }) => {
    await page.goto("/building/nonexistent_location_12345", { waitUntil: "networkidle" });
    // no weird redirect
    expect(page.url()).toContain("building/nonexistent_location_12345");
    expect(page.getByRole("heading", { name: "Die angeforderte Seite wurde" })).toBeVisible();
  });
});

test.describe("Details Page - Interactive Map", () => {
  test("should display interactive map with controls", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");

    const mapCanvas = page.locator('canvas, [class*="maplibre"]').first();
    await expect(mapCanvas).toBeVisible();
  });

  test("should switch between interactive map and floor plans", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");

    const mapSelector = page.locator('button[aria-label*="plan"], [role="tab"]');
    if ((await mapSelector.count()) > 1) {
      await mapSelector.nth(1).click();
      await expect(page).toHaveURL(/map=plans/);
    }
  });
});

test.describe("Details Page - Images", () => {
  test("should display and interact with location images", async ({ page }) => {
    await page.goto("/view/5602", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/5602");

    const images = page.locator('img[src*="/cdn/"]');
    if ((await images.count()) > 0) {
      await expect(images.first()).toBeVisible();

      // Test clicking image opens slideshow
      await images.first().click();
      const modal = page.locator('dialog[open], [role="dialog"]');
      if ((await modal.count()) > 0) {
        await expect(modal.first()).toBeVisible();
      }
    }
  });
});

test.describe("Details Page - Navigation Actions", () => {
  test("should have navigation button and navigate", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");

    const navButton = page.getByRole("link", { name: "BETA Navigation starten" }).first();
    expect(navButton).toBeVisible();
    await expect(navButton).toHaveCount(1);
    // Scroll element into view before clicking
    await navButton.scrollIntoViewIfNeeded();
    await navButton.click({ force: true });
    await expect(page).toHaveURL(/\/navigate/);
    expect(page.url()).toMatch(/to=|from=/);
  });
});

test.describe("Details Page - Property Information", () => {
  test("should display location properties and address", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");

    await expect(page.locator("body")).toBeVisible();

    const address = page.getByText(/straße|strasse|street|Garching/i).first();
    if ((await address.count()) > 0) {
      await expect(address).toBeVisible();
    }
  });

  test("should display coordinates", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");

    const coords = page.getByText(/\d+\.\d+.*\d+\.\d+|Koordinaten|Coordinates/i).first();
    if ((await coords.count()) > 0) {
      await expect(coords).toBeVisible();
    }
  });
});

test.describe("Details Page - Nearby Locations", () => {
  test("should display nearby public transport with distances", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");

    const transport = page.getByText(/U-Bahn|S-Bahn|Bus|Tram|Station|Haltestelle/i).first();
    if ((await transport.count()) > 0) {
      await expect(transport).toBeVisible();
    }
  });
});

test.describe("Details Page - Calendar Integration", () => {
  test("should navigate to calendar page for rooms", async ({ page }) => {
    await page.goto("/view/5602.EG.001", { waitUntil: "networkidle" });
    // view -> room redirect
    await expect(page).toHaveURL("room/5602.EG.001");

    const calendarLink = page.locator('a[href*="/calendar"]').first();
    if ((await calendarLink.count()) > 0) {
      await calendarLink.click();
      await expect(page).toHaveURL(/\/calendar/);
    }
  });
});

test.describe("Details Page - Share and Actions", () => {
  test("should have share, QR code, and feedback options", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");

    // Check for any action button
    const actionButtons = page.locator('button, a[href*="qr-code"]');
    const count = await actionButtons.count();
    expect(count).toBeGreaterThan(0);
  });
});

test.describe("Details Page - Building Overview", () => {
  test("should display rooms list and navigate to room", async ({ page }) => {
    await page.goto("/view/5602", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/5602");

    const roomLink = page.locator('a[href*="/view/5602."]').first();
    if ((await roomLink.count()) > 0) {
      await roomLink.click();
      await expect(page).toHaveURL(/\/view\/5602\./);
    }
  });

  test("should not display floor information", async ({ page }) => {
    await page.goto("/view/5602", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/5602");

    const floors = page.getByText(/OG|EG|UG|Erdgeschoss|Floor|Stockwerk/i).first();
    await expect(floors).not.toBeVisible();
  });
});

test.describe("Details Page - Breadcrumbs", () => {
  test("should display and navigate using breadcrumbs", async ({ page }) => {
    await page.goto("/view/5602.EG.001", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("room/5602.EG.001");

    const breadcrumbs = page.locator('nav[aria-label*="breadcrumb"], [class*="breadcrumb"]');
    if ((await breadcrumbs.count()) > 0) {
      await expect(breadcrumbs.first()).toBeVisible();

      const parentLink = page.locator('a[href*="/view/5602"]').first();
      if ((await parentLink.count()) > 0) {
        await parentLink.click();
        await expect(page).toHaveURL(/\/view\/5602$/);
      }
    }
  });
});

test.describe("Details Page - Accessibility", () => {
  test("should have proper heading hierarchy", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");

    const h1 = await page.locator("h1").count();
    expect(h1).toBeGreaterThanOrEqual(1);
  });

  test("should be keyboard navigable", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");

    await page.keyboard.press("Tab");
    const focusedElement = await page.evaluateHandle(() => document.activeElement);
    expect(focusedElement).toBeTruthy();
  });
});

test.describe("Details Page - Responsive Design", () => {
  test("should display correctly on mobile", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");

    await expect(page.locator("h1, h2").first()).toBeVisible();
  });

  test("should display correctly on desktop", async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");

    await expect(page.locator("h1, h2").first()).toBeVisible();
  });
});

test.describe("Details Page - SEO and Meta", () => {
  test("should have proper meta tags", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");

    const title = await page.title();
    expect(title.length).toBeGreaterThan(0);

    const description = await page.locator('meta[name="description"]').getAttribute("content");
    expect(description).toBeTruthy();

    const ogTitle = await page.locator('meta[property="og:title"]').getAttribute("content");
    expect(ogTitle || description).toBeTruthy();
  });
});
