import {
  tryOnScopeDispose,
  useDocumentVisibility,
  useElementVisibility,
  useTimeoutPoll,
} from "@vueuse/core";
import { useIrisSnapshot } from "~/composables/irisSnapshot";
import { buildRoomRows, type IrisRoomRow, roomsForBuildings } from "~/utils/iris";

// Iris recomputes room status roughly once a minute, so polling faster would only repeat work.
const POLL_INTERVAL_MS = 60_000;

/**
 * Drive the live Iris learning-room availability for one or more buildings.
 *
 * Builds on {@link useIrisSnapshot} for the browser-side roster fetch, filters it to the buildings,
 * resolves each room's `raum_nr_architekt` to a NavigaTUM room via the alias lookup (caching results,
 * omitting misses), and refreshes every {@link POLL_INTERVAL_MS} while the card is both on-screen and
 * in a foreground tab. Every fetch degrades silently: a failed poll keeps the last good snapshot.
 */
export function useIrisAvailability(
  buildingIds: MaybeRefOrGetter<readonly string[]>,
  target: MaybeRefOrGetter<HTMLElement | null | undefined>
) {
  // Explicit global scope: the bare useI18n() call defaults to 'local' when the
  // consuming component has an <i18n> block, which then double-registers a
  // local Composer and triggers vue-i18n's duplicate-call warning.
  const { locale } = useI18n({ useScope: "global" });
  const runtimeConfig = useRuntimeConfig();
  const snapshot = useIrisSnapshot();

  const rooms = shallowRef<readonly IrisRoomRow[]>([]);
  // `loading` stays true only until the first snapshot settles, so the card can show a placeholder
  // until then and hide afterwards if nothing resolved (the silent-degradation path).
  const loading = ref(true);

  // archName → resolved NavigaTUM path, or `null` once looked up but unmatched. Aliases are stable
  // across polls, so each is resolved at most once; `redirect_url` is locale-independent.
  const resolved = new Map<string, string | null>();

  let resolveBatch: AbortController | null = null;
  tryOnScopeDispose(() => resolveBatch?.abort());

  async function resolveAlias(archName: string, signal: AbortSignal): Promise<void> {
    if (resolved.has(archName)) return;
    try {
      const res = await fetch(
        `${runtimeConfig.public.apiURL}/api/locations/${encodeURIComponent(archName)}?lang=${locale.value}`,
        { credentials: "omit", signal }
      );
      if (!res.ok) {
        // A 404 means the room has no NavigaTUM alias; record the miss so it is never retried.
        // Other errors (5xx, network) are left unresolved so a later poll can try again.
        if (res.status === 404) resolved.set(archName, null);
        return;
      }
      const body = (await res.json()) as { redirect_url?: string | null };
      resolved.set(archName, body.redirect_url ?? null);
    } catch {
      // Transient failure - leave the alias unresolved and retry on the next poll.
    }
  }

  async function refresh(): Promise<void> {
    const all = await snapshot.refresh();
    // On failure or abort keep the last good rows; only the loading placeholder needs clearing.
    if (all === null) {
      loading.value = false;
      return;
    }

    resolveBatch?.abort();
    const controller = new AbortController();
    resolveBatch = controller;
    const buildingRooms = roomsForBuildings(all, toValue(buildingIds));
    await Promise.all(buildingRooms.map((room) => resolveAlias(room.archName, controller.signal)));
    if (controller.signal.aborted) return;

    rooms.value = buildRoomRows(buildingRooms, resolved);
    loading.value = false;
  }

  // `useTimeoutPoll` awaits each async refresh before scheduling the next, so a slow resolve cannot
  // pile up overlapping polls; `immediateCallback` refreshes the moment polling resumes.
  const poll = useTimeoutPoll(refresh, POLL_INTERVAL_MS, {
    immediate: false,
    immediateCallback: true,
  });

  // Poll only while the card is both on-screen and in a foreground tab; pause otherwise.
  const documentVisible = useDocumentVisibility();
  const onScreen = useElementVisibility(target);
  const active = computed(() => documentVisible.value === "visible" && onScreen.value);
  watch(
    active,
    (isActive) => {
      if (isActive) poll.resume();
      else poll.pause();
    },
    { immediate: true }
  );

  return { rooms, loading };
}
