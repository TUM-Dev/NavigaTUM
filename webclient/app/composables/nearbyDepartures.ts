import {
  tryOnScopeDispose,
  useDocumentVisibility,
  useIntervalFn,
  useTimestamp,
} from "@vueuse/core";
import type { components } from "~/api_types";

type NearbyLocationsResponse = components["schemas"]["NearbyLocationsResponse"];
type TransportationResponse = components["schemas"]["TransportationResponse"];
type ModeResponse = components["schemas"]["ModeResponse"];

// Endorsed by the motis maintainers: CORS is `*` and the endpoint is cheap enough for per-pageview hits.
const STOPTIMES_URL = "https://api.transitous.org/api/v4/stoptimes";
const REFRESH_INTERVAL_MS = 180_000;
const TICK_INTERVAL_MS = 1_000;
const N_DEPARTURES = 3;
const LOOKAHEAD_S = 86_400;
// Ask the API for everything within this window (seconds), with N_DEPARTURES
// as a floor. Per spec, response size = max(n, count-in-window) - so a busy
// station returns *all* events in the window (often more than 3), while a
// sparse station still returns at least 3 departures spanning longer.
const NEAR_WINDOW_S = 10 * 60;

type PickupDropoffType = "NORMAL" | "NOT_ALLOWED" | "PHONE_AGENCY" | "COORDINATE_WITH_DRIVER";

export interface StopTimeEntry {
  readonly mode?: ModeResponse;
  readonly headsign?: string | null;
  readonly cancelled?: boolean;
  readonly tripCancelled?: boolean;
  // Whether the operator feeds live data for this trip; absent on feeds that never do.
  readonly realTime?: boolean;
  readonly displayName?: string | null;
  readonly routeShortName?: string | null;
  readonly routeColor?: string | null;
  readonly routeTextColor?: string | null;
  readonly agencyName?: string | null;
  readonly pickupDropoffType?: PickupDropoffType | null;
  readonly tripTo?: { readonly name?: string | null } | null;
  readonly place?: {
    readonly departure?: string | null;
    readonly scheduledDeparture?: string | null;
    readonly cancelled?: boolean;
    readonly track?: string | null;
    readonly scheduledTrack?: string | null;
    readonly pickupType?: PickupDropoffType | null;
    readonly dropoffType?: PickupDropoffType | null;
  } | null;
}
interface StopTimesResponse {
  readonly stopTimes?: readonly StopTimeEntry[];
}

export interface DepartureState {
  loading: boolean;
  error: string | null;
  entries: readonly StopTimeEntry[];
}

export interface StationView {
  readonly station: TransportationResponse;
  readonly state: DepartureState | undefined;
}

export type CountdownPhase =
  | { kind: "empty" }
  | { kind: "departed" }
  | { kind: "now" }
  | { kind: "minutes"; count: number }
  | { kind: "hours"; hours: number }
  | { kind: "hoursMinutes"; hours: number; minutes: number };

export type BoardingRestriction =
  | "none"
  | "no_boarding_alighting"
  | "alighting_only"
  | "boarding_only";

export function countdownPhase(iso: string | null | undefined, nowMs: number): CountdownPhase {
  if (!iso) return { kind: "empty" };
  const departure = Date.parse(iso);
  if (Number.isNaN(departure)) return { kind: "empty" };
  const diffMs = departure - nowMs;
  if (diffMs < -30_000) return { kind: "departed" };
  if (diffMs < 30_000) return { kind: "now" };
  const totalMinutes = Math.round(diffMs / 60_000);
  if (totalMinutes < 60) return { kind: "minutes", count: totalMinutes };
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  if (minutes === 0) return { kind: "hours", hours };
  return { kind: "hoursMinutes", hours, minutes };
}

export function trackOf(entry: StopTimeEntry): string | null {
  return entry.place?.track ?? entry.place?.scheduledTrack ?? null;
}

// `pickupDropoffType` summarizes the per-stop pickup/dropoff at the queried stop.
// NOT_ALLOWED means the train passes without picking up *or* dropping off here -
// e.g. sightseeing routes that only board/alight at fixed termini.
export function boardingRestriction(entry: StopTimeEntry): BoardingRestriction {
  if (entry.pickupDropoffType === "NOT_ALLOWED") return "no_boarding_alighting";
  if (entry.place?.pickupType === "NOT_ALLOWED" && entry.place?.dropoffType !== "NOT_ALLOWED") {
    return "alighting_only";
  }
  if (entry.place?.dropoffType === "NOT_ALLOWED" && entry.place?.pickupType !== "NOT_ALLOWED") {
    return "boarding_only";
  }
  return "none";
}

export function routeBadgeStyle(entry: StopTimeEntry): { backgroundColor: string; color: string } {
  return {
    backgroundColor: entry.routeColor ? `#${entry.routeColor}` : "#3f3f46",
    color: entry.routeTextColor ? `#${entry.routeTextColor}` : "#ffffff",
  };
}

export function isStopCancelled(entry: StopTimeEntry): boolean {
  return Boolean(entry.cancelled || entry.place?.cancelled);
}

export async function useNearbyDepartures(id: MaybeRefOrGetter<string>) {
  const { locale } = useI18n({ useScope: "global" });
  const runtimeConfig = useRuntimeConfig();

  const { data } = await useFetch<NearbyLocationsResponse, string>(
    () => `${runtimeConfig.public.apiURL}/api/locations/${toValue(id)}/nearby`,
    { dedupe: "cancel", credentials: "omit" }
  );

  const sortedStations = computed<readonly TransportationResponse[]>(() => {
    const list = data.value?.public_transport ?? [];
    return [...list].sort((a, b) => a.distance_meters - b.distance_meters);
  });

  const stationState = reactive(new Map<string, DepartureState>());

  const stations = computed<readonly StationView[]>(() =>
    sortedStations.value.map((station) => ({
      station,
      state: stationState.get(station.id),
    }))
  );
  const now = useTimestamp({ interval: TICK_INTERVAL_MS });
  // Non-reactive: cancels superseded in-flight fetches so a slow earlier
  // response can't overwrite a faster later one (toggle off→on, locale flip,
  // periodic refresh racing a manual toggle, …).
  const inFlight = new Map<string, AbortController>();

  tryOnScopeDispose(() => {
    for (const c of inFlight.values()) c.abort();
    inFlight.clear();
  });

  async function fetchDepartures(stationId: string): Promise<void> {
    inFlight.get(stationId)?.abort();
    const controller = new AbortController();
    inFlight.set(stationId, controller);

    const existing = stationState.get(stationId);
    if (existing) {
      existing.loading = true;
      existing.error = null;
    } else {
      stationState.set(stationId, { loading: true, error: null, entries: [] });
    }
    const params = new URLSearchParams({
      stopId: stationId,
      n: String(N_DEPARTURES),
      window: String(NEAR_WINDOW_S),
      language: locale.value,
    });
    try {
      const res = await fetch(`${STOPTIMES_URL}?${params.toString()}`, {
        credentials: "omit",
        signal: controller.signal,
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = (await res.json()) as StopTimesResponse;
      if (controller.signal.aborted) return;
      const entry = stationState.get(stationId);
      if (!entry) return;
      const nowMs = Date.now();
      const floor = nowMs - 30_000;
      const cutoff = nowMs + LOOKAHEAD_S * 1_000;
      entry.entries = (body.stopTimes ?? []).filter((e) => {
        const iso = e.place?.departure ?? e.place?.scheduledDeparture;
        if (!iso) return true;
        const departureMs = Date.parse(iso);
        if (Number.isNaN(departureMs)) return true;
        return departureMs >= floor && departureMs <= cutoff;
      });
      entry.loading = false;
    } catch (e) {
      if (controller.signal.aborted) return;
      const entry = stationState.get(stationId);
      if (entry) {
        entry.error = e instanceof Error ? e.message : String(e);
        entry.loading = false;
      }
    } finally {
      if (inFlight.get(stationId) === controller) inFlight.delete(stationId);
    }
  }

  function toggleExpand(stationId: string): void {
    if (stationState.has(stationId)) {
      inFlight.get(stationId)?.abort();
      inFlight.delete(stationId);
      stationState.delete(stationId);
      return;
    }
    void fetchDepartures(stationId);
  }

  watch(locale, () => {
    for (const stationId of stationState.keys()) {
      void fetchDepartures(stationId);
    }
  });

  const refresh = useIntervalFn(() => {
    for (const stationId of stationState.keys()) {
      void fetchDepartures(stationId);
    }
  }, REFRESH_INTERVAL_MS);

  // Suspend the Transitous polling while the tab is hidden - saves third-party
  // hits on backgrounded tabs and keeps the CORS-* allowance polite.
  const visibility = useDocumentVisibility();
  watch(visibility, (v) => {
    if (v === "visible") refresh.resume();
    else refresh.pause();
  });

  return { stations, toggleExpand, now };
}
