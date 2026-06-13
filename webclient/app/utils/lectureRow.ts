import { type ComputedRef, computed, type InjectionKey, type Ref, readonly, ref } from "vue";
import type { components } from "~/api_types/index.js";
import type { FilterId } from "~/composables/mapLayers";
import { type EntityPath, entityPath } from "~/utils/entityPath";

type LocationEntry = components["schemas"]["LocationEntry"];
type AddressEntry = components["schemas"]["AddressEntry"];
type LectureEntry = components["schemas"]["LectureEntry"];
type ResultsSection = components["schemas"]["ResultsSection"];
type UpcomingEvent = components["schemas"]["UpcomingEvent"];

/**
 * The discriminator the search API groups result sections by (plural, and
 * includes `addresses`). Distinct from the singular filter taxonomy `Facet`
 * in `searchFilters.ts`.
 */
export type ResultsSectionFacet = ResultsSection["facet"];

/**
 * A single search entry tagged with the `kind` implied by its section's facet.
 *
 * The API groups entries into sections and discriminates on the section's
 * `facet`, not on a per-entry field. We re-attach a `kind` here so a row stays
 * self-describing once it is detached from its section (flattened into the
 * keyboard-navigation list, passed to a row component, ...).
 */
export type SearchResultEntry =
  | ({ readonly kind: "location" } & LocationEntry)
  | ({ readonly kind: "address" } & AddressEntry)
  | ({ readonly kind: "lecture" } & LectureEntry);

/** The lecture variant of a tagged search entry. */
export type LectureResultEntry = Extract<SearchResultEntry, { kind: "lecture" }>;

/** The location variant of a tagged search entry. */
export type LocationResultEntry = Extract<SearchResultEntry, { kind: "location" }>;

/** The Nominatim-address variant of a tagged search entry. */
export type AddressResultEntry = Extract<SearchResultEntry, { kind: "address" }>;

/** Tag a section's entries with the `kind` implied by its facet. */
export function tagSectionEntries(section: ResultsSection): SearchResultEntry[] {
  if (section.facet === "lectures") {
    return section.entries.map((entry) => ({ kind: "lecture", ...entry }));
  }
  if (section.facet === "addresses") {
    return section.entries.map((entry) => ({ kind: "address", ...entry }));
  }
  // Event sections only feed the propose page's AddEventSearch picker, which reads them
  // directly; the general search UIs keep the default-off facet disabled.
  if (section.facet === "events") {
    return [];
  }
  return section.entries.map((entry) => ({ kind: "location", ...entry }));
}

// TUM lectures are scheduled in Europe/Berlin regardless of the visitor's zone.
const TZ = "Europe/Berlin";

export type LectureLocale = "de" | "en";

export function lectureTitle(entry: LectureResultEntry, locale: LectureLocale): string {
  const preferred = locale === "de" ? entry.title_de : entry.title_en;
  const fallback = locale === "de" ? entry.title_en : entry.title_de;
  // Titles are non-null over the wire, but stay defensive against empty/missing.
  return (preferred || "").trim() || (fallback || "").trim() || entry.name;
}

export function firstUpcoming(entry: LectureResultEntry): UpcomingEvent | null {
  const list = entry.upcoming;
  if (!list || list.length === 0) return null;
  return list[0] ?? null;
}

export function lectureEventPath(event: Pick<UpcomingEvent, "room_code">): EntityPath {
  return entityPath(event.room_code, "room");
}

export function formatUpcoming(
  event: Pick<UpcomingEvent, "start_at" | "end_at">,
  locale: LectureLocale
): string {
  const bcp47 = locale === "de" ? "de-DE" : "en-GB";
  const start = new Date(event.start_at);
  const end = new Date(event.end_at);
  const time = new Intl.DateTimeFormat(bcp47, {
    timeZone: TZ,
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
  const date = new Intl.DateTimeFormat(bcp47, {
    timeZone: TZ,
    weekday: "short",
    day: "numeric",
    month: locale === "de" ? "long" : "short",
  });
  if (sameBerlinDay(start, end)) {
    return `${date.format(start)}, ${time.format(start)}-${time.format(end)}`;
  }
  return `${date.format(start)}, ${time.format(start)} - ${date.format(end)}, ${time.format(end)}`;
}

function sameBerlinDay(a: Date, b: Date): boolean {
  const key = new Intl.DateTimeFormat("en-CA", {
    timeZone: TZ,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
  return key.format(a) === key.format(b);
}

export interface LectureRowExpansion {
  readonly expanded: Readonly<Ref<boolean>>;
  readonly isCollapsed: ComputedRef<boolean>;
  toggle(): void;
  collapse(): void;
}

export function useLectureRowExpansion(initial = false): LectureRowExpansion {
  const expanded = ref(initial);
  return {
    expanded: readonly(expanded),
    isCollapsed: computed(() => !expanded.value),
    toggle() {
      expanded.value = !expanded.value;
    },
    collapse() {
      expanded.value = false;
    },
  };
}

// Per-lecture event cap before the show-more stop; keeps recurring lectures from drowning the dropdown.
export const LECTURE_EVENT_NAV_CAP = 3;

export type VisibleSearchEntry =
  | { readonly kind: "category_shortcut"; readonly category: FilterId }
  | {
      readonly kind: "result";
      readonly sectionFacet: ResultsSectionFacet;
      readonly entry: SearchResultEntry;
    }
  | {
      readonly kind: "event";
      readonly lectureId: string;
      readonly eventIndex: number;
      readonly event: UpcomingEvent;
    }
  | { readonly kind: "show_more_events"; readonly lectureId: string; readonly hiddenCount: number };

export interface LectureExpansionState {
  readonly expandedFacets: ReadonlySet<ResultsSectionFacet>;
  readonly expandedLectures: ReadonlySet<string>;
  readonly lectureShowAll: ReadonlySet<string>;
}

export function buildVisibleSearchEntries(
  sections: readonly ResultsSection[],
  state: LectureExpansionState,
  eventNavCap: number = LECTURE_EVENT_NAV_CAP
): VisibleSearchEntry[] {
  const out: VisibleSearchEntry[] = [];
  for (const section of sections) {
    const sectionCap = state.expandedFacets.has(section.facet)
      ? Number.POSITIVE_INFINITY
      : section.n_visible;
    const sectionEntries = tagSectionEntries(section).slice(0, sectionCap);
    for (const entry of sectionEntries) {
      out.push({ kind: "result", sectionFacet: section.facet, entry });
      if (entry.kind !== "lecture") continue;
      if (!state.expandedLectures.has(entry.id)) continue;

      const events = entry.upcoming;
      const cap = state.lectureShowAll.has(entry.id) ? events.length : eventNavCap;
      const visibleEvents = events.slice(0, cap);
      for (let i = 0; i < visibleEvents.length; i++) {
        const event = visibleEvents[i];
        if (event === undefined) continue;
        out.push({ kind: "event", lectureId: entry.id, eventIndex: i, event });
      }
      const hidden = events.length - visibleEvents.length;
      if (hidden > 0) {
        out.push({ kind: "show_more_events", lectureId: entry.id, hiddenCount: hidden });
      }
    }
  }
  return out;
}

export function findLectureHeaderIndex(
  visibleElements: readonly VisibleSearchEntry[],
  lectureId: string
): number {
  return visibleElements.findIndex(
    (entry) =>
      entry.kind === "result" && entry.entry.kind === "lecture" && entry.entry.id === lectureId
  );
}

export function collapsedHighlightTarget(
  visibleElements: readonly VisibleSearchEntry[],
  currentHighlightIndex: number,
  lectureId: string
): number {
  if (currentHighlightIndex === visibleElements.length - 1) return 0;
  const headerIndex = findLectureHeaderIndex(visibleElements, lectureId);
  if (headerIndex < 0) return 0;
  return headerIndex + 1;
}

export function collapsedUpwardHighlightTarget(
  currentHighlightIndex: number,
  postCollapseLength: number
): number {
  if (currentHighlightIndex === 0) return Math.max(0, postCollapseLength - 1);
  return currentHighlightIndex - 1;
}

export interface MouseToggleResult {
  readonly expandedLectures: ReadonlySet<string>;
  readonly lectureShowAll: ReadonlySet<string>;
}

// Mouse expansion flips both gates; the keyboard path opts into the full list separately via revealMore.
export function toggleLectureFromMouse(
  state: LectureExpansionState,
  lectureId: string
): MouseToggleResult {
  const wasExpanded = state.expandedLectures.has(lectureId);
  const expandedLectures = new Set(state.expandedLectures);
  const lectureShowAll = new Set(state.lectureShowAll);
  if (wasExpanded) {
    expandedLectures.delete(lectureId);
    lectureShowAll.delete(lectureId);
  } else {
    expandedLectures.add(lectureId);
    lectureShowAll.add(lectureId);
  }
  return { expandedLectures, lectureShowAll };
}

// Provided by AppSearchBar so rows can read their own slot without prop-drilling; absent on /search.
export interface LectureNavController {
  expanded(id: string): boolean;
  showAll(id: string): boolean;
  highlightedEventIndex(id: string): number | null;
  showMoreHighlighted(id: string): boolean;
  toggle(id: string): void;
  revealMore(id: string): void;
}

export const LectureNavKey: InjectionKey<LectureNavController> = Symbol("LectureNav");
