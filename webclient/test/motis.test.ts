import { encode } from "@googlemaps/polyline-codec";
import { describe, expect, it } from "vitest";
import type { components } from "../app/api_types";
import { calculateStepBounds, delayMinutes } from "../app/utils/motis";
import { floorLevelForSelection } from "../app/utils/motisLevels";

type MotisLegResponse = components["schemas"]["MotisLegResponse"];
type PlaceResponse = components["schemas"]["PlaceResponse"];
type StepInstructionResponse = components["schemas"]["StepInstructionResponse"];

function place(level: number): PlaceResponse {
  return { lat: 48.26, lon: 11.67, level, name: "somewhere" };
}

function step(fromLevel: number, toLevel: number, polyline = ""): StepInstructionResponse {
  return {
    area: false,
    distance: 10,
    exit: "",
    from_level: fromLevel,
    to_level: toLevel,
    polyline,
    relative_direction: "continue",
    stay_on: false,
    street_name: "corridor",
  };
}

function walkLeg(opts: {
  fromLevel: number;
  toLevel: number;
  steps?: StepInstructionResponse[];
}): MotisLegResponse {
  return {
    duration: 60,
    start_time: "2026-07-15T12:00:00Z",
    end_time: "2026-07-15T12:01:00Z",
    scheduled_start_time: "2026-07-15T12:00:00Z",
    scheduled_end_time: "2026-07-15T12:01:00Z",
    from: place(opts.fromLevel),
    to: place(opts.toLevel),
    leg_geometry: "",
    mode: "walk",
    real_time: false,
    scheduled: true,
    route_color: "FFFFFF",
    route_text_color: "000000",
    steps: opts.steps,
  };
}

describe("floorLevelForSelection", () => {
  it("leaves the floor untouched for an all-ground-level leg", () => {
    const leg = walkLeg({ fromLevel: 0, toLevel: 0, steps: [step(0, 0)] });
    expect(floorLevelForSelection(leg)).toBeNull();
    expect(floorLevelForSelection(leg, step(0, 0))).toBeNull();
  });

  it("switches to the leg's starting level when the leg is level-aware", () => {
    expect(floorLevelForSelection(walkLeg({ fromLevel: 2, toLevel: 0 }))).toBe(2);
    // An indoor leg legitimately starting on the ground floor still selects it.
    expect(floorLevelForSelection(walkLeg({ fromLevel: 0, toLevel: 2 }))).toBe(0);
  });

  it("treats a leg with only step-level transitions as level-aware", () => {
    const leg = walkLeg({ fromLevel: 0, toLevel: 0, steps: [step(0, 1), step(1, 0)] });
    expect(floorLevelForSelection(leg)).toBe(0);
  });

  it("switches to the step's starting level when a step is selected", () => {
    const stairs = step(1, 2);
    const leg = walkLeg({ fromLevel: 0, toLevel: 2, steps: [step(0, 1), stairs] });
    expect(floorLevelForSelection(leg, stairs)).toBe(1);
  });

  it("skips levels the floor selector cannot represent", () => {
    // Half-levels and deep basements have no floor selector button.
    expect(floorLevelForSelection(walkLeg({ fromLevel: 1.5, toLevel: 2 }))).toBeNull();
    expect(floorLevelForSelection(walkLeg({ fromLevel: -2, toLevel: 0 }))).toBeNull();
    const basementStep = step(-2, -3);
    const leg = walkLeg({ fromLevel: -2, toLevel: -3, steps: [basementStep] });
    expect(floorLevelForSelection(leg, basementStep)).toBeNull();
  });
});

describe("calculateStepBounds", () => {
  it("spans the step's polyline", () => {
    const polyline = encode(
      [
        [48.1, 11.6],
        [48.2, 11.5],
      ],
      6
    );
    expect(calculateStepBounds(step(0, 0, polyline))).toEqual({
      minLat: 48.1,
      maxLat: 48.2,
      minLon: 11.5,
      maxLon: 11.6,
    });
  });

  it("is null for an undecodable polyline", () => {
    expect(calculateStepBounds(step(0, 0, ""))).toBeNull();
  });
});

describe("delayMinutes", () => {
  const scheduled = "2026-07-15T14:51:00+02:00";

  it("reports a positive delay when the trip runs late", () => {
    expect(delayMinutes(scheduled, "2026-07-15T14:58:00+02:00")).toBe(7);
  });

  it("reports a negative delay when the trip runs early", () => {
    expect(delayMinutes(scheduled, "2026-07-15T14:48:00+02:00")).toBe(-3);
  });

  it("compares instants, not wall-clock offsets", () => {
    expect(delayMinutes(scheduled, "2026-07-15T12:58:00Z")).toBe(7);
  });

  it("is null on the timetable slot", () => {
    expect(delayMinutes(scheduled, scheduled)).toBeNull();
  });

  it("is null within the deadband, so feed jitter never reads as a delay", () => {
    expect(delayMinutes(scheduled, "2026-07-15T14:51:29+02:00")).toBeNull();
    expect(delayMinutes(scheduled, "2026-07-15T14:50:31+02:00")).toBeNull();
  });

  it("reports the first minute once the deadband is cleared", () => {
    expect(delayMinutes(scheduled, "2026-07-15T14:51:31+02:00")).toBe(1);
  });

  it("is null when either side is missing or unparseable", () => {
    expect(delayMinutes(null, "2026-07-15T14:58:00+02:00")).toBeNull();
    expect(delayMinutes(scheduled, undefined)).toBeNull();
    expect(delayMinutes(scheduled, "not a timestamp")).toBeNull();
  });
});
