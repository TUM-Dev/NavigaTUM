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
