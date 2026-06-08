import type { ComputedRef } from "vue";
import type { components } from "~/api_types";
import {
  buildVisibleSearchEntries,
  collapsedHighlightTarget,
  collapsedUpwardHighlightTarget,
  findLectureHeaderIndex,
  type LectureNavController,
  toggleLectureFromMouse,
  type VisibleSearchEntry,
} from "~/utils/lectureRow";

type ResultsSection = components["schemas"]["ResultsSection"];

// Keyboard-navigable dropdown state for AppSearchBar. Owns the four ref Sets
// that drive the flattened visibleElements list, plus the per-key actions
// (ArrowDown / ArrowUp) whose ordering is subtle - re-expand on landing, but
// not when the wrap brings us back to the same lecture's header. The
// LectureNavController is exposed so AppSearchBar can `provide()` it to the
// LectureSearchResultRow descendants without prop-drilling.
export function useSearchDropdownNav(sections: ComputedRef<readonly ResultsSection[] | undefined>) {
  const expandedFacets = ref<Set<string>>(new Set());
  // Sticky for the dropdown session so ArrowUp back into a lecture doesn't
  // recollapse it and shift every downstream highlight index.
  const expandedLectures = ref<Set<string>>(new Set());
  const lectureShowAll = ref<Set<string>>(new Set());
  const highlighted = ref<number | undefined>(undefined);

  const visibleElements = computed<VisibleSearchEntry[]>(() =>
    sections.value
      ? buildVisibleSearchEntries(sections.value, {
          expandedFacets: expandedFacets.value,
          expandedLectures: expandedLectures.value,
          lectureShowAll: lectureShowAll.value,
        })
      : []
  );

  const highlightedEntry = computed<VisibleSearchEntry | undefined>(() =>
    highlighted.value === undefined ? undefined : visibleElements.value[highlighted.value]
  );

  function expandHighlightedLecture(): void {
    const current = highlightedEntry.value;
    if (current?.kind !== "result") return;
    if (current.entry.type !== "lecture") return;
    if (expandedLectures.value.has(current.entry.id)) return;
    expandedLectures.value = new Set([...expandedLectures.value, current.entry.id]);
  }

  function revealMoreEvents(lectureId: string): void {
    if (lectureShowAll.value.has(lectureId)) return;
    lectureShowAll.value = new Set([...lectureShowAll.value, lectureId]);
  }

  function collapseLecturePastShowMore(lectureId: string): void {
    if (highlighted.value === undefined) return;
    const target = collapsedHighlightTarget(visibleElements.value, highlighted.value, lectureId);
    if (expandedLectures.value.has(lectureId)) {
      const next = new Set(expandedLectures.value);
      next.delete(lectureId);
      expandedLectures.value = next;
    }
    highlighted.value = target;
  }

  function collapseLectureOverTheTop(lectureId: string): void {
    if (highlighted.value === undefined) return;
    const oldIdx = highlighted.value;
    if (expandedLectures.value.has(lectureId)) {
      const next = new Set(expandedLectures.value);
      next.delete(lectureId);
      expandedLectures.value = next;
    }
    // Recompute against the post-collapse list so a wrap lands on the new tail.
    highlighted.value = collapsedUpwardHighlightTarget(oldIdx, visibleElements.value.length);
  }

  function arrowDown(): void {
    if (visibleElements.value.length === 0) return;
    const current = highlightedEntry.value;
    if (current?.kind === "show_more_events") {
      const collapsedId = current.lectureId;
      collapseLecturePastShowMore(collapsedId);
      // Don't re-expand if wrap landed back on the same lecture's header.
      const newCurrent = highlightedEntry.value;
      const sameLecture =
        newCurrent?.kind === "result" &&
        newCurrent.entry.type === "lecture" &&
        newCurrent.entry.id === collapsedId;
      if (!sameLecture) expandHighlightedLecture();
      return;
    }
    highlighted.value =
      highlighted.value === undefined ? 0 : (highlighted.value + 1) % visibleElements.value.length;
    expandHighlightedLecture();
  }

  function arrowUp(): void {
    if (visibleElements.value.length === 0) {
      highlighted.value = undefined;
      return;
    }
    const current = highlightedEntry.value;
    if (
      current?.kind === "result" &&
      current.entry.type === "lecture" &&
      expandedLectures.value.has(current.entry.id)
    ) {
      const collapsedId = current.entry.id;
      collapseLectureOverTheTop(collapsedId);
      const newCurrent = highlightedEntry.value;
      const sameLecture =
        newCurrent?.kind === "result" &&
        newCurrent.entry.type === "lecture" &&
        newCurrent.entry.id === collapsedId;
      if (!sameLecture) expandHighlightedLecture();
      return;
    }
    if (highlighted.value === 0 || highlighted.value === undefined) {
      highlighted.value = visibleElements.value.length - 1;
    } else {
      highlighted.value -= 1;
    }
    expandHighlightedLecture();
  }

  function expandFacet(facet: string): void {
    expandedFacets.value = new Set([...expandedFacets.value, facet]);
  }

  function resetAll(): void {
    expandedLectures.value = new Set();
    lectureShowAll.value = new Set();
    highlighted.value = undefined;
  }

  function clearLectureExpansion(): void {
    expandedLectures.value = new Set();
    lectureShowAll.value = new Set();
  }

  const lectureNav: LectureNavController = {
    expanded: (id) => expandedLectures.value.has(id),
    showAll: (id) => lectureShowAll.value.has(id),
    highlightedEventIndex: (id) => {
      const current = highlightedEntry.value;
      if (current?.kind === "event" && current.lectureId === id) return current.eventIndex;
      return null;
    },
    showMoreHighlighted: (id) => {
      const current = highlightedEntry.value;
      return current?.kind === "show_more_events" && current.lectureId === id;
    },
    toggle: (id) => {
      const next = toggleLectureFromMouse(
        {
          expandedFacets: expandedFacets.value,
          expandedLectures: expandedLectures.value,
          lectureShowAll: lectureShowAll.value,
        },
        id
      );
      expandedLectures.value = new Set(next.expandedLectures);
      lectureShowAll.value = new Set(next.lectureShowAll);
      // Snap keyboard state to the just-toggled row so ArrowDown picks up
      // where the mouse left off; visibleElements has recomputed against the
      // new sets by the time we read it here.
      const headerIdx = findLectureHeaderIndex(visibleElements.value, id);
      highlighted.value = headerIdx >= 0 ? headerIdx : undefined;
    },
    revealMore: revealMoreEvents,
  };

  return {
    expandedFacets,
    highlighted,
    visibleElements,
    highlightedEntry,
    lectureNav,
    arrowDown,
    arrowUp,
    expandFacet,
    revealMoreEvents,
    resetAll,
    clearLectureExpansion,
  };
}
