import { mdiFoodVariant } from "@mdi/js";
import { describe, expect, it } from "vitest";
import type { components } from "../app/api_types";
import {
  allergenIcon,
  allergenLabels,
  dietMarker,
  formatEuro,
  groupAllergensByIcon,
  groupDishesByCategory,
  isMensaPriceRole,
  isSelectableAllergen,
  MENSA_PRICE_ROLES,
  matchedAllergens,
} from "../app/utils/mensaMenu";

type MenuDish = components["schemas"]["MensaMenuDishResponse"];

function dish(overrides: Partial<MenuDish> = {}): MenuDish {
  return { name: "Dish", labels: [], prices: {}, ...overrides };
}

describe("dietMarker", () => {
  it("prefers vegan over the vegetarian/meat labels upstream also attaches", () => {
    expect(dietMarker(["vegetarian", "vegan", "milk"])?.kind).toBe("vegan");
  });

  it("falls back to vegetarian before any meat signal", () => {
    expect(dietMarker(["vegetarian", "gluten"])?.kind).toBe("vegetarian");
  });

  it("reads as the specific meat, not the generic `meat`, when both are present", () => {
    const marker = dietMarker(["meat", "pork"]);
    expect(marker?.kind).toBe("meat");
    expect(marker?.labelCode).toBe("pork");
  });

  it("treats fish as its own marker ahead of generic meat", () => {
    expect(dietMarker(["fish", "meat"])?.kind).toBe("fish");
  });

  it("returns null when upstream gave no diet label", () => {
    expect(dietMarker(["gluten", "milk"])).toBeNull();
  });
});

describe("allergenLabels", () => {
  it("drops the labels already shown as the diet marker", () => {
    expect(allergenLabels(["vegan", "gluten", "pork", "milk"])).toEqual(["gluten", "milk"]);
  });

  it("keeps additives and certifications that are not diet markers", () => {
    expect(allergenLabels(["preservatives", "msc"])).toEqual(["preservatives", "msc"]);
  });
});

describe("groupDishesByCategory", () => {
  it("keeps categories in first-appearance order and groups repeats", () => {
    const groups = groupDishesByCategory([
      dish({ dish_type: "Pasta", name: "A" }),
      dish({ dish_type: "Suppe", name: "B" }),
      dish({ dish_type: "Pasta", name: "C" }),
    ]);
    expect(groups.map((g) => g.category)).toEqual(["Pasta", "Suppe"]);
    expect(groups[0]?.dishes.map((d) => d.name)).toEqual(["A", "C"]);
  });

  it("buckets blank or missing dish_type under a null category", () => {
    const groups = groupDishesByCategory([
      dish({ name: "A" }),
      dish({ dish_type: "  ", name: "B" }),
    ]);
    expect(groups).toHaveLength(1);
    expect(groups[0]?.category).toBeNull();
    expect(groups[0]?.dishes).toHaveLength(2);
  });
});

describe("allergenIcon", () => {
  it("gives every member of a family the same glyph", () => {
    expect(allergenIcon("wheat")).toBe(allergenIcon("barley"));
    expect(allergenIcon("almonds")).toBe(allergenIcon("cashews"));
  });

  it("normalizes casing", () => {
    expect(allergenIcon("MILK")).toBe(allergenIcon("milk"));
  });

  it("falls back to a neutral food glyph for unmapped codes", () => {
    expect(allergenIcon("definitely_not_a_label")).toBe(mdiFoodVariant);
  });
});

describe("groupAllergensByIcon", () => {
  it("merges codes that share an icon into a single row", () => {
    const rows = groupAllergensByIcon(["milk", "lactose"]);
    expect(rows).toHaveLength(1);
    expect(rows[0]?.codes).toEqual(["milk", "lactose"]);
  });

  it("keeps distinct icons as separate rows in first-appearance order", () => {
    const rows = groupAllergensByIcon(["wheat", "milk", "gluten"]);
    expect(rows.map((r) => r.codes)).toEqual([["wheat", "gluten"], ["milk"]]);
  });
});

describe("isSelectableAllergen", () => {
  it("accepts every granular allergen and rejects additives, diet, or unknown codes", () => {
    expect(isSelectableAllergen("gluten")).toBe(true);
    expect(isSelectableAllergen("wheat")).toBe(true);
    expect(isSelectableAllergen("lactose")).toBe(true);
    expect(isSelectableAllergen("preservatives")).toBe(false);
    expect(isSelectableAllergen("pork")).toBe(false);
    expect(isSelectableAllergen("")).toBe(false);
  });
});

describe("matchedAllergens", () => {
  it("matches each flagged code directly against the dish labels", () => {
    expect(matchedAllergens(["wheat", "gluten"], ["wheat"])).toEqual(["wheat"]);
    expect(matchedAllergens(["cashews"], ["cashews"])).toEqual(["cashews"]);
  });

  it("does not expand an umbrella to its grains - selection is code-for-code", () => {
    expect(matchedAllergens(["wheat"], ["gluten"])).toEqual([]);
  });

  it("matches diet-promoted labels such as fish", () => {
    expect(matchedAllergens(["fish", "vegan"], ["fish"])).toEqual(["fish"]);
  });

  it("returns matches in canonical order regardless of label or selection order", () => {
    expect(matchedAllergens(["milk", "peanuts"], ["peanuts", "milk"])).toEqual(["peanuts", "milk"]);
  });

  it("is empty when nothing is selected or nothing matches", () => {
    expect(matchedAllergens(["wheat"], [])).toEqual([]);
    expect(matchedAllergens(["soy"], ["milk"])).toEqual([]);
  });

  it("normalizes label casing before matching", () => {
    expect(matchedAllergens(["WHEAT"], ["wheat"])).toEqual(["wheat"]);
  });
});

describe("isMensaPriceRole", () => {
  it("accepts the known roles and rejects anything else", () => {
    for (const role of MENSA_PRICE_ROLES) expect(isMensaPriceRole(role)).toBe(true);
    expect(isMensaPriceRole("teacher")).toBe(false);
    expect(isMensaPriceRole("")).toBe(false);
  });
});

describe("formatEuro", () => {
  it("formats with the locale-appropriate symbol placement", () => {
    const normalizeSpaces = (value: string) => value.replace(/\s/g, " ");
    expect(normalizeSpaces(formatEuro(3.5, "de"))).toBe("3,50 €");
    expect(normalizeSpaces(formatEuro(3.5, "en"))).toBe("€3.50");
  });
});
