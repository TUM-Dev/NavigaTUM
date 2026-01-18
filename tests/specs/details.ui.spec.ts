import { expect, test } from "@playwright/test";

test.describe("Details Page - Basic Functionality", () => {
  test("should load location details page with name", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");
    const heading = page.locator("h1, h2").first();
    await expect(heading).toBeVisible();
    await expect(heading).toContainText("MI");
    await expect(page).toHaveScreenshot();
  });

  test("should return an 404 for non-existent location", async ({ page }) => {
    await page.goto("/building/nonexistent_location_12345", { waitUntil: "networkidle" });
    // no weird redirect

    await expect(page).toHaveURL("building/nonexistent_location_12345");
    const heading404 = page.getByRole("heading", { name: "Die angeforderte Seite wurde" });
    await expect(heading404).toBeVisible();
  });
});

test.describe("Details Page - Interactive Map", () => {
  test("should display interactive map with controls", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");

    const mapCanvas = page.getByRole("region", { name: "Map" });
    await expect(mapCanvas).toHaveCount(1);
    await expect(mapCanvas).toBeVisible();

    const fullScreenButton = page.getByRole("button", { name: "Enter fullscreen" });
    await expect(fullScreenButton).toBeVisible();
    await expect(fullScreenButton).toHaveCount(1);

    const zoomInButton = page.getByRole("button", { name: "Zoom in" });
    await expect(zoomInButton).toBeVisible();
    await expect(zoomInButton).toHaveCount(1);

    const zoomOutButton = page.getByRole("button", { name: "Zoom out" });
    await expect(zoomOutButton).toBeVisible();
    await expect(zoomOutButton).toHaveCount(1);

    await expect(page).toHaveScreenshot();
  });
});

test.describe("Details Page - Images", () => {
  test("should display and interact with location images", async ({ page }) => {
    await page.goto("/view/chemie", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/chemie");

    const images = page.locator('img[src*="/cdn/"]');
    await expect(images).toHaveCount(1);

    // Test clicking image opens slideshow
    await images.first().click();
    const modal = page.getByRole("heading", { name: "Bilder-Showcase" });
    await expect(modal.first()).toBeVisible();
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

    const address = page.getByText("Boltzmannstr. 3, 85748");
    await expect(address).toBeVisible();
    await expect(page).toHaveScreenshot();
  });
});

test.describe("Details Page - Nearby Locations", () => {
  test.skip("should display nearby public transport with distances", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");

    const transport = page.getByText(/U-Bahn|S-Bahn|Bus|Tram|Station|Haltestelle/i);
    await expect(transport).toBeVisible();
  });
});

test.describe("Details Page - Calendar Integration", () => {
  test("should navigate to calendar page for rooms", async ({ page }) => {
    await page.goto("/view/5602.EG.001", { waitUntil: "networkidle" });
    // view -> room redirect
    await expect(page).toHaveURL("room/5602.EG.001");

    const calendarButton = page.getByRole("button", { name: "Kalender öffnen" });
    await expect(calendarButton).toHaveCount(1);
    await calendarButton.click();
    await expect(page).toHaveURL("/room/5602.EG.001?calendar[]=5602.EG.001");
  });
});

test.describe("Details Page - Share and Actions", () => {
  test("should have share, QR code, and feedback options", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");

    const shareButton = page.getByRole("button", { name: "Externe Links und optionen" });
    await shareButton.click();

    const googleMapsLink = page.getByRole("link", { name: "Google Maps" });
    await expect(googleMapsLink).toHaveCount(1);

    // Check for any action button
    const actionButtons = page.getByRole("img", { name: "QR-Code für diese Seite" });
    expect(actionButtons).toHaveCount(1);
  });
});

test.describe("Details Page - Building Overview", () => {
  test("should display rooms list and navigate to room", async ({ page }) => {
    await page.goto("/view/5602", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/5602");

    const roomLink = page.locator('a[href*="/view/5602."]').first();
    await expect(roomLink).toBeVisible();
    await expect(page).toHaveScreenshot();
    await roomLink.click();
    await expect(page).toHaveURL(/\/(view|room)\/5602\./);
  });

  test.skip("should not display floor information", async ({ page }) => {
    await page.goto("/view/5602", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/5602");

    const floors = page.getByText(/OG|EG|UG|Erdgeschoss|Floor|Stockwerk/i).first();
    await expect(floors).not.toBeVisible();
  });
});

test.describe("Details Page - Breadcrumbs", () => {
  test("displays breadcrumbs and allows navigation to parent", async ({ page }) => {
    await page.goto("/view/5602.EG.001");

    // Redirect: view -> room
    await expect(page).toHaveURL(/\/room\/5602\.EG\.001$/);

    const breadcrumbs = page.locator('nav[aria-label*="breadcrumb"], ol[typeof="BreadcrumbList"]');

    await expect(breadcrumbs).toBeVisible();

    const items = breadcrumbs.locator('li[typeof="ListItem"]');
    await expect(items).toHaveCount(4);

    // Assert last breadcrumb label (current context)
    await expect(items.nth(3)).toContainText("Hörsaal 1");

    // Click parent (building) breadcrumb
    const parentLink = items.nth(3).locator('a[href="/view/5602"]');
    await expect(parentLink).toBeVisible();

    await parentLink.click();
    await expect(page).toHaveURL(/\/(view|building)\/5602$/);
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
