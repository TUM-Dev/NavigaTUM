import type { components } from "@/api_types";
type SearchResponse = components["schemas"]["SearchResponse"];

function _allowHighlighting(text: string) {
  /// This function does still parse content only from our internal API (which should not try to pawn us in the
  // first place), but for extra redundancy we sanitise this anyway.
  // It is not done by Vue, as we use `v-html`-Tag to include it in the frontend.
  const opt = new Option(text).innerHTML;
  return opt.replace("\x19", "<em>").replaceAll("\x17", "</em>");
}

export type SectionFacet = RoomFacet | SiteBuildingFacet;
type RoomFacet = {
  name: string;
  entries: EntryFacet[];
  estimatedTotalHits: number;
};
type SiteBuildingFacet = RoomFacet & { expanded: false; n_visible: number };
type EntryFacet = {
  id: string;
  name: string;
  type: string;
  subtext: string;
  subtext_bold: string;
  parsed_id: string;
};

export function extractFacets(data: SearchResponse, t) {
  const sections: SectionFacet[] = [];

  data.sections.forEach((section) => {
    const entries: EntryFacet[] = [];

    section.entries.forEach((entry) => {
      entries.push({
        id: entry.id,
        name: _allowHighlighting(entry.name), // we explicitly dont let vue sanitise this text
        type: entry.type,
        subtext: entry.subtext,
        subtext_bold: _allowHighlighting(entry.subtext_bold), // we explicitly dont let vue sanitise this text
        parsed_id: _allowHighlighting(entry.parsed_id), // we explicitly dont let vue sanitise this text
      });
    });
    if (section.facet === "sites_buildings") {
      sections.push({
        name: t("search.sections.buildings"),
        expanded: false,
        entries: entries,
        estimatedTotalHits: section.estimatedTotalHits,
        n_visible: section.n_visible,
      });
    } else if (section.facet === "rooms") {
      sections.push({
        name: t("search.sections.rooms"),
        entries: entries,
        estimatedTotalHits: section.estimatedTotalHits,
      });
    }
  });

  return sections;
}
