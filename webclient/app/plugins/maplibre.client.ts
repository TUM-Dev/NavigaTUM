// MapLibre GL v6 loads its worker via `new URL('./maplibre-gl-worker.mjs', import.meta.url)`.
// Bundlers (Vite/Rollup) do not statically pick this pattern up, so the worker file is never
// emitted as an asset and the fetch 404s at runtime - the map renders the attribution but no
// tiles. Pin the worker URL through Vite's `?url` import so the bundler emits the asset and
// returns its public URL.
import { setWorkerUrl } from "maplibre-gl";
import maplibreWorkerUrl from "maplibre-gl/dist/maplibre-gl-worker.mjs?url";

export default defineNuxtPlugin(() => {
  setWorkerUrl(maplibreWorkerUrl);
});
