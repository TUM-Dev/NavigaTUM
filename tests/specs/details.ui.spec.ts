import { expect, test } from "@playwright/test";

test.describe("Details Page - Basic Functionality", () => {
  test("should load location details page with name", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/mi");
    const heading = page.locator("h1, h2").first();
    await expect(heading).toBeVisible();
    await expect(heading).toContainText("MI");
    // await expect(page).toHaveScreenshot();
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
    // The interactive map fetches its style from the production Martin
    // tileserver. When that endpoint is unavailable (intermittent 404s),
    // MapLibre never fires `load`, so the navigation/fullscreen controls
    // are never added and the test fails for an upstream reason. Stub the
    // style with a minimal valid maplibre style so the controls render
    // regardless of the upstream tileserver state.
    await page.route(
      "https://nav.tum.de/martin/style/navigatum-basemap.json",
      async (route) => {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({ version: 8, sources: {}, layers: [] }),
        });
      },
    );

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

    // await expect(page).toHaveScreenshot();
  });
});

test.describe("Details Page - POI Floor Inheritance", () => {
  // Regression for TUM-Dev/NavigaTUM#1696: POIs used to leave FloorControl empty
  // (every button dimmed), so the indoor overlay never showed. POIs now inherit
  // floors from their immediate parent in the data pipeline.
  const stubBasemap = async (page: import("@playwright/test").Page) => {
    await page.route(
      "https://nav.tum.de/martin/style/navigatum-basemap.json",
      async (route) => {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({ version: 8, sources: {}, layers: [] }),
        });
      },
    );
  };

  test("room-parented POI auto-selects the room's floor", async ({ page }) => {
    await stubBasemap(page);
    await page.goto("/view/validierungsautomat-5", { waitUntil: "networkidle" });
    await expect(page).toHaveURL("poi/validierungsautomat-5");

    const floorCtrl = page.locator(".floor-ctrl");
    await expect(floorCtrl).toBeVisible();

    // Single inherited floor → DetailsInteractiveMap auto-calls setLevel(EG.id)
    const egButton = floorCtrl.locator("button", { hasText: /^EG$/ });
    await expect(egButton).toHaveClass(/active/);
  });

  test("building-parented POI exposes building floors as clickable", async ({ page }) => {
    await stubBasemap(page);
    await page.goto("/view/validierungsautomat-9", { waitUntil: "networkidle" });
    await expect(page).toHaveURL("poi/validierungsautomat-9");

    const floorCtrl = page.locator(".floor-ctrl");
    await expect(floorCtrl).toBeVisible();

    // Multiple inherited floors → no auto-select, EG button is enabled.
    const egButton = floorCtrl.locator("button", { hasText: /^EG$/ });
    await expect(egButton).toBeEnabled();
    await expect(egButton).not.toHaveCSS("cursor", "not-allowed");
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
    await expect(page).toHaveURL((url) => url.pathname === "/navigate");
    await expect(page).toHaveURL((url) => url.searchParams.get("coming_from") === "mi");
    await expect(page).toHaveURL((url) => url.searchParams.get("to") === "mi");
    await expect(page).toHaveURL((url) => !!url.searchParams.get("q_to"));
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
    // await expect(page).toHaveScreenshot();
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

    // Share tab is the default selected tab and now also lists external "Open in" links
    const googleMapsLink = page.getByRole("link", { name: "Google Maps" });
    await expect(googleMapsLink).toHaveCount(1);

    // QR code lives behind its own tab
    await page.getByRole("tab", { name: /QR-Code/i }).click();
    const qrImage = page.getByRole("img", { name: "QR-Code für diese Seite" });
    await expect(qrImage).toHaveCount(1);
  });

  test("share modal exposes a copyable iframe embed snippet", async ({ page }) => {
    await page.goto("/view/mi", { waitUntil: "networkidle" });
    await expect(page).toHaveURL("building/mi");

    const shareButton = page.getByRole("button", { name: "Externe Links und optionen" });
    await shareButton.click();

    // share dialog uses tabs - switch to the Embed tab
    const embedTab = page.getByRole("tab", { name: /Einbetten/i });
    await expect(embedTab).toBeVisible();
    await embedTab.click();

    const snippet = page.locator("textarea[readonly]");
    await expect(snippet).toHaveCount(1);
    const value = await snippet.inputValue();
    expect(value).toContain('src="https://nav.tum.de/embed/mi"');
    expect(value).toContain("<iframe");
    expect(value).toContain('allow="fullscreen; geolocation"');

    const copyButton = page.getByRole("button", { name: /Einbettungs-Code kopieren/i });
    await expect(copyButton).toBeVisible();
  });
});

test.describe("Embed Page - Basic Rendering", () => {
  test("should render minimal embed view with map and CTA", async ({ page }) => {
    await page.goto("/embed/mi", { waitUntil: "networkidle" });
    await expect(page).toHaveURL("/embed/mi");

    const mapCanvas = page.getByRole("region", { name: "Map" });
    await expect(mapCanvas).toHaveCount(1);
    await expect(mapCanvas).toBeVisible();

    const detailsLink = page.getByRole("link", { name: /In NavigaTUM ansehen/i });
    await expect(detailsLink).toBeVisible();
    await expect(detailsLink).toHaveAttribute("target", "_blank");
    await expect(detailsLink).toHaveAttribute("href", "https://nav.tum.de/building/mi");

    // The embed layout strips the main app nav header
    await expect(page.locator("header")).toHaveCount(0);
  });

  test("should set noindex robots meta", async ({ page }) => {
    await page.goto("/embed/mi", { waitUntil: "networkidle" });
    const robots = await page.locator('meta[name="robots"]').getAttribute("content");
    expect(robots).toMatch(/noindex/i);
  });

  test("should 404 on non-existent location", async ({ page }) => {
    await page.goto("/embed/nonexistent_location_12345", { waitUntil: "networkidle" });
    await expect(page).toHaveURL("/embed/nonexistent_location_12345");
    const heading404 = page.getByRole("heading", { name: "Die angeforderte Seite wurde" });
    await expect(heading404).toBeVisible();
  });

  test("should not be blocked by X-Frame-Options for iframe usage", async ({ page }) => {
    const response = await page.goto("/embed/mi", { waitUntil: "domcontentloaded" });
    expect(response).not.toBeNull();
    const xfo = response?.headers()["x-frame-options"];
    // route-rule clears X-Frame-Options so embed can be iframed by third parties
    expect(xfo === undefined || xfo === "").toBeTruthy();

    const csp = response?.headers()["content-security-policy"];
    if (csp) {
      expect(csp).toMatch(/frame-ancestors\s+\*/);
    }
  });
});

test.describe("Details Page - Building Overview", () => {
  test("should display rooms list and navigate to room", async ({ page }) => {
    await page.goto("/view/5602", { waitUntil: "networkidle" });
    // view -> building redirect
    await expect(page).toHaveURL("building/5602");

    const roomLink = page.locator('a[href*="/view/5602."]').first();
    await expect(roomLink).toBeVisible();
    // await expect(page).toHaveScreenshot();
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
    await expect(page).toHaveURL("/room/5602.EG.001");

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
