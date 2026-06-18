// Presentation helpers for the canteen menu card. These keep the diet/allergen split, the
// category grouping, and the price formatting out of the templates so the rendering components
// stay declarative and the logic stays unit-testable.

import { mdiFish, mdiFoodDrumstick, mdiLeaf, mdiSprout } from "@mdi/js";
import type { components } from "~/api_types";

type MenuDish = components["schemas"]["MensaMenuDishResponse"];

/** Price roles eat-api distinguishes, in the order we surface them in the toggle. */
export const MENSA_PRICE_ROLES = ["students", "staff", "guests"] as const;
export type MensaPriceRole = (typeof MENSA_PRICE_ROLES)[number];

export function isMensaPriceRole(value: string): value is MensaPriceRole {
  return (MENSA_PRICE_ROLES as readonly string[]).includes(value);
}

export type DietKind = "vegan" | "vegetarian" | "fish" | "meat";

// Most specific wins: a schnitzel tagged both `meat` and `pork` should read as "Pork", and a
// dish tagged `vegan` is also `vegetarian` upstream but should show the stronger claim.
const MEAT_LABELS = ["pork", "beef", "veal", "lamb", "wild_meat", "poultry", "meat"] as const;

const DIET_ICONS: Record<DietKind, string> = {
  vegan: mdiLeaf,
  vegetarian: mdiSprout,
  fish: mdiFish,
  meat: mdiFoodDrumstick,
};

export interface DietMarker {
  readonly kind: DietKind;
  readonly icon: string;
  /** Upstream label code to localize for the visible text (e.g. `pork`). */
  readonly labelCode: string;
}

/**
 * The single most decision-relevant diet fact for a dish, or `null` when upstream gave no diet
 * label. This is the marker shown prominently; everything else stays in the allergen disclosure.
 */
export function dietMarker(labels: readonly string[]): DietMarker | null {
  if (labels.includes("vegan"))
    return { kind: "vegan", icon: DIET_ICONS.vegan, labelCode: "vegan" };
  if (labels.includes("vegetarian"))
    return { kind: "vegetarian", icon: DIET_ICONS.vegetarian, labelCode: "vegetarian" };
  if (labels.includes("fish")) return { kind: "fish", icon: DIET_ICONS.fish, labelCode: "fish" };
  for (const code of MEAT_LABELS) {
    if (labels.includes(code)) return { kind: "meat", icon: DIET_ICONS.meat, labelCode: code };
  }
  return null;
}

// Labels already carried by the diet marker; dropped from the allergen list so we never say the
// same thing twice (e.g. a fish dish shows the fish marker, not a redundant "fish" allergen chip).
const DIET_LABELS: ReadonlySet<string> = new Set<string>([
  "vegan",
  "vegetarian",
  "fish",
  ...MEAT_LABELS,
]);

/** Allergen, additive, and certification labels — everything not promoted to the diet marker. */
export function allergenLabels(labels: readonly string[]): readonly string[] {
  return labels.filter((code) => !DIET_LABELS.has(code));
}

export interface MensaCategoryGroup {
  /** Upstream `dish_type`, or `null` for dishes upstream left unclassified. */
  readonly category: string | null;
  readonly dishes: readonly MenuDish[];
}

/**
 * Split a day's dishes into the categories upstream serves them in (Pasta, Suppe, Studitopf, …),
 * preserving first-appearance order so the menu reads top-to-bottom as it does on the tray line.
 */
export function groupDishesByCategory(dishes: readonly MenuDish[]): readonly MensaCategoryGroup[] {
  const order: string[] = [];
  const byCategory = new Map<string, MenuDish[]>();
  for (const dish of dishes) {
    const key = dish.dish_type?.trim() || "";
    let bucket = byCategory.get(key);
    if (!bucket) {
      bucket = [];
      byCategory.set(key, bucket);
      order.push(key);
    }
    bucket.push(dish);
  }
  return order.map((key) => ({
    category: key === "" ? null : key,
    dishes: byCategory.get(key) ?? [],
  }));
}

// One formatter per locale; rebuilding Intl.NumberFormat for every dish row is needless work.
const EURO_FORMATTERS = new Map<"de" | "en", Intl.NumberFormat>();

export function formatEuro(amount: number, locale: "de" | "en"): string {
  let formatter = EURO_FORMATTERS.get(locale);
  if (!formatter) {
    formatter = new Intl.NumberFormat(locale === "de" ? "de-DE" : "en-GB", {
      style: "currency",
      currency: "EUR",
    });
    EURO_FORMATTERS.set(locale, formatter);
  }
  return formatter.format(amount);
}
