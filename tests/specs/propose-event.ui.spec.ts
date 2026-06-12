import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page, test } from "@playwright/test";

// The propose-page event flow (#3258) needs the event search facet (#3256) and
// upsert-by-key replace semantics (#3257), neither of which the test backend is
// guaranteed to have data for. All API and CDN traffic the flow depends on is
// mocked, so these tests exercise the webclient end-to-end and hermetically.

const EVENT_KEY = "event_4a3e5d2fd5b338e4";
const FIXTURE_IMAGE = fileURLToPath(
  new URL(`../../data/sources/img/lg/${EVENT_KEY}_0.webp`, import.meta.url),
);

const DAY_MS = 24 * 60 * 60 * 1000;

function rfc3339(offsetDays: number): string {
  const date = new Date(Date.now() + offsetDays * DAY_MS);
  // The pre-fill round-trips through minute-precision `datetime-local` values.
  date.setSeconds(0, 0);
  return date.toISOString();
}

function eventEntry(startsAt: string, endsAt: string) {
  return {
    id: EVENT_KEY,
    name: "GARNIX Festival",
    description: "Open-air student festival.",
    starts_at: startsAt,
    ends_at: endsAt,
    lat: 48.262908,
    lon: 11.669102,
    organising_org_id: 51897,
    image: `/cdn/thumb/${EVENT_KEY}_0.webp`,
    image_author: "Studentische Vertretung TUM",
  };
}

async function mockProposeBackend(
  page: Page,
  event: ReturnType<typeof eventEntry>,
): Promise<unknown[]> {
  const image = readFileSync(FIXTURE_IMAGE);
  await page.route("**/api/search**", (route) =>
    route.fulfill({
      json: {
        sections: [{ facet: "events", entries: [event], estimatedTotalHits: 1, n_visible: 1 }],
        time_ms: 1,
      },
    }),
  );
  await page.route("**/cdn/lg/event_*", (route) =>
    route.fulfill({ status: 200, contentType: "image/webp", body: image }),
  );
  await page.route("**/cdn/thumb/event_*", (route) =>
    route.fulfill({ status: 200, contentType: "image/webp", body: image }),
  );
  await page.route("**/cdn/known_orgs.json", (route) =>
    route.fulfill({
      json: [
        {
          org_id: 51897,
          code: "SVTUM",
          name_de: "Studentische Vertretung TUM",
          name_en: "Student Council TUM",
        },
      ],
    }),
  );
  // Old enough to skip the client's minimum-token-age wait, young enough to not refetch.
  await page.route("**/api/feedback/get_token", (route) =>
    route.fulfill({ status: 201, json: { created_at: Date.now() - 60_000, token: "e2e-token" } }),
  );
  const posted: unknown[] = [];
  await page.route("**/api/feedback/propose_edits", (route) => {
    posted.push(route.request().postDataJSON());
    return route.fulfill({
      status: 201,
      contentType: "text/plain",
      body: "https://github.com/TUM-Dev/NavigaTUM/pull/9999",
    });
  });
  // The location picker pulls its style from the production Martin tileserver; stub it
  // so the tests exercise our form rather than live tiles.
  await page.route("**/martin/style/navigatum-basemap.json", (route) =>
    route.fulfill({ json: { version: 8, sources: {}, layers: [] } }),
  );
  return posted;
}

async function openEventForm(page: Page): Promise<void> {
  await page.goto("/propose", { waitUntil: "networkidle" });
  await page.getByRole("tab", { name: "Event" }).click();
}

async function pickSuggestion(page: Page): Promise<void> {
  await page.locator("#add-event-search").fill("garnix");
  const suggestion = page.getByRole("button", { name: /GARNIX Festival/ });
  await expect(suggestion).toBeVisible();
  await suggestion.click();
}

const banner = (page: Page) => page.locator('[data-cy="event-update-banner"]');

test.describe("Propose page - event update mode", () => {
  test("picking a search hit pre-fills every field and locks onto the adopted key", async ({
    page,
  }) => {
    const posted = await mockProposeBackend(page, eventEntry(rfc3339(5), rfc3339(6)));
    await openEventForm(page);
    await pickSuggestion(page);

    // Locked mode: banner with name and last-held dates, search bar collapsed.
    await expect(banner(page)).toBeVisible();
    await expect(banner(page)).toContainText("GARNIX Festival");
    await expect(banner(page)).toContainText("Zuletzt:");
    await expect(page.locator("#add-event-search")).toHaveCount(0);

    // Every field pre-filled, including the image fetched from the CDN.
    await expect(page.locator("#add-event-name")).toHaveValue("GARNIX Festival");
    await expect(page.locator("#add-event-description")).toHaveValue(
      "Open-air student festival.",
    );
    await expect(page.locator("#add-event-image-author")).toHaveValue(
      "Studentische Vertretung TUM",
    );
    await expect(page.getByText(`${EVENT_KEY}_0.webp`)).toBeVisible();
    // The event has not ended, so its dates ride along.
    await expect(page.locator("#add-event-start")).toHaveValue(/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}/);
    await expect(page.locator("#add-event-end")).toHaveValue(/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}/);

    // The submit copy switched from proposing a new event to proposing an update.
    const submit = page.getByRole("button", { name: "Aktualisierung vorschlagen" });
    await expect(submit).toBeVisible();

    await page.locator("#feedback-privacy-checked").check();
    await submit.click();
    await expect(page.getByText("Vielen Dank!")).toBeVisible({ timeout: 20_000 });

    // The submission rides under the adopted (old) event key, so the server replaces the row.
    expect(posted).toHaveLength(1);
    const body = posted[0] as {
      additions: Record<string, { kind: string; name: string; organising_org_id: number }>;
    };
    expect(Object.keys(body.additions)).toEqual([EVENT_KEY]);
    expect(body.additions[EVENT_KEY]).toMatchObject({
      kind: "event",
      name: "GARNIX Festival",
      organising_org_id: 51897,
    });
  });

  test("an already-ended event leaves the date fields empty", async ({ page }) => {
    await mockProposeBackend(page, eventEntry(rfc3339(-30), rfc3339(-29)));
    await openEventForm(page);
    await pickSuggestion(page);

    await expect(banner(page)).toBeVisible();
    await expect(banner(page)).toContainText("bitte trage die neuen Termine ein");
    await expect(page.locator("#add-event-start")).toHaveValue("");
    await expect(page.locator("#add-event-end")).toHaveValue("");
    // The rest of the pre-fill is unaffected by the date cutoff.
    await expect(page.locator("#add-event-name")).toHaveValue("GARNIX Festival");
  });

  test("unlinking returns to a clean new-event flow", async ({ page }) => {
    await mockProposeBackend(page, eventEntry(rfc3339(5), rfc3339(6)));
    await openEventForm(page);
    await pickSuggestion(page);
    await expect(banner(page)).toBeVisible();

    await page
      .getByRole("button", { name: "Stattdessen eine neue Veranstaltung vorschlagen" })
      .click();

    await expect(banner(page)).toHaveCount(0);
    await expect(page.locator("#add-event-search")).toBeVisible();
    await expect(page.locator("#add-event-name")).toHaveValue("");
    await expect(page.getByText("Bild hierher ziehen oder klicken")).toBeVisible();
  });

  test("a fresh submission still derives the id from the image hash", async ({ page }) => {
    const posted = await mockProposeBackend(page, eventEntry(rfc3339(5), rfc3339(6)));
    await openEventForm(page);

    await page.locator("#add-event-name").fill("Testfest");
    await page.locator("#add-event-description").fill("A brand-new event.");
    const startWall = new Date(Date.now() + 5 * DAY_MS);
    const endWall = new Date(Date.now() + 6 * DAY_MS);
    const wall = (d: Date) =>
      `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}T10:00`;
    await page.locator("#add-event-start").fill(wall(startWall));
    await page.locator("#add-event-end").fill(wall(endWall));

    await page.getByPlaceholder("Organisation suchen…").fill("Stud");
    await page.getByRole("option", { name: /Studentische Vertretung TUM/ }).click();

    // The click handler only exists once the (stubbed) style has loaded; retry until it lands.
    const canvas = page.locator(".maplibregl-canvas").first();
    await expect(canvas).toBeVisible();
    await expect(async () => {
      await canvas.click();
      await expect(page.getByText(/-?\d+\.\d{5}, -?\d+\.\d{5}/)).toBeVisible({ timeout: 1_000 });
    }).toPass({ timeout: 15_000 });

    const [chooser] = await Promise.all([
      page.waitForEvent("filechooser"),
      page.getByText("Bild hierher ziehen oder klicken").click(),
    ]);
    await chooser.setFiles(FIXTURE_IMAGE);
    await expect(page.getByText(`${EVENT_KEY}_0.webp`)).toBeVisible();
    await page.locator("#add-event-image-author").fill("Studi");

    await page.locator("#feedback-privacy-checked").check();
    await page.getByRole("button", { name: "Senden", exact: true }).click();
    await expect(page.getByText("Vielen Dank!")).toBeVisible({ timeout: 20_000 });

    const expectedKey = `event_${createHash("sha256").update(readFileSync(FIXTURE_IMAGE)).digest("hex").slice(0, 16)}`;
    expect(posted).toHaveLength(1);
    const body = posted[0] as { additions: Record<string, { kind: string }> };
    expect(Object.keys(body.additions)).toEqual([expectedKey]);
  });
});
