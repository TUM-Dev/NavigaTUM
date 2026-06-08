import { describe, expect, it } from "vitest";
import {
  bookedUntilTime,
  buildRoomRows,
  isKnownStatus,
  occupancyPercent,
  parseIrisRooms,
  roomsForBuildings,
} from "../app/utils/iris";

// A small slice of a real `GET https://iris.asta.tum.de/api/` response, covering each status and
// the WAAS occupancy fields. The `BC2 ...` arch-name and `D 5@4113` are genuine roster entries that
// have no NavigaTUM alias - they exercise the silent-omission path.
const IRIS_FIXTURE = {
  raeume: [
    {
      raum_nr_architekt: "1450@5504",
      raum_name: "Zeichensaal",
      raum_code: "MW 1450",
      gebaeude_code: "5504",
      status: "WAAS",
      belegung_durch: "",
      belegung_bis: "",
      color: "#5cb85c",
      percent: 0.03,
      subtitle: "frei",
    },
    {
      raum_nr_architekt: "D 5@4113",
      raum_name: "Karaokeraum",
      raum_code: "D 5",
      gebaeude_code: "4113",
      status: "belegt",
      belegung_durch: "IRIS",
      belegung_bis: "2026-06-06 19:28:40",
    },
    {
      raum_nr_architekt: "BC2 0.01.18@8102",
      raum_name: "Seminarraum",
      raum_code: "BC2 0.01.18",
      gebaeude_code: "8102",
      status: "frei",
      belegung_durch: "",
      belegung_bis: "",
    },
    {
      raum_nr_architekt: "0.01.19@8102",
      raum_name: "Übungsraum",
      raum_code: "BC2 0.01.19",
      gebaeude_code: "8102",
      status: "unbekannt",
      belegung_durch: "",
      belegung_bis: "",
    },
    {
      // A WAAS reading can dip slightly negative around an empty room; it must clamp to 0.
      raum_nr_architekt: "U1@5532",
      raum_name: "Foyer",
      raum_code: "MW U1",
      gebaeude_code: "5532",
      status: "WAAS",
      belegung_durch: "",
      belegung_bis: "",
      color: "#5cb85c",
      percent: -0.0667,
      subtitle: "frei",
    },
  ],
};

describe("parseIrisRooms", () => {
  it("extracts every room with its core fields", () => {
    const rooms = parseIrisRooms(IRIS_FIXTURE);
    expect(rooms.map((r) => r.archName)).toEqual([
      "1450@5504",
      "D 5@4113",
      "BC2 0.01.18@8102",
      "0.01.19@8102",
      "U1@5532",
    ]);
    expect(rooms[0]).toMatchObject({
      name: "Zeichensaal",
      code: "MW 1450",
      buildingId: "5504",
      status: "WAAS",
    });
  });

  it("reads booker and booked-until only for booked rooms, mapping empty strings to null", () => {
    const [waas, belegt, frei] = parseIrisRooms(IRIS_FIXTURE);
    expect(belegt).toMatchObject({ bookedBy: "IRIS", bookedUntil: "2026-06-06 19:28:40" });
    expect(waas).toMatchObject({ bookedBy: null, bookedUntil: null });
    expect(frei).toMatchObject({ bookedBy: null, bookedUntil: null });
  });

  it("attaches occupancy only to WAAS rooms and clamps a negative reading to zero", () => {
    const rooms = parseIrisRooms(IRIS_FIXTURE);
    expect(rooms[0]?.occupancy).toEqual({ percent: 0.03, color: "#5cb85c" });
    expect(rooms[1]?.occupancy).toBeNull();
    expect(rooms[2]?.occupancy).toBeNull();
    expect(rooms[4]?.occupancy).toEqual({ percent: 0, color: "#5cb85c" });
  });

  it("degrades gracefully on malformed input", () => {
    expect(parseIrisRooms(null)).toEqual([]);
    expect(parseIrisRooms({})).toEqual([]);
    expect(parseIrisRooms({ raeume: "nope" })).toEqual([]);
    expect(parseIrisRooms("garbage")).toEqual([]);
  });

  it("skips rooms missing a key field rather than emitting partial rows", () => {
    const rooms = parseIrisRooms({
      raeume: [
        { raum_name: "no arch name", gebaeude_code: "5504", status: "frei" },
        { raum_nr_architekt: "9@5504", gebaeude_code: "5504", status: "frei" },
      ],
    });
    expect(rooms.map((r) => r.archName)).toEqual(["9@5504"]);
  });
});

describe("roomsForBuildings", () => {
  it("keeps only the rooms Iris attributes to the buildings", () => {
    const rooms = parseIrisRooms(IRIS_FIXTURE);
    expect(roomsForBuildings(rooms, ["8102"]).map((r) => r.archName)).toEqual([
      "BC2 0.01.18@8102",
      "0.01.19@8102",
    ]);
    expect(roomsForBuildings(rooms, ["0000"])).toEqual([]);
  });

  it("unions rooms across several buildings (a joined building's fingers)", () => {
    const rooms = parseIrisRooms(IRIS_FIXTURE);
    expect(roomsForBuildings(rooms, ["5504", "8102"]).map((r) => r.buildingId)).toEqual([
      "5504",
      "8102",
      "8102",
    ]);
    expect(roomsForBuildings(rooms, [])).toEqual([]);
  });
});

describe("buildRoomRows (alias join)", () => {
  it("keeps resolved rooms in order, omitting unmatched (null) and unresolved (absent) aliases", () => {
    const rooms = parseIrisRooms(IRIS_FIXTURE);
    const resolved = new Map<string, string | null>([
      ["1450@5504", "/room/5504.01.450"],
      ["D 5@4113", null], // looked up, no NavigaTUM match → omit.
      ["U1@5532", "/room/5532.EG.U1"],
      // "BC2 0.01.18@8102" and "0.01.19@8102" are absent → not yet resolved → omit.
    ]);
    const rows = buildRoomRows(rooms, resolved);
    expect(rows.map((r) => r.archName)).toEqual(["1450@5504", "U1@5532"]);
    expect(rows[0]?.path).toBe("/room/5504.01.450");
  });

  it("returns nothing when no alias resolved", () => {
    const rooms = parseIrisRooms(IRIS_FIXTURE);
    expect(buildRoomRows(rooms, new Map())).toEqual([]);
  });
});

describe("display helpers", () => {
  it("rounds occupancy to a whole percentage", () => {
    expect(occupancyPercent({ percent: 0.75, color: "#000" })).toBe(75);
    expect(occupancyPercent({ percent: 0.3, color: "#000" })).toBe(30);
    expect(occupancyPercent({ percent: 0, color: "#000" })).toBe(0);
  });

  it("extracts a padded HH:MM from an Iris timestamp, falling back to the raw value", () => {
    expect(bookedUntilTime("2026-06-06 19:28:40")).toBe("19:28");
    expect(bookedUntilTime("2026-06-06 9:05:00")).toBe("09:05");
    expect(bookedUntilTime("sometime")).toBe("sometime");
  });

  it("recognizes the four documented statuses and nothing else", () => {
    expect(isKnownStatus("frei")).toBe(true);
    expect(isKnownStatus("WAAS")).toBe(true);
    expect(isKnownStatus("reserviert")).toBe(false);
  });
});
