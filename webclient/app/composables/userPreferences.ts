export interface UserRoutingPreferences {
  /** @description Preferred Transport mode the user wants to use */
  route_costing: "pedestrian" | "bicycle" | "motorcycle" | "car" | "public_transit";
  /** @description Does the user have specific walking restrictions? (affects narration and routing) */
  pedestrian_type?: "none" | "blind";
  /** @description Does the user prefer mopeds or motorcycles for powered two-wheeled (ptw)? */
  ptw_type?: "motorcycle" | "moped";
  /** @description Which kind of bicycle do you ride? */
  bicycle_type?: "road" | "hybrid" | "cross" | "mountain";
}

const defaultPreferences: UserRoutingPreferences = {
  route_costing: "pedestrian",
  pedestrian_type: "none",
  ptw_type: "motorcycle",
  bicycle_type: "road",
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
