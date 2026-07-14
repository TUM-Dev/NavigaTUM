// MapLibre v6 initialises WebGL2 inside the `Map` constructor and, when it fails, synchronously fires
// and logs a `GPUInitializationError` before any `map.on("error")` handler can exist. Probe up-front
// so callers skip building (and logging errors for) a doomed map; the listener covers a runtime loss.
import { GPUInitializationError, type Map as MapLibreMap } from "maplibre-gl";

export interface WebglGuard {
  readonly supported: Readonly<Ref<boolean>>;
  attach(map: MapLibreMap): void;
}

function hasWebgl2(): boolean {
  return document.createElement("canvas").getContext("webgl2") !== null;
}

export function useWebglGuard(): WebglGuard {
  const supported = ref(import.meta.client ? hasWebgl2() : true);
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
