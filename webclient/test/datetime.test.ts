import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { wallTimeToRfc3339 } from "../app/utils/datetime";

describe("wallTimeToRfc3339", () => {
  // Pin the zone so the UTC output is deterministic regardless of where the suite runs.
  const original = process.env.TZ;
  beforeAll(() => {
    process.env.TZ = "Europe/Berlin";
  });
  afterAll(() => {
    process.env.TZ = original;
  });

  it("converts a summer (CEST, +02:00) wall time to UTC", () => {
    expect(wallTimeToRfc3339("2026-06-10T16:00")).toBe("2026-06-10T14:00:00.000Z");
  });

  it("converts a winter (CET, +01:00) wall time to UTC", () => {
    expect(wallTimeToRfc3339("2026-01-10T16:00")).toBe("2026-01-10T15:00:00.000Z");
  });

  it("returns null for malformed input", () => {
    expect(wallTimeToRfc3339("")).toBeNull();
    expect(wallTimeToRfc3339("not-a-date")).toBeNull();
  });
});
