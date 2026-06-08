import { describe, expect, it } from "vitest";
import { validateAddition } from "../app/composables/additionSchema";
import { type AdditionDraft, emptyAdditionDraft } from "../app/composables/editProposal";

const DAY_MS = 24 * 60 * 60 * 1000;

// A `datetime-local` wall string `now + offsetDays`. Built from the machine's local clock; the
// schema reads it as Europe/Berlin, but every threshold here is days wide, so the <=2h tz skew is
// irrelevant.
function wall(offsetDays: number): string {
  const d = new Date(Date.now() + offsetDays * DAY_MS);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function validEvent(overrides: Partial<AdditionDraft> = {}): AdditionDraft {
  return {
    ...emptyAdditionDraft(),
    kind: "event",
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

  // Mirrors the `validate_failure_cases` rstest table in
  // `server/src/routes/feedback/proposed_edits/addition/event.rs`.
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
