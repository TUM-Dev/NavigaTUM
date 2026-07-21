import type { ComponentPublicInstance, InjectionKey, Ref } from "vue";

// One highlightable segment of a computed route, addressed the same way by the list and the map.
// Valhalla routes are a single leg of maneuvers; Motis itineraries are legs, each with optional
// self-navigated steps. `stepIndex === null` targets a whole leg.
export type RouteHighlight =
  | { readonly router: "valhalla"; readonly maneuverIndex: number }
  | {
      readonly router: "motis";
      readonly itineraryIndex: number;
      readonly legIndex: number;
      readonly stepIndex: number | null;
    };

// Whether the current highlight was driven by the results list or by the map, so only map-driven
// highlights scroll the matching list card into view (a list hover must not fight the pointer).
export type HighlightOrigin = "list" | "map";

export interface RouteHighlightController {
  readonly hovered: Ref<RouteHighlight | null>;
  readonly selected: Ref<RouteHighlight | null>;
  readonly hoverOrigin: Ref<HighlightOrigin | null>;
  setHover(target: RouteHighlight | null, origin: HighlightOrigin): void;
  setSelected(target: RouteHighlight | null): void;
}

export function sameHighlight(a: RouteHighlight | null, b: RouteHighlight | null): boolean {
  if (!a || !b || a.router !== b.router) return false;
  if (a.router === "valhalla" && b.router === "valhalla")
    return a.maneuverIndex === b.maneuverIndex;
  if (a.router === "motis" && b.router === "motis")
    return (
      a.itineraryIndex === b.itineraryIndex &&
      a.legIndex === b.legIndex &&
      a.stepIndex === b.stepIndex
    );
  return false;
}

// Feature-state id for a Motis step, unique across a drawn itinerary. Legs cap well below 1000, so
// this stays collision-free without a running counter both the map and the list would have to share.
export function motisStepFeatureId(legIndex: number, stepIndex: number): number {
  return legIndex * 1000 + stepIndex;
}

export const routeHighlightKey: InjectionKey<RouteHighlightController> = Symbol("route-highlight");

export function provideRouteHighlight(): RouteHighlightController {
  const hovered = ref<RouteHighlight | null>(null);
  const selected = ref<RouteHighlight | null>(null);
  const hoverOrigin = ref<HighlightOrigin | null>(null);
  const controller: RouteHighlightController = {
    hovered,
    selected,
    hoverOrigin,
    // Map `mousemove` re-fires the same target continuously, so collapse no-op updates here rather
    // than repaint feature-state on every pixel.
    setHover(target, origin) {
      const nextOrigin = target ? origin : null;
      const unchanged =
        (target === null ? hovered.value === null : sameHighlight(hovered.value, target)) &&
        hoverOrigin.value === nextOrigin;
      if (unchanged) return;
      hovered.value = target;
      hoverOrigin.value = nextOrigin;
    },
    setSelected(target) {
      selected.value = target;
    },
  };
  provide(routeHighlightKey, controller);
  return controller;
}

export function useRouteHighlight(): RouteHighlightController {
  const controller = inject(routeHighlightKey);
  if (!controller) throw new Error("useRouteHighlight() called outside a provideRouteHighlight()");
  return controller;
}

// Row-level glue shared by every result list that highlights route segments: report list hover,
// reflect the active segment's emphasis, and scroll a row into view when the map drove the hover.
export function useHighlightRows() {
  const highlight = useRouteHighlight();
  const rows = new Map<string, HTMLElement>();
  const keyOf = (target: RouteHighlight): string => JSON.stringify(target);

  function registerRow(target: RouteHighlight, el: Element | ComponentPublicInstance | null): void {
    const key = keyOf(target);
    if (el instanceof HTMLElement) rows.set(key, el);
    else rows.delete(key);
  }
  function isEmphasised(target: RouteHighlight): boolean {
    return (
      sameHighlight(highlight.hovered.value, target) ||
      sameHighlight(highlight.selected.value, target)
    );
  }
  function hover(target: RouteHighlight | null): void {
    highlight.setHover(target, "list");
  }

  watch(
    () => highlight.hovered.value,
    (target) => {
      if (highlight.hoverOrigin.value !== "map" || !target) return;
      rows.get(keyOf(target))?.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  );

  return { registerRow, isEmphasised, hover };
}

// The Motis results tree carries its itinerary index down so leaf rows can build a full
// `RouteHighlight` without every level re-wrapping an emit.
export const motisItineraryIndexKey: InjectionKey<Ref<number>> = Symbol("motis-itinerary-index");

export function useMotisItineraryIndex(): Ref<number> {
  const index = inject(motisItineraryIndexKey);
  if (!index) throw new Error("useMotisItineraryIndex() called outside a Motis connection card");
  return index;
}
