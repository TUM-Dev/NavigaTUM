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
  const preferences = useCookie<UserRoutingPreferences>("user-routing-preferences", {
    default: () => ({ ...defaultPreferences }),
    sameSite: "lax",
    secure: process.env.NODE_ENV === "production",
    httpOnly: false,
  });

  const updatePreference = <K extends keyof UserRoutingPreferences>(
    key: K,
    value: UserRoutingPreferences[K]
  ) => {
    if (preferences.value) {
      preferences.value[key] = value;
    }
  };

  return {
    preferences: readonly(preferences),
    updatePreference,
  };
};
