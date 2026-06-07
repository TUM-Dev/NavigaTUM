import type { paths } from "@/api_types";

type RoutingQuery = paths["/api/maps/route"]["get"]["parameters"]["query"];
interface UserRoutingPreferences {
  route_costing: NonNullable<RoutingQuery["route_costing"]>;
  pedestrian_type: NonNullable<RoutingQuery["pedestrian_type"]>;
  ptw_type: NonNullable<RoutingQuery["ptw_type"]>;
  bicycle_type: NonNullable<RoutingQuery["bicycle_type"]>;
}

const defaultPreferences: UserRoutingPreferences = {
  route_costing: "pedestrian",
  pedestrian_type: "standard",
  ptw_type: "motorcycle",
  bicycle_type: "hybrid",
};

export const useUserPreferences = () => {
  // No `default` factory: that would make `useCookie` write the default back during SSR, and on
  // `swr`-cached routes (see `routeRules` in `nuxt.config.ts`) that bakes a per-user `Set-Cookie`
  // into the shared response — leaking values across visitors and crashing Nitro with "Cannot
  // append headers after they are sent to the client". The defaults are merged in app code instead,
  // so the server only ever reads this cookie and the client is the sole writer.
  const stored = useCookie<Partial<UserRoutingPreferences> | null>("user-routing-preferences", {
    sameSite: "lax",
    secure: import.meta.env.PROD,
    httpOnly: false,
  });

  const preferences = computed<UserRoutingPreferences>(() => ({
    ...defaultPreferences,
    ...stored.value,
  }));

  const updatePreference = <K extends keyof UserRoutingPreferences>(
    key: K,
    value: UserRoutingPreferences[K]
  ) => {
    stored.value = { ...preferences.value, [key]: value };
  };

  return {
    preferences,
    updatePreference,
  };
};
