import type { components } from "@/api_types";
import type { useI18n } from "vue-i18n";
type SearchResponse = components["schemas"]["SearchResponse"];
type RoomEntry = components["schemas"]["RoomEntry"];
type SitesBuildingsEntry = components["schemas"]["SitesBuildingsEntry"];
function _allowHighlighting(text: string): string {
  /// This function does still parse content only from our internal API (which should not try to pawn us in the
  // first place), but for extra redundancy we sanitise this anyway.
  // It is not done by Vue, as we use `v-html`-Tag to include it in the frontend.
  const opt = new Option(text).innerHTML;
  return opt.replaceAll("\x19", "<em>").replaceAll("\x17", "</em>");
}

export type SectionFacet = RoomFacet | SiteBuildingFacet;
type RoomFacet = {
  facet: "rooms";
  name: string;
  entries: EntryFacet[];
  estimatedTotalHits: number;
};
type SiteBuildingFacet = {
  facet: "sites_buildings";
  name: string;
  entries: EntryFacet[];
  estimatedTotalHits: number;
  expanded: boolean;
  n_visible: number;
};
type EntryFacet = {
  id: string;
  name: string;
  type: string;
  subtext: string;
  subtext_bold: string | null;
  parsed_id: string | null;
};

export function extractFacets(data: SearchResponse, t: ReturnType<typeof useI18n>["t"]) {
  const sections: SectionFacet[] = [];

  data.sections.forEach((section) => {
    const entries: EntryFacet[] = [];

    switch (section.facet) {
      case "rooms":
        section.entries.forEach((entry: RoomEntry) => {
          entries.push({
            id: entry.id,
            name: _allowHighlighting(entry.name), // we explicitly dont let vue sanitise this text
            type: entry.type,
            subtext: entry.subtext,
            subtext_bold: _allowHighlighting(entry.subtext_bold), // we explicitly dont let vue sanitise this text
            parsed_id: _allowHighlighting(entry.parsed_id || ""), // we explicitly dont let vue sanitise this text
          });
        });
        sections.push({
          facet: "rooms",
          name: t("search.sections.rooms"),
          entries: entries,
          estimatedTotalHits: section.estimatedTotalHits,
        });
        break;
      case "sites_buildings":
        section.entries.forEach((entry: SitesBuildingsEntry) => {
          entries.push({
            id: entry.id,
            name: _allowHighlighting(entry.name), // we explicitly dont let vue sanitise this text
            type: entry.type,
            subtext: entry.subtext,
            subtext_bold: null,
            parsed_id: null,
          });
        });
        sections.push({
          facet: "sites_buildings",
          name: t("search.sections.buildings"),
          expanded: false,
          entries: entries,
          estimatedTotalHits: section.estimatedTotalHits,
          n_visible: section.n_visible || entries.length,
        });
    }
  });

  return sections;
}
