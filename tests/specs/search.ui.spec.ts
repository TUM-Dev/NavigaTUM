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
  test("should display search results as canonical /{type}/{id} links", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    // Wait for search results to load
    await page.waitForLoadState("networkidle");

    const resultLinks = page.locator(
      'a[href*="/building/"], a[href*="/room/"], a[href*="/site/"], a[href*="/campus/"], a[href*="/poi/"]'
    );
    const count = await resultLinks.count();
    expect(count).toBeGreaterThan(0);
    // await expect(page).toHaveScreenshot();
  });

  test("should navigate to the canonical details page when clicking a result", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    const firstResult = page.locator('a[href*="/building/mi"]').first();
    await expect(firstResult).toBeVisible();
    await firstResult.click();
    await expect(page).toHaveURL(/\/building\/mi/);
  });
});

test.describe("Search Page - English ↔ German synonyms (#960)", () => {
  for (const query of ["library", "libraries", "bibliothek", "Teilbibliothek"]) {
    test(`q=${query} returns at least one building result`, async ({ page }) => {
      await page.goto(`/search?q=${query}`, { waitUntil: "networkidle" });
      await page.waitForLoadState("networkidle");

      const resultLinks = page.locator(
        'a[href*="/building/"], a[href*="/room/"], a[href*="/site/"], a[href*="/campus/"], a[href*="/poi/"]'
      );
      expect(await resultLinks.count()).toBeGreaterThan(0);
    });
  }
});

test.describe("Search Page - Empty and Error States", () => {
  test("should handle empty search query", async ({ page }) => {
    await page.goto("/search?q=", { waitUntil: "networkidle" });
    await expect(page.locator("body")).toBeVisible();
  });

  test("should handle search with no results", async ({ page }) => {
    await page.goto("/search?q=xyznonexistentbuilding12345", { waitUntil: "networkidle" });
    await expect(page.locator("body")).toBeVisible();
    // await expect(page).toHaveScreenshot();
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

  // #3324: tapping a dropdown result did nothing on Mobile Safari; Chromium focuses the link and can't reproduce it.
  test.describe("dropdown result navigation (#3324)", () => {
    test.skip(({ browserName }) => browserName !== "webkit", "WebKit/iOS-only focus race");

    test("clicking a dropdown result navigates to its details page", async ({ page }) => {
      await page.goto("/", { waitUntil: "networkidle" });

      const searchInput = page.getByRole("textbox", { name: "Suchfeld" }).first();
      await searchInput.fill("MI");

      const firstResult = page
        .locator('a[href*="/building/"], a[href*="/room/"], a[href*="/site/"], a[href*="/campus/"]')
        .first();
      await expect(firstResult).toBeVisible();
      await firstResult.click();

      await expect(page).toHaveURL(/\/(building|room|site|campus)\//);
    });
  });

  test("should not focus search bar when typing on search results page", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    const searchInput = page.getByRole("textbox", { name: "Suchfeld" }).first();
    
    // Initially not focused
    await expect(searchInput).not.toBeFocused();
    
    // Type a character - should NOT focus search bar on non-index page
    await page.keyboard.press("a");
    
    // Should still not be focused
    await expect(searchInput).not.toBeFocused();
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
  test("should return to the search results when navigating back", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    const firstResult = page.locator('a[href*="/building/mi"]').first();
    await firstResult.click();
    await expect(page).toHaveURL(/\/building\/mi/);

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

const TYPE_CHIP = /^Typ/;
const USAGE_CHIP = /^Nutzung/;
const LOCATION_CHIP = /^Standort/;

test.describe("Search Filters - URL to chip state", () => {
  const badgeCases = [
    { name: "single type bucket", query: "type=building", chip: TYPE_CHIP, badge: "(1)" },
    {
      name: "multiple type buckets",
      query: "type=building&type=room",
      chip: TYPE_CHIP,
      badge: "(2)",
    },
    { name: "usage filter", query: "usage=hoersaal", chip: USAGE_CHIP, badge: "(1)" },
    { name: "location (in) filter", query: "in=mi", chip: LOCATION_CHIP, badge: "(1)" },
  ] as const;

  for (const { name, query, chip, badge } of badgeCases) {
    test(`${name} from URL shows count badge on chip`, async ({ page }) => {
      await page.goto(`/search?q=MI&${query}`, { waitUntil: "networkidle" });

      const chipButton = page.getByRole("button", { name: chip }).first();
      await expect(chipButton).toBeVisible();
      await expect(chipButton).toContainText(badge);
    });
  }

  test("no filter params means no count badges", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    await expect(page.getByRole("button", { name: TYPE_CHIP }).first()).not.toContainText("(");
    await expect(page.getByRole("button", { name: USAGE_CHIP }).first()).not.toContainText("(");
    await expect(page.getByRole("button", { name: LOCATION_CHIP }).first()).not.toContainText("(");
  });
});

// Bucket labels also appear inside search-result links (e.g. "Mathematik /
// Informatik Gebäude"), so unscoped getByText('Gebäude') is ambiguous. Scope
// to the headlessui PopoverPanel that the type chip opens.
const typePopover = (page: import("@playwright/test").Page) =>
  page.locator('[id^="headlessui-popover-panel"]').first();

test.describe("Search Filters - Type popover", () => {
  test("clicking type chip opens popover with all four buckets", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    await page.getByRole("button", { name: TYPE_CHIP }).first().click();

    const popover = typePopover(page);
    await expect(popover.getByText("Raum", { exact: true })).toBeVisible();
    await expect(popover.getByText("Gebäude", { exact: true })).toBeVisible();
    await expect(popover.getByText(/Gelände/)).toBeVisible();
    await expect(popover.getByText(/POI/)).toBeVisible();
  });

  test("toggling a type bucket updates the URL", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    await page.getByRole("button", { name: TYPE_CHIP }).first().click();
    await typePopover(page).getByText("Gebäude", { exact: true }).click();

    await expect(page).toHaveURL(/type=building/);
  });

  test("toggling the same bucket twice removes it from URL", async ({ page }) => {
    await page.goto("/search?q=MI&type=building", { waitUntil: "networkidle" });

    await page.getByRole("button", { name: TYPE_CHIP }).first().click();
    await typePopover(page).getByText("Gebäude", { exact: true }).click();

    await expect(page).not.toHaveURL(/type=building/);
  });
});

test.describe("Search Filters - Usage panel", () => {
  test("usage panel close button hides the panel and reveals the search input", async ({
    page,
  }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    await page.getByRole("button", { name: USAGE_CHIP }).first().click();
    const panelTitle = page.getByText("Nach Nutzung filtern");
    await expect(panelTitle).toBeVisible();
    await expect(page.getByPlaceholder("Nutzungsart suchen...")).toBeVisible();

    await page.getByRole("button", { name: "Schließen" }).first().click();
    await expect(panelTitle).not.toBeVisible();
  });

  test("usage chip toggles the panel and flips aria-expanded", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    const chip = page.getByRole("button", { name: USAGE_CHIP }).first();
    const panelTitle = page.getByText("Nach Nutzung filtern");

    await expect(chip).toHaveAttribute("aria-expanded", "false");
    await chip.click();
    await expect(chip).toHaveAttribute("aria-expanded", "true");
    await expect(panelTitle).toBeVisible();

    await chip.click();
    await expect(chip).toHaveAttribute("aria-expanded", "false");
    await expect(panelTitle).not.toBeVisible();
  });
});

test.describe("Search Filters - Location panel", () => {
  test("clicking location chip opens the inline panel with hint and search input", async ({
    page,
  }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    await page.getByRole("button", { name: LOCATION_CHIP }).first().click();

    await expect(page.getByText("Standort einschränken")).toBeVisible();
    await expect(page.getByPlaceholder("Gebäude oder Standort suchen...")).toBeVisible();
    await expect(page.getByText(/Beginne zu tippen/)).toBeVisible();
  });

  test("typing in location panel triggers a suggestion fetch", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    await page.getByRole("button", { name: LOCATION_CHIP }).first().click();
    const input = page.getByPlaceholder("Gebäude oder Standort suchen...");
    await input.fill("garching");

    // Wait for either suggestions or a "no results" message - both prove the
    // fetch ran (loading, then settled).
    const suggestions = page.locator("#location-filter-panel ul li");
    const noResults = page.locator("#location-filter-panel").getByText("Keine Ergebnisse");
    await expect(suggestions.first().or(noResults)).toBeVisible({ timeout: 8000 });
  });

  test("active in-filter renders a removable pill in the panel", async ({ page }) => {
    await page.goto("/search?q=MI&in=mi", { waitUntil: "networkidle" });

    await page.getByRole("button", { name: LOCATION_CHIP }).first().click();

    const removeBtn = page.getByRole("button", { name: /Standort mi entfernen/ });
    await expect(removeBtn).toBeVisible();
    await removeBtn.click();
    await expect(page).not.toHaveURL(/in=mi/);
  });
});

test.describe("Search Filters - Clear all", () => {
  test("clear-all button is hidden when no filters are active", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    await expect(page.getByRole("button", { name: "Leeren" })).not.toBeVisible();
  });

  test("clear-all button removes every filter param from URL", async ({ page }) => {
    await page.goto("/search?q=MI&type=building&usage=hoersaal&in=mi", {
      waitUntil: "networkidle",
    });

    const clearButton = page.getByRole("button", { name: "Leeren" }).first();
    await expect(clearButton).toBeVisible();
    await clearButton.click();

    await expect(page).not.toHaveURL(/type=/);
    await expect(page).not.toHaveURL(/usage=/);
    await expect(page).not.toHaveURL(/[?&]in=/);
    await expect(page).toHaveURL(/q=MI/);
  });
});

test.describe("Search Filters - Sort control", () => {
  test("sort button is visible on search page", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    const sortButton = page.getByRole("button", { name: /Sortieren: Relevanz/ }).first();
    await expect(sortButton).toBeVisible();
  });

  test("clicking sort button reveals relevance + distance options", async ({ page }) => {
    await page.goto("/search?q=MI", { waitUntil: "networkidle" });

    await page.getByRole("button", { name: /Sortieren: Relevanz/ }).first().click();

    // The chip's own accessible name is "Sortieren: Relevanz", so a non-exact
    // match would also resolve to it. Pin to the option labels.
    await expect(page.getByRole("button", { name: "Relevanz", exact: true })).toBeVisible();
    await expect(page.getByRole("button", { name: "Entfernung", exact: true })).toBeVisible();
  });

  test("URL near= param drives the sort label to distance", async ({ page }) => {
    await page.goto("/search?q=MI&near=48.262,11.668", { waitUntil: "networkidle" });

    const sortButton = page.getByRole("button", { name: /Sortieren: Entfernung/ }).first();
    await expect(sortButton).toBeVisible();
  });

  test("disabling distance sort drops the near param from URL", async ({ page }) => {
    await page.goto("/search?q=MI&near=48.262,11.668", { waitUntil: "networkidle" });

    await page.getByRole("button", { name: /Sortieren: Entfernung/ }).first().click();
    await page.getByRole("button", { name: "Relevanz", exact: true }).click();

    await expect(page).not.toHaveURL(/near=/);
  });
});

test.describe("Search Filters - Autocomplete dropdown integration", () => {
  test("filter chips render inside the autocomplete dropdown on homepage", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    const searchInput = page.getByRole("textbox", { name: "Suchfeld" }).first();
    await searchInput.fill("MI");

    await expect(page.getByRole("button", { name: TYPE_CHIP }).first()).toBeVisible();
    await expect(page.getByRole("button", { name: USAGE_CHIP }).first()).toBeVisible();
    await expect(page.getByRole("button", { name: LOCATION_CHIP }).first()).toBeVisible();
  });

  test("opening a filter popover keeps the autocomplete dropdown open", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    const searchInput = page.getByRole("textbox", { name: "Suchfeld" }).first();
    await searchInput.fill("MI");

    // Buildings header is data-dependent (and the prod backend currently
    // returns no sections when a type filter is active). The chip wrapper
    // only exists while the dropdown is open, so use it as the open-signal.
    const typeChip = page.getByRole("button", { name: TYPE_CHIP }).first();
    await expect(typeChip).toBeVisible();

    await typeChip.click();
    await typePopover(page).getByText("Gebäude", { exact: true }).click();

    await expect(typeChip).toBeVisible();
  });

  test("staged filter selections survive the form submission into URL", async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });

    const searchInput = page.getByRole("textbox", { name: "Suchfeld" }).first();
    await searchInput.fill("MI");
    await page.getByRole("button", { name: TYPE_CHIP }).first().click();
    await typePopover(page).getByText("Gebäude", { exact: true }).click();

    await searchInput.press("Enter");

    await expect(page).toHaveURL(/q=MI/);
    await expect(page).toHaveURL(/type=building/);
  });
});

test.describe("Search Filters - API parameter passthrough", () => {
  const passthroughCases = [
    { bucket: "building", label: "Gebäude" },
    { bucket: "room", label: "Raum" },
  ] as const;

  for (const { bucket, label } of passthroughCases) {
    test(`type=${bucket} in URL is sent verbatim (server buckets internally)`, async ({
      page,
    }) => {
      // The server's `facet` field already buckets subtypes (e.g. joined_building
      // is filed under `building`), so the frontend does not expand the bucket
      // name client-side anymore - it just passes the chosen bucket through.
      await page.goto("/search?q=MI", { waitUntil: "networkidle" });

      const requestPromise = page.waitForRequest(
        (req) => req.url().includes("/api/search") && req.url().includes(`type=${bucket}`),
        { timeout: 15000 },
      );

      await page.getByRole("button", { name: TYPE_CHIP }).first().click();
      await typePopover(page).getByText(label, { exact: true }).click();

      const request = await requestPromise;
      const occurrences = request.url().match(new RegExp(`type=${bucket}\\b`, "g")) ?? [];
      expect(occurrences).toHaveLength(1);
    });
  }
});

test.describe("Search - Category shortcut", () => {
  // /map pulls its style from the production Martin tileserver; stub it so the tests
  // exercise our navigation rather than live tiles.
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
  const SHORTCUT_LABEL = "Toiletten & Duschen auf der Karte erkunden";

  test("typing toilets in the top bar surfaces the shortcut into the filtered map", async ({ page }) => {
    await stubBasemap(page);
    await page.goto("/", { waitUntil: "networkidle" });

    const searchInput = page.getByRole("textbox", { name: "Suchfeld" }).first();
    await searchInput.fill("toilets");

    const shortcut = page.getByRole("link", { name: SHORTCUT_LABEL });
    await expect(shortcut).toBeVisible();
    await shortcut.click();

    await expect(page).toHaveURL(/\/map\?filter=wcs/);
    await expect(page.getByRole("checkbox", { name: "Toiletten & Duschen" })).toBeChecked();
  });

  test("the shortcut participates in the keyboard highlight cursor", async ({ page }) => {
    await stubBasemap(page);
    await page.goto("/", { waitUntil: "networkidle" });

    const searchInput = page.getByRole("textbox", { name: "Suchfeld" }).first();
    await searchInput.fill("klo");
    await expect(page.getByRole("link", { name: SHORTCUT_LABEL })).toBeVisible();

    // The shortcut sits above all sections, so the first ArrowDown lands on it.
    await searchInput.press("ArrowDown");
    await searchInput.press("Enter");

    await expect(page).toHaveURL(/\/map\?filter=wcs/);
  });

  test("the /search page renders the shortcut above all sections", async ({ page }) => {
    await page.goto("/search?q=toilette", { waitUntil: "networkidle" });

    const shortcut = page.getByRole("link", { name: SHORTCUT_LABEL });
    await expect(shortcut).toBeVisible();
  });

  test("room-code-like queries surface no shortcut", async ({ page }) => {
    await page.goto("/search?q=GWC 101", { waitUntil: "networkidle" });

    await expect(page.getByRole("link", { name: SHORTCUT_LABEL })).toHaveCount(0);
  });
});
