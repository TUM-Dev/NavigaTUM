import type { ComputedRef, Ref } from "vue";
import { allValues, firstOrDefault } from "~/composables/common";

type FilterKind = "in" | "usage" | "type" | "near";
type ListKind = "in" | "usage" | "type";

// The backend's `facet` field already buckets indexed types into these five
// categories; the frontend uses them directly as the user-facing taxonomy.
// Subtypes (`virtual_room`, `joined_building`, `campus`, `area`) are an
// internal detail of the data pipeline and not surfaced to users.
export const FACET_OPTIONS = ["site", "building", "room", "poi", "lecture"] as const;
export type Facet = (typeof FACET_OPTIONS)[number];

export interface SearchFilters {
  inFilter: Readonly<Ref<readonly string[]>> | ComputedRef<readonly string[]>;
  usageFilter: Readonly<Ref<readonly string[]>> | ComputedRef<readonly string[]>;
  typeFilter: Readonly<Ref<readonly string[]>> | ComputedRef<readonly string[]>;
  nearFilter: Readonly<Ref<string>> | ComputedRef<string>;
  hasActiveFilters: ComputedRef<boolean>;
  buildQueryObject: () => Record<string, string | string[]>;
  appendToParams: (params: URLSearchParams) => void;
  removeFilter: (kind: FilterKind, value?: string) => void;
  clearAll: () => void;
  toggleFilterValue: (kind: "type" | "usage", value: string) => void;
  addInFilter: (value: string) => void;
  setNear: (enabled: boolean) => void;
}

// The full set of filter values at one instant. Mutations are expressed as a
// partial snapshot handed to the backing store's `commit`.
interface FilterSnapshot {
  in: readonly string[];
  usage: readonly string[];
  type: readonly string[];
  near: string;
}

// The only real difference between the page and the dropdown is where filter
// state lives and how it is written: the page backs it by the URL (mutations go
// through the router), the dropdown stages it in local refs until submit. A
// backing exposes the four reactive values plus a `commit` that persists a
// partial update; the mutator logic on top is identical for both.
interface FilterBacking {
  inFilter: Readonly<Ref<readonly string[]>> | ComputedRef<readonly string[]>;
  usageFilter: Readonly<Ref<readonly string[]>> | ComputedRef<readonly string[]>;
  typeFilter: Readonly<Ref<readonly string[]>> | ComputedRef<readonly string[]>;
  nearFilter: Readonly<Ref<string>> | ComputedRef<string>;
  commit: (next: Partial<FilterSnapshot>) => void;
}

function activateNearFilter(setter: (coords: string) => void) {
  const geo = useSharedGeolocation();
  if (geo.value.userLocation) {
    setter(`${geo.value.userLocation.lat},${geo.value.userLocation.lon}`);
    return;
  }
  if (typeof navigator !== "undefined" && navigator.geolocation) {
    navigator.geolocation.getCurrentPosition(
      (pos) => {
        geo.value.userLocation = { lat: pos.coords.latitude, lon: pos.coords.longitude };
        setter(`${pos.coords.latitude},${pos.coords.longitude}`);
      },
      () => {
        // Geolocation denied or unavailable - do nothing
      }
    );
  }
}

function makeFilters(backing: FilterBacking): SearchFilters {
  const { inFilter, usageFilter, typeFilter, nearFilter, commit } = backing;

  const hasActiveFilters = computed(
    () =>
      inFilter.value.length > 0 ||
      usageFilter.value.length > 0 ||
      typeFilter.value.length > 0 ||
      nearFilter.value !== ""
  );

  function buildQueryObject(): Record<string, string | string[]> {
    const obj: Record<string, string | string[]> = {};
    if (inFilter.value.length) obj.in = [...inFilter.value];
    if (usageFilter.value.length) obj.usage = [...usageFilter.value];
    if (typeFilter.value.length) obj.type = [...typeFilter.value];
    if (nearFilter.value) obj.near = nearFilter.value;
    return obj;
  }

  function appendToParams(params: URLSearchParams) {
    for (const v of inFilter.value) params.append("in", v);
    for (const v of usageFilter.value) params.append("usage", v);
    for (const v of typeFilter.value) params.append("type", v);
    if (nearFilter.value) params.append("near", nearFilter.value);
  }

  function listFor(kind: ListKind): readonly string[] {
    if (kind === "in") return inFilter.value;
    if (kind === "usage") return usageFilter.value;
    return typeFilter.value;
  }

  function commitList(kind: ListKind, list: string[]) {
    if (kind === "in") commit({ in: list });
    else if (kind === "usage") commit({ usage: list });
    else commit({ type: list });
  }

  function removeFilter(kind: FilterKind, value?: string) {
    if (kind === "near") {
      commit({ near: "" });
      return;
    }
    if (!value) return;
    const current = [...listFor(kind)];
    const idx = current.indexOf(value);
    if (idx === -1) return;
    current.splice(idx, 1);
    commitList(kind, current);
  }

  function clearAll() {
    commit({ in: [], usage: [], type: [], near: "" });
  }

  function toggleFilterValue(kind: "type" | "usage", value: string) {
    const current = [...listFor(kind)];
    const idx = current.indexOf(value);
    if (idx === -1) current.push(value);
    else current.splice(idx, 1);
    commitList(kind, current);
  }

  function addInFilter(value: string) {
    if (inFilter.value.includes(value)) return;
    commit({ in: [...inFilter.value, value] });
  }

  function setNear(enabled: boolean) {
    if (!enabled) {
      if (nearFilter.value) commit({ near: "" });
      return;
    }
    if (nearFilter.value) return;
    activateNearFilter((coords) => commit({ near: coords }));
  }

  return {
    inFilter,
    usageFilter,
    typeFilter,
    nearFilter,
    hasActiveFilters,
    buildQueryObject,
    appendToParams,
    removeFilter,
    clearAll,
    toggleFilterValue,
    addInFilter,
    setNear,
  };
}

// Page filters: state is the URL, so every mutation rewrites the query. Unrelated
// params (`q`, the `limit_*` knobs) are preserved verbatim; only the four filter
// keys are owned here, and empty ones drop out of the URL.
function urlBacking(): FilterBacking {
  const route = useRoute();
  const router = useRouter();

  const inFilter = computed(() => allValues(route.query.in ?? []));
  const usageFilter = computed(() => allValues(route.query.usage ?? []));
  const typeFilter = computed(() => allValues(route.query.type ?? []));
  const nearFilter = computed(() => firstOrDefault(route.query.near, ""));

  function commit(next: Partial<FilterSnapshot>) {
    const snapshot: FilterSnapshot = {
      in: next.in ?? inFilter.value,
      usage: next.usage ?? usageFilter.value,
      type: next.type ?? typeFilter.value,
      near: next.near ?? nearFilter.value,
    };
    const query = { ...route.query };
    delete query.in;
    delete query.usage;
    delete query.type;
    delete query.near;
    if (snapshot.in.length) query.in = [...snapshot.in];
    if (snapshot.usage.length) query.usage = [...snapshot.usage];
    if (snapshot.type.length) query.type = [...snapshot.type];
    if (snapshot.near) query.near = snapshot.near;
    router.replace({ query });
  }

  return { inFilter, usageFilter, typeFilter, nearFilter, commit };
}

// Dropdown filters: state is staged in local refs that the caller commits to the
// URL on submit. The URL is still authoritative, so re-sync whenever it changes
// out from under us (e.g., a chip toggled on the /search page).
function localBacking(): FilterBacking {
  const route = useRoute();

  const inFilter = ref<string[]>(allValues(route.query.in ?? []));
  const usageFilter = ref<string[]>(allValues(route.query.usage ?? []));
  const typeFilter = ref<string[]>(allValues(route.query.type ?? []));
  const nearFilter = ref<string>(firstOrDefault(route.query.near, ""));

  watch(
    () => route.query,
    (q) => {
      inFilter.value = allValues(q.in ?? []);
      usageFilter.value = allValues(q.usage ?? []);
      typeFilter.value = allValues(q.type ?? []);
      nearFilter.value = firstOrDefault(q.near, "");
    }
  );

  function commit(next: Partial<FilterSnapshot>) {
    if (next.in !== undefined) inFilter.value = [...next.in];
    if (next.usage !== undefined) usageFilter.value = [...next.usage];
    if (next.type !== undefined) typeFilter.value = [...next.type];
    if (next.near !== undefined) nearFilter.value = next.near;
  }

  return { inFilter, usageFilter, typeFilter, nearFilter, commit };
}

export function useSearchFilters(): SearchFilters {
  return makeFilters(urlBacking());
}

export function useStagedSearchFilters(): SearchFilters {
  return makeFilters(localBacking());
}
