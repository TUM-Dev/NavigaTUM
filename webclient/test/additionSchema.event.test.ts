import { describe, expect, it } from "vitest";
import type { components } from "../app/api_types";
import {
  additionRegistry,
  buildAddition,
  type EventDraft,
  eventDraftFromEntry,
  eventSourceImageUrl,
  validateAddition,
} from "../app/composables/additionSchema";
import { wallTimeToRfc3339 } from "../app/utils/datetime";

type EventEntry = components["schemas"]["EventEntry"];

const DAY_MS = 24 * 60 * 60 * 1000;

function wall(offsetDays: number): string {
  const d = new Date(Date.now() + offsetDays * DAY_MS);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function validEvent(overrides: Partial<EventDraft> = {}): EventDraft {
  return {
    ...additionRegistry.event.empty(),
    id: "event_9d02ddd940c43f87",
    name: "GARNIX Festival",
    description: "Live music, food trucks, and stands.",
    starts_at: wall(2),
    ends_at: wall(3),
    coords: { lat: 48.262908, lon: 11.669102, picked: true },
    organising_org_id: 51897,
    image: { base64: "Zm9v", fileName: "event.png" },
    image_width: 300,
    image_height: 300,
    image_author: "Studi",
    ...overrides,
  };
}

describe("validateAddition (event)", () => {
  it("accepts a well-formed event", () => {
    expect(validateAddition(validEvent())).toEqual({});
  });

  it("builds the event payload from a valid draft", () => {
    const built = buildAddition(validEvent());
    // The build sets the discriminant, threads the image through CC BY 4.0, and renames the
    // wall-clock times into RFC 3339. Spot-check the shape rather than every field.
    expect(built).toMatchObject({
      kind: "event",
      name: "GARNIX Festival",
      organising_org_id: 51897,
      image: { content: "Zm9v" },
    });
  });

  it.each([
    ["bad key", { id: "event_NOTHEX" }, "id", "error.event_key_format"],
    ["missing prefix", { id: "9d02ddd940c43f87" }, "id", "error.event_key_format"],
    ["empty name", { name: "  " }, "name", "error.name_required"],
    ["empty description", { description: "" }, "description", "error.description_required"],
    [
      "unpicked coords",
      { coords: { lat: 0, lon: 0, picked: false } },
      "coords.picked",
      "error.coords_required",
    ],
    ["no org", { organising_org_id: null }, "organising_org_id", "error.org_required"],
    ["no image", { image: null }, "image", "error.image_required"],
    ["image too small", { image_width: 100, image_height: 100 }, "image", "error.image_too_small"],
    [
      "ends before start",
      { starts_at: wall(3), ends_at: wall(2) },
      "ends_at",
      "error.event_ends_before_start",
    ],
    ["already ended", { starts_at: wall(-3), ends_at: wall(-2) }, "ends_at", "error.event_ended"],
    [
      "too far out",
      { starts_at: wall(400), ends_at: wall(401) },
      "starts_at",
      "error.event_too_far_out",
    ],
    ["too long", { starts_at: wall(2), ends_at: wall(40) }, "ends_at", "error.event_too_long"],
    ["missing author", { image_author: "" }, "image_author", "error.image_author_required"],
  ] as const)("rejects %s", (_label, overrides, path, message) => {
    expect(validateAddition(validEvent(overrides))[path]).toBe(message);
  });
});

describe("eventDraftFromEntry", () => {
  const NOW = Date.parse("2026-06-13T12:00:00Z");

  function entry(overrides: Partial<EventEntry> = {}): EventEntry {
    return {
      id: "event_4a3e5d2fd5b338e4",
      name: "GARNIX Festival",
      description: "Open-air student festival.",
      starts_at: "2026-06-15T14:00:00Z",
      ends_at: "2026-06-19T21:59:00Z",
      lat: 48.262908,
      lon: 11.669102,
      organising_org_id: 51897,
      image: "/cdn/thumb/event_4a3e5d2fd5b338e4_0.webp",
      image_author: "Studentische Vertretung TUM",
      ...overrides,
    };
  }

  it("adopts the key and pre-fills every text field", () => {
    const draft = eventDraftFromEntry(entry(), NOW);
    expect(draft.id).toBe("event_4a3e5d2fd5b338e4");
    expect(draft.based_on).toEqual({
      id: "event_4a3e5d2fd5b338e4",
      name: "GARNIX Festival",
      starts_at: "2026-06-15T14:00:00Z",
      ends_at: "2026-06-19T21:59:00Z",
    });
    expect(draft.name).toBe("GARNIX Festival");
    expect(draft.description).toBe("Open-air student festival.");
    expect(draft.organising_org_id).toBe(51897);
    expect(draft.coords).toEqual({ lat: 48.262908, lon: 11.669102, picked: true });
    expect(draft.image_author).toBe("Studentische Vertretung TUM");
    // The image rides in separately through the upload path.
    expect(draft.image).toBeNull();
  });

  it("pre-fills the dates of a not-yet-ended event as the same instants", () => {
    const draft = eventDraftFromEntry(entry(), NOW);
    expect(Date.parse(wallTimeToRfc3339(draft.starts_at) ?? "")).toBe(
      Date.parse("2026-06-15T14:00:00Z")
    );
    expect(Date.parse(wallTimeToRfc3339(draft.ends_at) ?? "")).toBe(
      Date.parse("2026-06-19T21:59:00Z")
    );
  });

  it("clears the dates of an already-ended event", () => {
    const draft = eventDraftFromEntry(
      entry({ starts_at: "2025-06-15T14:00:00Z", ends_at: "2025-06-19T21:59:00Z" }),
      NOW
    );
    expect(draft.starts_at).toBe("");
    expect(draft.ends_at).toBe("");
    // The rest of the pre-fill is unaffected by the date cutoff.
    expect(draft.name).toBe("GARNIX Festival");
    expect(draft.based_on?.id).toBe("event_4a3e5d2fd5b338e4");
  });

  it("validates clean once an image is attached", () => {
    // `validateAddition` reads the real clock, so the entry's dates must be relative to it.
    const upcoming = entry({
      starts_at: new Date(Date.now() + 2 * DAY_MS).toISOString(),
      ends_at: new Date(Date.now() + 3 * DAY_MS).toISOString(),
    });
    const draft = eventDraftFromEntry(upcoming, Date.now());
    draft.image = { base64: "Zm9v", fileName: "event_4a3e5d2fd5b338e4_0.webp" };
    draft.image_width = 1448;
    draft.image_height = 2048;
    expect(validateAddition(draft)).toEqual({});
  });
});

describe("eventSourceImageUrl", () => {
  // Building on an event re-submits its image, so we must pull the exact, unchanged bytes
  // of the git source rather than the /cdn/lg artifact, which is a processed copy.
  it("points at the raw committed source on GitHub, not the served CDN artifact", () => {
    const url = eventSourceImageUrl("event_4a3e5d2fd5b338e4");
    expect(url).toBe(
      "https://raw.githubusercontent.com/TUM-Dev/NavigaTUM/refs/heads/main/data/sources/img/lg/event_4a3e5d2fd5b338e4_0.webp"
    );
    expect(url).not.toContain("/cdn/lg/");
  });
});
