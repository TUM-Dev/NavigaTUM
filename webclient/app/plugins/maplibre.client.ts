import { setWorkerUrl } from "maplibre-gl";
import maplibreWorkerUrl from "maplibre-gl/dist/maplibre-gl-worker.mjs?worker&url";

export default defineNuxtPlugin(() => {
  setWorkerUrl(maplibreWorkerUrl);
});
