// Allergen, ingredient, and certification labels emitted by the TUM-Dev eat-api in their
// `enum_name` form. The JSON next to this module mirrors `enums/labels.json` from
// `https://tum-dev.github.io/eat-api/` verbatim so a refresh is a literal file copy rather
// than a hand-merge. Vite inlines the JSON at build time, so visitors do not pay a
// 3rd-party round-trip (which would also need the privacy disclosure).
//
// Unknown codes degrade gracefully to their raw upstream form.

import labelsJson from "~/data/eat_api_labels.json";

interface RawLabel {
  readonly enum_name: string;
  readonly text: { readonly DE: string; readonly EN: string };
  readonly abbreviation: string;
}

export type EatApiLocale = "de" | "en";

// Keyed by the lower-cased `enum_name`: upstream's dictionary uses `GLUTEN`, but our API serializes
// the same labels as lower-case snake_case (`gluten`), so we normalize both sides to bridge them.
const BY_CODE: ReadonlyMap<string, RawLabel> = new Map(
  (labelsJson as readonly RawLabel[]).map((label) => [label.enum_name.toLowerCase(), label])
);

/** Resolve a label code to its localized text, falling back to the raw code when unknown. */
export function labelText(code: string, locale: EatApiLocale): string {
  const entry = BY_CODE.get(code.toLowerCase());
  if (!entry) return code;
  return locale === "de" ? entry.text.DE : entry.text.EN;
}

/** Resolve a label code to upstream's short abbreviation, or the raw code when unknown. */
export function labelAbbreviation(code: string): string {
  return BY_CODE.get(code.toLowerCase())?.abbreviation ?? code;
}
