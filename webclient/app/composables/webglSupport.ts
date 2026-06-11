// MapLibre GL v6 surfaces unavailable GPU contexts (no WebGL2 etc.) through the map's `error`
// event as a `GPUInitializationError`, instead of letting consumers probe support up-front. We
// keep an optimistic ref, attach to a constructed map, and flip the ref on the first such error
// so the calling component can render the not-supported fallback.
import { GPUInitializationError, type Map as MapLibreMap } from "maplibre-gl";

export interface WebglGuard {
  /** Optimistic; flips to false once a `GPUInitializationError` is observed on the map. */
  readonly supported: Readonly<Ref<boolean>>;
  /** Wire up the guard against a freshly-constructed map. Safe to call once per map. */
  attach(map: MapLibreMap): void;
}

export function useWebglGuard(): WebglGuard {
  const supported = ref(true);
  return {
    supported,
    attach(map) {
      const onError = (event: { error: unknown }): void => {
        if (event.error instanceof GPUInitializationError) {
          supported.value = false;
          map.off("error", onError);
        }
      };
      map.on("error", onError);
    },
  };
}
