import { encode } from "@googlemaps/polyline-codec";
import { describe, expect, it } from "vitest";
import type { components } from "../app/api_types";
import { buildItineraryTimeline, calculateStepBounds, delayMinutes } from "../app/utils/motis";
import { floorLevelForSelection } from "../app/utils/motisLevels";

type MotisLegResponse = components["schemas"]["MotisLegResponse"];
type ItineraryResponse = components["schemas"]["ItineraryResponse"];
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

describe("buildItineraryTimeline", () => {
  function stop(name: string, track?: string): PlaceResponse {
    return { lat: 48.26, lon: 11.67, level: 0, name, track };
  }

  function ride(opts: {
    fromName: string;
    toName: string;
    fromTrack?: string;
    toTrack?: string;
    start: string;
    end: string;
    scheduledEnd?: string;
  }): MotisLegResponse {
    return {
      duration: 600,
      mode: "subway",
      from: stop(opts.fromName, opts.fromTrack),
      to: stop(opts.toName, opts.toTrack),
      start_time: opts.start,
      end_time: opts.end,
      scheduled_start_time: opts.start,
      scheduled_end_time: opts.scheduledEnd ?? opts.end,
      leg_geometry: "",
      real_time: true,
      scheduled: true,
      route_color: "0065AE",
      route_text_color: "FFFFFF",
      route_short_name: "U6",
    };
  }

  function walk(opts: {
    fromName: string;
    toName: string;
    fromTrack?: string;
    toTrack?: string;
    start: string;
    end: string;
  }): MotisLegResponse {
    return {
      duration: 240,
      mode: "walk",
      from: stop(opts.fromName, opts.fromTrack),
      to: stop(opts.toName, opts.toTrack),
      start_time: opts.start,
      end_time: opts.end,
      scheduled_start_time: opts.start,
      scheduled_end_time: opts.end,
      leg_geometry: "",
      real_time: false,
      scheduled: true,
      route_color: "FFFFFF",
      route_text_color: "000000",
    };
  }

  function itinerary(legs: MotisLegResponse[]): ItineraryResponse {
    return {
      duration: 0,
      start_time: legs[0]?.start_time ?? "",
      end_time: legs[legs.length - 1]?.end_time ?? "",
      transfer_count: 0,
      legs,
    };
  }

  it("has one more node than edges", () => {
    const { nodes, edges } = buildItineraryTimeline(
      itinerary([walk({ fromName: "A", toName: "B", start: "08:12", end: "08:16" })])
    );
    expect(nodes.length).toBe(2);
    expect(edges.length).toBe(1);
    expect(edges[0]?.selfNavigated).toBe(true);
  });

  it("returns nothing for an itinerary without legs", () => {
    expect(buildItineraryTimeline(itinerary([]))).toEqual({ nodes: [], edges: [] });
  });

  it("prints each stop once and renders a platform change as two nodes", () => {
    const { nodes, edges } = buildItineraryTimeline(
      itinerary([
        walk({
          fromName: "Boltzmannstraße",
          toName: "Garching-Forschungszentrum",
          start: "08:12",
          end: "08:16",
        }),
        ride({
          fromName: "Garching-Forschungszentrum",
          toName: "Odeonsplatz",
          fromTrack: "1",
          toTrack: "2",
          start: "08:17",
          end: "08:40",
          scheduledEnd: "08:38",
        }),
        walk({
          fromName: "Odeonsplatz",
          toName: "Odeonsplatz",
          fromTrack: "2",
          toTrack: "4",
          start: "08:40",
          end: "08:43",
        }),
        ride({
          fromName: "Odeonsplatz",
          toName: "Marienplatz",
          fromTrack: "4",
          toTrack: "3",
          start: "08:44",
          end: "08:47",
        }),
        walk({ fromName: "Marienplatz", toName: "Marienplatz 8", start: "08:47", end: "08:52" }),
      ])
    );

    expect(nodes.map((n) => n.name)).toEqual([
      "Boltzmannstraße",
      "Garching-Forschungszentrum",
      "Odeonsplatz",
      "Odeonsplatz",
      "Marienplatz",
      "Marienplatz 8",
    ]);
    expect(nodes[2]?.track).toBe("2");
    expect(nodes[3]?.track).toBe("4");
    expect(edges.map((e) => e.selfNavigated)).toEqual([true, false, true, false, true]);
  });

  it("shows the boarding time at a transit departure and the arrival time when a walk follows", () => {
    const { nodes } = buildItineraryTimeline(
      itinerary([
        walk({
          fromName: "Boltzmannstraße",
          toName: "Garching-Forschungszentrum",
          start: "08:12",
          end: "08:16",
        }),
        ride({
          fromName: "Garching-Forschungszentrum",
          toName: "Odeonsplatz",
          fromTrack: "1",
          start: "08:17",
          end: "08:40",
          scheduledEnd: "08:38",
        }),
        walk({ fromName: "Odeonsplatz", toName: "Marienplatz 8", start: "08:40", end: "08:52" }),
      ])
    );

    expect(nodes[1]?.time.scheduled).toBe("08:17");
    expect(nodes[1]?.track).toBe("1");
    expect(nodes[2]?.time.scheduled).toBe("08:38");
    expect(nodes[2]?.time.actual).toBe("08:40");
  });
});
