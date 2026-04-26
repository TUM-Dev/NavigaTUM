import type { ComputedRef, Ref } from "vue";
import { allValues, firstOrDefault } from "~/composables/common";

type FilterKind = "in" | "usage" | "type" | "near";
type ListKind = "in" | "usage" | "type";

// The backend's `facet` field already buckets indexed types into these four
// categories; the frontend uses them directly as the user-facing taxonomy.
// Subtypes (`virtual_room`, `joined_building`, `campus`, `area`) are an
// internal detail of the data pipeline and not surfaced to users.
export const FACET_OPTIONS = ["site", "building", "room", "poi"] as const;
export type Facet = (typeof FACET_OPTIONS)[number];

export type SearchFilters = {
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
};

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
        // Geolocation denied or unavailable — do nothing
      }
    );
  }
}

function makeShared(values: {
  inFilter: ComputedRef<string[]> | Ref<string[]>;
  usageFilter: ComputedRef<string[]> | Ref<string[]>;
  typeFilter: ComputedRef<string[]> | Ref<string[]>;
  nearFilter: ComputedRef<string> | Ref<string>;
}): Pick<SearchFilters, "hasActiveFilters" | "buildQueryObject" | "appendToParams"> {
  const hasActiveFilters = computed(
    () =>
      values.inFilter.value.length > 0 ||
      values.usageFilter.value.length > 0 ||
      values.typeFilter.value.length > 0 ||
      values.nearFilter.value !== ""
  );

  function buildQueryObject(): Record<string, string | string[]> {
    const obj: Record<string, string | string[]> = {};
    if (values.inFilter.value.length) obj.in = [...values.inFilter.value];
    if (values.usageFilter.value.length) obj.usage = [...values.usageFilter.value];
    if (values.typeFilter.value.length) obj.type = [...values.typeFilter.value];
    if (values.nearFilter.value) obj.near = values.nearFilter.value;
    return obj;
  }

  function appendToParams(params: URLSearchParams) {
    for (const v of values.inFilter.value) params.append("in", v);
    for (const v of values.usageFilter.value) params.append("usage", v);
    for (const v of values.typeFilter.value) params.append("type", v);
    if (values.nearFilter.value) params.append("near", values.nearFilter.value);
  }

  return { hasActiveFilters, buildQueryObject, appendToParams };
}

export function useSearchFilters(): SearchFilters {
  const route = useRoute();
  const router = useRouter();

  const inFilter = computed(() => allValues(route.query.in ?? []));
  const usageFilter = computed(() => allValues(route.query.usage ?? []));
  const typeFilter = computed(() => allValues(route.query.type ?? []));
  const nearFilter = computed(() => firstOrDefault(route.query.near, ""));

  const shared = makeShared({ inFilter, usageFilter, typeFilter, nearFilter });

  function replaceQuery(updates: Record<string, string | string[] | undefined>) {
    const current: Record<string, string | string[] | undefined> = {
      q: route.query.q as string | undefined,
      limit_buildings: route.query.limit_buildings as string | undefined,
      limit_rooms: route.query.limit_rooms as string | undefined,
      ...shared.buildQueryObject(),
      ...updates,
    };
    for (const key of Object.keys(current)) {
      if (current[key] === undefined) delete current[key];
    }
    router.replace({ query: current });
  }

  function listFor(kind: ListKind): string[] {
    if (kind === "in") return [...inFilter.value];
    if (kind === "usage") return [...usageFilter.value];
    return [...typeFilter.value];
  }

  function removeFilter(kind: FilterKind, value?: string) {
    if (kind === "near") {
      replaceQuery({ near: undefined });
      return;
    }
    if (!value) return;
    const current = listFor(kind);
    const idx = current.indexOf(value);
    if (idx !== -1) current.splice(idx, 1);
    replaceQuery({ [kind]: current.length ? current : undefined });
  }

  function clearAll() {
    replaceQuery({ in: undefined, usage: undefined, type: undefined, near: undefined });
  }

  function toggleFilterValue(kind: "type" | "usage", value: string) {
    const current = listFor(kind);
    const idx = current.indexOf(value);
    if (idx !== -1) current.splice(idx, 1);
    else current.push(value);
    replaceQuery({ [kind]: current.length ? current : undefined });
  }

  function addInFilter(value: string) {
    if (inFilter.value.includes(value)) return;
    replaceQuery({ in: [...inFilter.value, value] });
  }

  function setNear(enabled: boolean) {
    if (!enabled) {
      if (nearFilter.value) replaceQuery({ near: undefined });
      return;
    }
    if (nearFilter.value) return;
    activateNearFilter((coords) => replaceQuery({ near: coords }));
  }

  return {
    inFilter,
    usageFilter,
    typeFilter,
    nearFilter,
    ...shared,
    removeFilter,
    clearAll,
    toggleFilterValue,
    addInFilter,
    setNear,
  };
}

export function useStagedSearchFilters(): SearchFilters {
  const route = useRoute();
  const inFilter = ref<string[]>(allValues(route.query.in ?? []));
  const usageFilter = ref<string[]>(allValues(route.query.usage ?? []));
  const typeFilter = ref<string[]>(allValues(route.query.type ?? []));
  const nearFilter = ref<string>(firstOrDefault(route.query.near, ""));

  // Re-sync when the URL changes (e.g., user toggles a chip on the /search page)
  watch(
    () => route.query,
    (q) => {
      inFilter.value = allValues(q.in ?? []);
      usageFilter.value = allValues(q.usage ?? []);
      typeFilter.value = allValues(q.type ?? []);
      nearFilter.value = firstOrDefault(q.near, "");
    }
  );

  const shared = makeShared({ inFilter, usageFilter, typeFilter, nearFilter });

  function listRef(kind: ListKind): Ref<string[]> {
    if (kind === "in") return inFilter;
    if (kind === "usage") return usageFilter;
    return typeFilter;
  }

  function removeFilter(kind: FilterKind, value?: string) {
    if (kind === "near") {
      nearFilter.value = "";
      return;
    }
    if (!value) return;
    const list = listRef(kind);
    const idx = list.value.indexOf(value);
    if (idx !== -1) {
      const next = [...list.value];
      next.splice(idx, 1);
      list.value = next;
    }
  }

  function clearAll() {
    inFilter.value = [];
    usageFilter.value = [];
    typeFilter.value = [];
    nearFilter.value = "";
  }

  function toggleFilterValue(kind: "type" | "usage", value: string) {
    const list = listRef(kind);
    const idx = list.value.indexOf(value);
    if (idx !== -1) {
      const next = [...list.value];
      next.splice(idx, 1);
      list.value = next;
    } else {
      list.value = [...list.value, value];
    }
  }

  function addInFilter(value: string) {
    if (inFilter.value.includes(value)) return;
    inFilter.value = [...inFilter.value, value];
  }

  function setNear(enabled: boolean) {
    if (!enabled) {
      nearFilter.value = "";
      return;
    }
    if (nearFilter.value) return;
    activateNearFilter((coords) => {
      nearFilter.value = coords;
    });
  }

  return {
    inFilter,
    usageFilter,
    typeFilter,
    nearFilter,
    ...shared,
    removeFilter,
    clearAll,
    toggleFilterValue,
    addInFilter,
    setNear,
  };
}
