import { describe, expect, it } from "vitest";
import { labelAbbreviation, labelText } from "../app/utils/eatApiLabels";

describe("labelText", () => {
  // The dictionary keys are upstream's upper-case `enum_name`, but our API serializes labels as
  // lower-case snake_case — the resolver must bridge that casing or every label falls back untranslated.
  it("resolves the lower-case codes our API emits", () => {
    expect(labelText("gluten", "de")).toBe("Gluten");
    expect(labelText("gluten", "en")).toBe("gluten-containing cereals");
    expect(labelText("chicken_eggs", "de")).toBe("Eier");
  });

  it("translates by locale", () => {
    expect(labelText("fish", "de")).toBe("Fisch");
    expect(labelText("fish", "en")).toBe("fish");
  });

  it("falls back to the raw code when unknown", () => {
    expect(labelText("not_a_label", "de")).toBe("not_a_label");
  });
});

describe("labelAbbreviation", () => {
  it("resolves lower-case codes to the upstream abbreviation", () => {
    expect(labelAbbreviation("gluten")).toBe("Gl");
    expect(labelAbbreviation("fish")).toBe("Fi");
  });
});
