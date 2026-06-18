import { describe, expect, it } from "vitest";
import type { components } from "../app/api_types";
import {
  allergenLabels,
  dietMarker,
  formatEuro,
  groupDishesByCategory,
  isMensaPriceRole,
  MENSA_PRICE_ROLES,
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

describe("isMensaPriceRole", () => {
  it("accepts the known roles and rejects anything else", () => {
    for (const role of MENSA_PRICE_ROLES) expect(isMensaPriceRole(role)).toBe(true);
    expect(isMensaPriceRole("teacher")).toBe(false);
    expect(isMensaPriceRole("")).toBe(false);
  });
});

describe("formatEuro", () => {
  it("formats with the locale-appropriate symbol placement", () => {
    expect(formatEuro(3.5, "de")).toBe("3,50 €");
    expect(formatEuro(3.5, "en")).toBe("€3.50");
  });
});
