// Per-variant coverage for the addition-proposal registry. Each kind walks the
// empty -> validate -> build path: the empty seed must be rejected, a known-good draft must
// pass and produce the expected payload, and the obvious failing fields keep their messages.

import { describe, expect, it } from "vitest";
import {
  type AdditionDraft,
  additionRegistry,
  type BuildingDraft,
  buildAddition,
  emptyAdditionDraft,
  isAdditionValid,
  type PoiDraft,
  type RoomDraft,
  validateAddition,
} from "../app/composables/additionSchema";

function validRoom(overrides: Partial<RoomDraft> = {}): RoomDraft {
  return {
    ...additionRegistry.room.empty(),
    id: "5510.01.001",
    parent_id: "5510",
    parent_name: "Maschinenwesen",
    alt_name: "Lecture Hall 1",
    arch_name: "001@5510",
    usage_id: 20,
    coords: { lat: 48.262908, lon: 11.669102, picked: true },
    ...overrides,
  };
}

function validBuilding(overrides: Partial<BuildingDraft> = {}): BuildingDraft {
  return {
    ...additionRegistry.building.empty(),
    id: "5511",
    parent_id: "garching",
    parent_name: "Garching",
    name: "New annex",
    node_kind: "building",
    building_prefixes: ["5511"],
    coords: { lat: 48.262908, lon: 11.669102, picked: true },
    ...overrides,
  };
}

function validPoi(overrides: Partial<PoiDraft> = {}): PoiDraft {
  return {
    ...additionRegistry.poi.empty(),
    id: "garching-cafeteria",
    parent_id: "garching",
    parent_name: "Garching",
    name: "Garching Cafeteria",
    usage_name: "Cafeteria",
    coords: { lat: 48.262908, lon: 11.669102, picked: true },
    ...overrides,
  };
}

describe("emptyAdditionDraft", () => {
  it("starts with no kind selected and no coords picked", () => {
    const draft = emptyAdditionDraft();
    expect(draft.kind).toBeNull();
    expect(draft.id).toBe("");
    expect(draft.coords).toEqual({ lat: 0, lon: 0, picked: false });
  });

  it("treats the kind-less draft as not valid but produces no field errors", () => {
    const draft: AdditionDraft = emptyAdditionDraft();
    expect(isAdditionValid(draft)).toBe(false);
    // No kind means no schema to surface field-level errors against - the UI gates this state
    // with the kind-picker, not with an inline error.
    expect(validateAddition(draft)).toEqual({});
    expect(buildAddition(draft)).toBeNull();
  });
});

describe("room variant", () => {
  it("the empty seed fails validation on every required field", () => {
    const empty = additionRegistry.room.empty();
    const errors = validateAddition(empty);
    expect(errors.parent_id).toBe("error.parent_required");
    expect(errors.alt_name).toBe("error.name_required");
    expect(errors.arch_name).toBe("error.arch_name_required");
    expect(errors.usage_id).toBe("error.usage_required");
    expect(errors["coords.picked"]).toBe("error.coords_required");
    expect(isAdditionValid(empty)).toBe(false);
  });

  it("accepts a well-formed draft and builds the expected payload", () => {
    const draft = validRoom();
    expect(validateAddition(draft)).toEqual({});
    expect(isAdditionValid(draft)).toBe(true);
    const built = buildAddition(draft);
    // The build renames `parent_id` to `parent_building_id` and drops the empty seats block.
    expect(built).toMatchObject({
      kind: "room",
      parent_building_id: "5510",
      alt_name: "Lecture Hall 1",
      arch_name: "001@5510",
      usage_id: 20,
      seats: null,
    });
  });

  it("flags a malformed room id", () => {
    expect(validateAddition(validRoom({ id: "" })).id).toBe("error.id_required");
    expect(validateAddition(validRoom({ id: "5510.01" })).id).toBe("error.id_room_incomplete");
    expect(validateAddition(validRoom({ id: "5510..001" })).id).toBe("error.id_room_incomplete");
    expect(validateAddition(validRoom({ id: "5510.01.001@bad" })).id).toBe("error.id_room_format");
  });

  it("flags a malformed architectural name", () => {
    expect(validateAddition(validRoom({ arch_name: "" })).arch_name).toBe(
      "error.arch_name_required"
    );
    expect(validateAddition(validRoom({ arch_name: "001" })).arch_name).toBe(
      "error.arch_name_format"
    );
  });

  it("carries optional seats and links into the built payload", () => {
    const built = buildAddition(
      validRoom({
        seats: { sitting: 30, standing: 5, wheelchair: 1 },
        room_links: [{ url: "https://example.org", text_de: "Info", text_en: "Info" }],
      })
    );
    expect(built).toMatchObject({
      seats: { sitting: 30, standing: 5, wheelchair: 1 },
      links: [{ url: "https://example.org", text_de: "Info", text_en: "Info" }],
    });
  });
});

describe("building variant", () => {
  it("the empty seed fails validation on every required field", () => {
    const empty = additionRegistry.building.empty();
    const errors = validateAddition(empty);
    expect(errors.id).toBe("error.id_required");
    expect(errors.parent_id).toBe("error.parent_required");
    expect(errors.name).toBe("error.name_required");
    expect(errors.node_kind).toBe("error.node_kind_required");
    expect(errors["coords.picked"]).toBe("error.coords_required");
    expect(isAdditionValid(empty)).toBe(false);
  });

  it("accepts a well-formed single building and builds the expected payload", () => {
    const draft = validBuilding();
    expect(validateAddition(draft)).toEqual({});
    expect(isAdditionValid(draft)).toBe(true);
    expect(buildAddition(draft)).toMatchObject({
      kind: "building",
      parent_id: "garching",
      name: "New annex",
      node_kind: "building",
      building_prefixes: ["5511"],
    });
  });

  it("requires exactly one prefix for kind=building", () => {
    expect(validateAddition(validBuilding({ building_prefixes: [] })).building_prefixes).toBe(
      "error.building_needs_one_prefix"
    );
    expect(
      validateAddition(validBuilding({ building_prefixes: ["5511", "5512"] })).building_prefixes
    ).toBe("error.building_needs_one_prefix");
  });

  it("requires at least two prefixes for kind=joined_building", () => {
    expect(
      validateAddition(validBuilding({ node_kind: "joined_building", building_prefixes: ["5511"] }))
        .building_prefixes
    ).toBe("error.joined_building_needs_multi_prefix");
    expect(
      validateAddition(
        validBuilding({ node_kind: "joined_building", building_prefixes: ["5511", "5512"] })
      )
    ).toEqual({});
  });

  it("rejects a prefix that isn't four digits", () => {
    expect(
      validateAddition(validBuilding({ building_prefixes: ["abc"] }))["building_prefixes.0"]
    ).toBe("error.building_prefix_format");
  });
});

describe("poi variant", () => {
  it("the empty seed fails validation on every required field", () => {
    const empty = additionRegistry.poi.empty();
    const errors = validateAddition(empty);
    expect(errors.id).toBe("error.id_required");
    expect(errors.parent_id).toBe("error.parent_required");
    expect(errors.name).toBe("error.name_required");
    expect(errors.usage_name).toBe("error.usage_name_required");
    expect(errors["coords.picked"]).toBe("error.coords_required");
    expect(isAdditionValid(empty)).toBe(false);
  });

  it("accepts a well-formed draft and builds the expected payload", () => {
    const draft = validPoi();
    expect(validateAddition(draft)).toEqual({});
    expect(isAdditionValid(draft)).toBe(true);
    expect(buildAddition(draft)).toMatchObject({
      kind: "poi",
      parent: "garching",
      name: "Garching Cafeteria",
      usage_name: "Cafeteria",
      comment: null,
    });
  });

  it("rejects malformed POI keys", () => {
    expect(validateAddition(validPoi({ id: "BadCase" })).id).toBe("error.poi_key_format");
    expect(validateAddition(validPoi({ id: "_leading-underscore" })).id).toBe(
      "error.poi_key_format"
    );
    expect(validateAddition(validPoi({ id: "x".repeat(65) })).id).toBe("error.poi_key_too_long");
  });

  it("propagates optional comment, links, and generic_props into the build", () => {
    const built = buildAddition(
      validPoi({
        comment_de: "Hallo",
        comment_en: "Hi",
        poi_links: [{ url: "https://example.org", text_de: "Web", text_en: "Web" }],
        generic_props: [{ name_de: "Sitz", name_en: "Seats", text: "50" }],
      })
    );
    expect(built).toMatchObject({
      comment: { de: "Hallo", en: "Hi" },
      links: [{ url: "https://example.org", text: { de: "Web", en: "Web" } }],
      generic_props: [{ name: { de: "Sitz", en: "Seats" }, text: "50" }],
    });
  });
});
