import type { components } from "~/api_types";

type SearchResponse = components["schemas"]["SearchResponse"];

function _allowHighlighting(text: string): string {
  /// This function does still parse content only from our internal API (which should not try to pawn us in the
  // first place), but for extra redundancy we sanitise this anyway.
  // It is not done by Vue, as we use `v-html`-Tag to include it in the frontend.
  const opt = new Option(text).innerHTML;
  return opt.replaceAll("\x19", "<b class='text-blue-500'>").replaceAll("\x17", "</b>");
}

export interface SectionFacet {
  facet: "sites" | "buildings" | "rooms" | "pois";
  name: string;
  entries: EntryFacet[];
  estimatedTotalHits: number;
  expanded: boolean;
  n_visible: number;
}
interface EntryFacet {
  id: string;
  name: string;
  type: string;
  subtext: string;
  subtext_bold: string | null;
  parsed_id: string | null;
}

export function extractFacets(
  data: SearchResponse,
  labels: Record<SectionFacet["facet"], string>
): SectionFacet[] {
  const sections: SectionFacet[] = [];

  for (const section of data.sections) {
    if (
      section.facet !== "sites" &&
      section.facet !== "buildings" &&
      section.facet !== "rooms" &&
      section.facet !== "pois"
    ) {
      continue;
    }
    const entries: EntryFacet[] = section.entries.map((entry) => ({
      id: entry.id,
      name: _allowHighlighting(entry.name),
      type: entry.type,
      subtext: entry.subtext,
      subtext_bold: _allowHighlighting(entry.subtext_bold || ""),
      parsed_id: _allowHighlighting(entry.parsed_id || ""),
    }));
    sections.push({
      facet: section.facet,
      name: labels[section.facet],
      entries,
      estimatedTotalHits: section.estimatedTotalHits,
      expanded: false,
      n_visible: section.n_visible || entries.length,
    });
  }

  return sections;
}
