import type { SearchResponse } from "@/codegen";

function getVisibleElements() {
  const visible: string[] = [];

  navigatum.app.search.autocomplete.sections.forEach((section) => {
    section.entries.forEach((entry, index: number) => {
      if (
        section.n_visible === undefined ||
        index < section.n_visible ||
        section.expanded
      )
        visible.push(entry.id);
    });
  });
  return visible;
}

function _allowHighlighting(text: string) {
  /// This function does still parse content only from our internal API (which should not try to pawn us in the
  // first place), but for extra redundancy we sanitise this anyway.
  // It is not done by Vue, as we use `v-html`-Tag to include it in the frontend.
  const opt = new Option(text).innerHTML;
  return opt.replaceAll("\x19", "<em>").replaceAll("\x17", "</em>");
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

export function extractFacets(data: SearchResponse) {
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
        name: $t("search.sections.buildings "),
        expanded: false,
        entries: entries,
        estimatedTotalHits: section.estimatedTotalHits,
        n_visible: section.n_visible,
      });
    } else if (section.facet === "rooms") {
      sections.push({
        name: $t("search.sections.rooms "),
        entries: entries,
        estimatedTotalHits: section.estimatedTotalHits,
      });
    }
  });

  return sections;
}

// As a simple measure against out-of-order responses
// to the autocompletion, we count queries and make sure
// that late results will not overwrite the currently
// visible results.
let queryCounter = 0;
let latestUsedQueryId: number | null = null;

export function onInput(q: string) {
  navigatum.app.search.autocomplete.highlighted = null;

  if (q.length === 0) {
    navigatum.app.search.autocomplete.sections = [];
  } else {
    const queryId = queryCounter;
    queryCounter += 1;

    /* global cachedFetch */
    // no-cache instructs browser, because the cachedFetch will store the reponse.
    fetch(
      `/api/search?q=${window.encodeURIComponent(q)}`
    ).then((data) => {
      // Data will be cached anyway in case the user hits backspace,
      // but we need to discard the data here if it arrived out of order.
      if (!latestUsedQueryId || queryId > latestUsedQueryId) {
        latestUsedQueryId = queryId;

        navigatum.app.search.autocomplete.sections = extractFacets(data);
      }
    });
  }
}
export function onKeyDown(e) {
  let visible;
  let index;
  switch (e.keyCode) {
    case 27: // ESC
      document.getElementById("search")?.blur();
      break;

    case 40: // Arrow down
      visible = getVisibleElements();
      index = visible.indexOf(navigatum.app.search.autocomplete.highlighted);
      if (index === -1 && visible.length > 0) {
        navigatum.app.search.autocomplete.highlighted = visible[0];
      } else if (index >= 0 && index < visible.length - 1) {
        navigatum.app.search.autocomplete.highlighted = visible[index + 1];
      }
      e.preventDefault();
      break;

    case 38: // Arrow up
      visible = getVisibleElements();
      index = visible.indexOf(navigatum.app.search.autocomplete.highlighted);
      if (index === 0) {
        navigatum.app.search.autocomplete.highlighted = null;
      } else if (index > 0) {
        navigatum.app.search.autocomplete.highlighted = visible[index - 1];
      }
      e.preventDefault();
      break;

    case 13: // Enter
      if (navigatum.app.search.autocomplete.highlighted !== null) {
        navigatum.app.searchGoTo(
          navigatum.app.search.autocomplete.highlighted,
          true
        );
      } else {
        navigatum.app.searchGo(false);
      }
      break;
    default:
      break;
  }
}
