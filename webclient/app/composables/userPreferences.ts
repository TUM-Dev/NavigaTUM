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
  // A `default` factory would write the default back during SSR, baking a per-user `Set-Cookie`
  // into `swr`-cached shared responses (see PR #3147). Merge defaults client-side instead.
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
