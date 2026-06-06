import { tryOnScopeDispose } from "@vueuse/core";
import { IRIS_API_URL, type IrisRoom, parseIrisRooms } from "~/utils/iris";

/**
 * Own the browser-side fetch of the full AStA Iris room roster.
 *
 * The roster is fetched directly from `iris.asta.tum.de` (public, CORS-`*`) and never proxied
 * through the NavigaTUM server. Each {@link refresh} aborts a still-pending one so a slow earlier
 * response cannot overwrite a faster later one, and any failure degrades silently: `refresh` resolves
 * to `null` and sets {@link failed} instead of throwing, leaving the last good {@link rooms} in place.
 */
export function useIrisSnapshot() {
  const rooms = shallowRef<readonly IrisRoom[]>([]);
  const failed = ref(false);

  let inFlight: AbortController | null = null;
  tryOnScopeDispose(() => inFlight?.abort());

  async function refresh(): Promise<readonly IrisRoom[] | null> {
    inFlight?.abort();
    const controller = new AbortController();
    inFlight = controller;
    try {
      const res = await fetch(IRIS_API_URL, { credentials: "omit", signal: controller.signal });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      if (controller.signal.aborted) return null;
      rooms.value = parseIrisRooms(body);
      failed.value = false;
      return rooms.value;
    } catch {
      if (controller.signal.aborted) return null;
      failed.value = true;
      return null;
    } finally {
      if (inFlight === controller) inFlight = null;
    }
  }

  return { rooms, failed, refresh };
}
