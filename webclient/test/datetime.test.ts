import { describe, expect, it } from "vitest";
import { berlinWallTimeToRfc3339 } from "../app/utils/datetime";

describe("berlinWallTimeToRfc3339", () => {
  it("stamps the summer (CEST) offset", () => {
    expect(berlinWallTimeToRfc3339("2026-06-10T16:00")).toBe("2026-06-10T16:00:00+02:00");
  });

  it("stamps the winter (CET) offset", () => {
    expect(berlinWallTimeToRfc3339("2026-01-10T16:00")).toBe("2026-01-10T16:00:00+01:00");
  });

  it("uses the offset in effect just after the spring-forward cutover", () => {
    // 2026 DST starts 2026-03-29 02:00 CET → 03:00 CEST.
    expect(berlinWallTimeToRfc3339("2026-03-29T04:00")).toBe("2026-03-29T04:00:00+02:00");
  });

  it("uses the offset in effect just before the autumn cutover", () => {
    // 2026 DST ends 2026-10-25 03:00 CEST → 02:00 CET.
    expect(berlinWallTimeToRfc3339("2026-10-25T00:30")).toBe("2026-10-25T00:30:00+02:00");
  });

  it("rejects malformed input", () => {
    expect(berlinWallTimeToRfc3339("")).toBeNull();
    expect(berlinWallTimeToRfc3339("2026-06-10")).toBeNull();
    expect(berlinWallTimeToRfc3339("not-a-date")).toBeNull();
  });
});
